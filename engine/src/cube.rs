//! Cubie-level model: where each corner and edge piece is, and how it is twisted.
//!
//! Corner slots: URF UFL ULB UBR DFR DLF DBL DRB (0..8).
//! Edge slots:   UR UF UL UB DR DF DL DB FR FL BL BR (0..12).
//! `cp[i]` is the piece sitting in slot `i`; `co[i]` its twist (0..3), `eo[i]` its flip (0..2).

use std::fmt;

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

// Sticker layout: faces in order U R F D L B, 9 stickers each, row by row as seen from
// outside with U on top of F (U and D: F side at the bottom / top). Index = face * 9 + n.
// Corner slot i has stickers CORNER_FACELETS[i]; piece j has colors CORNER_COLORS[j],
// listed clockwise starting from its U/D sticker. Same for edges, starting from the
// sticker that defines edge orientation.
const CORNER_FACELETS: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];
const CORNER_COLORS: [[u8; 3]; 8] = [
    [0, 1, 2],
    [0, 2, 4],
    [0, 4, 5],
    [0, 5, 1],
    [3, 2, 1],
    [3, 4, 2],
    [3, 5, 4],
    [3, 1, 5],
];
const EDGE_FACELETS: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];
const EDGE_COLORS: [[u8; 2]; 12] = [
    [0, 1],
    [0, 2],
    [0, 4],
    [0, 5],
    [3, 1],
    [3, 2],
    [3, 4],
    [3, 5],
    [2, 1],
    [2, 4],
    [5, 4],
    [5, 1],
];

impl Cube {
    /// The 54 sticker colors (0..6 = color of the U R F D L B center), for display.
    pub fn facelets(&self) -> [u8; 54] {
        let mut f = [0; 54];
        for (face, center) in f.chunks_mut(9).enumerate() {
            center[4] = face as u8;
        }
        for i in 0..8 {
            let (piece, twist) = (self.cp[i] as usize, self.co[i] as usize);
            for n in 0..3 {
                f[CORNER_FACELETS[i][(n + twist) % 3]] = CORNER_COLORS[piece][n];
            }
        }
        for i in 0..12 {
            let (piece, flip) = (self.ep[i] as usize, self.eo[i] as usize);
            for n in 0..2 {
                f[EDGE_FACELETS[i][(n + flip) % 2]] = EDGE_COLORS[piece][n];
            }
        }
        f
    }
}

/// Why 54 sticker colours cannot be a real cube.
#[derive(Debug, PartialEq, Eq)]
pub enum FaceletError {
    /// A sticker has no colour (any value above 5).
    Unpainted,
    /// Centers must be U R F D L B in order (white on top, green in front).
    WrongCenters,
    /// Every colour must appear 9 times.
    ColorCount {
        color: u8,
        count: usize,
    },
    /// No piece has this combination of colours (or a piece appears twice).
    BadCorner {
        slot: usize,
    },
    BadEdge {
        slot: usize,
    },
    /// Real cube rules: impossible without taking the cube apart.
    TwistedCorner,
    FlippedEdge,
    SwappedPieces,
}

impl fmt::Display for FaceletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const NAMES: [&str; 6] = ["white", "red", "green", "yellow", "orange", "blue"];
        const CORNERS: [&str; 8] = ["UFR", "UFL", "UBL", "UBR", "DFR", "DFL", "DBL", "DBR"];
        const EDGES: [&str; 12] = [
            "UR", "UF", "UL", "UB", "DR", "DF", "DL", "DB", "FR", "FL", "BL", "BR",
        ];
        match self {
            FaceletError::Unpainted => write!(f, "some stickers have no colour yet"),
            FaceletError::WrongCenters => {
                write!(f, "centers must stay white on top, green in front")
            }
            FaceletError::ColorCount { color, count } => write!(
                f,
                "{} appears {count} times, it must appear 9 times",
                NAMES[*color as usize]
            ),
            FaceletError::BadCorner { slot } => write!(
                f,
                "the {} corner has colours no real corner has (or a corner is there twice)",
                CORNERS[*slot]
            ),
            FaceletError::BadEdge { slot } => write!(
                f,
                "the {} edge has colours no real edge has (or an edge is there twice)",
                EDGES[*slot]
            ),
            FaceletError::TwistedCorner => {
                write!(f, "one corner is twisted: this cube cannot be solved")
            }
            FaceletError::FlippedEdge => {
                write!(f, "one edge is flipped: this cube cannot be solved")
            }
            FaceletError::SwappedPieces => {
                write!(f, "two pieces are swapped: this cube cannot be solved")
            }
        }
    }
}

impl std::error::Error for FaceletError {}

impl Cube {
    /// The cube showing these 54 sticker colours (same layout as [`Cube::facelets`]),
    /// or why no real cube can look like this.
    pub fn from_facelets(f: &[u8; 54]) -> Result<Cube, FaceletError> {
        if f.iter().any(|&c| c > 5) {
            return Err(FaceletError::Unpainted);
        }
        if (0..6).any(|face| f[face * 9 + 4] != face as u8) {
            return Err(FaceletError::WrongCenters);
        }
        for color in 0..6 {
            let count = f.iter().filter(|&&c| c == color).count();
            if count != 9 {
                return Err(FaceletError::ColorCount { color, count });
            }
        }
        let mut c = Cube::SOLVED;
        for (slot, stickers) in CORNER_FACELETS.iter().enumerate() {
            // Twist = which of the 3 stickers shows the U/D colour.
            let twist = (0..3)
                .find(|&t| matches!(f[stickers[t]], 0 | 3))
                .ok_or(FaceletError::BadCorner { slot })?;
            let colors = [0, 1, 2].map(|n| f[stickers[(n + twist) % 3]]);
            let piece = CORNER_COLORS
                .iter()
                .position(|&p| p == colors)
                .ok_or(FaceletError::BadCorner { slot })?;
            c.cp[slot] = piece as u8;
            c.co[slot] = twist as u8;
        }
        for (slot, stickers) in EDGE_FACELETS.iter().enumerate() {
            let colors = stickers.map(|i| f[i]);
            let (piece, flip) = EDGE_COLORS
                .iter()
                .enumerate()
                .find_map(|(j, &p)| {
                    (p == colors)
                        .then_some((j, 0))
                        .or((p == [colors[1], colors[0]]).then_some((j, 1)))
                })
                .ok_or(FaceletError::BadEdge { slot })?;
            c.ep[slot] = piece as u8;
            c.eo[slot] = flip;
        }
        if let Some(slot) = first_duplicate(&c.cp) {
            return Err(FaceletError::BadCorner { slot });
        }
        if let Some(slot) = first_duplicate(&c.ep) {
            return Err(FaceletError::BadEdge { slot });
        }
        if c.co.iter().map(|&x| x as u32).sum::<u32>() % 3 != 0 {
            return Err(FaceletError::TwistedCorner);
        }
        if c.eo.iter().map(|&x| x as u32).sum::<u32>() % 2 != 0 {
            return Err(FaceletError::FlippedEdge);
        }
        if parity(&c.cp) != parity(&c.ep) {
            return Err(FaceletError::SwappedPieces);
        }
        Ok(c)
    }
}

/// Slot of the second occurrence of a piece, if any piece appears twice.
fn first_duplicate(pieces: &[u8]) -> Option<usize> {
    (1..pieces.len()).find(|&i| pieces[..i].contains(&pieces[i]))
}

/// `true` if the permutation is odd.
pub(crate) fn parity(perm: &[u8]) -> bool {
    let mut odd = false;
    for i in 0..perm.len() {
        for j in i + 1..perm.len() {
            odd ^= perm[i] > perm[j];
        }
    }
    odd
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
    fn solved_facelets_are_one_color_per_face() {
        let f = Cube::SOLVED.facelets();
        for (face, stickers) in f.chunks(9).enumerate() {
            assert!(stickers.iter().all(|&c| c == face as u8), "face {face}");
        }
    }

    #[test]
    fn r_turn_moves_front_stickers_up() {
        let f = run("R").facelets();
        let (u, f_, d, b) = (0, 2, 3, 5);
        assert_eq!([f[2], f[5], f[8]], [f_; 3]); // U right column comes from F
        assert_eq!([f[20], f[23], f[26]], [d; 3]); // F right column comes from D
        assert_eq!([f[45], f[48], f[51]], [u; 3]); // B left column (seen from behind) from U
        assert_eq!([f[29], f[32], f[35]], [b; 3]); // D right column comes from B
        assert!(f[9..18].iter().all(|&c| c == 1)); // R face keeps its color
    }

    #[test]
    fn facelets_always_have_nine_of_each_color() {
        let c = run("F R U2 B' L' D' R2 F' U L2 D B2 R' F");
        let f = c.facelets();
        for color in 0..6 {
            assert_eq!(f.iter().filter(|&&x| x == color).count(), 9);
            assert_eq!(f[color as usize * 9 + 4], color);
        }
        // Each corner shows three different colors (catches wrong facelet tables).
        for corner in CORNER_FACELETS {
            let colors = corner.map(|i| f[i]);
            assert!(colors[0] != colors[1] && colors[1] != colors[2] && colors[0] != colors[2]);
        }
    }

    #[test]
    fn from_facelets_reads_back_any_cube() {
        let mut rng = crate::random::Rng::new(11);
        for _ in 0..1000 {
            let c = crate::random::random_cube(&mut rng);
            assert_eq!(Cube::from_facelets(&c.facelets()), Ok(c));
        }
    }

    #[test]
    fn from_facelets_explains_impossible_cubes() {
        let solved = Cube::SOLVED.facelets();
        let with = |changes: &[(usize, u8)]| {
            let mut f = solved;
            for &(i, c) in changes {
                f[i] = c;
            }
            Cube::from_facelets(&f)
        };
        assert_eq!(with(&[(0, 6)]), Err(FaceletError::Unpainted));
        assert_eq!(with(&[(4, 1), (13, 0)]), Err(FaceletError::WrongCenters));
        assert_eq!(
            with(&[(0, 1)]),
            Err(FaceletError::ColorCount { color: 0, count: 8 })
        );
        // URF corner turned in place: U R F stickers -> R F U.
        assert_eq!(
            with(&[(8, 1), (9, 2), (20, 0)]),
            Err(FaceletError::TwistedCorner)
        );
        // UF edge flipped in place.
        assert_eq!(with(&[(7, 2), (19, 0)]), Err(FaceletError::FlippedEdge));
        // UR and UF edges swapped (stickers U6 R2 <-> U8 F2): odd permutation.
        assert_eq!(with(&[(10, 2), (19, 1)]), Err(FaceletError::SwappedPieces));
        // Two stickers of different corners swapped: one corner has impossible colours.
        assert!(matches!(
            with(&[(9, 4), (38, 1)]),
            Err(FaceletError::BadCorner { .. })
        ));
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
