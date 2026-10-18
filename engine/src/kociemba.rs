//! Kociemba's two-phase algorithm: shorter solutions (about 20 moves) than Thistlethwaite.
//!
//! Phase 1 does Thistlethwaite's stages 1 and 2 at once (orient everything, E-slice edges
//! into the E slice) with all 18 moves. Phase 2 solves the rest with `U D L2 R2 F2 B2`.
//! Both phases are too big for one exact distance table, so each uses IDA* search guided by
//! two exact tables of smaller views of the cube (the larger of the two is a safe estimate).
//!
//! The trick: a short phase 1 can lead to a long phase 2, so after the first solution we keep
//! trying longer phase 1 solutions, as long as the total keeps shrinking, until the solution
//! is short enough or the time budget is spent.

use std::time::{Duration, Instant};

use crate::moves::{merge, Face, Move, ALL_MOVES};
use crate::stage::{Ids, Stage};
use crate::thistlethwaite::{corner_orientation, e_slice_positions, edge_orientation, moves, pack};
use crate::Cube;

/// Phase 2 never needs more than 18 moves.
const MAX_PHASE2: usize = 18;
const MAX_PHASE1: usize = 20;

/// What each phase does, in plain words.
pub const PHASE_GOALS: [&str; 2] = [
    "orient everything, middle-layer edges into the middle layer",
    "solve with U D L2 R2 F2 B2",
];

pub struct Kociemba {
    phase1: [Stage; 2],
    phase2: [Stage; 2],
}

impl Kociemba {
    /// Builds the four tables (about 1M states each).
    pub fn new() -> Kociemba {
        let all = ALL_MOVES.to_vec();
        let p2 = moves(&[Face::U, Face::D], &[Face::L, Face::R, Face::F, Face::B]);
        Kociemba {
            phase1: [
                Stage::new(
                    all.clone(),
                    vec![Box::new(corner_orientation), Box::new(e_slice_positions)],
                ),
                Stage::new(
                    all,
                    vec![Box::new(edge_orientation), Box::new(e_slice_positions)],
                ),
            ],
            phase2: [
                Stage::new(
                    p2.clone(),
                    vec![Box::new(|c: &Cube| pack(c.cp)), Box::new(e_slice_perm)],
                ),
                Stage::new(p2, vec![Box::new(ud_edge_perm), Box::new(e_slice_perm)]),
            ],
        }
    }

    /// Phase 1 and phase 2 moves. Stops as soon as the total is at most `target` moves,
    /// or when `budget` is spent (the first solution is always found, budget or not).
    pub fn solve_phases(&self, cube: &Cube, target: usize, budget: Duration) -> Vec<Vec<Move>> {
        let mut search = Search {
            k: self,
            cube: *cube,
            path1: Vec::new(),
            path2: Vec::new(),
            best: None,
            target,
            deadline: Instant::now() + budget,
        };
        let start = self
            .phase1
            .each_ref()
            .map(|s| s.ids_of(cube).expect("phase 1 tables cover every cube"));
        for depth in 0..=MAX_PHASE1 {
            let done = search.phase1(start, depth);
            let best_len = search
                .best
                .as_ref()
                .map_or(usize::MAX, |(a, b)| a.len() + b.len());
            if done || depth + 1 >= best_len {
                break;
            }
        }
        let (p1, p2) = search
            .best
            .expect("phase 2 always finishes within 18 moves");
        vec![p1, p2]
    }

    /// A full solution of at most `target` moves if found within `budget`.
    pub fn solve(&self, cube: &Cube, target: usize, budget: Duration) -> Vec<Move> {
        merge(&self.solve_phases(cube, target, budget).concat())
    }
}

impl Default for Kociemba {
    fn default() -> Self {
        Self::new()
    }
}

struct Search<'a> {
    k: &'a Kociemba,
    cube: Cube,
    path1: Vec<Move>,
    path2: Vec<Move>,
    best: Option<(Vec<Move>, Vec<Move>)>,
    target: usize,
    deadline: Instant,
}

impl Search<'_> {
    /// Looks for phase 1 solutions of exactly `left` more moves. Returns `true` to stop.
    fn phase1(&mut self, ids: [Ids; 2], left: usize) -> bool {
        let estimate = estimate(&self.k.phase1, &ids);
        if estimate > left {
            return false;
        }
        if left == 0 {
            // Ending on a phase 2 move means a shorter phase 1 was already tried.
            if let Some(&last) = self.path1.last() {
                if self.k.phase2[0].moves().contains(&last) {
                    return false;
                }
            }
            return self.phase2();
        }
        for (m, &mv) in self.k.phase1[0].moves().iter().enumerate() {
            if !mv.can_follow(self.path1.last()) {
                continue;
            }
            self.path1.push(mv);
            let next = [0, 1].map(|i| self.k.phase1[i].step(&ids[i], m));
            let stop = self.phase1(next, left - 1);
            self.path1.pop();
            if stop {
                return true;
            }
        }
        false
    }

    /// Finishes the current phase 1 path with the shortest phase 2 that beats the best
    /// solution so far. Returns `true` to stop the whole search.
    fn phase2(&mut self) -> bool {
        let cube = self.cube.apply_all(&self.path1);
        let ids = self.k.phase2.each_ref().map(|s| {
            s.ids_of(&cube)
                .expect("phase 1 leaves the cube in the phase 2 group")
        });
        let limit = match &self.best {
            Some((a, b)) => (a.len() + b.len()).saturating_sub(self.path1.len() + 1),
            None => MAX_PHASE2,
        }
        .min(MAX_PHASE2);
        for depth in estimate(&self.k.phase2, &ids)..=limit {
            self.path2.clear();
            if self.phase2_dfs(ids, depth) {
                self.best = Some((self.path1.clone(), self.path2.clone()));
                break;
            }
        }
        let best_len = self
            .best
            .as_ref()
            .map_or(usize::MAX, |(a, b)| a.len() + b.len());
        best_len <= self.target || (self.best.is_some() && Instant::now() >= self.deadline)
    }

    fn phase2_dfs(&mut self, ids: [Ids; 2], left: usize) -> bool {
        let estimate = estimate(&self.k.phase2, &ids);
        if estimate > left {
            return false;
        }
        if left == 0 {
            return true;
        }
        for (m, &mv) in self.k.phase2[0].moves().iter().enumerate() {
            let prev = self.path2.last().or(self.path1.last());
            if !mv.can_follow(prev) {
                continue;
            }
            self.path2.push(mv);
            let next = [0, 1].map(|i| self.k.phase2[i].step(&ids[i], m));
            if self.phase2_dfs(next, left - 1) {
                return true;
            }
            self.path2.pop();
        }
        false
    }
}

/// Lower bound on the moves left: the larger of the two exact partial distances.
fn estimate(stages: &[Stage; 2], ids: &[Ids; 2]) -> usize {
    stages[0].dist(&ids[0]).max(stages[1].dist(&ids[1])) as usize
}

/// Order of the 4 E-slice edges inside the E slice (valid in the phase 2 group).
fn e_slice_perm(c: &Cube) -> u128 {
    pack([c.ep[8], c.ep[9], c.ep[10], c.ep[11]])
}

/// Order of the 8 U/D-layer edges (valid in the phase 2 group).
fn ud_edge_perm(c: &Cube) -> u128 {
    pack([
        c.ep[0], c.ep[1], c.ep[2], c.ep[3], c.ep[4], c.ep[5], c.ep[6], c.ep[7],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{random_cube, Rng};
    use std::sync::LazyLock;

    static SOLVER: LazyLock<Kociemba> = LazyLock::new(Kociemba::new);

    #[test]
    fn table_sizes() {
        let sizes: Vec<_> = SOLVER
            .phase1
            .iter()
            .chain(&SOLVER.phase2)
            .map(Stage::size)
            .collect();
        assert_eq!(sizes, [2187 * 495, 2048 * 495, 40320 * 24, 40320 * 24]);
    }

    #[test]
    fn solved_cube_needs_no_moves() {
        assert!(SOLVER
            .solve(&Cube::SOLVED, 20, Duration::from_secs(1))
            .is_empty());
    }

    #[test]
    fn short_mix_gets_optimal_solution() {
        let mix = crate::parse("R U F").unwrap();
        let cube = Cube::SOLVED.apply_all(&mix);
        let solution = SOLVER.solve(&cube, 3, Duration::from_secs(5));
        assert_eq!(crate::format(&solution), "F' U' R'");
    }

    #[test]
    fn solves_random_states_in_few_moves() {
        let mut rng = Rng::new(5);
        for _ in 0..20 {
            let cube = random_cube(&mut rng);
            let phases = SOLVER.solve_phases(&cube, 22, Duration::from_secs(5));
            let after1 = cube.apply_all(&phases[0]);
            assert!(SOLVER.phase2[0].ids_of(&after1).is_some());
            assert!(phases[1]
                .iter()
                .all(|m| SOLVER.phase2[0].moves().contains(m)));
            let solution = merge(&phases.concat());
            assert!(cube.apply_all(&solution).is_solved());
            assert!(solution.len() <= 22, "{} moves", solution.len());
            assert!(solution.windows(2).all(|w| w[0].face != w[1].face));
        }
    }
}
