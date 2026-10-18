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
