// Where each of the engine's 54 stickers sits in 3D, and how a face turn rotates its layer.
// Display only: which colour goes where always comes from the engine (`facelets`).
//
// Axes: x to the right (R), y up (U), z towards the viewer (F). Cubies sit at -1, 0 or 1.

export type Vec3 = [number, number, number];

export const FACES = ["U", "R", "F", "D", "L", "B"] as const;
export type Face = (typeof FACES)[number];

export const NORMALS: Record<Face, Vec3> = {
  U: [0, 1, 0],
  R: [1, 0, 0],
  F: [0, 0, 1],
  D: [0, -1, 0],
  L: [-1, 0, 0],
  B: [0, 0, -1],
};

// Sticker `row`, `col` of each face (engine order: row by row, as seen from outside,
// U seen with B at the top, D seen with F at the top, side faces with U at the top).
const PLACE: Record<Face, (row: number, col: number) => Vec3> = {
  U: (r, c) => [c - 1, 1, r - 1],
  R: (r, c) => [1, 1 - r, 1 - c],
  F: (r, c) => [c - 1, 1 - r, 1],
  D: (r, c) => [c - 1, -1, 1 - r],
  L: (r, c) => [-1, 1 - r, c - 1],
  B: (r, c) => [1 - c, 1 - r, -1],
};

export interface Sticker {
  /** Position of the cubie carrying the sticker. */
  position: Vec3;
  /** Outward direction of the sticker. */
  normal: Vec3;
}

/** The 54 stickers, in the engine's facelet order (index = face * 9 + row * 3 + col). */
export const STICKERS: Sticker[] = FACES.flatMap((face) =>
  Array.from({ length: 9 }, (_, i) => ({
    position: PLACE[face](Math.floor(i / 3), i % 3),
    normal: NORMALS[face],
  })),
);

/** The 26 visible cubie positions. */
export const CUBIES: Vec3[] = [-1, 0, 1]
  .flatMap((x) => [-1, 0, 1].flatMap((y) => [-1, 0, 1].map((z): Vec3 => [x, y, z])))
  .filter(([x, y, z]) => x !== 0 || y !== 0 || z !== 0);

export interface Turn {
  face: Face;
  /** Clockwise quarter turns: 1, 2 or 3. */
  turns: number;
}

export function parseMove(move: string): Turn {
  const face = move[0] as Face;
  const turns = move.endsWith("'") ? 3 : move.endsWith("2") ? 2 : 1;
  return { face, turns };
}

export function inverseMove(move: string): string {
  const { face, turns } = parseMove(move);
  return face + ["", "'", "2", ""][turns];
}

/** Rotation angle around the face's normal. Clockwise seen from the face = negative. */
export function turnAngle({ turns }: Turn): number {
  return turns === 3 ? Math.PI / 2 : (-turns * Math.PI) / 2;
}

export function inLayer(position: Vec3, face: Face): boolean {
  const n = NORMALS[face];
  return position[0] * n[0] + position[1] * n[1] + position[2] * n[2] === 1;
}

const SOLVED_FACE = (i: number) => Math.floor(i / 9);

/** True when every face shows a single colour. */
export function isSolved(colors: ArrayLike<number>): boolean {
  return Array.from(colors).every((c, i) => c === SOLVED_FACE(i));
}
