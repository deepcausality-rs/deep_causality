// The three steps behind the dynamic fail-safe.
import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { C, MONO } from "../tokens";
import script from "../script.json";
import { appear, Text } from "./common";

export const SUMMARY_FRAMES = 360;
/** Frames at which the heading and step `k` start to appear. */
export const SUMMARY_HEADING_AT = 4;
export const SUMMARY_STEP_AT = (k: number) => 30 + k * 45;

export const Summary: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <div style={{ position: "absolute", left: 140, top: 150, width: 1640 }}>
        <Text size={64} weight={500} style={{ opacity: appear(frame, SUMMARY_HEADING_AT) }}>{script.summary.heading}</Text>
        <div style={{ display: "flex", gap: 40, marginTop: 80 }}>
          {script.summary.steps.map((step, k) => (
            <div key={k} style={{ flex: 1, padding: "36px 34px 42px", borderRadius: 12, background: C.bg1, border: `1px solid ${k === 2 ? C.accent : C.line2}`, opacity: appear(frame, SUMMARY_STEP_AT(k), 18) }}>
              <div style={{ fontFamily: MONO, fontSize: 24, color: k === 2 ? C.accent : C.fg2 }}>{`0${k + 1}`}</div>
              <Text size={40} weight={500} color={k === 2 ? C.accent : C.fg0} style={{ marginTop: 16 }}>{step.name}</Text>
              <Text size={30} color={C.fg1} style={{ marginTop: 20 }}>{step.text}</Text>
            </div>
          ))}
        </div>
      </div>
    </AbsoluteFill>
  );
};
