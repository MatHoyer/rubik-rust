import { readFileSync } from "node:fs";
import { Vector3 } from "three";
import { beforeAll, describe, expect, test } from "vitest";
import init, { facelets } from "rubik-wasm";
import {
  CUBIES,
  FACES,
  NORMALS,
  STICKERS,
  inLayer,
  inverseMove,
  isSolved,
  parseMove,
  turnAngle,
  type Vec3,
} from "./geometry";

beforeAll(async () => {
  const wasm = readFileSync(new URL("../../wasm/pkg/rubik_wasm_bg.wasm", import.meta.url));
  await init({ module_or_path: wasm });
});

const key = (v: Vec3) => v.join(",");

function rotate(v: Vec3, axis: Vec3, angle: number): Vec3 {
  const r = new Vector3(...v).applyAxisAngle(new Vector3(...axis), angle);
  return [Math.round(r.x), Math.round(r.y), Math.round(r.z)];
}

describe("sticker layout", () => {
  test("each cubie carries as many stickers as it has outer faces", () => {
    for (const cubie of CUBIES) {
      const outer = cubie.filter((c) => c !== 0).length;
      const count = STICKERS.filter((s) => key(s.position) === key(cubie)).length;
      expect(count, key(cubie)).toBe(outer);
    }
  });

  test("no two stickers share a place", () => {
    const places = STICKERS.map((s) => key(s.position) + "|" + key(s.normal));
    expect(new Set(places).size).toBe(54);
  });
});

// The real check: animating a turn must put each sticker exactly where the engine says
// its colour goes. Otherwise the cube would "jump" at the end of every animation.
describe("animation matches the engine", () => {
  for (const face of FACES) {
    for (const suffix of ["", "2", "'"]) {
      const move = face + suffix;
      test(move, () => {
        const solved = Array.from(facelets(""));
        const after = Array.from(facelets(move));
        const turn = parseMove(move);
        STICKERS.forEach((s, i) => {
          if (!inLayer(s.position, face)) {
            expect(after[i]).toBe(solved[i]);
            return;
          }
          const angle = turnAngle(turn);
          const pos = rotate(s.position, NORMALS[face], angle);
          const normal = rotate(s.normal, NORMALS[face], angle);
          const j = STICKERS.findIndex(
            (t) => key(t.position) === key(pos) && key(t.normal) === key(normal),
          );
          expect(after[j], `${move}: sticker ${i} -> ${j}`).toBe(solved[i]);
        });
      });
    }
  }
});

test("move helpers", () => {
  expect(inverseMove("R")).toBe("R'");
  expect(inverseMove("R'")).toBe("R");
  expect(inverseMove("R2")).toBe("R2");
  expect(isSolved(facelets(""))).toBe(true);
  expect(isSolved(facelets("R"))).toBe(false);
});
