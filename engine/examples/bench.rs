//! Benchmark on uniformly random cube states.
//!
//! cargo run --release -p engine --example bench -- [--algo thistlethwaite|kociemba]
//!     [--states N] [--seed S] [--target MOVES] [--budget MS] [--cli PATH] [--runs N]
//!
//! `--target` and `--budget` tune Kociemba (stop at that length, or after that time per cube).
//!
//! In-process: solution lengths (average, worst, per stage) and solve time.
//! With `--cli`: runs the real binary on `--runs` scrambles and measures wall time,
//! table building included. Exits 1 if any limit of the subject is broken.

use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use engine::random::{random_cube, Rng};
use engine::{format, inverse, parse, Cube, Kociemba, Move, Thistlethwaite};

type StageSolver<'a> = Box<dyn Fn(&Cube) -> Vec<Vec<Move>> + 'a>;

const MAX_AVERAGE: f64 = 50.0;
const MAX_TIME: Duration = Duration::from_secs(3);

struct Args {
    algo: String,
    target: usize,
    budget: Duration,
    states: usize,
    seed: u64,
    cli: Option<String>,
    runs: usize,
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("bench: {e}");
            eprintln!("usage: bench [--algo A] [--states N] [--seed S] [--target M] [--budget MS] [--cli PATH] [--runs N]");
            return ExitCode::FAILURE;
        }
    };
    let mut ok = true;

    let start = Instant::now();
    let thistlethwaite = Thistlethwaite::new();
    let solve: StageSolver = match args.algo.as_str() {
        "thistlethwaite" => Box::new(|c| thistlethwaite.solve_stages(c)),
        "kociemba" => {
            let kociemba = Kociemba::new();
            let (target, budget) = (args.target, args.budget);
            Box::new(move |c| {
                let deadline = Instant::now() + budget;
                kociemba.solve_phases(c, target, &|| Instant::now() >= deadline)
            })
        }
        other => {
            eprintln!("bench: unknown algorithm {other}");
            return ExitCode::FAILURE;
        }
    };
    println!("{}: tables built in {:?}", args.algo, start.elapsed());

    let mut rng = Rng::new(args.seed);
    let mut lengths = Vec::with_capacity(args.states);
    let mut stage_totals = Vec::new();
    let mut stage_worst = Vec::new();
    let mut worst_solve = Duration::ZERO;
    let start = Instant::now();
    for _ in 0..args.states {
        let cube = random_cube(&mut rng);
        let t = Instant::now();
        let stages = solve(&cube);
        stage_totals.resize(stages.len(), 0);
        stage_worst.resize(stages.len(), 0);
        let solution = engine::merge(&stages.concat());
        worst_solve = worst_solve.max(t.elapsed());
        if !cube.apply_all(&solution).is_solved() {
            println!("FAIL: wrong solution for {cube:?}");
            ok = false;
        }
        for (i, s) in stages.iter().enumerate() {
            stage_totals[i] += s.len();
            stage_worst[i] = stage_worst[i].max(s.len());
        }
        lengths.push(solution.len());
    }
    let n = args.states as f64;
    let average = lengths.iter().sum::<usize>() as f64 / n;
    println!(
        "{} random states: average {average:.2} moves, min {}, max {}",
        args.states,
        lengths.iter().min().unwrap_or(&0),
        lengths.iter().max().unwrap_or(&0),
    );
    for i in 0..stage_totals.len() {
        println!(
            "  stage {}: average {:.2}, max {}",
            i + 1,
            stage_totals[i] as f64 / n,
            stage_worst[i]
        );
    }
    println!(
        "solve time: average {:?}, worst {worst_solve:?}",
        start.elapsed() / args.states.max(1) as u32
    );
    print_histogram(&lengths);
    if average > MAX_AVERAGE {
        println!("FAIL: average above {MAX_AVERAGE}");
        ok = false;
    }

    if let Some(cli) = &args.cli {
        ok &= bench_cli(cli, &args, &thistlethwaite, &mut rng);
    }

    if ok {
        println!("OK");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Runs the binary on scrambles of random states (the inverse of their solution).
fn bench_cli(cli: &str, args: &Args, solver: &Thistlethwaite, rng: &mut Rng) -> bool {
    let runs = args.runs;
    let mut ok = true;
    let mut worst = Duration::ZERO;
    let mut total = Duration::ZERO;
    for _ in 0..runs {
        let mix = format(&inverse(&solver.solve(&random_cube(rng))));
        let t = Instant::now();
        let out = match Command::new(cli).args(["-a", &args.algo, &mix]).output() {
            Ok(out) => out,
            Err(e) => {
                println!("FAIL: cannot run {cli}: {e}");
                return false;
            }
        };
        let elapsed = t.elapsed();
        worst = worst.max(elapsed);
        total += elapsed;
        let solution = String::from_utf8_lossy(&out.stdout);
        let solved = parse(&format!("{mix} {solution}"))
            .map(|moves| Cube::SOLVED.apply_all(&moves).is_solved())
            .unwrap_or(false);
        if !out.status.success() || !solved {
            println!("FAIL: {cli} did not solve \"{mix}\"");
            ok = false;
        }
    }
    println!(
        "{cli} on {runs} scrambles: average {:?}, worst {worst:?} (limit {MAX_TIME:?})",
        total / runs.max(1) as u32
    );
    if worst > MAX_TIME {
        println!("FAIL: over the time limit");
        ok = false;
    }
    ok
}

fn print_histogram(lengths: &[usize]) {
    let max = lengths.iter().copied().max().unwrap_or(0);
    let mut counts = vec![0usize; max + 1];
    for &l in lengths {
        counts[l] += 1;
    }
    let peak = counts.iter().copied().max().unwrap_or(1).max(1);
    for (len, &count) in counts.iter().enumerate().filter(|(_, &c)| c > 0) {
        println!("  {len:>2} | {:<40} {count}", "#".repeat(count * 40 / peak));
    }
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        algo: "thistlethwaite".into(),
        target: 20,
        budget: Duration::from_millis(500),
        states: 10_000,
        seed: 1,
        cli: None,
        runs: 100,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let value = it.next().ok_or(format!("missing value for {flag}"))?;
        let number = || {
            value
                .parse()
                .map_err(|_| format!("bad number for {flag}: {value}"))
        };
        match flag.as_str() {
            "--states" => args.states = number()? as usize,
            "--seed" => args.seed = number()?,
            "--runs" => args.runs = number()? as usize,
            "--cli" => args.cli = Some(value),
            "--algo" => args.algo = value,
            "--target" => args.target = number()? as usize,
            "--budget" => args.budget = Duration::from_millis(number()?),
            _ => return Err(format!("unknown option {flag}")),
        }
    }
    Ok(args)
}
