// The frame around the views: header chips, the fault timeline, captions and outcome cards.
import React from "react";
import { C, MONO, SANS, W } from "../tokens";
import { flights, type Part } from "../data";

export const Eyebrow: React.FC<{ text: string; x?: number; y?: number }> = ({ text, x = 96, y = 72 }) => (
  <text x={x} y={y} fill={C.fg2} fontFamily={MONO} fontSize={20} letterSpacing={2}>{text}</text>
);

/** Chip text size; JetBrains Mono advances every glyph by 0.6 em. */
const CHIP_FONT = 17;
const CHAR = 0.6 * CHIP_FONT;
const [PAD, GAP] = [18, 14];

export const Chips: React.FC<{ items: [string, string, boolean?][] }> = ({ items }) => {
  let x = 96;
  return (
    <g>
      {items.map(([k, v, hot]) => {
        const vx = x + PAD + k.length * CHAR + GAP;
        const w = vx - x + v.length * CHAR + PAD;
        const chip = (
          <g key={k}>
            <rect x={x} y={92} width={w} height={40} rx={6} fill={C.bg1} stroke={C.line1} opacity={0.94} />
            <text x={x + PAD} y={118} fill={C.fg2} fontFamily={MONO} fontSize={CHIP_FONT}>{k}</text>
            <text x={vx} y={118} fill={hot ? C.accent : C.fg0} fontFamily={MONO} fontSize={CHIP_FONT}>{v}</text>
          </g>
        );
        x += w + 12;
        return chip;
      })}
    </g>
  );
};

const URGENCY: Record<string, string> = { Routine: "routine", Practicable: "contingency", Possible: "emergency", LastResort: "last resort" };
const label = (d: string) => URGENCY[d] ?? d.replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase();

/** The faults of the default night: the flight second and what fails. */
const FAULTS: { t: number; what: string }[] = [
  { t: 55, what: "fix lost" },
  { t: 65, what: "link lost" },
  { t: 85, what: "cell fails" },
];

/** Flight time from 0 to 100 s, the fault markers of the default night, and the playhead. */
export const Timeline: React.FC<{ part: Part; t: number }> = ({ part, t }) => {
  const [x0, x1, y] = [96, 1824, 1028];
  const X = (s: number) => x0 + ((x1 - x0) * Math.min(100, s)) / 100;
  const rows = flights[part].rows;
  const changes = rows.filter((r, k) => k === 0 || r.decision !== rows[k - 1].decision);
  return (
    <g>
      <line x1={X(0)} y1={y} x2={X(100)} y2={y} stroke={C.line2} strokeWidth={3} />
      <line x1={X(0)} y1={y} x2={X(t)} y2={y} stroke={C.fg2} strokeWidth={3} />
      {changes.filter((r) => r.t > 0 && r.t <= t).map((r) => (
        <text key={r.t} x={X(r.t)} y={y - 14} fill={C.fg1} fontFamily={MONO} fontSize={13} textAnchor="middle">{label(r.decision)}</text>
      ))}
      {FAULTS.map((f) => {
        const passed = t >= f.t;
        return (
          <g key={f.t}>
            <line x1={X(f.t)} y1={y - 9} x2={X(f.t)} y2={y + 9} stroke={passed ? C.fg0 : C.fg2} strokeWidth={2} />
            <text x={X(f.t)} y={y + 30} fill={passed ? C.fg0 : C.fg2} fontFamily={MONO} fontSize={14} textAnchor="middle">{`${f.what} ${f.t} s`}</text>
          </g>
        );
      })}
      <circle cx={X(t)} cy={y} r={7} fill={C.accent} />
      <text x={X(t)} y={y + 30} fill={C.accent} fontFamily={MONO} fontSize={14} textAnchor="middle" opacity={FAULTS.every((f) => Math.abs(t - f.t) > 7) ? 1 : 0}>
        {`${Math.floor(t)} s`}
      </text>
    </g>
  );
};

export const Caption: React.FC<{ text: string; width?: number; x?: number }> = ({ text, width = 1180, x = 56 }) =>
  text ? (
    <g>
      <rect x={x} y={932} width={width} height={56} rx={6} fill={C.bg1} stroke={C.line1} opacity={0.95} />
      <foreignObject x={x + 20} y={932} width={width - 40} height={56}>
        <div style={{ height: 56, display: "flex", alignItems: "center", justifyContent: "center", textAlign: "center", color: C.fg0, fontFamily: SANS, fontSize: text.length > 95 ? 21 : 26, lineHeight: 1.15 }}>
          {text}
        </div>
      </foreignObject>
    </g>
  ) : null;

export const OutcomeCard: React.FC<{ lines: string[]; danger?: boolean; opacity: number }> = ({ lines, danger, opacity }) => (
  <g opacity={opacity}>
    <rect x={96} y={760} width={560} height={40 + lines.length * 44} rx={8} fill={C.bg1} stroke={danger ? C.danger : C.line2} strokeWidth={danger ? 2 : 1} />
    {lines.map((l, k) => (
      <text key={k} x={124} y={808 + k * 44} fill={k === 0 && danger ? C.danger : C.fg0} fontFamily={SANS} fontSize={k === 0 ? 34 : 24}>{l}</text>
    ))}
  </g>
);

export const Frame1080: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <svg width={W} height={1080} viewBox={`0 0 ${W} 1080`} style={{ position: "absolute", left: 0, top: 0 }}>
    {children}
  </svg>
);
