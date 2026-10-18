import { COLORS, COLOR_NAMES } from "./Cube";

interface Props {
  colors: number[];
  brush: number;
  onBrush: (color: number) => void;
  onClear: () => void;
  onDone: () => void;
  onCancel: () => void;
  error: string;
}

/** Controls for entering a real cube sticker by sticker. */
export function PaintPanel({ colors, brush, onBrush, onClear, onDone, onCancel, error }: Props) {
  const count = (c: number) => colors.filter((x) => x === c).length;
  const unpainted = count(6);

  return (
    <section>
      <label>Set my cube</label>
      <p className="muted">
        Hold your cube with the <b>white</b> center on top and the <b>green</b> center facing you.
        Pick a colour, then click stickers to paint them. Drag to see the other sides.
      </p>
      <div className="palette">
        {COLOR_NAMES.map((name, c) => (
          <button
            key={name}
            className={c === brush ? "swatch selected" : "swatch"}
            style={{ background: COLORS[c] }}
            onClick={() => onBrush(c)}
            aria-label={name}
            aria-pressed={c === brush}
            title={name}
          >
            <span className={count(c) === 9 ? "count" : "count off"}>{count(c)}</span>
          </button>
        ))}
      </div>
      <p className="muted">
        {unpainted > 0 ? `${unpainted} stickers left to paint · ` : ""}each colour must appear 9 times
      </p>
      {error && <p className="error">{error}</p>}
      <div className="row">
        <button className="primary" onClick={onDone}>
          Use this cube
        </button>
        <button onClick={onClear}>Clear</button>
        <button onClick={onCancel}>Cancel</button>
      </div>
    </section>
  );
}
