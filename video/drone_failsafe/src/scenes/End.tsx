// The end screen: the closing line, the two logos, and their links.
import React from "react";
import { AbsoluteFill, Img, staticFile, useCurrentFrame } from "remotion";
import { C, MONO } from "../tokens";
import script from "../script.json";
import { appear, Text } from "./common";

export const END_FRAMES = 330;
/** Frames at which the closing line and the logos start to appear, and the addresses under them. */
export const END_LINE_AT = 4;
export const END_LOGOS_AT = 30;
export const TUTORIAL_URL = "deepcausality.com/tutorials/dynamic-drone-failsafe";
export const RUN_COMMAND = "cargo run -p dynamic_drone_failsafe --example drone_failsafe_part_4";
export const CENTER_URL = "causalcenter.com";

export const End: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <Text size={56} weight={500} style={{ position: "absolute", left: 0, right: 0, top: 130, textAlign: "center", opacity: appear(frame, END_LINE_AT) }}>{script.end.line}</Text>
      <div style={{ position: "absolute", left: 0, right: 0, top: 290, display: "flex", justifyContent: "center", gap: 160, opacity: appear(frame, END_LOGOS_AT, 20) }}>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
          <Img src={staticFile("logos/logo_background.jpg")} style={{ width: 640, height: 336, borderRadius: 10, border: `1px solid ${C.line1}` }} />
          <div style={{ fontFamily: MONO, fontSize: 26, color: C.accent, marginTop: 34 }}>{TUTORIAL_URL}</div>
          <div style={{ marginTop: 18, padding: "10px 18px", borderRadius: 6, background: C.bg1, border: `1px solid ${C.line1}`, fontFamily: MONO, fontSize: 19, color: C.fg1 }}>
            {RUN_COMMAND}
          </div>
        </div>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
          <Img src={staticFile("logos/causal_center_logo_dark.svg")} style={{ width: 336, height: 336 }} />
          <div style={{ fontFamily: MONO, fontSize: 26, color: C.accent, marginTop: 34 }}>{CENTER_URL}</div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
