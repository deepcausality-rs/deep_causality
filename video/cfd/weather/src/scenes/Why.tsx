/**
 * Scene 5, why: first the window's share, the squared ratio of the longest to the shortest blackout
 * against polar winter's measured factor. Then the instrument: the bias departure each world flies,
 * 1 + k·|dT| from the table, against the 1.00 the filter assumes. Last, the prediction: the
 * baseline's mean drift scaled by each world's departure and squared dwell, against the measured
 * mean and its standard deviation.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Weather, World } from '../data/weather';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, Stage } from '@cfd-video/shared';
import { kelvin } from './Worlds';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Why: React.FC<{ weather: Weather; timing: SceneTiming }> = ({ weather: w, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const span = (i: number, a = 0, b = 1) => interpolate(frame, [at(ph, i, a), at(ph, i, b)], [0, 1], { ...clamp, easing: ease });
  const base = w.worlds[0];
  const polar = w.worlds.find((x) => x.name === 'polar_winter')!;
  const dwells = w.worlds.map((x) => x.dwell);
  const windowFactor = (Math.max(...dwells) / Math.min(...dwells)) ** 2;

  // Phrase 0: the window's factor against the measured one, on one scale.
  const scale = fade(frame, at(ph, 0, 0.05), at(ph, 0, 0.2)) * (1 - fade(frame, at(ph, 1, 0), at(ph, 1, 0.12)));
  const S = { x: 260, y: 480, w: 1400, lo: 1, hi: 1.5 };
  const sx = (f: number) => S.x + ((f - S.lo) / (S.hi - S.lo)) * S.w;
  const grow = (f: number, p: number) => sx(S.lo + (f - S.lo) * p);
  const windowBar = span(0, 0.3, 0.55);
  const measuredBar = span(0, 0.6, 0.85);

  // Phrases 1-2: the departure each world flies against its temperature offset.
  const V = { x: 190, y: 330, w: 700, h: 400, tLo: -45, tHi: 25, dLo: 0.95, dHi: 1.45 };
  const vx = (dT: number) => V.x + ((dT - V.tLo) / (V.tHi - V.tLo)) * V.w;
  const vy = (d: number) => V.y + V.h - ((d - V.dLo) / (V.dHi - V.dLo)) * V.h;
  const instrument = fade(frame, at(ph, 1, 0.05), at(ph, 1, 0.2));
  const vLine = span(1, 0.15, 0.55);
  const assumed = span(1, 0.6, 0.85);
  const polarOn = span(2, 0.1, 0.4);
  // Labels sit inside the V, clear of the line: above-right on the cold branch, above-left on the hot
  // one, beside the two points next to the calibration point, and under the calibration point.
  const labelAt = (x: World): { dx: number; dy: number; anchor: 'start' | 'end' | 'middle' } =>
    x.dTemp === 0
      ? { dx: 0, dy: 34, anchor: 'middle' }
      : Math.abs(x.dTemp) <= 5
        ? { dx: x.dTemp < 0 ? -16 : 16, dy: 14, anchor: x.dTemp < 0 ? 'end' : 'start' }
        : { dx: x.dTemp < 0 ? 16 : -16, dy: -14, anchor: x.dTemp < 0 ? 'start' : 'end' };

  // Phrase 3: predicted against measured.
  const P = { x: 1180, y: 330, s: 400 };
  const all = w.worlds.flatMap((x) => [x.predicted, x.driftMean - x.driftSd, x.driftMean + x.driftSd]);
  const lo = Math.floor(Math.min(...all) / 5) * 5;
  const hi = Math.ceil(Math.max(...all) / 5) * 5;
  const px = (m: number) => P.x + ((m - lo) / (hi - lo)) * P.s;
  const py = (m: number) => P.y + P.s - ((m - lo) / (hi - lo)) * P.s;
  const parity = fade(frame, at(ph, 3, 0.0), at(ph, 3, 0.15));
  const dots = (k: number) => span(3, 0.15 + 0.06 * k, 0.3 + 0.06 * k);
  const verdict = span(3, 0.6, 0.85);
  const ptTone = (x: World) => (x === polar ? color.accent : x === base ? color.fg0 : color.fg1);
  const ticks = Array.from({ length: (hi - lo) / 5 + 1 }, (_, i) => lo + i * 5);

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 45% 45%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text="Answer 2 · why polar winter drifts further" />

        <div style={{ opacity: scale }}>
          <div style={{ position: 'absolute', left: S.x, top: 300, fontFamily: font.sans, fontSize: 44, fontWeight: 500, color: color.fg0 }}>Not the window</div>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            <line x1={S.x} y1={S.y + 120} x2={S.x + S.w} y2={S.y + 120} stroke={color.line2} strokeWidth={2} />
            {[1, 1.1, 1.2, 1.3, 1.4, 1.5].map((f) => (
              <line key={f} x1={sx(f)} y1={S.y + 112} x2={sx(f)} y2={S.y + 128} stroke={color.line2} strokeWidth={2} />
            ))}
            <rect x={S.x} y={S.y} width={grow(windowFactor, windowBar) - S.x} height={34} fill={color.fg1} opacity={0.8} />
            <rect x={S.x} y={S.y + 56} width={grow(w.coldFactor, measuredBar) - S.x} height={34} fill={color.accent} opacity={0.85} />
          </svg>
          {[1, 1.1, 1.2, 1.3, 1.4, 1.5].map((f) => (
            <div key={f} style={{ position: 'absolute', left: sx(f) - 40, top: S.y + 136, width: 80, textAlign: 'center', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
              {f.toFixed(1)}×
            </div>
          ))}
          <div style={{ position: 'absolute', left: grow(windowFactor, windowBar) + 16, top: S.y + 2, fontFamily: font.mono, fontSize: 21, color: color.fg1, opacity: windowBar, whiteSpace: 'nowrap' }}>
            {windowFactor.toFixed(2)}× · longest blackout over shortest, squared
          </div>
          <div style={{ position: 'absolute', left: grow(w.coldFactor, measuredBar) + 16, top: S.y + 58, fontFamily: font.mono, fontSize: 21, color: color.accent, opacity: measuredBar, whiteSpace: 'nowrap' }}>
            {w.coldFactor.toFixed(2)}× · polar winter over standard day, measured
          </div>
        </div>

        <div style={{ opacity: instrument }}>
          <div style={{ position: 'absolute', left: V.x - 40, top: 220, fontFamily: font.sans, fontSize: 40, fontWeight: 500, color: color.fg0 }}>The instrument</div>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0, overflow: 'visible' }}>
            {[1, 1.1, 1.2, 1.3, 1.4].map((d) => (
              <line key={d} x1={V.x} y1={vy(d)} x2={V.x + V.w} y2={vy(d)} stroke={color.line1} strokeWidth={1} />
            ))}
            <line x1={vx(0)} y1={V.y} x2={vx(0)} y2={V.y + V.h} stroke={color.line1} strokeWidth={1} />
            <line x1={V.x} y1={V.y + V.h} x2={V.x + V.w} y2={V.y + V.h} stroke={color.line2} strokeWidth={2} />
            <path
              d={`M${vx(V.tLo)},${vy(1 + w.imuPerK * Math.abs(V.tLo))} L${vx(0)},${vy(1)} L${vx(V.tHi)},${vy(1 + w.imuPerK * V.tHi)}`}
              fill="none"
              stroke={color.fg1}
              strokeWidth={2}
              pathLength={1}
              strokeDasharray={1}
              strokeDashoffset={1 - vLine}
            />
            <line x1={V.x} y1={vy(1)} x2={V.x + V.w} y2={vy(1)} stroke={color.fg0} strokeWidth={1.6} strokeDasharray="8 8" opacity={assumed} />
            {w.worlds.map((x) => (
              <circle key={x.name} cx={vx(x.dTemp)} cy={vy(x.departure)} r={x === polar ? 10 * (1 + 0.3 * polarOn) : 8} fill={x === polar ? (polarOn > 0 ? color.accent : color.fg0) : color.fg0} opacity={vLine} />
            ))}
            <line x1={vx(polar.dTemp)} y1={vy(polar.departure)} x2={vx(polar.dTemp)} y2={V.y + V.h} stroke={color.accent} strokeWidth={1.5} strokeDasharray="4 6" opacity={polarOn} />
            {w.worlds.map((x) => {
              const side = labelAt(x);
              return (
                <text key={`l${x.name}`} x={vx(x.dTemp) + side.dx} y={vy(x.departure) + side.dy} textAnchor={side.anchor} fontFamily={font.mono} fontSize={18} fill={x === polar ? color.accent : color.fg1} opacity={vLine}>
                  {x.label}
                </text>
              );
            })}
          </svg>
          {[1, 1.1, 1.2, 1.3, 1.4].map((d) => (
            <div key={d} style={{ position: 'absolute', left: V.x - 80, top: vy(d) - 12, width: 64, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
              {d.toFixed(2)}
            </div>
          ))}
          {[-40, -20, 0, 20].map((dT) => (
            <div key={dT} style={{ position: 'absolute', left: vx(dT) - 40, top: V.y + V.h + 12, width: 80, textAlign: 'center', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
              {kelvin(dT)}
            </div>
          ))}
          <div style={{ position: 'absolute', left: V.x - 40, top: V.y - 44, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>accelerometer bias flown, × calibrated</div>
          <div style={{ position: 'absolute', left: V.x + V.w - 440, top: V.y + V.h + 44, width: 440, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg1 }}>
            temperature offset from standard →
          </div>
          <div style={{ position: 'absolute', left: vx(polar.dTemp) + 12, top: vy(1) + 8, width: 330, fontFamily: font.mono, fontSize: 18, color: color.fg0, opacity: assumed }}>
            the filter assumes 1.00
          </div>
          <div style={{ position: 'absolute', left: vx(polar.dTemp) + 160, top: vy(polar.departure) - 34, fontFamily: font.mono, fontSize: 22, color: color.accent, opacity: polarOn, whiteSpace: 'nowrap' }}>
            {Math.abs(polar.dTemp)} K out · {polar.departure.toFixed(2)}×
          </div>
        </div>

        <div style={{ opacity: parity }}>
          <div style={{ position: 'absolute', left: P.x - 40, top: 220, fontFamily: font.sans, fontSize: 40, fontWeight: 500, color: color.fg0 }}>The prediction</div>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            {ticks.map((m) => (
              <g key={m}>
                <line x1={P.x} y1={py(m)} x2={P.x + P.s} y2={py(m)} stroke={color.line1} strokeWidth={1} />
                <line x1={px(m)} y1={P.y} x2={px(m)} y2={P.y + P.s} stroke={color.line1} strokeWidth={1} />
              </g>
            ))}
            <line x1={px(lo)} y1={py(lo)} x2={px(hi)} y2={py(hi)} stroke={color.fg2} strokeWidth={1.6} strokeDasharray="6 6" />
            {w.worlds.map((x, k) => (
              <g key={x.name} opacity={dots(k)}>
                <line x1={px(x.predicted)} y1={py(x.driftMean - x.driftSd)} x2={px(x.predicted)} y2={py(x.driftMean + x.driftSd)} stroke={ptTone(x)} strokeWidth={1.5} />
                <circle cx={px(x.predicted)} cy={py(x.driftMean)} r={8} fill={ptTone(x)} />
              </g>
            ))}
          </svg>
          {ticks.map((m) => (
            <div key={`y${m}`} style={{ position: 'absolute', left: P.x - 70, top: py(m) - 12, width: 54, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
              {m}
            </div>
          ))}
          {ticks.map((m) => (
            <div key={`x${m}`} style={{ position: 'absolute', left: px(m) - 30, top: P.y + P.s + 10, width: 60, textAlign: 'center', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
              {m}
            </div>
          ))}
          <div style={{ position: 'absolute', left: P.x - 40, top: P.y - 44, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>measured drift, m (mean ± 1 sd)</div>
          <div style={{ position: 'absolute', left: P.x + P.s - 300, top: P.y + P.s + 40, width: 300, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg1 }}>
            predicted drift, m →
          </div>
          {[base, polar].map((x) => (
            <div
              key={`p${x.name}`}
              style={{
                position: 'absolute',
                left: px(x.predicted) + (x === base ? 30 : 16),
                top: py(x.driftMean) + (x === base ? 22 : 4),
                fontFamily: font.mono,
                fontSize: 18,
                color: ptTone(x),
                opacity: dots(5),
                whiteSpace: 'nowrap',
              }}
            >
              {x.label}
            </div>
          ))}
          <div style={{ position: 'absolute', left: P.x - 40, top: P.y + P.s + 80, width: 640, fontFamily: font.mono, fontSize: 20, lineHeight: 1.5, color: color.fg1, opacity: verdict }}>
            {base.driftMean.toFixed(2)} m × departure × (dwell / {base.dwell.toFixed(1)} s)²
            <br />
            <span style={{ color: color.accent }}>every world within {(Math.ceil(w.predictionError * 1000) / 10).toFixed(1)}%</span>
          </div>
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
