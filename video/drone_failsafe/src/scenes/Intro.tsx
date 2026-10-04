// The setting: an inspection drone on a power line at night, a repair crew below, and the three
// faults ahead. The drone's position is part 1's recorded flight.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO } from "../tokens";
import { world } from "../data";
import { WorldView } from "../views/WorldView";
import { elevation, LINE_ACROSS } from "../world/terrain";
import script from "../script.json";
import { appear, clamp, Text } from "./common";
import { STILL_WORLD } from "./world";

export const INTRO_FRAMES = 300;
/** Frame at which line `k` of the setting starts to appear; the faults follow the last line. */
export const SETTING_LINE_AT = (k: number) => 10 + k * 65;
export const FAULTS_LINE = "55 s GPS lost · 65 s link lost · 85 s cell fails";

export const Intro: React.FC = () => {
  const frame = useCurrentFrame();
  const t = interpolate(frame, [0, INTRO_FRAMES], [44, 52], clamp);
  const [crewX, crewY] = world.crew[4];
  const labels = [
    { at: [-10, 452, 0.5] as [number, number, number], text: "creek" },
    { at: [crewX + 13, crewY, elevation(crewX, crewY) + 3] as [number, number, number], text: "repair crew" },
    { at: [30, 446, elevation(30, 446) + 1.5] as [number, number, number], text: "terrace" },
    { at: [LINE_ACROSS + 9, 466, elevation(LINE_ACROSS, 466) + 22] as [number, number, number], text: "power line" },
  ];
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <WorldView part={1} t={t} wind labels={labels} {...STILL_WORLD} />
      <div style={{ position: "absolute", left: 96, top: 90, width: 940 }}>
        {script.setting.lines.map((line, k) => (
          <Text key={k} size={k === 0 ? 50 : 32} color={k === 0 ? C.fg0 : C.fg1} style={{ marginTop: k === 0 ? 26 : 22, opacity: appear(frame, SETTING_LINE_AT(k)) }}>{line}</Text>
        ))}
        <div style={{ fontFamily: MONO, fontSize: 22, color: C.accent, marginTop: 18, opacity: appear(frame, SETTING_LINE_AT(script.setting.lines.length)) }}>
          {FAULTS_LINE}
        </div>
      </div>
    </AbsoluteFill>
  );
};
