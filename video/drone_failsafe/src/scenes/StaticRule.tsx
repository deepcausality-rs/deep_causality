// The issue with the textbook fail-safe's static rule. Behind the text, the textbook drone where it
// came down.
import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { C } from "../tokens";
import { flights } from "../data";
import script from "../script.json";
import { WorldView } from "../views/WorldView";
import { appear, Eyebrow, Text } from "./common";
import { STILL_WORLD } from "./world";

export const STATIC_RULE_FRAMES = 300;

export const StaticRule: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <div style={{ position: "absolute", inset: 0, opacity: 0.2 }}>
        <WorldView part={1} t={flights[1].touchdown.t + 1} track {...STILL_WORLD} />
      </div>
      <div style={{ position: "absolute", left: 200, top: 300, width: 1500 }}>
        <Eyebrow text="Tutorial part 1 · The issue" />
        <Text size={72} weight={500} style={{ marginTop: 22, opacity: appear(frame, 6) }}>{script.issue.heading}</Text>
        {script.issue.lines.map((l, k) => (
          <Text key={k} size={42} color={k === script.issue.lines.length - 1 ? C.fg0 : C.fg1} style={{ marginTop: 26, opacity: appear(frame, 50 + k * 70) }}>{l}</Text>
        ))}
      </div>
    </AbsoluteFill>
  );
};
