//! Cubie-level model: where each corner and edge piece is, and how it is twisted.
//!
//! Corner slots: URF UFL ULB UBR DFR DLF DBL DRB (0..8).
//! Edge slots:   UR UF UL UB DR DF DL DB FR FL BL BR (0..12).
//! `cp[i]` is the piece sitting in slot `i`; `co[i]` its twist (0..3), `eo[i]` its flip (0..2).

use crate::moves::Move;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cube {
    pub cp: [u8; 8],
    pub co: [u8; 8],
    pub ep: [u8; 12],
    pub eo: [u8; 12],
}

impl Cube {
    pub const SOLVED: Cube = Cube {
        cp: [0, 1, 2, 3, 4, 5, 6, 7],
        co: [0; 8],
        ep: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        eo: [0; 12],
    };

    pub fn is_solved(&self) -> bool {
        *self == Self::SOLVED
    }

    /// `self` followed by `other`.
    pub fn multiply(&self, other: &Cube) -> Cube {
        let mut r = Cube::SOLVED;
        for i in 0..8 {
            let from = other.cp[i] as usize;
            r.cp[i] = self.cp[from];
            r.co[i] = (self.co[from] + other.co[i]) % 3;
        }
        for i in 0..12 {
            let from = other.ep[i] as usize;
            r.ep[i] = self.ep[from];
            r.eo[i] = (self.eo[from] + other.eo[i]) % 2;
        }
        r
    }

    pub fn apply(&self, m: Move) -> Cube {
        let quarter = &FACE_TURNS[m.face as usize];
        let mut c = *self;
        for _ in 0..m.turns {
            c = c.multiply(quarter);
        }
        c
    }

    pub fn apply_all(&self, moves: &[Move]) -> Cube {
        moves.iter().fold(*self, |c, &m| c.apply(m))
    }
}

/// One clockwise quarter turn of each face, indexed by `Face`.
const FACE_TURNS: [Cube; 6] = [
    // U
    Cube {
        cp: [3, 0, 1, 2, 4, 5, 6, 7],
        co: [0; 8],
        ep: [3, 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11],
        eo: [0; 12],
    },
    // R
    Cube {
        cp: [4, 1, 2, 0, 7, 5, 6, 3],
        co: [2, 0, 0, 1, 1, 0, 0, 2],
        ep: [8, 1, 2, 3, 11, 5, 6, 7, 4, 9, 10, 0],
        eo: [0; 12],
    },
    // F
    Cube {
        cp: [1, 5, 2, 3, 0, 4, 6, 7],
        co: [1, 2, 0, 0, 2, 1, 0, 0],
        ep: [0, 9, 2, 3, 4, 8, 6, 7, 1, 5, 10, 11],
        eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
    },
    // D
    Cube {
        cp: [0, 1, 2, 3, 5, 6, 7, 4],
        co: [0; 8],
        ep: [0, 1, 2, 3, 5, 6, 7, 4, 8, 9, 10, 11],
        eo: [0; 12],
    },
    // L
    Cube {
        cp: [0, 2, 6, 3, 4, 1, 5, 7],
        co: [0, 1, 2, 0, 0, 2, 1, 0],
        ep: [0, 1, 10, 3, 4, 5, 9, 7, 8, 2, 6, 11],
        eo: [0; 12],
    },
    // B
    Cube {
        cp: [0, 1, 3, 7, 4, 5, 2, 6],
        co: [0, 0, 1, 2, 0, 0, 2, 1],
        ep: [0, 1, 2, 11, 4, 5, 6, 10, 8, 9, 3, 7],
        eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::{inverse, parse, Face, ALL_MOVES};

    fn run(s: &str) -> Cube {
        Cube::SOLVED.apply_all(&parse(s).unwrap())
    }

    fn repeat(s: &str, n: usize) -> Cube {
        let seq = parse(s).unwrap();
        (0..n).fold(Cube::SOLVED, |c, _| c.apply_all(&seq))
    }

    #[test]
    fn every_quarter_turn_four_times_is_identity() {
        for f in Face::ALL {
            let m = Move { face: f, turns: 1 };
            assert!(Cube::SOLVED.apply_all(&[m; 4]).is_solved(), "{f:?}");
            assert!(!Cube::SOLVED.apply(m).is_solved(), "{f:?}");
        }
    }

    #[test]
    fn move_then_inverse_is_identity() {
        for m in ALL_MOVES {
            assert!(Cube::SOLVED.apply(m).apply(m.inverse()).is_solved(), "{m}");
        }
    }

    #[test]
    fn known_orders() {
        assert!(repeat("R U", 105).is_solved());
        assert!(!repeat("R U", 35).is_solved());
        assert!(repeat("R U R' U'", 6).is_solved());
        assert!(!repeat("R U R' U'", 3).is_solved());
    }

    #[test]
    fn sequence_inverse_undoes_it() {
        let mix = parse("F R U2 B' L' D' R2 F' U L2 D B2").unwrap();
        let c = Cube::SOLVED.apply_all(&mix).apply_all(&inverse(&mix));
        assert!(c.is_solved());
    }

    #[test]
    fn orientation_sums_stay_valid() {
        let c = run("F R U2 B' L' D' R2 F' U L2 D B2 R' F");
        assert_eq!(c.co.iter().map(|&x| x as u32).sum::<u32>() % 3, 0);
        assert_eq!(c.eo.iter().map(|&x| x as u32).sum::<u32>() % 2, 0);
    }

    #[test]
    fn superflip_flips_every_edge_in_place() {
        let c = run("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2");
        assert_eq!(c.cp, Cube::SOLVED.cp);
        assert_eq!(c.co, Cube::SOLVED.co);
        assert_eq!(c.ep, Cube::SOLVED.ep);
        assert_eq!(c.eo, [1; 12]);
    }
}
