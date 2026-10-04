// A chapter card: tells the viewer what comes next.
import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { C, MONO } from "../tokens";
import script from "../script.json";
import { appear, Text } from "./common";

export const PART1_CARD_FRAMES = 165;
export const PART2_CARD_FRAMES = 255;
/** Frames at which a card's label, heading and line `k` start to appear. */
export const CARD_LABEL_AT = 4;
export const CARD_HEADING_AT = 10;
export const CARD_LINE_AT = (k: number) => 40 + k * 40;

const Card: React.FC<{ label: string; heading: string; lines: string[]; accent: boolean }> = ({ label, heading, lines, accent }) => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0, justifyContent: "center" }}>
      <div style={{ marginLeft: 200, width: 1500 }}>
        <div style={{ fontFamily: MONO, fontSize: 26, letterSpacing: 3, color: accent ? C.accent : C.fg1, textTransform: "uppercase", opacity: appear(frame, CARD_LABEL_AT) }}>{label}</div>
        <Text size={80} weight={500} style={{ marginTop: 18, opacity: appear(frame, CARD_HEADING_AT) }}>{heading}</Text>
        {lines.map((l, k) => (
          <Text key={k} size={40} color={C.fg1} style={{ marginTop: 28, opacity: appear(frame, CARD_LINE_AT(k)) }}>{l}</Text>
        ))}
      </div>
    </AbsoluteFill>
  );
};

export const Part1Card: React.FC = () => <Card label={script.part1Card.label} heading={script.part1Card.heading} lines={[script.part1Card.line]} accent={false} />;
export const Part2Card: React.FC = () => <Card label={script.part2Card.label} heading={script.part2Card.heading} lines={script.part2Card.lines} accent />;
