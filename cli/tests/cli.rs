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
