// Solving runs here, off the main thread: Kociemba searches for up to 0.5 s and building
// its tables takes longer, which would freeze the animation.
import init, { prepare, solve } from "rubik-wasm";

const ready = init().then(() => {
  prepare("best");
  postMessage({ type: "ready" });
});

onmessage = async (e: MessageEvent<{ id: number; moves: string; algo: string }>) => {
  await ready;
  const { id, moves, algo } = e.data;
  try {
    const s = solve(moves, algo);
    postMessage({ type: "solution", id, labels: s.labels, steps: s.steps, moves: s.moves });
    s.free();
  } catch (err) {
    postMessage({ type: "error", id, error: String(err) });
  }
};
