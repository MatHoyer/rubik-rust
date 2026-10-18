//! Uniformly random cube states, with a tiny seeded RNG (no dependency).

use crate::moves::{Move, ALL_MOVES};
use crate::Cube;

/// SplitMix64: small, fast, good enough for scrambles. Not for cryptography.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n`.
    pub fn below(&mut self, n: u64) -> u64 {
        // Rejection sampling avoids modulo bias.
        let limit = u64::MAX - u64::MAX % n;
        loop {
            let x = self.next_u64();
            if x < limit {
                return x % n;
            }
        }
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.below(i as u64 + 1) as usize);
        }
    }
}

/// A uniformly random solvable cube: random permutations and orientations,
/// keeping the three rules every real cube follows (equal permutation parities,
/// corner twist sum divisible by 3, edge flip sum even).
pub fn random_cube(rng: &mut Rng) -> Cube {
    let mut c = Cube::SOLVED;
    rng.shuffle(&mut c.cp);
    rng.shuffle(&mut c.ep);
    if parity(&c.cp) != parity(&c.ep) {
        c.ep.swap(0, 1);
    }
    for i in 0..7 {
        c.co[i] = rng.below(3) as u8;
        c.eo[i] = rng.below(2) as u8;
    }
    for i in 7..11 {
        c.eo[i] = rng.below(2) as u8;
    }
    c.co[7] = (3 - c.co[..7].iter().sum::<u8>() % 3) % 3;
    c.eo[11] = c.eo[..11].iter().sum::<u8>() % 2;
    c
}

/// A random mix of `len` moves in which no move can be merged with or commutes past its
/// neighbour (no `R R`, no `R L R`; opposite faces always in the same order).
pub fn random_moves(rng: &mut Rng, len: usize) -> Vec<Move> {
    let mut mix: Vec<Move> = Vec::with_capacity(len);
    while mix.len() < len {
        let m = ALL_MOVES[rng.below(18) as usize];
        if m.can_follow(mix.last()) {
            mix.push(m);
        }
    }
    mix
}

/// `true` if the permutation is odd.
fn parity(perm: &[u8]) -> bool {
    let mut odd = false;
    for i in 0..perm.len() {
        for j in i + 1..perm.len() {
            odd ^= perm[i] > perm[j];
        }
    }
    odd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_cube() {
        assert_eq!(random_cube(&mut Rng::new(7)), random_cube(&mut Rng::new(7)));
        assert_ne!(random_cube(&mut Rng::new(7)), random_cube(&mut Rng::new(8)));
    }

    #[test]
    fn random_cubes_follow_cube_rules() {
        let mut rng = Rng::new(1);
        for _ in 0..1000 {
            let c = random_cube(&mut rng);
            assert_eq!(parity(&c.cp), parity(&c.ep));
            assert_eq!(c.co.iter().sum::<u8>() % 3, 0);
            assert_eq!(c.eo.iter().sum::<u8>() % 2, 0);
            assert!(c.co.iter().all(|&o| o < 3) && c.eo.iter().all(|&o| o < 2));
        }
    }

    #[test]
    fn random_moves_never_merge() {
        let mut rng = Rng::new(9);
        for len in [0, 1, 2, 25, 200] {
            let mix = random_moves(&mut rng, len);
            assert_eq!(mix.len(), len);
            assert_eq!(crate::merge(&mix), mix);
        }
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::new(3);
        assert!((0..10_000).all(|_| rng.below(7) < 7));
    }
}
