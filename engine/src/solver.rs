//! Algorithm choice shared by the CLI and the web: solve a cube, keep every stage labelled.
//! Tables are built once per process and reused (matters in the browser, where the page
//! solves many cubes).

use std::str::FromStr;
use std::sync::OnceLock;

use crate::kociemba::{Kociemba, PHASE_GOALS};
use crate::moves::{merge, Move};
use crate::thistlethwaite::{Thistlethwaite, STAGE_GOALS};
use crate::Cube;

/// Kociemba stops at this many moves, or after `KOCIEMBA_BUDGET_MS` of search.
pub const KOCIEMBA_TARGET: usize = 20;
pub const KOCIEMBA_BUDGET_MS: f64 = 500.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algo {
    Thistlethwaite,
    Kociemba,
    /// Both, keep the shorter.
    Best,
}

impl FromStr for Algo {
    type Err = String;

    fn from_str(s: &str) -> Result<Algo, String> {
        match s {
            "thistlethwaite" => Ok(Algo::Thistlethwaite),
            "kociemba" => Ok(Algo::Kociemba),
            "best" => Ok(Algo::Best),
            other => Err(format!(
                "unknown algorithm '{other}' (expected thistlethwaite, kociemba or best)"
            )),
        }
    }
}

pub struct Step {
    /// e.g. "Stage 1 - orient all edges".
    pub label: String,
    pub moves: Vec<Move>,
}

pub struct Solution {
    pub steps: Vec<Step>,
    /// All steps with moves merged across step boundaries.
    pub moves: Vec<Move>,
}

/// `now_ms` is any clock in milliseconds (`Instant` natively, `Date.now()` in the browser).
pub fn solve(cube: &Cube, algo: Algo, now_ms: &dyn Fn() -> f64) -> Solution {
    match algo {
        Algo::Thistlethwaite => {
            solution("Stage", &STAGE_GOALS, thistlethwaite().solve_stages(cube))
        }
        Algo::Kociemba => {
            let solver = kociemba(); // build tables before starting the clock
            let deadline = now_ms() + KOCIEMBA_BUDGET_MS;
            let phases = solver.solve_phases(cube, KOCIEMBA_TARGET, &|| now_ms() >= deadline);
            solution("Phase", &PHASE_GOALS, phases)
        }
        Algo::Best => {
            let a = solve(cube, Algo::Thistlethwaite, now_ms);
            let b = solve(cube, Algo::Kociemba, now_ms);
            if a.moves.len() <= b.moves.len() {
                a
            } else {
                b
            }
        }
    }
}

pub fn thistlethwaite() -> &'static Thistlethwaite {
    static SOLVER: OnceLock<Thistlethwaite> = OnceLock::new();
    SOLVER.get_or_init(Thistlethwaite::new)
}

pub fn kociemba() -> &'static Kociemba {
    static SOLVER: OnceLock<Kociemba> = OnceLock::new();
    SOLVER.get_or_init(Kociemba::new)
}

fn solution(kind: &str, goals: &[&str], stages: Vec<Vec<Move>>) -> Solution {
    let moves = merge(&stages.concat());
    let steps = goals
        .iter()
        .zip(stages)
        .enumerate()
        .map(|(i, (goal, moves))| Step {
            label: format!("{kind} {} - {goal}", i + 1),
            moves,
        })
        .collect();
    Solution { steps, moves }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn never() -> f64 {
        0.0
    }

    #[test]
    fn parses_algorithm_names() {
        assert_eq!("kociemba".parse(), Ok(Algo::Kociemba));
        assert!("Kociemba".parse::<Algo>().is_err());
    }

    #[test]
    fn every_algorithm_solves_with_labels() {
        let cube = Cube::SOLVED.apply_all(&crate::parse("F R U2 B' L' D' R2 F' U L2").unwrap());
        for (algo, steps) in [
            (Algo::Thistlethwaite, 4),
            (Algo::Kociemba, 2),
            (Algo::Best, 2),
        ] {
            let s = solve(&cube, algo, &never);
            assert_eq!(s.steps.len(), steps, "{algo:?}");
            assert!(cube.apply_all(&s.moves).is_solved(), "{algo:?}");
            let all: Vec<Move> = s.steps.iter().flat_map(|s| s.moves.clone()).collect();
            assert_eq!(merge(&all), s.moves);
        }
    }
}
