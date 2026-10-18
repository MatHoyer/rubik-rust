use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use engine::random::{random_moves, Rng};
use engine::solver::{solve, Algo};
use engine::{format, Cube};

const USAGE: &str = "usage: ./rubik [-a thistlethwaite|kociemba|best] [-s] \"<mix>\"
       ./rubik -g LENGTH [-n COUNT]

  -a ALGO    solving algorithm (default: thistlethwaite; best = shortest of both)
  -s         show the solution stage by stage
  -g LENGTH  print a random mix of LENGTH moves instead of solving
  -n COUNT   with -g: print COUNT mixes, one per line";

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

    let start = Instant::now();
    let solution = solve(&cube, opts.algo, &|| start.elapsed().as_secs_f64() * 1000.0);
    if !opts.stages {
        return Ok(format(&solution.moves));
    }
    let mut lines: Vec<String> = solution
        .steps
        .iter()
        .map(|step| {
            let moves = if step.moves.is_empty() {
                "(nothing to do)".to_string()
            } else {
                format(&step.moves)
            };
            format!("{}: {moves}", step.label)
        })
        .collect();
    lines.push(format!(
        "Solution ({} moves): {}",
        solution.moves.len(),
        format(&solution.moves)
    ));
    Ok(lines.join("\n"))
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
            "-a" => opts.algo = value("-a")?.parse()?,
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
