// The night every run flies: the slope, the creek, the terrace, the line and its tower, the crew,
// and the drone on its inspection line. The drone's position is part 1's recorded flight.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO, SANS } from "../tokens";
import { droneAt, type SceneSpec } from "../data";
import { WorldView } from "../views/WorldView";
import { Chips, Eyebrow, Frame1080, Timeline } from "../views/Hud";
import { elevation, LINE_ACROSS } from "../world/terrain";

export const WorldIntro: React.FC<{ spec: SceneSpec }> = ({ spec }) => {
  const frame = useCurrentFrame();
  const t = interpolate(frame, [0, spec.durationInFrames], [44, 54], { extrapolateRight: "clamp" });
  const orbit = interpolate(frame, [0, spec.durationInFrames], [-14, 0], { extrapolateRight: "clamp" });
  const cutY = droneAt(1, t).y;
  const show = (k: number) => interpolate(frame, [spec.cues[k].from, spec.cues[k].from + 12], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const labels = [
    { at: [-8, cutY + 30, 0.5] as [number, number, number], text: "creek" },
    { at: [86, cutY + 12, elevation(86, cutY + 12) + 1] as [number, number, number], text: "grass, 30°" },
    { at: [30, Math.max(cutY + 8, 424), elevation(30, 440) + 1] as [number, number, number], text: "terrace" },
    { at: [LINE_ACROSS, cutY + 44, elevation(LINE_ACROSS, cutY + 44) + 25] as [number, number, number], text: "power line" },
  ];
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <WorldView part={1} t={t} wind orbit={orbit} labels={labels} lift={16} fov={34} />
      <Frame1080>
        <rect x={0} y={0} width={1266} height={150} fill={C.bg0} opacity={0.82} />
        <rect x={1266} y={0} width={654} height={1080} fill={C.bg0} />
        <Eyebrow text="DEEPCAUSALITY · THE NIGHT EVERY RUN FLIES" />
        <Chips items={[["launch", "22:00"], ["height", "40 m"], ["wind", "2 m/s downslope"]]} />
        <g opacity={show(1)}>
          <text x={1300} y={240} fill={C.fg2} fontFamily={MONO} fontSize={15} letterSpacing={1.4}>THE MISSION</text>
          <text x={1300} y={286} fill={C.fg0} fontFamily={SANS} fontSize={30}>inspect a power line</text>
          <text x={1300} y={326} fill={C.fg1} fontFamily={SANS} fontSize={24}>40 m above a 30° grass slope, at 8 m/s</text>
        </g>
        <g opacity={show(2)}>
          <text x={1300} y={410} fill={C.fg2} fontFamily={MONO} fontSize={15} letterSpacing={1.4}>BELOW</text>
          <text x={1300} y={452} fill={C.fg1} fontFamily={SANS} fontSize={24}>a creek on the valley floor</text>
          <text x={1300} y={488} fill={C.fg1} fontFamily={SANS} fontSize={24}>a flat terrace in the slope</text>
          <text x={1300} y={524} fill={C.fg1} fontFamily={SANS} fontSize={24}>a crew on the road and at a tower</text>
        </g>
        <g opacity={show(3)}>
          <text x={1300} y={608} fill={C.fg2} fontFamily={MONO} fontSize={15} letterSpacing={1.4}>THE FAULTS</text>
          {[["55 s", "satellite positioning lost", 3], ["65 s", "link to the pilot lost", 4], ["85 s", "a battery cell fails", 5]].map(([s, text, k]) => (
            <g key={s as string} opacity={show(k as number)}>
              <text x={1300} y={652 + ((k as number) - 3) * 40} fill={C.accent} fontFamily={MONO} fontSize={22}>{s}</text>
              <text x={1390} y={652 + ((k as number) - 3) * 40} fill={C.fg0} fontFamily={SANS} fontSize={24}>{text}</text>
            </g>
          ))}
        </g>
        <Timeline part={1} t={0} />
      </Frame1080>
    </AbsoluteFill>
  );
};
