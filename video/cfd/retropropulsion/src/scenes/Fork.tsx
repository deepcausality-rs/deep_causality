/**
 * Scene 5, the fork: the five throttle branches, each flown 12 s from the paused state, drawn from
 * `retropropulsion_branch_trace.csv` and the roster table. Left, the velocity each sheds over its
 * continuation; right, the deceleration each ends on against the throttle it flew, with coasting as
 * the reference. Each step lands on its narration phrase.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Branch, Retro } from '../data/retro';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);
const fail = (msg: string): never => {
  throw new Error(`fork scene: ${msg}`);
};

export const Fork: React.FC<{ retro: Retro; timing: SceneTiming }> = ({ retro: r, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const span = (i: number, a = 0, b = 1) => interpolate(frame, [at(ph, i, a), at(ph, i, b)], [0, 1], { ...clamp, easing: ease });
  const branches = [...r.branches].sort((a, b) => a.flown - b.flown);
  const coast = branches.find((b) => b.flown === 0) ?? fail('no branch flies throttle 0, the coasting reference');
  const burning = branches.filter((b) => b.flown > 0);
  if (burning.length === 0) fail('no branch flies a throttle above 0');
  const dip = burning.reduce((a, b) => (b.deceleration < a.deceleration ? b : a));
  const shown = (b: Branch) => (b === coast ? span(1, 0, 0.4) : b === dip ? span(2, 0, 0.4) : span(4, 0.05, 0.5));
  const tone = (b: Branch) => (b === coast ? color.fg0 : b === dip ? color.accent : color.fg1);
  const name = (b: Branch) => (b.flown === 0 ? 'coast' : b.commanded.toFixed(2));

  // Left: velocity shed since the fork.
  const L = { x: 130, y: 250, w: 640, h: 470 };
  const dvMax = Math.ceil(Math.max(...branches.map((b) => b.dvActual)) / 50) * 50;
  const lx = (t: number) => L.x + (t / r.branchSeconds) * L.w;
  const ly = (dv: number) => L.y + L.h - (dv / dvMax) * L.h;
  const curve = (b: Branch) => b.t.map((t, i) => `${i === 0 ? 'M' : 'L'}${lx(t).toFixed(1)},${ly(b.dv[i]).toFixed(1)}`).join(' ');
  const draw = span(0, 0.1, 0.95);

  // Right: deceleration against the throttle flown.
  const R = { x: 1100, y: 250, w: 640, h: 470 };
  const thrMax = Math.ceil(Math.max(...branches.map((b) => b.commanded)) * 10) / 10;
  const dMin = Math.floor(Math.min(...branches.map((b) => b.deceleration)) - 1);
  const dMax = Math.ceil(Math.max(...branches.map((b) => b.deceleration)) + 1);
  const rx = (thr: number) => R.x + (thr / thrMax) * R.w;
  const ry = (d: number) => R.y + R.h - ((d - dMin) / (dMax - dMin)) * R.h;
  const line = branches.map((b, i) => `${i === 0 ? 'M' : 'L'}${rx(b.flown).toFixed(1)},${ry(b.deceleration).toFixed(1)}`).join(' ');
  const hard = branches[branches.length - 1];
  const kept = span(3, 0.1, 0.5);
  const note = span(5, 0, 0.3);

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 50% 40%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text={`Answer 1 · ${r.branches.length} counterfactual burns`} />
        <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0, overflow: 'visible' }}>
          {/* Left axes and curves. */}
          {Array.from({ length: dvMax / 50 + 1 }, (_, i) => i * 50).map((v) => (
            <line key={v} x1={L.x} y1={ly(v)} x2={L.x + L.w} y2={ly(v)} stroke={color.line1} strokeWidth={1} />
          ))}
          <line x1={L.x} y1={ly(0)} x2={L.x + L.w} y2={ly(0)} stroke={color.line2} strokeWidth={1.5} />
          {branches.map((b) => (
            <path
              key={b.name}
              d={curve(b)}
              fill="none"
              stroke={tone(b)}
              strokeWidth={b === coast || b === dip ? 3.2 : 2}
              strokeLinecap="round"
              pathLength={1}
              strokeDasharray={1}
              strokeDashoffset={1 - draw}
            />
          ))}
          {/* Right axes, coasting reference, points. */}
          {Array.from({ length: dMax - dMin + 1 }, (_, i) => dMin + i).filter((d) => d % 2 === 0).map((d) => (
            <line key={d} x1={R.x} y1={ry(d)} x2={R.x + R.w} y2={ry(d)} stroke={color.line1} strokeWidth={1} />
          ))}
          <line x1={R.x} y1={R.y + R.h} x2={R.x + R.w} y2={R.y + R.h} stroke={color.line2} strokeWidth={1.5} />
          <line x1={R.x} y1={ry(coast.deceleration)} x2={R.x + R.w} y2={ry(coast.deceleration)} stroke={color.fg0} strokeWidth={1.6} strokeDasharray="7 7" opacity={span(1, 0.2, 0.6)} />
          <path d={line} fill="none" stroke={color.fg2} strokeWidth={1.5} opacity={span(4, 0.4, 0.9)} />
          {branches.map((b) => (
            <circle key={b.name} cx={rx(b.flown)} cy={ry(b.deceleration)} r={b === dip ? 11 : 8} fill={tone(b)} opacity={shown(b)} />
          ))}
          <g opacity={span(4, 0.3, 0.7)}>
            <circle cx={rx(hard.commanded)} cy={ry(hard.deceleration)} r={8} fill="none" stroke={color.fg2} strokeWidth={2} strokeDasharray="3 3" />
            <line x1={rx(hard.commanded) - 10} y1={ry(hard.deceleration)} x2={rx(hard.flown) + 14} y2={ry(hard.deceleration)} stroke={color.fg2} strokeWidth={1.5} />
          </g>
        </svg>

        {/* Left labels. */}
        <div style={{ position: 'absolute', left: L.x - 30, top: L.y - 54, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>velocity shed since the fork, m/s</div>
        {Array.from({ length: dvMax / 50 + 1 }, (_, i) => i * 50).map((v) => (
          <div key={v} style={{ position: 'absolute', left: L.x - 76, top: ly(v) - 12, width: 60, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
            {v}
          </div>
        ))}
        <div style={{ position: 'absolute', left: L.x - 10, top: L.y + L.h + 14, fontFamily: font.mono, fontSize: 18, color: color.accent }}>fork</div>
        <div style={{ position: 'absolute', left: L.x + L.w - 80, top: L.y + L.h + 14, width: 80, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
          +{r.branchSeconds.toFixed(0)} s
        </div>
        {branches.map((b) => (
          <div
            key={b.name}
            style={{
              position: 'absolute',
              left: lx(r.branchSeconds) + 16,
              top: ly(b.dvActual) - 13,
              fontFamily: font.mono,
              fontSize: 19,
              color: tone(b),
              opacity: fade(frame, at(ph, 0, 0.8), at(ph, 0, 0.95)),
              whiteSpace: 'nowrap',
            }}
          >
            {name(b)} <span style={{ color: color.fg2 }}>{b.dvActual.toFixed(1)}</span>
          </div>
        ))}

        {/* Right labels. */}
        <div style={{ position: 'absolute', left: R.x - 30, top: R.y - 54, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: span(1, 0, 0.3) }}>
          deceleration at the end of the branch, m/s²
        </div>
        {Array.from({ length: dMax - dMin + 1 }, (_, i) => dMin + i).filter((d) => d % 2 === 0).map((d) => (
          <div key={d} style={{ position: 'absolute', left: R.x - 64, top: ry(d) - 12, width: 48, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2, opacity: span(1, 0, 0.3) }}>
            {d}
          </div>
        ))}
        {[0, 0.2, 0.4, 0.6, 0.8].map((v) => (
          <div key={v} style={{ position: 'absolute', left: rx(v) - 30, top: R.y + R.h + 14, width: 60, textAlign: 'center', fontFamily: font.mono, fontSize: 18, color: color.fg2, opacity: span(1, 0, 0.3) }}>
            {v.toFixed(1)}
          </div>
        ))}
        <div style={{ position: 'absolute', left: R.x + R.w - 220, top: R.y + R.h + 44, width: 220, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg1, opacity: span(1, 0, 0.3) }}>
          throttle flown
        </div>
        <div style={{ position: 'absolute', left: R.x + R.w - 260, top: ry(coast.deceleration) - 36, width: 260, textAlign: 'right', fontFamily: font.mono, fontSize: 19, color: color.fg0, opacity: span(1, 0.3, 0.7) }}>
          coasting: {coast.deceleration.toFixed(2)}
        </div>
        <div style={{ position: 'absolute', left: rx(dip.flown) + 20, top: ry(dip.deceleration) + 6, fontFamily: font.mono, fontSize: 21, color: color.accent, opacity: span(2, 0.3, 0.7), whiteSpace: 'nowrap' }}>
          {dip.flown.toFixed(2)}: {dip.deceleration.toFixed(2)}, less than coasting
        </div>
        <div style={{ position: 'absolute', left: R.x - 140, top: R.y + R.h + 78, width: 120, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg1, opacity: kept }}>
          drag kept
        </div>
        {burning.map((b) => (
          <div
            key={b.name}
            style={{
              position: 'absolute',
              left: rx(b.flown) - 40,
              top: R.y + R.h + 78,
              width: 80,
              textAlign: 'center',
              fontFamily: font.mono,
              fontSize: 19,
              color: b === dip ? color.accent : color.fg1,
              opacity: kept * shown(b),
            }}
          >
            {b.preserved !== null ? (b.preserved < 0 ? `−${Math.abs(b.preserved).toFixed(2)}` : b.preserved.toFixed(2)) : '—'}
          </div>
        ))}
        <div style={{ position: 'absolute', left: rx(hard.flown) - 420, top: ry(hard.deceleration) - 14, width: 390, textAlign: 'right', fontFamily: font.mono, fontSize: 17, color: color.fg2, opacity: span(4, 0.4, 0.8) }}>
          {hard.commanded.toFixed(2)} commanded · envelope flies {hard.flown.toFixed(2)}
        </div>
        <div style={{ position: 'absolute', left: R.x - 140, top: R.y + R.h + 116, width: 900, fontFamily: font.mono, fontSize: 17, color: note > 0.5 ? color.fg0 : color.fg2, opacity: kept }}>
          drag kept: the Jarvinen and Adams (1970) correlation at each branch's thrust coefficient, not the simulated plume
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
