import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Canvas } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { Cube, type ActiveTurn } from "./Cube";
import { facelets, mix_for, random_mix, solveInWorker, solverReady, type SolutionData } from "./engine";
import { PaintPanel } from "./PaintPanel";
import { FACES, inverseMove, isSolved } from "./geometry";

type Kind = "manual" | "forward" | "back" | "scramble";

/** Mixes are played this fast; longer mixes than `MAX_ANIMATED` are applied instantly. */
const SCRAMBLE_SPEED = 15;
const MAX_ANIMATED = 50;

const ALGOS = [
  ["kociemba", "Kociemba (~20 moves)"],
  ["thistlethwaite", "Thistlethwaite (~31 moves)"],
  ["best", "Best of both"],
] as const;

const words = (s: string) => s.split(/\s+/).filter(Boolean);

export function App() {
  /** Every move applied since the solved cube: the position is `facelets(history)`. */
  const [history, setHistory] = useState<string[]>([]);
  const [turn, setTurn] = useState<(ActiveTurn & { kind: Kind }) | null>(null);
  const [solution, setSolution] = useState<SolutionData | null>(null);
  /** Solution moves already played. */
  const [cursor, setCursor] = useState(0);
  /** Mix moves still to animate. */
  const [scramble, setScramble] = useState<string[]>([]);
  const [playing, setPlaying] = useState(false);
  const [algo, setAlgo] = useState("kociemba");
  const [speed, setSpeed] = useState(3);
  const [mix, setMix] = useState("");
  const [error, setError] = useState("");
  const [solving, setSolving] = useState(false);
  const [ready, setReady] = useState(false);
  /** Sticker colours being entered by hand, or null when not painting. */
  const [paint, setPaint] = useState<number[] | null>(null);
  const [brush, setBrush] = useState(0);
  const [paintError, setPaintError] = useState("");
  const ids = useRef(0);

  useEffect(() => void solverReady.then(() => setReady(true)), []);

  const colors = useMemo(() => facelets(history.join(" ")), [history]);
  const flat = useMemo(() => solution?.steps.flat() ?? [], [solution]);

  const start = useCallback((move: string, kind: Kind) => {
    setTurn((t) => t ?? { id: ids.current++, move, kind });
  }, []);

  const onTurnDone = useCallback(() => {
    if (!turn) return;
    if (turn.kind === "back") {
      setHistory((h) => h.slice(0, -1));
      setCursor((c) => c - 1);
    } else {
      setHistory((h) => [...h, turn.move]);
      if (turn.kind === "forward") setCursor((c) => c + 1);
      if (turn.kind === "scramble") setScramble((q) => q.slice(1));
    }
    setTurn(null);
  }, [turn]);

  const forget = () => {
    setSolution(null);
    setCursor(0);
    setPlaying(false);
  };

  const stepForward = useCallback(() => {
    if (cursor < flat.length) start(flat[cursor], "forward");
  }, [cursor, flat, start]);
  const stepBack = useCallback(() => {
    if (cursor > 0) start(inverseMove(flat[cursor - 1]), "back");
  }, [cursor, flat, start]);

  // Playback: start the next move as soon as the previous one is done.
  useEffect(() => {
    if (!playing || turn) return;
    if (cursor < flat.length) stepForward();
    else setPlaying(false);
  }, [playing, turn, cursor, flat, stepForward]);

  // Mix animation: same idea, from the queue.
  useEffect(() => {
    if (!turn && scramble.length) start(scramble[0], "scramble");
  }, [turn, scramble, start]);

  const manualTurn = useCallback(
    (move: string) => {
      if (turn || scramble.length || paint) return;
      forget();
      start(move, "manual");
    },
    [turn, scramble, paint, start],
  );

  const applyMix = (text: string) => {
    try {
      facelets(text); // validates the notation
    } catch (e) {
      setError(String(e).replace(/^Error: /, ""));
      return;
    }
    setError("");
    setTurn(null);
    forget();
    const moves = words(text);
    const animate = moves.length <= MAX_ANIMATED;
    setHistory(animate ? [] : moves);
    setScramble(animate ? moves : []);
  };

  const randomMix = () => {
    const m = random_mix(25, Math.floor(Math.random() * 2 ** 32));
    setMix(m);
    applyMix(m);
  };

  const startPaint = () => {
    setTurn(null);
    setScramble([]);
    forget();
    setError("");
    setPaintError("");
    setPaint(Array.from(colors));
  };

  const paintSticker = (i: number) => {
    if (i % 9 === 4) return; // centers define the faces, they never change
    setPaintError("");
    setPaint((p) => p && p.map((c, k) => (k === i ? brush : c)));
  };

  const finishPaint = () => {
    if (!paint) return;
    try {
      const m = mix_for(Uint8Array.from(paint));
      setHistory(words(m));
      setMix(m);
      setPaint(null);
    } catch (e) {
      setPaintError(String(e).replace(/^Error: /, ""));
    }
  };

  const solve = async () => {
    setSolving(true);
    setError("");
    try {
      forget();
      setSolution(await solveInWorker(history, algo));
    } catch (e) {
      setError(String(e));
    } finally {
      setSolving(false);
    }
  };

  // Keyboard: U R F D L B turn a face, Shift for counter-clockwise; Space plays;
  // arrows step through the solution.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement) return;
      const face = e.key.toUpperCase();
      if ((FACES as readonly string[]).includes(face)) manualTurn(face + (e.shiftKey ? "'" : ""));
      else if (e.key === " " && flat.length) {
        e.preventDefault();
        setPlaying((p) => !p);
      } else if (e.key === "ArrowRight") stepForward();
      else if (e.key === "ArrowLeft") stepBack();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [manualTurn, stepForward, stepBack, flat.length]);

  const solved = isSolved(colors) && !turn;
  // Index of the solution move being animated or last played, for highlighting.
  const current = turn?.kind === "forward" ? cursor : turn?.kind === "back" ? cursor - 1 : -1;

  return (
    <div className="app">
      <main className="stage">
        <Canvas camera={{ position: [5.6, 4.6, 7.2], fov: 38 }} dpr={[1, 2]}>
          <ambientLight intensity={0.9} />
          <directionalLight position={[5, 8, 6]} intensity={1.6} />
          <directionalLight position={[-6, -4, -5]} intensity={0.5} />
          <Cube
            colors={paint ?? colors}
            onStickerClick={paint ? paintSticker : undefined}
            turn={turn}
            speed={turn?.kind === "scramble" ? SCRAMBLE_SPEED : speed}
            onTurnDone={onTurnDone}
          />
          <OrbitControls enablePan={false} minDistance={5} maxDistance={14} />
        </Canvas>
        <p className="hint">
          {paint
            ? "Click a sticker to paint it · drag to rotate"
            : "Drag to rotate · U R F D L B to turn (Shift: counter-clockwise)"}
        </p>
        {solved && !paint && history.length > 0 && <p className="badge">Solved</p>}
      </main>

      <aside className="panel">
        <h1>Rubik solver</h1>

        {paint && (
          <PaintPanel
            colors={paint}
            brush={brush}
            onBrush={setBrush}
            onClear={() => setPaint(paint.map((c, i) => (i % 9 === 4 ? c : 6)))}
            onDone={finishPaint}
            onCancel={() => setPaint(null)}
            error={paintError}
          />
        )}
        {!paint && (
          <>

        <section>
          <label htmlFor="mix">Mix</label>
          <form
            className="row"
            onSubmit={(e) => {
              e.preventDefault();
              applyMix(mix);
            }}
          >
            <input
              id="mix"
              value={mix}
              onChange={(e) => setMix(e.target.value)}
              placeholder="F R U2 B' L' D'"
              spellCheck={false}
              autoComplete="off"
            />
            <button type="submit">Apply</button>
          </form>
          <div className="row">
            <button onClick={randomMix}>Random mix</button>
            <button
              onClick={() => {
                setMix("");
                applyMix("");
              }}
            >
              Reset
            </button>
            <button onClick={startPaint}>Set my cube</button>
          </div>
          {error && <p className="error">{error}</p>}
        </section>

        <section>
          <label>Turn a face</label>
          <div className="faces">
            {FACES.flatMap((f) => [f, f + "'"]).map((m) => (
              <button key={m} onClick={() => manualTurn(m)} disabled={!!turn || scramble.length > 0}>
                {m}
              </button>
            ))}
          </div>
        </section>

        <section>
          <label htmlFor="algo">Algorithm</label>
          <div className="row">
            <select id="algo" value={algo} onChange={(e) => setAlgo(e.target.value)}>
              {ALGOS.map(([value, name]) => (
                <option key={value} value={value}>
                  {name}
                </option>
              ))}
            </select>
            <button
              className="primary"
              onClick={solve}
              disabled={!ready || solving || !!turn || scramble.length > 0 || isSolved(colors)}
            >
              {!ready ? "Loading…" : solving ? "Solving…" : "Solve"}
            </button>
          </div>
        </section>

        {solution && (
          <section className="solution">
            <div className="row spread">
              <label>
                Solution · {flat.length} moves
                {solution.length !== flat.length && ` (${solution.length} merged)`}
              </label>
              <span className="muted">
                {cursor}/{flat.length}
              </span>
            </div>
            <div className="row">
              <button onClick={stepBack} disabled={!!turn || cursor === 0} aria-label="Previous move">
                ◀
              </button>
              <button
                className="primary"
                onClick={() => setPlaying((p) => !p)}
                disabled={cursor === flat.length && !playing}
              >
                {playing ? "Pause" : "Play"}
              </button>
              <button onClick={stepForward} disabled={!!turn || cursor === flat.length} aria-label="Next move">
                ▶
              </button>
            </div>
            <ol className="steps">
              {solution.labels.map((label, s) => {
                const offset = solution.steps.slice(0, s).flat().length;
                return (
                  <li key={label}>
                    <span className="step-label">{label}</span>
                    <span className="moves">
                      {solution.steps[s].length === 0 && <span className="muted">nothing to do</span>}
                      {solution.steps[s].map((m, k) => {
                        const i = offset + k;
                        const cls = i === current ? "move current" : i < cursor ? "move done" : "move";
                        return (
                          <span key={k} className={cls}>
                            {m}
                          </span>
                        );
                      })}
                    </span>
                  </li>
                );
              })}
            </ol>
          </section>
        )}

        <section>
          <label htmlFor="speed">Speed · {speed} turns/s</label>
          <input
            id="speed"
            type="range"
            min={1}
            max={10}
            value={speed}
            onChange={(e) => setSpeed(Number(e.target.value))}
          />
        </section>
          </>
        )}
      </aside>
    </div>
  );
}
