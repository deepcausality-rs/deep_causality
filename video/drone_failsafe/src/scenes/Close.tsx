// The verdicts part 4 logs at the emergency, then where to find the tutorial.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO, SANS } from "../tokens";
import { type SceneSpec } from "../data";
import { Frame1080 } from "../views/Hud";

const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
// Part 4's log at 85 s, as the tutorial page prints it.
export const LOG = [
  "t=85 s: 72 proposals put to the Effect Ethos under the emergency norms: 55 permitted.",
  "t=85 s:   17 forbidden: a gust could put the drone on steep ground beside it.",
  "t=85 s: Chosen: land on the patch 0 m away, at 30 m across and 430 m along; harm cost 0.",
];

export const Close: React.FC<{ spec: SceneSpec }> = ({ spec }) => {
  const frame = useCurrentFrame();
  const typed = (k: number) => {
    const start = 8 + k * 26;
    const n = Math.floor(interpolate(frame, [start, start + 24], [0, LOG[k].length], clamp));
    return LOG[k].slice(0, n);
  };
  const end = interpolate(frame, [spec.cues[1].from, spec.cues[1].from + 15], [0, 1], clamp);
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <Frame1080>
        <text x={120} y={200} fill={C.fg2} fontFamily={MONO} fontSize={16} letterSpacing={1.4}>PART 4 · THE CONTROLLER'S LOG</text>
        {LOG.map((_, k) => (
          <text key={k} x={120} y={252 + k * 40} fill={k === 2 ? C.accent : C.fg1} fontFamily={MONO} fontSize={22}>{typed(k)}</text>
        ))}
        <g opacity={end}>
          <text x={120} y={560} fill={C.fg2} fontFamily={MONO} fontSize={20} letterSpacing={2}>DEEPCAUSALITY</text>
          <text x={120} y={640} fill={C.fg0} fontFamily={SANS} fontSize={64}>Dynamic drone fail-safe</text>
          <text x={120} y={700} fill={C.fg1} fontFamily={SANS} fontSize={28}>an open-source tutorial in Rust, in five parts</text>
          <text x={120} y={800} fill={C.accent} fontFamily={MONO} fontSize={28}>deepcausality.com/tutorials/dynamic-drone-failsafe</text>
          <rect x={120} y={840} width={1060} height={52} rx={6} fill={C.bg1} stroke={C.line1} />
          <text x={140} y={874} fill={C.fg0} fontFamily={MONO} fontSize={20}>cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_1</text>
        </g>
      </Frame1080>
    </AbsoluteFill>
  );
};
