// The campaign's first finding about part 4, as the tutorial README records it: a norm layer that
// could only forbid held the drone until its battery died, and it fell beside the crew.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO, SANS } from "../tokens";
import { type SceneSpec } from "../data";
import { Eyebrow, Frame1080 } from "../views/Hud";

const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

export const Twist: React.FC<{ spec: SceneSpec }> = ({ spec }) => {
  const frame = useCurrentFrame();
  const at = (k: number) => spec.cues[k].from;
  const len = (k: number) => spec.cues[k].durationInFrames;
  const show = (k: number) => interpolate(frame, [at(k), at(k) + 12], [0, 1], clamp);
  // The battery drains while every proposal is forbidden; then the drone falls.
  const drain = interpolate(frame, [at(2), at(2) + len(2) * 0.8], [1, 0], clamp);
  const fallen = interpolate(frame, [at(2) + len(2) * 0.8, at(2) + len(2) * 0.8 + 8], [0, 1], clamp);
  const rounds = Math.min(6, Math.floor(interpolate(frame, [at(1), at(2) + len(2) * 0.8], [0, 6.99], clamp)));
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <Frame1080>
        <Eyebrow text="DEEPCAUSALITY · EFFECT ETHOS · THE FIRST VERSION" />
        <g opacity={show(0)}>
          <text x={96} y={210} fill={C.fg0} fontFamily={SANS} fontSize={48}>The pilots' emergency procedures,</text>
          <text x={96} y={270} fill={C.fg0} fontFamily={SANS} fontSize={48}>written as norms.</text>
        </g>
        <g opacity={show(1)}>
          <text x={96} y={380} fill={C.fg1} fontFamily={SANS} fontSize={34}>The first version could only forbid.</text>
        </g>
        <g opacity={show(1)}>
          <rect x={1000} y={170} width={820} height={470} rx={8} fill={C.bg1} stroke={C.line1} />
          <text x={1024} y={206} fill={C.fg2} fontFamily={MONO} fontSize={14} letterSpacing={1.4}>EFFECT ETHOS · EVERY ROUND</text>
          {Array.from({ length: rounds }, (_, k) => (
            <g key={k}>
              <text x={1024} y={252 + k * 44} fill={C.fg1} fontFamily={MONO} fontSize={19}>{`round ${k + 1} · every landing`}</text>
              <text x={1796} y={252 + k * 44} fill={C.fg2} fontFamily={MONO} fontSize={19} textAnchor="end">forbidden</text>
            </g>
          ))}
          <text x={1024} y={560} fill={C.fg2} fontFamily={MONO} fontSize={14} letterSpacing={1.4}>BATTERY</text>
          <rect x={1024} y={576} width={772} height={24} rx={4} fill={C.bg0} stroke={C.line2} />
          <rect x={1024} y={576} width={772 * drain} height={24} rx={4} fill={drain < 0.2 ? C.danger : C.fg2} />
        </g>
        <g opacity={fallen}>
          <text x={96} y={520} fill={C.danger} fontFamily={SANS} fontSize={38}>The drone hovered until its battery died</text>
          <text x={96} y={570} fill={C.danger} fontFamily={SANS} fontSize={38}>and fell beside the crew.</text>
        </g>
        <g opacity={show(2)}>
          <rect x={96} y={680} width={1724} height={130} rx={8} fill={C.bg1} stroke={C.line1} />
          <text x={124} y={736} fill={C.fg0} fontFamily={MONO} fontSize={40}>3.9 %</text>
          <text x={300} y={732} fill={C.fg1} fontFamily={SANS} fontSize={26}>of 1,000 simulated nights put a person at risk.</text>
          <text x={300} y={772} fill={C.fg1} fontFamily={SANS} fontSize={26}>Every one of them was such a fall.</text>
          <text x={1796} y={790} fill={C.fg2} fontFamily={MONO} fontSize={14} textAnchor="end">tutorial README · how the campaign changed part 4</text>
        </g>
        <g opacity={show(3)}>
          <text x={96} y={940} fill={C.accent} fontFamily={SANS} fontSize={52}>The safety rule became the hazard.</text>
        </g>
      </Frame1080>
    </AbsoluteFill>
  );
};
