import { useLayoutEffect, useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { RoundedBox } from "@react-three/drei";
import { Group, Vector3 } from "three";
import { CUBIES, NORMALS, STICKERS, inLayer, parseMove, turnAngle, type Vec3 } from "./geometry";

// U R F D L B, standard scheme.
const COLORS = ["#f4f4f4", "#c41e3a", "#009e60", "#ffd500", "#ff5800", "#0051ba"];

export interface ActiveTurn {
  /** New id for every turn, even when the same move is repeated. */
  id: number;
  move: string;
}

interface Props {
  /** The 54 sticker colours of the position before `turn`. */
  colors: ArrayLike<number>;
  turn: ActiveTurn | null;
  /** Turns per second. */
  speed: number;
  onTurnDone: () => void;
}

/**
 * Draws the position, and animates `turn` by rotating its 9 cubies. When the rotation
 * ends, the parent swaps in the engine's colours for the next position and the layer
 * snaps back to 0: stickers then sit exactly where they ended (checked by geometry.test).
 * No React state changes per frame: the angle lives in a ref.
 */
export function Cube({ colors, turn, speed, onTurnDone }: Props) {
  const layer = useRef<Group>(null);
  const progress = useRef(0);

  const parsed = turn ? parseMove(turn.move) : null;
  const axis = useMemo(() => (parsed ? new Vector3(...NORMALS[parsed.face]) : null), [parsed?.face]);

  // Runs before paint whenever the turn changes, together with the new colours.
  useLayoutEffect(() => {
    progress.current = 0;
    layer.current?.rotation.set(0, 0, 0);
  }, [turn?.id]);

  useFrame((_, dt) => {
    if (!parsed || !axis || !layer.current || progress.current >= 1) return;
    progress.current = Math.min(1, progress.current + dt * speed);
    layer.current.setRotationFromAxisAngle(axis, ease(progress.current) * turnAngle(parsed));
    if (progress.current === 1) onTurnDone();
  });

  const moving = (p: Vec3) => parsed !== null && inLayer(p, parsed.face);
  const cubie = (p: Vec3) => (
    <RoundedBox key={p.join()} args={[0.96, 0.96, 0.96]} radius={0.08} position={p}>
      <meshStandardMaterial color="#141414" roughness={0.6} />
    </RoundedBox>
  );
  const sticker = (i: number) => {
    const { position, normal } = STICKERS[i];
    const at = position.map((c, k) => c + normal[k] * 0.485) as Vec3;
    return (
      <mesh key={i} position={at} rotation={facing(normal)}>
        <planeGeometry args={[0.8, 0.8]} />
        <meshStandardMaterial color={COLORS[colors[i]]} roughness={0.35} />
      </mesh>
    );
  };
  const indices = STICKERS.map((_, i) => i);

  return (
    <>
      <group ref={layer}>
        {CUBIES.filter(moving).map(cubie)}
        {indices.filter((i) => moving(STICKERS[i].position)).map(sticker)}
      </group>
      <group>
        {CUBIES.filter((p) => !moving(p)).map(cubie)}
        {indices.filter((i) => !moving(STICKERS[i].position)).map(sticker)}
      </group>
    </>
  );
}

const ease = (t: number) => (t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2);

/** Euler rotation turning a plane (which faces +z) towards `normal`. */
function facing([x, y, z]: Vec3): [number, number, number] {
  if (x) return [0, (x * Math.PI) / 2, 0];
  if (y) return [(-y * Math.PI) / 2, 0, 0];
  return [0, z > 0 ? 0 : Math.PI, 0];
}
