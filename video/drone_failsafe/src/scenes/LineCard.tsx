// A clip's opening hook line and its closing line, spoken over a plain card.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO, SANS } from "../tokens";
import { type SceneSpec } from "../data";
import { Frame1080 } from "../views/Hud";

export const LineCard: React.FC<{ spec: SceneSpec }> = ({ spec }) => {
  const frame = useCurrentFrame();
  const show = interpolate(frame, [0, 10], [0, 1], { extrapolateRight: "clamp" });
  const end = spec.name === "endCard";
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <Frame1080>
        <g opacity={show}>
          <text x={960} y={end ? 380 : 470} fill={C.fg2} fontFamily={MONO} fontSize={20} letterSpacing={2} textAnchor="middle">DEEPCAUSALITY · DYNAMIC DRONE FAIL-SAFE</text>
          <foreignObject x={260} y={end ? 410 : 500} width={1400} height={180}>
            <div style={{ textAlign: "center", color: C.fg0, fontFamily: SANS, fontSize: 60, lineHeight: 1.2 }}>{spec.cues[0].text}</div>
          </foreignObject>
          {end && (
            <>
              <text x={960} y={720} fill={C.accent} fontFamily={MONO} fontSize={28} textAnchor="middle">deepcausality.com/tutorials/dynamic-drone-failsafe</text>
              <text x={960} y={772} fill={C.fg2} fontFamily={MONO} fontSize={20} textAnchor="middle">cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_1</text>
            </>
          )}
        </g>
      </Frame1080>
    </AbsoluteFill>
  );
};
