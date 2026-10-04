/**
 * Scene 4, question 2: does the weather move the navigation error? Left, the navigation error of
 * each world's reference draw through the whole flight, from `weather_trace.csv`, drawn as the clock
 * runs. Right, the largest error while GPS is lost, mean and one standard deviation over the draws,
 * from the table.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Weather, World } from '../data/weather';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, QuestionTitle, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Drift: React.FC<{ weather: Weather; timing: SceneTiming }> = ({ weather: w, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const span = (i: number, a = 0, b = 1) => interpolate(frame, [at(ph, i, a), at(ph, i, b)], [0, 1], { ...clamp, easing: ease });
  const base = w.worlds[0];
  const polar = w.worlds.find((x) => x.name === 'polar_winter')!;
  const lead = (x: World) => x === base || x === polar;
  const tone = (x: World) => (x === polar ? color.accent : x === base ? color.fg0 : color.fg2);

  // Left: the reference draws' navigation error against flight time.
  const L = { x: 150, y: 340, w: 760, h: 420 };
  const yMax = Math.ceil(Math.max(...w.worlds.map((x) => x.draws[0].drift)) / 10) * 10;
  const lx = (t: number) => L.x + (t / w.flightS) * L.w;
  const ly = (m: number) => L.y + L.h - (m / yMax) * L.h;
  const tNow = interpolate(frame, [at(ph, 1, 0.05), at(ph, 2, 0.2)], [0, w.flightS], clamp);
  const path = (x: World) =>
    x.trace
      .filter((s) => s.t <= tNow)
      .map((s, i) => `${i === 0 ? 'M' : 'L'}${lx(s.t).toFixed(1)},${ly(s.navErr).toFixed(1)}`)
      .join(' ');
  const darkFrom = Math.min(...w.worlds.map((x) => x.onset));
  const darkTo = Math.max(...w.worlds.map((x) => x.exit));
  const peak = (x: World) => x.trace.reduce((a, s) => (s.denied && s.navErr > a.navErr ? s : a));

  // Right: mean ± one sd of the largest dark error over the draws, smallest mean first.
  const R = { x: 1230, y: 360, w: 520, rowH: 62 };
  const rows = [...w.worlds].sort((a, b) => a.driftMean - b.driftMean);
  const xMax = Math.ceil((polar.driftMean + polar.driftSd) / 10) * 10 + 10;
  const rx = (m: number) => R.x + (m / xMax) * R.w;
  const bars = span(2, 0.05, 0.4);
  const ratio = span(2, 0.6, 0.9);
  const rowOf = (x: World) => rows.indexOf(x);
  const bx = rx(polar.driftMean + polar.driftSd) + 24;

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 40% 45%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text="Question 2 · the navigation error" />
        <QuestionTitle n={2} text="Does it move the navigation error?" x={96} y={150} opacity={fade(frame, 4, 20)} />

        <div style={{ opacity: fade(frame, at(ph, 0, 0.6), at(ph, 1, 0.05)) }}>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            <rect x={lx(darkFrom)} y={L.y} width={lx(darkTo) - lx(darkFrom)} height={L.h} fill={color.warn} opacity={0.08} />
            {Array.from({ length: yMax / 10 + 1 }, (_, i) => i * 10).map((m) => (
              <line key={m} x1={L.x} y1={ly(m)} x2={L.x + L.w} y2={ly(m)} stroke={color.line1} strokeWidth={1} />
            ))}
            <line x1={L.x} y1={ly(0)} x2={L.x + L.w} y2={ly(0)} stroke={color.line2} strokeWidth={2} />
            {[...w.worlds]
              .sort((a, b) => Number(lead(a)) - Number(lead(b)))
              .map((x) => (
                <path key={x.name} d={path(x)} fill="none" stroke={tone(x)} strokeWidth={lead(x) ? 3 : 1.6} strokeLinejoin="round" opacity={lead(x) ? 1 : 0.55} />
              ))}
          </svg>
          {Array.from({ length: yMax / 10 + 1 }, (_, i) => i * 10).map((m) => (
            <div key={m} style={{ position: 'absolute', left: L.x - 70, top: ly(m) - 12, width: 54, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
              {m}
            </div>
          ))}
          <div style={{ position: 'absolute', left: L.x - 30, top: L.y - 50, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>navigation error, m · draw 0 of each world</div>
          <div style={{ position: 'absolute', left: (lx(darkFrom) + lx(darkTo)) / 2 - 100, top: L.y + 10, width: 200, textAlign: 'center', fontFamily: font.mono, fontSize: 18, color: color.warn }}>
            GPS lost
          </div>
          <div style={{ position: 'absolute', left: L.x + L.w - 300, top: L.y + L.h + 14, width: 300, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg1 }}>
            flight time →
          </div>
          {[polar, base].map((x) => {
            const p = peak(x);
            return (
              <div
                key={x.name}
                style={{ position: 'absolute', left: lx(p.t) + 14, top: ly(p.navErr) - 30, fontFamily: font.mono, fontSize: 20, color: tone(x), opacity: interpolate(tNow, [p.t, p.t + 3], [0, 1], clamp) }}
              >
                {x.label}
              </div>
            );
          })}
        </div>

        <div style={{ opacity: bars }}>
          <div style={{ position: 'absolute', left: R.x, top: R.y - 70, width: R.w, fontFamily: font.mono, fontSize: 19, lineHeight: 1.4, color: color.fg1 }}>
            largest error while GPS is lost, m
            <br />
            <span style={{ color: color.fg2 }}>mean ± 1 sd over {w.drawsPerWorld} draws</span>
          </div>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            {rows.map((x, k) => {
              const y = R.y + k * R.rowH + R.rowH / 2;
              return (
                <g key={x.name}>
                  <line x1={R.x} y1={y} x2={rx(x.driftMean * bars)} y2={y} stroke={tone(x)} strokeWidth={lead(x) ? 14 : 10} opacity={lead(x) ? 0.9 : 0.5} />
                  <line x1={rx(x.driftMean - x.driftSd)} y1={y} x2={rx(x.driftMean + x.driftSd)} y2={y} stroke={color.fg0} strokeWidth={1.5} opacity={bars} />
                  <line x1={rx(x.driftMean - x.driftSd)} y1={y - 8} x2={rx(x.driftMean - x.driftSd)} y2={y + 8} stroke={color.fg0} strokeWidth={1.5} opacity={bars} />
                  <line x1={rx(x.driftMean + x.driftSd)} y1={y - 8} x2={rx(x.driftMean + x.driftSd)} y2={y + 8} stroke={color.fg0} strokeWidth={1.5} opacity={bars} />
                </g>
              );
            })}
            <g opacity={ratio}>
              <path
                d={`M${bx},${R.y + rowOf(base) * R.rowH + R.rowH / 2} h 14 V ${R.y + rowOf(polar) * R.rowH + R.rowH / 2} h -14`}
                fill="none"
                stroke={color.accent}
                strokeWidth={1.5}
              />
            </g>
          </svg>
          {rows.map((x, k) => (
            <div key={x.name} style={{ position: 'absolute', left: R.x - 220, top: R.y + k * R.rowH + R.rowH / 2 - 14, width: 200, textAlign: 'right', fontFamily: font.sans, fontSize: 22, color: lead(x) ? tone(x) : color.fg1 }}>
              {x.label}
            </div>
          ))}
          {[base, polar].map((x) => (
            <div key={`v${x.name}`} style={{ position: 'absolute', left: R.x + 10, top: R.y + rowOf(x) * R.rowH + R.rowH / 2 - 40, fontFamily: font.mono, fontSize: 19, color: tone(x) }}>
              {x.driftMean.toFixed(2)} m
            </div>
          ))}
          <div
            style={{
              position: 'absolute',
              left: bx + 30,
              top: R.y + ((rowOf(base) + rowOf(polar)) / 2) * R.rowH + R.rowH / 2 - 20,
              fontFamily: font.mono,
              fontSize: 30,
              color: color.accent,
              opacity: ratio,
            }}
          >
            {w.coldFactor.toFixed(2)}×
          </div>
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
