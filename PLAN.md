# Rubik: Base Plan

A Rust program that solves a 3x3x3 Rubik's Cube from a mix, following the 42 `Rubik` subject (v5).
One Rust engine, used by a native CLI (graded) and by a 3D web interface through WebAssembly (bonus).

## Subject requirements

### Input / output
- One argument: a mix, e.g. `./rubik "F R U2 B' L' D'"`.
- Allowed moves: `F R U B L D`, each with an optional `'` (counter-clockwise) or `2` (half turn).
- Forbidden: slice moves `M E S`, rotations `x y z`, any alternative notation.
- Output: the solution on stdout, moves separated by spaces, on one line.
- Score is `./rubik "<mix>" | wc -w` (half-turn metric, HTM).

### Validation criteria
- Average of **50 moves or fewer** across mixes.
- **3 seconds max** per run.
- No crash, memory error, or infinite loop.
- Proper error handling: invalid move, empty or missing argument, extra arguments.
- Printing the inverse of the mix, or inserting moves that cancel out, is cheating.
- A `Makefile` with the usual rules (`all`, `clean`, `fclean`, `re`).
- Must be able to explain the algorithm in simple words and visuals.
- No library that does the solving. Libraries are allowed if justified at defense.

### Bonus (only evaluated if the mandatory part is perfect)
- Real graphical display of the cube, ideally animated moves (web 3D view, see below).
- Algorithm that goes below standard solutions in reasonable time.
- Choice between several algorithms, or best-of-several.
- Integrated mix generator (length and/or count).
- Solution split into human-understandable substeps.
- Other puzzles (2x2x2, 4x4x4, ...).
- Options must come before the mix and be prefixed with `-`.

## Architecture

```
rubik/
├── Makefile           # mandatory: builds ./rubik only, no Node needed
├── Cargo.toml         # workspace, release profile
├── Cargo.lock         # committed, builds use --locked
├── Dockerfile         # bonus: builds the web app, serves it with nginx
├── engine/            # cube model, parser, solvers. Zero dependencies.
├── cli/               # bin "rubik": parse args, solve, print
├── wasm/              # thin wasm-bindgen wrapper around engine
└── web/               # Vite + React + React Three Fiber
```

- **Do not name a crate `core`**: it clashes with Rust's built-in `core` crate. The library is `engine`.
- `engine` has zero normal dependencies, so it is easy to test and defend. Dev-dependencies (`proptest`) are fine.
- `wasm-bindgen` lives only in `wasm/`. Justification at defense: it is the JS bridge, it does no solving.
- `make` builds only the CLI (`cargo build --release --locked -p cli`). The web part never runs during grading.
- Release profile: `opt-level = 3`, `lto = true`, `codegen-units = 1`.

### Makefile
- `all`: builds and copies the binary to `./rubik`.
- The `rubik` target depends on the Rust sources, so a second `make` does nothing (no relink).
- `clean`: removes build artifacts. `fclean`: `cargo clean` and removes `./rubik`. `re`: `fclean` then `all`.
- `web` (bonus): `wasm-pack build wasm --release --target web`, then `npm run build` in `web/`.

### Error handling
- `Result` everywhere, never `unwrap()` on user input, no panics reachable from the CLI.
- Errors go to stderr with a clear message and a non-zero exit code.
- Rejected: empty argument, no argument, extra arguments, unknown moves (`X`, `F3`, `M`, `x`), lowercase (`f`), `F2'`.
- Whitespace: one or more spaces or tabs between moves are accepted; leading and trailing spaces are ignored.
- A mix that solves to nothing (`"F F'"`) prints an empty line and exits 0.

## Algorithm

### Choice: Thistlethwaite (mandatory part)
Solves in 4 stages. Each stage moves the cube into a smaller group, where fewer moves are allowed.

| Stage | From group | To group | Goal | Coordinate size |
|-------|------------|----------|------|-----------------|
| 0 -> 1 | `<U D F B L R>` | `<U D L R F2 B2>` | Orient all 12 edges | 2,048 |
| 1 -> 2 | `<U D L R F2 B2>` | `<U D L2 R2 F2 B2>` | Orient all corners, put the 4 E-slice edges in the E slice | 1,082,565 |
| 2 -> 3 | `<U D L2 R2 F2 B2>` | `<U2 D2 F2 B2 L2 R2>` | Put M- and S-slice edges in their slices, put corners in their tetrads with a permutation reachable by half turns only (96 cases), even edge parity | 29,400 |
| 3 -> 4 | `<U2 D2 F2 B2 L2 R2>` | solved | Solve with half turns only | 663,552 |

- Axes follow the cube model's orientation convention: edge orientation changes only on F and B quarter turns,
  corner orientation never changes on U and D turns. So F/B lose quarter turns first, then L/R. Stage 2's goal
  group is the same as Kociemba's phase 2 group, which the bonus solver can reuse.
- Stage 2 -> 3 is where implementations usually go wrong: "tetrads + parity" alone is not enough. The corner
  permutation must be one of the 96 reachable with half turns only, or stage 4 never finds a solution.
- Search, same for every stage (`engine/src/stage.rs`):
  - the stage looks at the cube through a few components (e.g. corner twist and E-slice positions), each
    depending only on which piece is in which slot, so each gets a move table built by BFS;
  - a BFS over the product of the components, from the goal, gives the **exact** distance of every state
    (sizes above, a few MB in total, built at startup);
  - solving = always play a move that lowers the distance by one. No IDA* needed, and each stage is optimal,
    so it never turns the same face twice in a row.
- Length: at most 45 moves when each stage is solved optimally (7 + 10 + 13 + 15), about 30 on average.
- **Merge moves across stage boundaries.** A stage can end with `R2` and the next start with `R2`. A final
  `optimize(&[Move])` pass merges same-face neighbours (`R R` -> `R2`, `R2 R2` -> nothing), else the output
  contains moves that cancel out (cheating) and scores worse.
- The 4 stages give the "solution by stages" bonus. They are group stages, not human methods,
  so name and explain each one in the output and in the web view.

### Later: Kociemba two-phase (bonus)
- About 20 to 25 moves.
- Phase 1 and phase 2 pruning tables are about 1M entries each and build in a few hundred ms in release.
  Build them at startup; no `build.rs` or `include_bytes!` unless measurements say otherwise.
- Adds the "choice between algorithms" bonus (`-a thistle|kociemba`).

## Web interface (bonus)

### WebAssembly
- The engine compiles to WASM and runs in the browser. No backend, no API, same code as the CLI.
- Speed: WASM is about 1.2 to 2 times slower than native, still much faster than JS. It does not matter:
  the 3 second limit applies only to the native CLI, and Thistlethwaite runs in milliseconds.
- Exposed functions (one call per user action, never one call per move):

```rust
#[wasm_bindgen] pub fn solve(mix: &str, algo: &str) -> Result<String, JsError>;
#[wasm_bindgen] pub fn facelets(moves: &str) -> Vec<u8>; // 54 sticker colors
#[wasm_bindgen] pub fn random_mix(len: u32, seed: u64) -> String;
```

- **All cube logic stays in Rust.** JS only draws and animates; sticker colors always come from `facelets()`.
- Kociemba runs in a Web Worker, so building its tables does not freeze the page.
- Tables are built in the background when the page loads, before the user clicks Solve.

### 3D front
- Stack: Vite + React + TypeScript, `@react-three/fiber`, `@react-three/drei` (`OrbitControls`, `RoundedBox`).
  No state library. Pin exact versions.
- Components:

```
<App>
├── <Controls>        mix input, Solve, algorithm select, random mix
├── <Playback>        play/pause, step back/forward, speed, current move highlighted
└── <Canvas>
    ├── <OrbitControls />   rotate, zoom, pan the view with the mouse
    ├── lights
    └── <Cube moves={queue} />   26 cubies, animates layer turns
```

- Animation: never React state per frame. Use refs inside `useFrame`:
  1. Start of a move: attach the 9 cubies of the layer to a `pivot` group (selected by position).
  2. During the move: rotate the pivot with easing.
  3. End of the move: reattach the cubies to the cube root and round positions and rotations to the grid
     (otherwise float errors pile up and the cube drifts).
  4. In development, compare displayed stickers with `facelets()` after each move.
- Scope: mouse view rotation, face turns with buttons and keyboard (`R`, `Shift+R` for `R'`),
  animated solution playback labelled by stage. Turning faces by dragging the mouse on the cube is last, only if time allows.

### Deployment
- The web build is static files (HTML, JS, `.wasm`). Multi-stage Dockerfile:
  1. `rust` image: `wasm-pack build` (prebuilt `wasm-pack` binary to avoid a slow `cargo install`).
  2. `node` image: `npm ci && npm run build`.
  3. `nginx:alpine`: copy `dist/`. Final image about 50 MB, nothing but nginx.
- Check that `.wasm` is served as `application/wasm` (`curl -I`).
- WASM does not load from `file://`. Without Docker: `python3 -m http.server -d web/dist`.
- Defense: the machine may have no Docker, Node or internet. Commit the built `web/dist` or use a deployed URL,
  and bundle three.js locally (no CDN).

## Milestones

1. **Cube model** (`engine`): corner and edge permutation and orientation, 6 faces x 3 modifiers = 18 moves, apply and invert moves.
2. **Parser**: validate and parse the mix, with clear errors.
3. **CLI + Makefile**: end to end from day one, error handling, no relink.
4. **Tests + CI**: see checklist. CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
5. **Stage 1 solver** (edge orientation).
6. **Stage 2 solver**.
7. **Stage 3 solver**.
8. **Stage 4 solver**, plus the move-merging pass.
9. **Benchmark**: random-state generator, thousands of states, check average moves (<= 50) and max wall time (< 3 s).
10. **Mandatory part frozen.** Nothing below starts before this is perfect.
11. **CLI bonuses**: options (`-a`, `-s` for stages, mix generator), Kociemba.
12. **WASM wrapper**.
13. **Web front**: static cube from `facelets()`, then animation, then playback, then controls.
14. **Dockerfile** and deployment.

## Testing checklist

- [ ] Valid mix returns a solution that actually solves the cube (verify by applying it).
- [ ] Empty argument, no argument, extra arguments, bad moves (`X`, `F3`, `M`, `x`, `f`, `F2'`) all return a clean error on stderr with a non-zero exit code.
- [ ] Each move applied 4 times is identity; `X X'` is identity; `(R U)` x105 is identity; `R U R' U'` x6 is identity.
- [ ] Cubie model and coordinate move tables agree for every move and coordinate.
- [ ] No two adjacent moves in the output turn the same face.
- [ ] Solution is not the inverse of the mix.
- [ ] Benchmark uses uniformly random states (random permutation and orientation, respecting parity and orientation sums), not random move sequences.
- [ ] Average over 1000+ random states is <= 50.
- [ ] Wall time of the whole process (`time ./rubik ...`), table building included, stays under 3 s, measured on the slowest available machine.
- [ ] Very long mixes (1000+ moves) work.
- [ ] No panics: `proptest` on the parser with arbitrary strings.
- [ ] Web: displayed stickers match `facelets()` after every animated move.

## Risks

- 42 machines: check that rustup is available. Docker, Node or internet may be missing at defense.
- Ask past evaluators whether a web 3D view counts as "real graphical display" before investing in visuals.

## Git conventions

- Branches: `feat/...`, `fix/...`, `test/...`, `docs/...`, `refactor/...`
- Commits: `feat: ...`, `fix: ...`, `test: ...`, `docs: ...`, `refactor: ...`
