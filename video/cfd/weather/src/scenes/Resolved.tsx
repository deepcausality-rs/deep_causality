/**
 * Scene 6, resolved: is the difference real, or receiver noise? Left, every draw's largest error
 * while GPS is lost, one column per world, with the mean and one standard deviation; the
 * polar-standard separation against their combined sigma is gate (4b). Right, every draw's terminal
 * error after GPS returns, under gate (5)'s ceiling. All values from `weather_draws.csv`.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Weather, World } from '../data/weather';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Resolved: React.FC<{ weather: Weather; timing: SceneTiming }> = ({ weather: w, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const span = (i: number, a = 0, b = 1) => interpolate(frame, [at(ph, i, a), at(ph, i, b)], [0, 1], { ...clamp, easing: ease });
  const base = w.worlds[0];
  const polar = w.worlds.find((x) => x.name === 'polar_winter')!;
  const cols = [...w.worlds].sort((a, b) => a.driftMean - b.driftMean);
  const n = w.drawsPerWorld;
  const jitter = (d: number) => (d - (n - 1) / 2) * 7;

  // Left: the draws per world.
  const L = { x: 200, y: 330, w: 860, h: 420 };
  const drifts = w.worlds.flatMap((x) => x.draws.map((d) => d.drift));
  const lo = Math.floor(Math.min(...drifts) / 5) * 5;
  const hi = Math.ceil(Math.max(...drifts) / 5) * 5;
  const cx = (k: number) => L.x + ((k + 0.5) / cols.length) * L.w;
  const ly = (m: number) => L.y + L.h - ((m - lo) / (hi - lo)) * L.h;
  const shown = (k: number) => span(1, 0.05 + 0.1 * k, 0.2 + 0.1 * k);
  const focus = span(2, 0.05, 0.3);
  const lead = (x: World) => x === base || x === polar;
  const dim = (x: World) => 1 - (lead(x) ? 0 : 0.6 * focus);
  const tone = (x: World) => (x === polar ? color.accent : x === base ? color.fg0 : color.fg1);
  const bx = L.x + L.w + 30;
  const gap = span(2, 0.3, 0.6);
  const ticks = Array.from({ length: (hi - lo) / 5 + 1 }, (_, i) => lo + i * 5);

  // Right: every draw's terminal error.
  const T = { x: 1490, y: 330, w: 300, h: 420 };
  const ty = (m: number) => T.y + T.h - (m / w.reacquireM) * T.h;
  const terminal = span(3, 0.05, 0.35);
  const worst = Math.max(...w.worlds.map((x) => x.terminalMax));
  const worstWorld = w.worlds.find((x) => x.terminalMax === worst)!;

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 40% 45%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text="Answer 2 · real, or receiver noise?" />
        <div style={{ position: 'absolute', left: L.x - 60, top: 200, fontFamily: font.sans, fontSize: 44, fontWeight: 500, color: color.fg0, opacity: fade(frame, 4, 20) }}>
          Real, or receiver noise?
        </div>
        <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
          {ticks.map((m) => (
            <line key={m} x1={L.x} y1={ly(m)} x2={L.x + L.w} y2={ly(m)} stroke={color.line1} strokeWidth={1} />
          ))}
          {cols.map((x, k) => (
            <g key={x.name} opacity={shown(k) * dim(x)}>
              <rect x={cx(k) - 40} y={ly(x.driftMean + x.driftSd)} width={80} height={ly(x.driftMean - x.driftSd) - ly(x.driftMean + x.driftSd)} fill={tone(x)} opacity={0.14} />
              <line x1={cx(k) - 40} y1={ly(x.driftMean)} x2={cx(k) + 40} y2={ly(x.driftMean)} stroke={tone(x)} strokeWidth={2.5} />
              {x.draws.map((d) => (
                <circle key={d.draw} cx={cx(k) + jitter(d.draw)} cy={ly(d.drift)} r={6} fill={tone(x)} opacity={0.9} />
              ))}
            </g>
          ))}
          <g opacity={gap}>
            {[base, polar].map((x) => (
              <line key={x.name} x1={cx(cols.indexOf(x)) + 48} y1={ly(x.driftMean)} x2={bx} y2={ly(x.driftMean)} stroke={tone(x)} strokeWidth={1.2} strokeDasharray="4 6" />
            ))}
            <path d={`M${bx},${ly(base.driftMean)} h 14 V ${ly(polar.driftMean)} h -14`} fill="none" stroke={color.accent} strokeWidth={1.8} />
          </g>
        </svg>
        {ticks.map((m) => (
          <div key={m} style={{ position: 'absolute', left: L.x - 70, top: ly(m) - 12, width: 54, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
            {m}
          </div>
        ))}
        {cols.map((x, k) => (
          <div key={x.name} style={{ position: 'absolute', left: cx(k) - 80, top: L.y + L.h + 14, width: 160, textAlign: 'center', fontFamily: font.sans, fontSize: 21, color: tone(x), opacity: shown(k) * dim(x) }}>
            {x.label}
          </div>
        ))}
        <div style={{ position: 'absolute', left: L.x - 30, top: L.y - 46, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>
          largest error while GPS is lost, m · one dot per draw
        </div>
        <div
          style={{
            position: 'absolute',
            left: bx + 26,
            top: (ly(base.driftMean) + ly(polar.driftMean)) / 2 - 44,
            fontFamily: font.mono,
            fontSize: 22,
            lineHeight: 1.45,
            color: color.fg1,
            opacity: gap,
            whiteSpace: 'nowrap',
          }}
        >
          <span style={{ color: color.accent, fontSize: 28 }}>{w.resolved.separation.toFixed(2)} m</span> apart
          <br />= <span style={{ color: color.accent }}>{w.resolved.ratio.toFixed(1)}</span> × {w.resolved.sigma.toFixed(2)} m
          <br />
          <span style={{ fontSize: 18, color: color.fg2 }}>combined sigma</span>
        </div>

        <div style={{ opacity: terminal }}>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            <line x1={T.x} y1={ty(w.reacquireM)} x2={T.x + T.w} y2={ty(w.reacquireM)} stroke={color.fg0} strokeWidth={1.6} strokeDasharray="6 6" />
            <line x1={T.x} y1={ty(0)} x2={T.x + T.w} y2={ty(0)} stroke={color.line2} strokeWidth={2} />
            {w.worlds.map((x, k) =>
              x.draws.map((d) => (
                <circle
                  key={`${x.name}${d.draw}`}
                  cx={T.x + ((k + 0.5) / w.worlds.length) * T.w + jitter(d.draw) * 0.6}
                  cy={ty(d.terminal)}
                  r={5}
                  fill={color.ok}
                  opacity={0.85}
                />
              ))
            )}
          </svg>
          <div style={{ position: 'absolute', left: T.x - 20, top: T.y - 46, width: T.w + 40, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>error after GPS returns, m</div>
          <div style={{ position: 'absolute', left: T.x, top: ty(w.reacquireM) + 8, width: T.w, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg0 }}>
            gate (5): under {w.reacquireM.toFixed(1)} m
          </div>
          <div style={{ position: 'absolute', left: T.x - 70, top: ty(0) - 12, width: 54, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>0</div>
          <div
            style={{
              position: 'absolute',
              left: T.x + ((w.worlds.indexOf(worstWorld) + 0.5) / w.worlds.length) * T.w - 120,
              top: ty(worst) - 44,
              width: 240,
              textAlign: 'center',
              fontFamily: font.mono,
              fontSize: 20,
              color: color.ok,
            }}
          >
            worst {worst.toFixed(2)} m
          </div>
          <div style={{ position: 'absolute', left: T.x - 20, top: T.y + T.h + 14, width: T.w + 40, textAlign: 'center', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
            all {w.worlds.length * n} descents
          </div>
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
