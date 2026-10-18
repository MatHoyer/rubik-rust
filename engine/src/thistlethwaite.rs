//! Thistlethwaite's algorithm: four stages, each restricting the allowed moves further.
//!
//! | Stage  | Allowed moves              | Goal                                   |
//! |--------|----------------------------|----------------------------------------|
//! | 0 -> 1 | all 18                     | orient all edges                       |
//! | 1 -> 2 | U D L R, F2 B2             | (next)                                 |
//! | 2 -> 3 | U D, L2 R2 F2 B2           | (next)                                 |
//! | 3 -> 4 | U2 D2 L2 R2 F2 B2          | (next)                                 |

use crate::moves::ALL_MOVES;
use crate::stage::Stage;
use crate::Cube;

/// Stage 1: flip every edge to "good". Edge orientation here only changes on F and B
/// quarter turns, so once it is fixed the cube can be solved without them.
pub fn stage1() -> Stage {
    Stage::new(ALL_MOVES.to_vec(), vec![Box::new(edge_orientation)])
}

fn edge_orientation(c: &Cube) -> u128 {
    c.eo.iter().fold(0, |acc, &o| acc << 1 | o as u128)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::Move;
    use proptest::prelude::*;
    use std::sync::LazyLock;

    static STAGE1: LazyLock<Stage> = LazyLock::new(stage1);

    fn mix() -> impl Strategy<Value = Vec<Move>> {
        prop::collection::vec(prop::sample::select(ALL_MOVES.to_vec()), 0..60)
    }

    #[test]
    fn stage1_covers_all_edge_orientations() {
        assert_eq!(STAGE1.size(), 2048);
    }

    #[test]
    fn superflip_needs_seven_moves() {
        let superflip = crate::parse("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2").unwrap();
        let cube = Cube::SOLVED.apply_all(&superflip);
        assert_eq!(STAGE1.distance(&cube), Some(7));
    }

    proptest! {
        #[test]
        fn stage1_orients_all_edges(mix in mix()) {
            let cube = Cube::SOLVED.apply_all(&mix);
            let solution = STAGE1.solve(&cube).unwrap();
            prop_assert!(solution.len() <= 7);
            prop_assert_eq!(cube.apply_all(&solution).eo, [0; 12]);
        }
    }
}
