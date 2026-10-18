use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use engine::kociemba::PHASE_GOALS;
use engine::random::{random_moves, Rng};
use engine::thistlethwaite::STAGE_GOALS;
use engine::{format, merge, Cube, Kociemba, Move, Thistlethwaite};

const USAGE: &str = "usage: ./rubik [-a thistlethwaite|kociemba|best] [-s] \"<mix>\"
       ./rubik -g LENGTH [-n COUNT]

  -a ALGO    solving algorithm (default: thistlethwaite; best = shortest of both)
  -s         show the solution stage by stage
  -g LENGTH  print a random mix of LENGTH moves instead of solving
  -n COUNT   with -g: print COUNT mixes, one per line";

/// Kociemba stops at this many moves, or after the time budget.
const KOCIEMBA_TARGET: usize = 20;
const KOCIEMBA_BUDGET: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, PartialEq)]
enum Algo {
    Thistlethwaite,
    Kociemba,
    Best,
}

struct Options {
    algo: Algo,
    stages: bool,
    generate: Option<usize>,
    count: Option<usize>,
    mix: Option<String>,
}

// writeln! instead of println!/eprintln!: those panic when the pipe is closed (`| head -c0`).
fn main() -> ExitCode {
    match run() {
        Ok(output) => match writeln!(io::stdout(), "{output}") {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        },
        Err(msg) => {
            let _ = writeln!(io::stderr(), "rubik: {msg}\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let opts = parse_args()?;
    if let Some(len) = opts.generate {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        let mut rng = Rng::new(seed);
        let mixes: Vec<String> = (0..opts.count.unwrap_or(1))
            .map(|_| format(&random_moves(&mut rng, len)))
            .collect();
        return Ok(mixes.join("\n"));
    }

    let mix = opts.mix.ok_or("missing mix argument")?;
    let moves = engine::parse(&mix).map_err(|e| e.to_string())?;
    let cube = Cube::SOLVED.apply_all(&moves);
    if cube.is_solved() {
        return Ok(String::new());
    }

    let stages = solve(&cube, opts.algo);
    let solution = merge(
        &stages
            .iter()
            .flat_map(|(_, m)| m.clone())
            .collect::<Vec<_>>(),
    );
    if !opts.stages {
        return Ok(format(&solution));
    }
    let mut lines: Vec<String> = stages
        .iter()
        .map(|(label, moves)| {
            let moves = if moves.is_empty() {
                "(nothing to do)".to_string()
            } else {
                format(moves)
            };
            format!("{label}: {moves}")
        })
        .collect();
    lines.push(format!(
        "Solution ({} moves): {}",
        solution.len(),
        format(&solution)
    ));
    Ok(lines.join("\n"))
}

/// The moves of each stage with a label saying what the stage does.
fn solve(cube: &Cube, algo: Algo) -> Vec<(String, Vec<Move>)> {
    let thistlethwaite = || {
        let stages = Thistlethwaite::new().solve_stages(cube);
        label("Stage", &STAGE_GOALS, stages)
    };
    let kociemba = || {
        let phases = Kociemba::new().solve_phases(cube, KOCIEMBA_TARGET, KOCIEMBA_BUDGET);
        label("Phase", &PHASE_GOALS, phases)
    };
    match algo {
        Algo::Thistlethwaite => thistlethwaite(),
        Algo::Kociemba => kociemba(),
        Algo::Best => {
            let len = |s: &[(String, Vec<Move>)]| {
                merge(&s.iter().flat_map(|(_, m)| m.clone()).collect::<Vec<_>>()).len()
            };
            let (a, b) = (thistlethwaite(), kociemba());
            if len(&a) <= len(&b) {
                a
            } else {
                b
            }
        }
    }
}

fn label(kind: &str, goals: &[&str], stages: Vec<Vec<Move>>) -> Vec<(String, Vec<Move>)> {
    goals
        .iter()
        .zip(stages)
        .enumerate()
        .map(|(i, (goal, moves))| (format!("{kind} {} - {goal}", i + 1), moves))
        .collect()
}

/// Options first, each starting with `-`, then at most one mix.
fn parse_args() -> Result<Options, String> {
    let mut opts = Options {
        algo: Algo::Thistlethwaite,
        stages: false,
        generate: None,
        count: None,
        mix: None,
    };
    let mut args = env::args_os().skip(1).map(|a| {
        a.into_string()
            .map_err(|_| "argument is not valid UTF-8".to_string())
    });
    while let Some(arg) = args.next().transpose()? {
        if opts.mix.is_some() {
            return Err("too many arguments (put the whole mix in quotes)".into());
        }
        let mut value = |name: &str| {
            args.next()
                .transpose()?
                .ok_or(format!("option {name} needs a value"))
        };
        match arg.as_str() {
            "-a" => {
                opts.algo = match value("-a")?.as_str() {
                    "thistlethwaite" => Algo::Thistlethwaite,
                    "kociemba" => Algo::Kociemba,
                    "best" => Algo::Best,
                    other => return Err(format!("unknown algorithm '{other}'")),
                }
            }
            "-s" => opts.stages = true,
            "-g" => opts.generate = Some(positive(&value("-g")?, "-g")?),
            "-n" => opts.count = Some(positive(&value("-n")?, "-n")?),
            _ if arg.starts_with('-') => return Err(format!("unknown option '{arg}'")),
            _ => opts.mix = Some(arg),
        }
    }
    if opts.generate.is_some() {
        if opts.mix.is_some() || opts.stages || opts.algo != Algo::Thistlethwaite {
            return Err("-g generates a mix, it cannot be combined with a mix, -a or -s".into());
        }
    } else if opts.count.is_some() {
        return Err("-n only works with -g".into());
    }
    Ok(opts)
}

fn positive(value: &str, name: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(format!("{name} needs a positive number, got '{value}'")),
    }
}
