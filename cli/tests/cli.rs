use std::process::{Command, Output};

fn rubik(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rubik"))
        .args(args)
        .output()
        .expect("failed to run rubik")
}

fn assert_error(args: &[&str]) {
    let out = rubik(args);
    assert!(!out.status.success(), "{args:?} should fail");
    assert!(out.stdout.is_empty(), "{args:?} printed to stdout");
    assert!(!out.stderr.is_empty(), "{args:?} printed no error");
}

#[test]
fn rejects_bad_arguments() {
    assert_error(&[]);
    assert_error(&[""]);
    assert_error(&["   "]);
    assert_error(&["F", "R"]);
    assert_error(&["-x", "R"]);
    assert_error(&["-a"]);
    assert_error(&["-a", "foo", "R"]);
    assert_error(&["-g", "0"]);
    assert_error(&["-g", "abc"]);
    assert_error(&["-g", "5", "R"]);
    assert_error(&["-n", "3", "R"]);
    assert_error(&["R", "-s"]);
    assert_error(&["-s"]);
    for bad in ["X", "F3", "M", "x", "f", "F2'"] {
        assert_error(&[&format!("R {bad}")]);
    }
}

#[test]
fn cancelling_mix_prints_empty_line() {
    let out = rubik(&["F F'"]);
    assert!(out.status.success());
    assert_eq!(out.stdout, b"\n");
}

#[test]
fn solution_solves_the_mix() {
    let mix = "F R U2 B' L' D' R2 F' U L2 D B2 R' F U2 L";
    let out = rubik(&[mix]);
    assert!(out.status.success());
    let solution = String::from_utf8(out.stdout).unwrap();
    assert!(solution.ends_with('\n') && solution.lines().count() == 1);
    let moves = engine::parse(&format!("{mix} {solution}")).unwrap();
    assert!(engine::Cube::SOLVED.apply_all(&moves).is_solved());
}

#[test]
fn very_long_mix_is_solved() {
    let mix = "R U F' L2 D B' ".repeat(500);
    let out = rubik(&[&mix]);
    assert!(out.status.success());
    let solution = String::from_utf8(out.stdout).unwrap();
    let moves = engine::parse(&format!("{mix} {solution}")).unwrap();
    assert!(engine::Cube::SOLVED.apply_all(&moves).is_solved());
}

fn solves(args: &[&str], mix: &str) -> String {
    let mut all = args.to_vec();
    all.push(mix);
    let out = rubik(&all);
    assert!(out.status.success(), "{args:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let solution = stdout.lines().last().unwrap().rsplit(':').next().unwrap();
    let moves = engine::parse(&format!("{mix} {solution}")).unwrap();
    assert!(
        engine::Cube::SOLVED.apply_all(&moves).is_solved(),
        "{args:?}"
    );
    stdout
}

#[test]
fn every_algorithm_solves() {
    let mix = "F R U2 B' L' D' R2 F' U L2 D B2 R' F U2 L";
    for algo in ["thistlethwaite", "kociemba", "best"] {
        solves(&["-a", algo], mix);
    }
    let kociemba = solves(&["-a", "kociemba"], mix).split_whitespace().count();
    assert!(kociemba <= 22, "{kociemba} moves");
}

#[test]
fn stages_are_labelled() {
    let mix = "F R U2 B' L' D' R2 F' U L2 D B2";
    let out = solves(&["-s"], mix);
    assert_eq!(out.lines().count(), 5);
    assert!(out.starts_with("Stage 1 - "));
    assert!(out.lines().last().unwrap().starts_with("Solution ("));
    let out = solves(&["-a", "kociemba", "-s"], mix);
    assert_eq!(out.lines().count(), 3);
    assert!(out.starts_with("Phase 1 - "));
}

#[test]
fn generator_prints_valid_mixes() {
    let out = rubik(&["-g", "25", "-n", "4"]);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 4);
    for line in stdout.lines() {
        let moves = engine::parse(line).unwrap();
        assert_eq!(moves.len(), 25);
        assert_eq!(engine::merge(&moves), moves);
    }
}
