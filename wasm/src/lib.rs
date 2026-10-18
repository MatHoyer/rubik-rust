//! Browser bindings: a thin layer over `engine`. No cube logic or solving here.
//!
//! One call per user action (solve a mix, get the stickers of a position), never one per
//! move: crossing between JS and WASM is the expensive part.

use engine::solver::{self, Algo};
use engine::{format, Cube};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Solution {
    labels: Vec<String>,
    steps: Vec<String>,
    moves: String,
}

#[wasm_bindgen]
impl Solution {
    /// What each step does, e.g. "Stage 1 - orient all edges".
    #[wasm_bindgen(getter)]
    pub fn labels(&self) -> Vec<String> {
        self.labels.clone()
    }

    /// Moves of each step (same order as `labels`), e.g. "R U2 F'". May be empty.
    #[wasm_bindgen(getter)]
    pub fn steps(&self) -> Vec<String> {
        self.steps.clone()
    }

    /// The full solution, moves merged across steps.
    #[wasm_bindgen(getter)]
    pub fn moves(&self) -> String {
        self.moves.clone()
    }
}

/// Solves the cube reached by `mix` with `algo` (`thistlethwaite`, `kociemba` or `best`).
/// An empty mix is the solved cube.
#[wasm_bindgen]
pub fn solve(mix: &str, algo: &str) -> Result<Solution, JsError> {
    let cube = cube_after(mix)?;
    let algo: Algo = algo.parse().map_err(|e: String| JsError::new(&e))?;
    let s = solver::solve(&cube, algo, &js_sys::Date::now);
    Ok(Solution {
        labels: s.steps.iter().map(|step| step.label.clone()).collect(),
        steps: s.steps.iter().map(|step| format(&step.moves)).collect(),
        moves: format(&s.moves),
    })
}

/// The 54 sticker colors after `moves`, faces in order U R F D L B, 9 stickers each, row by
/// row. Color `n` is the color of face `n`'s center.
#[wasm_bindgen]
pub fn facelets(moves: &str) -> Result<Vec<u8>, JsError> {
    Ok(cube_after(moves)?.facelets().to_vec())
}

/// A mix that turns the solved cube into the cube showing these 54 colours (same layout as
/// `facelets`), so a real cube can be entered sticker by sticker. Errors say why the colours
/// cannot be a real cube ("one corner is twisted", ...).
#[wasm_bindgen]
pub fn mix_for(colors: &[u8]) -> Result<String, JsError> {
    let colors: &[u8; 54] = colors
        .try_into()
        .map_err(|_| JsError::new("expected 54 sticker colours"))?;
    let cube = Cube::from_facelets(colors).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(format(&engine::inverse(
        &solver::thistlethwaite().solve(&cube),
    )))
}

/// A random mix of `length` moves where no two neighbours merge or cancel.
#[wasm_bindgen]
pub fn random_mix(length: u32, seed: u32) -> String {
    let mut rng = engine::random::Rng::new(seed.into());
    format(&engine::random::random_moves(&mut rng, length as usize))
}

/// Builds the tables of `algo` now, so the first solve is instant. Call it right after
/// the page loads (Kociemba's tables take a few hundred milliseconds).
#[wasm_bindgen]
pub fn prepare(algo: &str) -> Result<(), JsError> {
    match algo.parse().map_err(|e: String| JsError::new(&e))? {
        Algo::Thistlethwaite => {
            solver::thistlethwaite();
        }
        Algo::Kociemba => {
            solver::kociemba();
        }
        Algo::Best => {
            solver::thistlethwaite();
            solver::kociemba();
        }
    }
    Ok(())
}

fn cube_after(mix: &str) -> Result<Cube, JsError> {
    if mix.trim().is_empty() {
        return Ok(Cube::SOLVED);
    }
    let moves = engine::parse(mix).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(Cube::SOLVED.apply_all(&moves))
}

// Run with `wasm-pack test --node wasm`: needs a JS runtime (Date.now, JsError).
#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    fn solves(mix: &str, solution: &str) -> bool {
        Cube::SOLVED
            .apply_all(&engine::parse(&format!("{mix} {solution}")).unwrap())
            .is_solved()
    }

    #[wasm_bindgen_test]
    fn solves_with_every_algorithm() {
        let mix = "F R U2 B' L' D' R2 F' U L2 D B2";
        for (algo, steps) in [("thistlethwaite", 4), ("kociemba", 2), ("best", 2)] {
            let s = solve(mix, algo).unwrap();
            assert_eq!(s.labels().len(), steps);
            assert_eq!(s.steps().len(), steps);
            assert!(solves(mix, &s.moves()));
        }
    }

    #[wasm_bindgen_test]
    fn kociemba_stays_short_in_the_browser() {
        let mix = random_mix(40, 1);
        let s = solve(&mix, "kociemba").unwrap();
        assert!(s.moves().split_whitespace().count() <= 22);
        assert!(solves(&mix, &s.moves()));
    }

    #[wasm_bindgen_test]
    fn rejects_bad_input() {
        assert!(solve("F3", "kociemba").is_err());
        assert!(solve("R", "fast").is_err());
        assert!(facelets("M").is_err());
        assert!(prepare("fast").is_err());
    }

    #[wasm_bindgen_test]
    fn facelets_match_engine() {
        assert_eq!(facelets("").unwrap(), Cube::SOLVED.facelets().to_vec());
        let r = facelets("R U").unwrap();
        assert_eq!(r.len(), 54);
        assert_eq!(
            r,
            Cube::SOLVED
                .apply_all(&engine::parse("R U").unwrap())
                .facelets()
        );
    }

    #[wasm_bindgen_test]
    fn mix_for_reproduces_painted_colours() {
        let colors = facelets("F R U2 B' L' D' R2").unwrap();
        let mix = mix_for(&colors).unwrap();
        assert_eq!(facelets(&mix).unwrap(), colors);
        assert_eq!(mix_for(&facelets("").unwrap()).unwrap(), "");
        assert!(mix_for(&colors[..53]).is_err());
        let mut twisted = facelets("").unwrap();
        (twisted[8], twisted[9], twisted[20]) = (1, 2, 0);
        assert!(mix_for(&twisted).is_err());
    }

    #[wasm_bindgen_test]
    fn random_mix_is_seeded() {
        assert_eq!(random_mix(25, 3), random_mix(25, 3));
        assert_eq!(engine::parse(&random_mix(25, 3)).unwrap().len(), 25);
    }
}
