/**
 * Scene 2, the plan: the weather campaign's dispersion table, drift against the day's temperature
 * departure, interpolated at the measured day; then the ignition margin each belief takes from it,
 * the mean plus k standard deviations. Every point is a row of `weather_table.csv`.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Retro } from '../data/retro';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, QuestionTitle, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);
const minus = (v: number) => (v < 0 ? `−${Math.abs(v)}` : `${v}`);

export const Plan: React.FC<{ retro: Retro; timing: SceneTiming }> = ({ retro: r, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const b = r.belief;
  const question = fade(frame, 4, 20) * (1 - fade(frame, at(ph, 1, 0), at(ph, 1, 0.15)));
  const rowsIn = fade(frame, at(ph, 1, 0.1), at(ph, 1, 0.3));
  const day = interpolate(frame, [at(ph, 2, 0.05), at(ph, 2, 0.5)], [0, 1], { ...clamp, easing: ease });
  const informed = interpolate(frame, [at(ph, 3, 0.05), at(ph, 3, 0.6)], [0, 1], { ...clamp, easing: ease });
  const uninformed = interpolate(frame, [at(ph, 4, 0.05), at(ph, 4, 0.6)], [0, 1], { ...clamp, easing: ease });

  // The table: drift against the temperature departure.
  const C = { x: 140, y: 230, w: 760, h: 520 };
  const dtLo = Math.min(...b.rows.map((x) => x.dTemp)) - 8;
  const dtHi = Math.max(...b.rows.map((x) => x.dTemp)) + 8;
  const mLo = Math.floor(Math.min(...b.rows.map((x) => x.mean - x.sd)) / 10) * 10;
  const mHi = Math.ceil(Math.max(b.informed.margin, ...b.rows.map((x) => x.mean + x.sd)) / 10) * 10;
  const px = (dT: number) => C.x + ((dT - dtLo) / (dtHi - dtLo)) * C.w;
  const py = (m: number) => C.y + C.h - ((m - mLo) / (mHi - mLo)) * C.h;
  const ticks = Array.from({ length: (mHi - mLo) / 10 + 1 }, (_, i) => mLo + i * 10);
  const inBracket = (dT: number) => dT === b.bracket[0] || dT === b.bracket[1];

  // The margins: mean plus k standard deviations, bars on one metre scale.
  const B = { x: 1060, w: 700, max: Math.ceil(b.informed.margin / 10) * 10 };
  const bx = (m: number) => (m / B.max) * B.w;
  const bar = (y: number, label: string, mean: number, sd: number, tone: string, p: number) => (
    <div style={{ position: 'absolute', left: B.x, top: y, width: B.w + 120, opacity: p }}>
      <div style={{ fontFamily: font.mono, fontSize: 21, color: tone }}>{label}</div>
      <svg width={B.w + 4} height={44} style={{ display: 'block', marginTop: 10 }}>
        <rect x={0} y={6} width={bx(mean) * p} height={32} fill={tone} opacity={0.85} />
        <rect x={bx(mean) * p} y={6} width={Math.max(0, bx(b.k * sd) * p)} height={32} fill={tone} opacity={0.35} />
      </svg>
      <div style={{ marginTop: 8, fontFamily: font.mono, fontSize: 20, color: color.fg1 }}>
        {mean.toFixed(2)} + {b.k} × {sd.toFixed(2)} = <span style={{ color: tone, fontSize: 26 }}>{(mean + b.k * sd).toFixed(2)} m</span>
      </div>
    </div>
  );

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 40% 45%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text="Question 2 · the landing margin, from the weather table" />
        <QuestionTitle n={2} text="When to light the landing burn?" x={140} y={400} opacity={question} />
        <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0, opacity: rowsIn }}>
          {ticks.map((m) => (
            <line key={m} x1={C.x} y1={py(m)} x2={C.x + C.w} y2={py(m)} stroke={color.line1} strokeWidth={1} />
          ))}
          <line x1={C.x} y1={C.y + C.h} x2={C.x + C.w} y2={C.y + C.h} stroke={color.line2} strokeWidth={1.5} />
          {b.rows.map((x) => (
            <g key={x.dTemp} opacity={day > 0.3 && !inBracket(x.dTemp) ? 0.45 : 1}>
              <line x1={px(x.dTemp)} y1={py(x.mean - x.sd)} x2={px(x.dTemp)} y2={py(x.mean + x.sd)} stroke={color.fg1} strokeWidth={2} />
              <circle cx={px(x.dTemp)} cy={py(x.mean)} r={7} fill={color.fg1} />
            </g>
          ))}
          <g opacity={day}>
            <line x1={px(b.measuredDT)} y1={C.y} x2={px(b.measuredDT)} y2={C.y + C.h} stroke={color.accent} strokeWidth={1.6} strokeDasharray="6 6" />
            <line x1={px(b.bracket[0])} y1={py(r.belief.rows.find((x) => x.dTemp === b.bracket[0])!.mean)} x2={px(b.bracket[1])} y2={py(r.belief.rows.find((x) => x.dTemp === b.bracket[1])!.mean)} stroke={color.accent} strokeWidth={1.5} opacity={0.6} />
            <line x1={px(b.measuredDT)} y1={py(b.informed.mean - b.informed.sd)} x2={px(b.measuredDT)} y2={py(b.informed.mean + b.informed.sd)} stroke={color.accent} strokeWidth={3} />
            <circle cx={px(b.measuredDT)} cy={py(b.informed.mean)} r={9} fill={color.accent} />
          </g>
        </svg>
        <div style={{ position: 'absolute', left: C.x - 20, top: C.y - 52, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: rowsIn }}>
          navigation drift through blackout, m · one point per table row, ± 1σ
        </div>
        {ticks.map((m) => (
          <div key={m} style={{ position: 'absolute', left: C.x - 70, top: py(m) - 12, width: 56, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2, opacity: rowsIn }}>
            {m}
          </div>
        ))}
        {b.rows.map((x) => (
          <div key={x.dTemp} style={{ position: 'absolute', left: px(x.dTemp) - 40, top: C.y + C.h + 14, width: 80, textAlign: 'center', fontFamily: font.mono, fontSize: 18, color: color.fg2, opacity: rowsIn }}>
            {minus(x.dTemp)}
          </div>
        ))}
        <div style={{ position: 'absolute', left: C.x + C.w - 460, top: C.y + C.h + 44, width: 460, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg1, opacity: rowsIn }}>
          the day's temperature departure, K
        </div>
        <div style={{ position: 'absolute', left: px(b.measuredDT) + 18, top: py(b.informed.mean) - 60, fontFamily: font.mono, fontSize: 21, color: color.accent, opacity: day, whiteSpace: 'nowrap' }}>
          today, {minus(b.measuredDT)} K
          <div style={{ fontSize: 19 }}>
            {b.informed.mean.toFixed(2)} ± {b.informed.sd.toFixed(2)} m
          </div>
        </div>

        <div style={{ position: 'absolute', left: B.x, top: C.y - 52, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: informed }}>
          ignition margin = mean + {b.k}σ
        </div>
        {bar(C.y + 30, `guidance sized for today, ${minus(b.measuredDT)} K`, b.informed.mean, b.informed.sd, color.accent, informed)}
        {bar(C.y + 250, 'guidance sized for a standard day, 0 K', b.uninformed.mean, b.uninformed.sd, color.fg1, uninformed)}
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
