// The title screen.
import React from "react";
import { AbsoluteFill, Img, staticFile, useCurrentFrame } from "remotion";
import { C } from "../tokens";
import script from "../script.json";
import { appear, Eyebrow, Text } from "./common";

export const TITLE_FRAMES = 165;

export const Title: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0, justifyContent: "center", alignItems: "center" }}>
      <div style={{ textAlign: "center", opacity: appear(frame, 4, 18) }}>
        <Img src={staticFile("logos/logo_background.jpg")} style={{ width: 560, borderRadius: 10, border: `1px solid ${C.line1}` }} />
        <Eyebrow text="DeepCausality tutorial" style={{ marginTop: 50 }} />
        <Text size={84} weight={500} style={{ marginTop: 18 }}>{script.title.title}</Text>
        <Text size={34} color={C.fg1} style={{ marginTop: 18, opacity: appear(frame, 30) }}>{script.title.subtitle}</Text>
      </div>
    </AbsoluteFill>
  );
};
