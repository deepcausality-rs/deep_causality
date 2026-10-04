// The four runs at touchdown, side by side, then the campaign's counts.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO, SANS } from "../tokens";
import { facts, flights, type Part, type SceneSpec } from "../data";
import { WorldView } from "../views/WorldView";
import { Frame1080 } from "../views/Hud";

const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
const TILES: { part: Part; title: string; outcome: string; danger?: boolean }[] = [
  { part: 1, title: "PART 1 · TEXTBOOK LADDER", outcome: "in the creek" },
  { part: 2, title: "PART 2 · CONTEXT", outcome: "in the creek" },
  { part: 3, title: "PART 3 · ACTION", outcome: "3 m from a person", danger: true },
  { part: 4, title: "PART 4 · EFFECT ETHOS", outcome: "upright, 34 m from the nearest person" },
];

export const Teaser: React.FC<{ spec: SceneSpec }> = ({ spec }) => {
  const frame = useCurrentFrame();
  const per = 75;
  const counts = interpolate(frame, [4 * per, 4 * per + 15], [0, 1], clamp);
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      {TILES.map((tile, k) => {
        const show = interpolate(frame, [k * per, k * per + 12], [0, 1], clamp);
        const [col, row] = [k % 2, Math.floor(k / 2)];
        return (
          <div key={k} style={{ position: "absolute", left: 40 + col * 940, top: 40 + row * 470, width: 900, height: 440, overflow: "hidden", opacity: show, border: `1px solid ${tile.danger ? C.danger : C.line1}`, borderRadius: 8, background: C.bg0 }}>
            <div style={{ position: "absolute", left: 90, top: -170, transform: "scale(0.66)", transformOrigin: "0 0" }}>
              <WorldView part={tile.part} t={flights[tile.part].touchdown.t + 1} knows={tile.part >= 2} track={tile.part <= 2} keepOut={tile.part === 4} />
            </div>
            <div style={{ position: "absolute", left: 20, top: 16, fontFamily: MONO, fontSize: 16, letterSpacing: 1.4, color: C.fg2 }}>{tile.title}</div>
            <div style={{ position: "absolute", left: 20, top: 44, fontFamily: SANS, fontSize: 30, color: tile.danger ? C.danger : C.fg0 }}>{tile.outcome}</div>
          </div>
        );
      })}
      <Frame1080>
        <g opacity={counts}>
          <rect x={360} y={440} width={1200} height={200} rx={10} fill={C.bg1} stroke={C.line2} />
          <text x={960} y={500} fill={C.fg1} fontFamily={SANS} fontSize={28} textAnchor="middle">1,000 randomised nights · nights with a person at risk</text>
          {[["textbook", 0], ["context and action", 1], ["Effect Ethos", 2]].map(([label, c], k) => (
            <g key={k}>
              <text x={560 + k * 400} y={590} fill={C.danger} fontFamily={MONO} fontSize={56} textAnchor="middle">{facts.campaign.nearPerson[c as number]}</text>
              <text x={560 + k * 400} y={622} fill={C.fg2} fontFamily={MONO} fontSize={16} textAnchor="middle">{label as string}</text>
            </g>
          ))}
        </g>
      </Frame1080>
    </AbsoluteFill>
  );
};
