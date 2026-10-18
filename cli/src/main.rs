use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(solution) => {
            println!("{solution}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("rubik: {msg}");
            eprintln!("usage: ./rubik \"<mix>\"   e.g. ./rubik \"F R U2 B' L' D'\"");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let mut args = env::args_os().skip(1);
    let mix = args.next().ok_or("missing mix argument")?;
    if args.next().is_some() {
        return Err("too many arguments (put the whole mix in quotes)".into());
    }
    let mix = mix.to_str().ok_or("mix is not valid UTF-8")?;
    let moves = engine::parse(mix).map_err(|e| e.to_string())?;

    let cube = engine::Cube::SOLVED.apply_all(&moves);
    if cube.is_solved() {
        return Ok(String::new());
    }
    // shortcut: no solver yet, replaced by Thistlethwaite in milestones 5-8.
    Err("solver not implemented yet".into())
}
