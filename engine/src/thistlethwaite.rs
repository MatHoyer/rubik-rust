//! Thistlethwaite's algorithm: four stages, each restricting the allowed moves further.
//!
//! | Stage  | Allowed moves              | Goal                                                  |
//! |--------|----------------------------|-------------------------------------------------------|
//! | 0 -> 1 | all 18                     | orient all edges                                      |
//! | 1 -> 2 | U D L R, F2 B2             | orient all corners, E-slice edges into the E slice    |
//! | 2 -> 3 | U D, L2 R2 F2 B2           | M/S-slice edges into their slices, corners into the   |
//! |        |                            | 96 permutations reachable by half turns               |
//! | 3 -> 4 | U2 D2 L2 R2 F2 B2          | solved                                                |

use crate::moves::{merge, Face, Move, ALL_MOVES};
use crate::stage::Stage;
use crate::Cube;

pub struct Thistlethwaite {
    stages: [Stage; 4],
}

impl Thistlethwaite {
    /// Builds every stage's tables.
    pub fn new() -> Thistlethwaite {
        Thistlethwaite {
            stages: [stage1(), stage2(), stage3(), stage4()],
        }
    }

    /// The moves of each stage, in order. Playing them all solves `cube`.
    pub fn solve_stages(&self, cube: &Cube) -> Vec<Vec<Move>> {
        let mut cube = *cube;
        self.stages
            .iter()
            .map(|stage| {
                let moves = stage
                    .solve(&cube)
                    .expect("each stage leaves the cube in the next stage's start group");
                cube = cube.apply_all(&moves);
                moves
            })
            .collect()
    }

    /// A full solution, with moves merged across stage boundaries.
    pub fn solve(&self, cube: &Cube) -> Vec<Move> {
        merge(&self.solve_stages(cube).concat())
    }
}

impl Default for Thistlethwaite {
    fn default() -> Self {
        Self::new()
    }
}

/// Stage 1: flip every edge to "good". Edge orientation only changes on F and B
/// quarter turns, so once it is fixed the cube can be solved without them.
pub fn stage1() -> Stage {
    Stage::new(ALL_MOVES.to_vec(), vec![Box::new(edge_orientation)])
}

/// Stage 2: twist every corner to "good" (only L and R quarter turns twist corners
/// among the allowed moves) and bring the 4 E-slice edges into the E slice.
pub fn stage2() -> Stage {
    Stage::new(
        moves(&[Face::U, Face::D, Face::L, Face::R], &[Face::F, Face::B]),
        vec![Box::new(corner_orientation), Box::new(e_slice_positions)],
    )
}

/// Stage 3: bring M-slice edges into the M slice (S-slice edges follow), and corners
/// into a permutation that half turns alone can solve.
pub fn stage3() -> Stage {
    let half_turn_corners = half_turn_corner_group();
    let corner_class = move |c: &Cube| {
        half_turn_corners
            .iter()
            .map(|g| pack(c.cp.map(|piece| g[piece as usize])))
            .min()
            .expect("group is never empty")
    };
    Stage::new(
        moves(&[Face::U, Face::D], &[Face::L, Face::R, Face::F, Face::B]),
        vec![Box::new(m_slice_positions), Box::new(corner_class)],
    )
}

/// Stage 4: solve with half turns only.
pub fn stage4() -> Stage {
    Stage::new(
        moves(&[], &Face::ALL),
        vec![
            Box::new(|c: &Cube| pack(c.cp)),
            Box::new(|c: &Cube| pack(c.ep)),
        ],
    )
}

/// Quarter and half turns of `quarter`, half turns only of `half`.
fn moves(quarter: &[Face], half: &[Face]) -> Vec<Move> {
    let quarters = quarter
        .iter()
        .flat_map(|&face| (1..=3).map(move |turns| Move { face, turns }));
    let halves = half.iter().map(|&face| Move { face, turns: 2 });
    quarters.chain(halves).collect()
}

/// Corner permutations reachable from solved with half turns only (96 of them).
fn half_turn_corner_group() -> Vec<[u8; 8]> {
    let half_turns = moves(&[], &Face::ALL);
    let mut group = vec![Cube::SOLVED.cp];
    let mut i = 0;
    while i < group.len() {
        let cube = Cube {
            cp: group[i],
            ..Cube::SOLVED
        };
        for &m in &half_turns {
            let cp = cube.apply(m).cp;
            if !group.contains(&cp) {
                group.push(cp);
            }
        }
        i += 1;
    }
    group
}

fn pack<const N: usize>(values: [u8; N]) -> u128 {
    values.iter().fold(0, |acc, &v| acc << 4 | v as u128)
}

fn edge_orientation(c: &Cube) -> u128 {
    pack(c.eo)
}

fn corner_orientation(c: &Cube) -> u128 {
    pack(c.co)
}

/// Which slots hold an E-slice edge (FR FL BL BR, pieces 8..12).
fn e_slice_positions(c: &Cube) -> u128 {
    pack(c.ep.map(|piece| (piece >= 8) as u8))
}

/// Which slots hold an M-slice edge (UF UB DF DB, pieces 1 3 5 7).
fn m_slice_positions(c: &Cube) -> u128 {
    pack(c.ep.map(|piece| (piece < 8 && piece % 2 == 1) as u8))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::LazyLock;

    static SOLVER: LazyLock<Thistlethwaite> = LazyLock::new(Thistlethwaite::new);

    fn mix() -> impl Strategy<Value = Vec<Move>> {
        prop::collection::vec(prop::sample::select(ALL_MOVES.to_vec()), 0..80)
    }

    #[test]
    fn stage_sizes_and_worst_cases() {
        let sizes: Vec<_> = SOLVER.stages.iter().map(Stage::size).collect();
        assert_eq!(sizes, [2048, 2187 * 495, 70 * 420, 96 * 6912]);
        let worst: Vec<_> = SOLVER.stages.iter().map(Stage::max_distance).collect();
        assert_eq!(worst, [7, 10, 13, 15]);
    }

    #[test]
    fn half_turn_corner_group_has_96_elements() {
        assert_eq!(half_turn_corner_group().len(), 96);
    }

    #[test]
    fn superflip_needs_seven_moves_in_stage1() {
        let superflip = crate::parse("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2").unwrap();
        let cube = Cube::SOLVED.apply_all(&superflip);
        assert_eq!(SOLVER.stages[0].distance(&cube), Some(7));
    }

    #[test]
    fn solved_cube_needs_no_moves() {
        assert!(SOLVER.solve(&Cube::SOLVED).is_empty());
    }

    proptest! {
        #[test]
        fn each_stage_reaches_its_goal(mix in mix()) {
            let mut cube = Cube::SOLVED.apply_all(&mix);
            for (i, moves) in SOLVER.solve_stages(&cube).iter().enumerate() {
                let allowed = SOLVER.stages[i].moves();
                prop_assert!(moves.iter().all(|m| allowed.contains(m)));
                cube = cube.apply_all(moves);
                if i + 1 < 4 {
                    prop_assert_eq!(SOLVER.stages[i + 1].distance(&cube).is_some(), true);
                }
            }
            prop_assert!(cube.is_solved());
        }

        #[test]
        fn solution_solves_and_is_short(mix in mix()) {
            let cube = Cube::SOLVED.apply_all(&mix);
            let solution = SOLVER.solve(&cube);
            prop_assert!(cube.apply_all(&solution).is_solved());
            prop_assert!(solution.len() <= 45);
            prop_assert!(solution.windows(2).all(|w| w[0].face != w[1].face));
        }
    }
}
