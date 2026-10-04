/**
 * Scene 8, the close. First the two results, each from the run: coasting against the light burn's
 * deceleration from the roster, and the two landings' burn-light altitudes from the belief table.
 * Then the descent condensed to one line, the sixteen gate lines as printed, and the end card with
 * the tutorial address and the command that reproduces every number in the cut.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Retro } from '../data/retro';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Close: React.FC<{ retro: Retro; timing: SceneTiming }> = ({ retro: r, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const draw = (from: number, to: number) => interpolate(frame, [from, to], [0, 1], { ...clamp, easing: ease });
  const e = r.events;
  const coast = r.branches.find((b) => b.flown === 0)!;
  const dip = r.branches.filter((b) => b.flown > 0).reduce((a, b) => (b.deceleration < a.deceleration ? b : a));
  const inf = r.landings.informed;
  const uni = r.landings.uninformed;

  const results = fade(frame, at(ph, 0, 0), at(ph, 0, 0.08)) * (1 - fade(frame, at(ph, 2, 0), at(ph, 2, 0.12)));
  const one = draw(at(ph, 0, 0.1), at(ph, 0, 0.6));
  const two = draw(at(ph, 1, 0.1), at(ph, 1, 0.6));
  // One layer at a time: the results leave before the run arrives, the run before the end card.
  const run = fade(frame, at(ph, 2, 0.08), at(ph, 2, 0.16)) * (1 - fade(frame, at(ph, 2, 0.72), at(ph, 2, 0.8)));
  const line = draw(at(ph, 2, 0.08), at(ph, 2, 0.4));
  const gatesIn = (i: number) => fade(frame, at(ph, 2, 0.16) + i * 3, at(ph, 2, 0.16) + i * 3 + 10);
  const card = fade(frame, at(ph, 2, 0.82), at(ph, 2, 0.95));
  const passed = r.gates.filter((g) => g.passed).length;

  // Result one: deceleration bars, coasting against the light burn.
  const D = { x: 160, y: 330, w: 640, max: Math.ceil(coast.deceleration / 2) * 2 };
  const dw = (v: number) => (v / D.max) * D.w * one;
  // Result two: the burn-light altitudes on one metre scale.
  const H = { x: 1100, y: 300, h: 420, max: Math.ceil(inf.printed.lightM / 100) * 100 };
  const hy = (m: number) => H.y + H.h - (m / H.max) * H.h * two;

  // The descent condensed: entry, blackout, ignition and the fork, burn-out, touchdown.
  const y = 300;
  const x = (t: number) => 160 + (t / e.end.t) * 1600;

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 50% 35%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <div style={{ opacity: results }}>
          <Eyebrow text="DeepCausality · the results" />
          <div style={{ position: 'absolute', left: D.x, top: 200, fontFamily: font.sans, fontSize: 36, fontWeight: 500, color: color.fg0, opacity: one }}>
            1 · How hard to burn?
            <div style={{ marginTop: 8, fontFamily: font.mono, fontSize: 21, fontWeight: 400, color: color.fg1 }}>a light burn slows the capsule less than coasting</div>
          </div>
          {[
            { label: 'coasting', v: coast.deceleration, tone: color.fg0 },
            { label: `burning at ${dip.flown.toFixed(2)}`, v: dip.deceleration, tone: color.accent },
          ].map((b, i) => (
            <div key={b.label} style={{ position: 'absolute', left: D.x, top: D.y + i * 130, opacity: one }}>
              <div style={{ fontFamily: font.mono, fontSize: 21, color: b.tone }}>{b.label}</div>
              <svg width={D.w + 10} height={40} style={{ display: 'block', marginTop: 8 }}>
                <rect x={0} y={4} width={dw(b.v)} height={32} fill={b.tone} opacity={0.85} />
              </svg>
              <div style={{ marginTop: 6, fontFamily: font.mono, fontSize: 22, color: b.tone }}>{b.v.toFixed(2)} m/s²</div>
            </div>
          ))}
          <div style={{ position: 'absolute', left: D.x, top: D.y + 270, fontFamily: font.mono, fontSize: 18, color: color.fg2, opacity: one }}>
            deceleration, {r.branchSeconds.toFixed(0)} s after the fork
          </div>

          <div style={{ position: 'absolute', left: H.x, top: 200, fontFamily: font.sans, fontSize: 36, fontWeight: 500, color: color.fg0, opacity: two }}>
            2 · When to light the final burn?
            <div style={{ marginTop: 8, fontFamily: font.mono, fontSize: 21, fontWeight: 400, color: color.fg1 }}>knowing the day lights it {(inf.printed.lightM - uni.printed.lightM).toFixed(2)} m higher</div>
          </div>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0, opacity: two }}>
            <line x1={H.x} y1={H.y + H.h} x2={H.x + 560} y2={H.y + H.h} stroke={color.line2} strokeWidth={2} />
            <line x1={H.x + 120} y1={hy(inf.printed.lightM)} x2={H.x + 120} y2={H.y + H.h} stroke={color.accent} strokeWidth={10} />
            <line x1={H.x + 360} y1={hy(uni.printed.lightM)} x2={H.x + 360} y2={H.y + H.h} stroke={color.fg1} strokeWidth={10} />
          </svg>
          {[
            { at: 120, l: inf, title: 'sized for today', tone: color.accent },
            { at: 360, l: uni, title: 'sized for a standard day', tone: color.fg1 },
          ].map((b) => (
            <div key={b.title} style={{ position: 'absolute', left: H.x + b.at - 160, top: hy(b.l.printed.lightM) - 76, width: 320, textAlign: 'center', whiteSpace: 'nowrap', fontFamily: font.mono, fontSize: 19, color: b.tone, opacity: two }}>
              {b.title}
              <div style={{ fontSize: 24 }}>{b.l.printed.lightM.toFixed(2)} m</div>
            </div>
          ))}
          <div style={{ position: 'absolute', left: H.x, top: H.y + H.h + 20, width: 620, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: two }}>
            where each lights its stopping burn · {(inf.printed.lightM - uni.printed.lightM).toFixed(2)} m apart, for{' '}
            <span style={{ color: color.accent }}>{inf.printed.reserveKg.toFixed(2)} kg</span>
          </div>
        </div>

        <div style={{ opacity: run }}>
          <Eyebrow text="DeepCausality · the descent, end to end" />
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            <rect x={x(e.blackout.from)} y={y - 7} width={x(e.blackout.to) - x(e.blackout.from)} height={14} fill={color.warn} opacity={0.3 * line} />
            <line x1={x(0)} y1={y} x2={x(e.end.t)} y2={y} stroke={color.accent} strokeWidth={3} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - line} />
            {r.branches.map((_, k) => (
              <path
                key={k}
                d={`M${x(e.fork.t)},${y} C${x(e.fork.t) + 40},${y} ${x(e.fork.t) + 80},${y - 14 - k * 12} ${x(e.fork.t) + 140},${y - 14 - k * 12}`}
                fill="none"
                stroke={color.accent}
                strokeOpacity={0.5}
                strokeWidth={1.6}
                pathLength={1}
                strokeDasharray={1}
                strokeDashoffset={1 - interpolate(line, [0.35, 0.7], [0, 1], clamp)}
              />
            ))}
            {[0, e.fork.t, e.subsonic.t, e.end.t].map((t) => (
              <circle key={t} cx={x(t)} cy={y} r={8} fill={color.accent} opacity={line} />
            ))}
          </svg>
          {[
            { t: 0, label: `entry, ${e.entry.altitudeKm.toFixed(0)} km` },
            { t: e.fork.t, label: `ignition, ${e.fork.altitudeKm.toFixed(1)} km` },
            { t: e.subsonic.t, label: `cutoff, ${e.subsonic.altitudeKm.toFixed(1)} km` },
            { t: e.end.t - 60, label: `touchdown, ${inf.contactMs.toFixed(2)} m/s` },
          ].map((l) => (
            <div key={l.label} style={{ position: 'absolute', left: x(l.t) - 10, top: y + 22, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: line, whiteSpace: 'nowrap' }}>
              {l.label}
            </div>
          ))}
          <div style={{ position: 'absolute', left: x(e.fork.t) + 160, top: y - 90, fontFamily: font.mono, fontSize: 19, color: color.accent, opacity: line }}>
            {r.branches.length} burns forked mid-burn
          </div>
          <div style={{ position: 'absolute', left: 160, top: 400, width: 1600, display: 'grid', gridTemplateColumns: '1fr 1fr', columnGap: 64 }}>
            {r.gates.map((g, i) => (
              <div
                key={g.label}
                style={{ display: 'flex', gap: 18, padding: '7px 0', borderBottom: `1px solid ${color.line1}`, fontFamily: font.mono, fontSize: 20, color: color.fg1, opacity: gatesIn(i) }}
              >
                <span style={{ color: g.passed ? color.ok : '#f47174', minWidth: 64 }}>{g.passed ? 'PASS' : 'FAIL'}</span>
                <span>{g.label}</span>
              </div>
            ))}
          </div>
          <div style={{ position: 'absolute', left: 160, top: 800, fontFamily: font.mono, fontSize: 20, color: color.fg2, opacity: gatesIn(r.gates.length) }}>
            {passed} of {r.gates.length} gates pass · copied from output.txt
          </div>
        </div>

        <div style={{ position: 'absolute', left: 160, top: 300, opacity: card }}>
          <div style={{ fontFamily: font.mono, fontSize: 24, letterSpacing: '0.16em', textTransform: 'uppercase', color: color.accent }}>DeepCausality CFD</div>
          <div style={{ marginTop: 24, fontFamily: font.sans, fontSize: 84, fontWeight: 500, letterSpacing: '-0.02em', color: color.fg0, lineHeight: 1.05 }}>
            Five counterfactual burns,
            <br />
            two counterfactual landings.
          </div>
          <div style={{ marginTop: 40, fontFamily: font.mono, fontSize: 30, color: color.fg0 }}>cfd.deepcausality.com/tutorial/stage-3-retropulsion</div>
          <div style={{ marginTop: 20, fontFamily: font.mono, fontSize: 23, color: color.fg1 }}>
            cargo run --release -p avionics_examples --example plasma_blackout_retropulsion
          </div>
          <div style={{ marginTop: 14, fontFamily: font.mono, fontSize: 19, color: color.fg2 }}>
            the whole descent: {r.runSeconds.toFixed(1)} s on an Apple M3 Max laptop · open source, in Rust
          </div>
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
