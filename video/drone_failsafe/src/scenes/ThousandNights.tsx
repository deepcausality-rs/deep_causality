// The evidence: the same scenario simulated 1,000 times with different parameters, flown by both
// fail-safes; one cell per simulation, from part_5_verification/campaign_1000.csv.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO } from "../tokens";
import { campaign, ENDINGS, facts } from "../data";
import script from "../script.json";
import { appear, clamp, Text } from "./common";

export const THOUSAND_FRAMES = 420;
const NEAR = ENDINGS.indexOf("NearPerson");
const COLS = 40;
const CELL = 13;
// Columns of the campaign record: part 1 is 0, part 4 is 2.
const SIDES = [
  { column: 0, x: 200, title: script.simulations.textbookLabel.toUpperCase(), rate: (p: number[]) => `${p[0].toFixed(1)} %` },
  { column: 2, x: 1080, title: script.simulations.ethosLabel.toUpperCase(), rate: (p: number[]) => `${p[2].toFixed(1)} % · at most ${facts.campaign.part4UpperPct.toFixed(2)} % at 95 % confidence` },
];

export const ThousandNights: React.FC = () => {
  const frame = useCurrentFrame();
  const filled = Math.round(interpolate(frame, [20, 130], [0, campaign.length], clamp));
  const reveal = appear(frame, 150, 18);
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <div style={{ position: "absolute", left: 200, top: 70, width: 1520 }}>
        <Text size={52} weight={500}>{script.simulations.heading}</Text>
        <Text size={28} color={C.fg1} style={{ marginTop: 14 }}>{script.simulations.description}</Text>
      </div>
      <svg width={1920} height={1080} style={{ position: "absolute", left: 0, top: 0 }}>
        {SIDES.map((side) => (
          <g key={side.column}>
            <text x={side.x} y={318} fill={side.column === 2 ? C.accent : C.fg1} fontFamily={MONO} fontSize={20} letterSpacing={2}>{side.title}</text>
            {campaign.slice(0, filled).map((s, n) => {
              const near = s[side.column] === NEAR;
              return (
                <rect
                  key={n}
                  x={side.x + (n % COLS) * CELL}
                  y={340 + Math.floor(n / COLS) * CELL}
                  width={CELL - 2}
                  height={CELL - 2}
                  fill={near && reveal > 0 ? C.danger : "#33404d"}
                  opacity={near && reveal > 0 ? 1 : 0.85}
                />
              );
            })}
            <g opacity={reveal}>
              <text x={side.x} y={790} fill={C.danger} fontFamily={MONO} fontSize={110}>{facts.campaign.nearPerson[side.column]}</text>
              <text x={side.x} y={840} fill={C.fg0} fontFamily="Geist, sans-serif" fontSize={28}>
                {facts.campaign.nearPerson[side.column] === 1 ? script.simulations.countLabelOne : script.simulations.countLabelMany}
              </text>
              <text x={side.x} y={878} fill={C.fg2} fontFamily={MONO} fontSize={18}>{side.rate(facts.campaign.nearPersonPct)}</text>
            </g>
          </g>
        ))}
      </svg>
    </AbsoluteFill>
  );
};
