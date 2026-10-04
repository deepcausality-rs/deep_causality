/**
 * The seventeen branches after the pause: sideways distance from the no-bank path against time,
 * drawn from `corridor_branch_trace.csv`. The axes come first, then round 1, the clamped branch,
 * the coarse winner, round 2 around it, and the committed branch. All lengths in stage pixels.
 */
import { interpolate } from 'remotion';
import type { Corridor } from '../data/corridor';
import { color, font } from '@cfd-video/shared';

export interface FanProgress {
  axes: number;
  roundOne: number;
  clamp: number;
  best: number;
  roundTwo: number;
  committed: number;
}

const clamp01 = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;

export const BranchFan: React.FC<{ corridor: Corridor; p: FanProgress; x: number; y: number; w: number; h: number }> = ({
  corridor,
  p,
  x,
  y,
  w,
  h,
}) => {
  const left = 64;
  const bottom = 56;
  const pw = w - left - 150;
  const ph = h - bottom - 40;
  const crMax = Math.ceil(Math.max(...corridor.branches.map((b) => b.end.cross)) / 10) * 10;
  const tMax = corridor.branchSeconds;
  const px = (t: number) => left + (t / tMax) * pw;
  const py = (m: number) => 40 + ph - (m / crMax) * ph;
  const path = (b: Corridor['branches'][number]) =>
    [`M${px(0)},${py(0)}`, ...b.t.map((t, i) => `L${px(t).toFixed(1)},${py(b.crossRange[i]).toFixed(1)}`)].join(' ');

  const roundOne = corridor.branches.filter((b) => b.round === 1).sort((a, b) => a.bank - b.bank);
  const committed = corridor.branches.find((b) => b.round === 2 && b.bank === corridor.committed.bank)!;
  const roundTwo = corridor.branches.filter((b) => b.round === 2 && b !== committed);
  const coarseBest = roundOne.reduce((a, b) => (b.miss < a.miss ? b : a));
  const clamped = roundOne.find((b) => b.flown < b.bank - 0.5);

  const draw = (v: number) => ({ strokeDasharray: 1, strokeDashoffset: 1 - v });

  return (
    <div style={{ position: 'absolute', left: x, top: y, width: w, height: h, opacity: p.axes }}>
      <svg width={w} height={h} style={{ position: 'absolute', inset: 0, overflow: 'visible' }}>
        {/* Axes and the aim. */}
        <line x1={px(0)} y1={py(0)} x2={px(tMax)} y2={py(0)} stroke={color.line2} strokeWidth={1.5} />
        {[10, 20, 30, 40, 50].filter((m) => m <= crMax).map((m) => (
          <line key={m} x1={px(0)} y1={py(m)} x2={px(tMax)} y2={py(m)} stroke={color.line1} strokeWidth={1} />
        ))}
        <line x1={px(tMax * 0.82)} y1={py(corridor.aimOffsetM)} x2={px(tMax) + 8} y2={py(corridor.aimOffsetM)} stroke={color.fg1} strokeWidth={2} strokeDasharray="6 6" />

        {roundOne.map((b) => (
          <path
            key={`r1-${b.bank}`}
            d={path(b)}
            pathLength={1}
            fill="none"
            stroke={b === clamped && p.clamp > 0 && p.best < 0.2 ? color.fg1 : color.fg2}
            strokeWidth={b === clamped && p.clamp > 0 && p.best < 0.2 ? 2.8 : 2}
            strokeLinecap="round"
            opacity={interpolate(p.roundTwo, [0, 1], [1, 0.55], clamp01)}
            style={draw(p.roundOne)}
          />
        ))}
        <path d={path(coarseBest)} pathLength={1} fill="none" stroke={color.fg0} strokeWidth={2.8} strokeLinecap="round" opacity={p.best} style={draw(p.roundOne)} />
        {roundTwo.map((b, i) => (
          <path
            key={`r2-${b.bank}`}
            d={path(b)}
            pathLength={1}
            fill="none"
            stroke={color.accent}
            strokeOpacity={0.5}
            strokeWidth={1.6}
            strokeLinecap="round"
            style={draw(interpolate(p.roundTwo, [i * 0.04, 0.6 + i * 0.04], [0, 1], clamp01))}
          />
        ))}
        <path d={path(committed)} pathLength={1} fill="none" stroke={color.accent} strokeWidth={4} strokeLinecap="round" style={draw(p.committed)} />
        <circle cx={px(0)} cy={py(0)} r={8} fill={color.accent} />
      </svg>

      {/* Labels in HTML, in the design system's type. */}
      <div style={{ position: 'absolute', left: 0, top: 0, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>
        sideways from the no-bank path, m
      </div>
      {[0, 10, 20, 30, 40, 50].filter((m) => m <= crMax).map((m) => (
        <div key={m} style={{ position: 'absolute', left: 0, top: py(m) - 12, width: left - 14, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
          {m}
        </div>
      ))}
      <div style={{ position: 'absolute', left: px(0) - 40, top: py(0) + 16, fontFamily: font.mono, fontSize: 18, color: color.accent }}>pause</div>
      <div style={{ position: 'absolute', left: px(tMax) - 70, top: py(0) + 16, fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
        +{tMax.toFixed(0)} s
      </div>
      <div
        style={{
          position: 'absolute',
          left: px(tMax * 0.82),
          top: py(corridor.aimOffsetM) - 34,
          fontFamily: font.mono,
          fontSize: 18,
          color: color.fg1,
        }}
      >
        aim · {corridor.aimOffsetM} m
      </div>
      {roundOne.map((b) => {
        const endY = py(b.end.cross);
        const o = interpolate(p.roundOne, [0.85, 1], [0, 1], clamp01);
        return (
          <div
            key={`l-${b.bank}`}
            style={{
              position: 'absolute',
              left: px(tMax) + 16,
              top: endY - 13,
              fontFamily: font.mono,
              fontSize: 20,
              color: b === coarseBest && p.best > 0.5 ? color.fg0 : color.fg1,
              opacity: o,
              whiteSpace: 'nowrap',
            }}
          >
            {b.bank}° <span style={{ color: color.fg2 }}>{b.miss.toFixed(1)} m</span>
          </div>
        );
      })}
      {clamped && (
        <div
          style={{
            position: 'absolute',
            left: px(0) + 24,
            top: py(crMax * 0.9),
            fontFamily: font.mono,
            fontSize: 19,
            color: color.fg0,
            opacity: p.clamp,
            padding: '6px 12px',
            border: `1px solid ${color.line2}`,
            borderRadius: 6,
            background: 'rgba(11,17,24,0.85)',
          }}
        >
          {clamped.bank}° commanded · gate flies {clamped.flown.toFixed(1)}°
        </div>
      )}
    </div>
  );
};
