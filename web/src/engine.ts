// Access to the Rust engine: cheap calls (stickers, random mixes) on the main thread,
// solving in a worker.
import init, { facelets, random_mix } from "rubik-wasm";

export { facelets, random_mix };

/** Resolves once the main-thread copy of the engine is loaded. */
export const engineReady = init();

export interface SolutionData {
  labels: string[];
  /** Moves of each step, e.g. ["R", "U2"]. */
  steps: string[][];
  /** Length of the full solution once moves are merged across steps. */
  length: number;
}

const worker = new Worker(new URL("./solver.worker.ts", import.meta.url), { type: "module" });
const pending = new Map<number, { resolve: (s: SolutionData) => void; reject: (e: Error) => void }>();
let nextId = 0;
let markReady: () => void;

/** Resolves once the worker has built every solving table. */
export const solverReady = new Promise<void>((resolve) => (markReady = resolve));

const words = (s: string) => s.split(/\s+/).filter(Boolean);

worker.onmessage = (e) => {
  const d = e.data;
  if (d.type === "ready") return markReady();
  const p = pending.get(d.id);
  pending.delete(d.id);
  if (d.type === "error") p?.reject(new Error(d.error));
  else p?.resolve({ labels: d.labels, steps: d.steps.map(words), length: words(d.moves).length });
};

export function solveInWorker(moves: string[], algo: string): Promise<SolutionData> {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    worker.postMessage({ id, moves: moves.join(" "), algo });
  });
}
