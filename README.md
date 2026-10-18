# rubik

Solves a 3x3x3 Rubik's Cube from a mix, in Rust (42 `Rubik` subject).

```
$ ./rubik "F R U2 B' L' D'"
B R' D' L' B' U' F2 D L D' L U2 L2 U F2 U2 L2 U L2 D F2 U2 D2 R2 F2 U2 L2 F2 U2 L2
```

## Build and run

```bash
make            # builds ./rubik (needs a Rust toolchain)
./rubik "F R U2 B' L' D'"
make test       # unit, property and CLI tests
make bench      # 10,000 random cubes + 100 runs of ./rubik, fails if a limit is broken
make fclean     # removes build files and ./rubik (also: clean, re)
```

- Moves: `F R U B L D`, each optionally followed by `'` (counter-clockwise) or `2` (half turn).
- Moves are separated by spaces or tabs. The whole mix is one argument, in quotes.
- The solution is printed on one line. A mix that leaves the cube solved (`"F F'"`) prints an empty line.
- Any error (no argument, empty mix, extra arguments, unknown move like `F3`, `M`, `x`, `f`, `F2'`)
  prints a message on stderr and exits with code 1.

## Results

`make bench`, 10,000 uniformly random cubes:

| | |
|---|---|
| Average solution | **30.9 moves** (limit 50) |
| Shortest / longest | 20 / 39 moves |
| Worst `./rubik` run | **0.19 s** (limit 3 s), table building included |
| Memory | about 50 MB |

## How it works

### The cube in memory

The cube is not stored as 54 stickers but as its **20 moving pieces**: 8 corners and 12 edges.
For each slot we store which piece is there and how it is turned:

- corners: which piece (0-7) and its twist (0, 1 or 2)
- edges: which piece (0-11) and its flip (0 or 1)

A move is a fixed rearrangement of these slots, so applying a move is just copying values around.

### Thistlethwaite's algorithm, in simple words

Solving a random cube in one search is hopeless: there are 43 quintillion positions.
Instead, we solve it in **4 stages**. After each stage, the cube is "tidy" enough that some moves
are never needed again, so the next stage works with fewer moves and far fewer positions.

```
 all 18 moves allowed            stage 1: fix edge flips
          │                      (F, B now only as half turns)
          ▼
 U D L R  F2 B2                  stage 2: fix corner twists, put the 4 middle-layer edges
          │                      in the middle layer   (L, R now only as half turns)
          ▼
 U D  L2 R2 F2 B2                stage 3: put every other edge in its slice, put corners
          │                      in an order half turns can finish   (U, D only half turns)
          ▼
 U2 D2 L2 R2 F2 B2               stage 4: solve with half turns only
          │
          ▼
       solved
```

| Stage | Moves allowed | Goal | Positions | Average / max moves |
|-------|---------------|------|-----------|---------------------|
| 1 | all 18 | every edge flipped correctly | 2,048 | 4.6 / 7 |
| 2 | U D L R, F2 B2 | every corner twisted correctly, middle-layer edges in the middle layer | 1,082,565 | 7.8 / 10 |
| 3 | U D, L2 R2 F2 B2 | other edges in their slices, corners in one of the 96 orders half turns can solve | 29,400 | 8.8 / 13 |
| 4 | half turns only | solved | 663,552 | 10.1 / 15 |

Why stage 1 means "no more F and B quarter turns": in this cube model, only a quarter turn of F or B
flips edges. Once every edge is correct, using F/B quarter turns would only break them again.
The same idea applies to corner twists (only L and R quarter turns change them in stage 2) and so on.

### How each stage finds its moves

Each stage only looks at what matters for its goal (for stage 1: which edges are flipped,
ignoring everything else). That gives a small number of positions, from 2,048 to about a million.

1. **Map out every position** of the stage, starting from the goal and going outward one move at
   a time (breadth-first search). Each position gets its exact distance to the goal.
2. **Solve** by always playing a move that brings the distance down by one, until it reaches 0.

```
 distance:   3 ──move──▶ 2 ──move──▶ 1 ──move──▶ 0  (goal of the stage)
```

Each stage's answer is therefore the shortest possible for that stage. Building all four maps
takes about 0.18 s when the program starts; solving afterwards takes microseconds.

### Cleaning up the result

When stage 1 ends with `R` and stage 2 starts with `R`, they are merged into `R2`
(and `R R'` is removed). This also works across a move of the opposite face, which does not
interfere: `U D U` becomes `U2 D`. The output therefore never contains moves that cancel out.

## Project layout

```
engine/   cube model, parser, solver (no dependencies)
  src/cube.rs            pieces, moves
  src/moves.rs           notation, parser, merging moves
  src/stage.rs           generic stage: position map + solving by distance
  src/thistlethwaite.rs  the 4 stages
  src/random.rs          uniformly random cubes for tests and the benchmark
  examples/bench.rs      the benchmark behind `make bench`
cli/      the ./rubik binary: arguments, errors, output
PLAN.md   roadmap and design decisions
```

`engine` has no dependencies. `proptest` is used only by tests (random inputs for the parser and
the solver); no library does any solving.

## Testing

- Cube model: every move 4 times is identity, `(R U)` 105 times is identity, the superflip flips
  every edge and nothing else.
- Parser: valid and invalid moves, and random strings never crash it.
- Solver: position counts and worst cases of every stage match the known values
  (2,048 / 1,082,565 / 29,400 / 663,552 positions, 7 / 10 / 13 / 15 moves); for random cubes,
  each stage only uses its allowed moves and the solution really solves the cube.
- CLI: every error case, a cancelling mix, a 3,000-move mix, and the printed solution is
  applied back to the cube.

## Defense notes

- **Is the solution just the inverse of the mix?** No: the solver only sees the final cube, not the
  mix. For a very short mix the shortest answer can be the inverse (`R` → `R'`); there is no other choice.
- **Why does a 6-move mix give a 30-move solution?** Each stage is optimal for its own goal, but the
  4 stages together are not the shortest overall solution. That is the price of being fast and simple.
- **Limits:** Linux caps one command-line argument at 128 KB (about 45,000 moves).
