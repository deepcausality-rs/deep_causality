// Part 5's campaign: parts 1, 3 and 4 over the same 1000 scenarios, one cell per scenario, from
// part_5_verification/campaign_1000.csv.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO, SANS } from "../tokens";
import { campaign, ENDINGS, facts, type SceneSpec } from "../data";
import { Eyebrow, Frame1080 } from "../views/Hud";

const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
const NEAR = ENDINGS.indexOf("NearPerson");
const KEPT = [ENDINGS.indexOf("Safe"), ENDINGS.indexOf("Resumed")];
const COLS = 40;
const CELL = 10;
const GRIDS = [
  { x: 120, title: "PART 1 · TEXTBOOK LADDER" },
  { x: 700, title: "PART 3 · CONTEXT AND ACTION" },
  { x: 1280, title: "PART 4 · EFFECT ETHOS" },
];

export const Campaign: React.FC<{ spec: SceneSpec }> = ({ spec }) => {
  const frame = useCurrentFrame();
  const at = (k: number) => spec.cues[k].from;
  const filled = Math.round(interpolate(frame, [6, at(1) + spec.cues[1].durationInFrames], [0, campaign.length], clamp));
  const reveal = [2, 3, 4].map((k) => interpolate(frame, [at(k), at(k) + 12], [0, 1], clamp));
  const zoom = interpolate(frame, [at(5), at(5) + 15], [0, 1], clamp);
  const pct = facts.campaign.nearPersonPct;
  const rates = [`${pct[0].toFixed(1)} %`, `${pct[1].toFixed(1)} %`, `${pct[2].toFixed(1)} % · at most ${facts.campaign.part4UpperPct.toFixed(2)} %`];
  const y0 = 300;
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <Frame1080>
        <Eyebrow text="DEEPCAUSALITY · PART 5 · VERIFICATION" />
        <text x={96} y={150} fill={C.fg0} fontFamily={SANS} fontSize={44}>1,000 randomised nights, flown by each controller</text>
        <text x={96} y={196} fill={C.fg1} fontFamily={SANS} fontSize={24}>any hour, any wind, the faults at any time, up to eight people where the fix drops out</text>
        {GRIDS.map((g, c) => (
          <g key={c}>
            <text x={g.x} y={y0 - 24} fill={C.fg2} fontFamily={MONO} fontSize={16} letterSpacing={1.4}>{g.title}</text>
            {campaign.slice(0, filled).map((s, n) => {
              const ending = s[c];
              const near = ending === NEAR;
              const fill = near ? (reveal[c] > 0 ? C.danger : C.fg2) : KEPT.includes(ending) ? C.accent : "#33404d";
              const op = near ? 1 : KEPT.includes(ending) ? 0.55 : 0.9;
              return <rect key={n} x={g.x + (n % COLS) * CELL} y={y0 + Math.floor(n / COLS) * CELL} width={CELL - 2} height={CELL - 2} fill={fill} opacity={op} />;
            })}
            <g opacity={reveal[c]}>
              <text x={g.x} y={y0 + 25 * CELL + 70} fill={C.danger} fontFamily={MONO} fontSize={64}>{facts.campaign.nearPerson[c]}</text>
              <text x={g.x + (c === 2 ? 60 : 110)} y={y0 + 25 * CELL + 60} fill={C.fg1} fontFamily={SANS} fontSize={22}>nights with a person at risk</text>
              <text x={g.x + (c === 2 ? 60 : 110)} y={y0 + 25 * CELL + 92} fill={C.fg2} fontFamily={MONO} fontSize={17}>{rates[c]}</text>
            </g>
          </g>
        ))}
        <g>
          {[[C.accent, 0.55, "landed upright or resumed"], ["#33404d", 0.9, "drone lost"], [C.danger, 1, "came down within 10 m of a person"]].map(([col, op, label], k) => (
            <g key={k}>
              <rect x={120 + k * 420} y={y0 + 25 * CELL + 150} width={14} height={14} fill={col as string} opacity={op as number} />
              <text x={144 + k * 420} y={y0 + 25 * CELL + 163} fill={C.fg2} fontFamily={MONO} fontSize={16}>{label}</text>
            </g>
          ))}
        </g>
        <g opacity={zoom}>
          {(() => {
            const n = facts.campaign.part4NearScenario;
            const [cx, cy] = [GRIDS[2].x + (n % COLS) * CELL + 4, y0 + Math.floor(n / COLS) * CELL + 4];
            return (
              <>
                <circle cx={cx} cy={cy} r={6 + 30 * zoom} fill="none" stroke={C.danger} strokeWidth={2} />
                <rect x={96} y={820} width={1728} height={96} rx={8} fill={C.bg1} stroke={C.danger} />
                <text x={124} y={860} fill={C.danger} fontFamily={MONO} fontSize={22}>{`scenario ${n}`}</text>
                <text x={124} y={896} fill={C.fg1} fontFamily={SANS} fontSize={22}>
                  The cell failed 6 s after the lost link was confirmed, with the drone at 40 m. Every patch it could reach lay beside ground where a person was not ruled out.
                </text>
              </>
            );
          })()}
        </g>
      </Frame1080>
    </AbsoluteFill>
  );
};
