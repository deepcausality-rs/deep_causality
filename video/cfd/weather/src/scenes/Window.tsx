/**
 * Scene 3, question 1: does the weather move the blackout? One bar per world, from the step GPS is
 * lost to the last step without it, from the table, ordered by density. The axis zooms onto the
 * onsets while the narration names them, then out to the whole flight to compare the lengths.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Weather } from '../data/weather';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, QuestionTitle, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.45, 0, 0.55, 1);

export const Window: React.FC<{ weather: Weather; timing: SceneTiming }> = ({ weather: w, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const rows = [...w.worlds].sort((a, b) => b.rho - a.rho);
  const early = w.worlds.reduce((a, x) => (x.onset < a.onset ? x : a));
  const late = w.worlds.reduce((a, x) => (x.onset > a.onset ? x : a));

  // The axis domain: the whole flight, then the onsets, then the whole flight again.
  const zoom = interpolate(frame, [at(ph, 1, 0.0), at(ph, 1, 0.3), at(ph, 2, 0.0), at(ph, 2, 0.3)], [0, 1, 1, 0], { ...clamp, easing: ease });
  const lo = interpolate(zoom, [0, 1], [0, Math.floor(early.onset) - 1]);
  const hi = interpolate(zoom, [0, 1], [w.flightS, Math.ceil(late.onset) + 1]);
  const P = { x: 560, y: 330, w: 1180, rowH: 62 };
  const px = (t: number) => P.x + ((t - lo) / (hi - lo)) * P.w;
  const ry = (k: number) => P.y + k * P.rowH;
  const H = rows.length * P.rowH;

  const chart = fade(frame, at(ph, 0, 0.6), at(ph, 1, 0.05));
  const question = fade(frame, 4, 20);
  const onsetLabels = fade(frame, at(ph, 1, 0.3), at(ph, 1, 0.45)) * (1 - fade(frame, at(ph, 2, 0.0), at(ph, 2, 0.1)));
  const dwellLabels = fade(frame, at(ph, 2, 0.35), at(ph, 2, 0.55));
  const verdict = fade(frame, at(ph, 3, 0.1), at(ph, 3, 0.35));
  const tickStep = hi - lo > 20 ? 10 : 1;
  const ticks = Array.from({ length: Math.floor(hi / tickStep) - Math.ceil(lo / tickStep) + 1 }, (_, i) => (Math.ceil(lo / tickStep) + i) * tickStep);
  const tone = (name: string) => (name === early.name || name === late.name ? color.fg0 : color.fg1);

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 50% 40%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text="Question 1 · the blackout window" />
        <QuestionTitle n={1} text="Does the weather move the GPS blackout?" x={96} y={150} opacity={question} />
        <div style={{ opacity: chart }}>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            <defs>
              <clipPath id="plot">
                <rect x={P.x} y={P.y - 10} width={P.w} height={H + 20} />
              </clipPath>
            </defs>
            {ticks.map((t) => (
              <line key={t} x1={px(t)} y1={P.y - 6} x2={px(t)} y2={P.y + H} stroke={color.line1} strokeWidth={1} />
            ))}
            <g clipPath="url(#plot)">
              {rows.map((x, k) => (
                <g key={x.name}>
                  <rect x={px(x.onset)} y={ry(k) + 14} width={px(x.exit) - px(x.onset)} height={P.rowH - 28} fill={color.warn} opacity={0.28} />
                  <line x1={px(x.onset)} y1={ry(k) + 8} x2={px(x.onset)} y2={ry(k) + P.rowH - 8} stroke={color.warn} strokeWidth={3} />
                  <line x1={px(x.exit)} y1={ry(k) + 8} x2={px(x.exit)} y2={ry(k) + P.rowH - 8} stroke={color.warn} strokeWidth={1.5} opacity={0.7} />
                </g>
              ))}
            </g>
            <line x1={P.x} y1={P.y + H} x2={P.x + P.w} y2={P.y + H} stroke={color.line2} strokeWidth={2} />
          </svg>
          {rows.map((x, k) => (
            <div key={x.name} style={{ position: 'absolute', left: 130, top: ry(k) + 12, width: 400, display: 'flex', justifyContent: 'space-between', alignItems: 'baseline' }}>
              <span style={{ fontFamily: font.sans, fontSize: 28, color: tone(x.name) }}>{x.label}</span>
              <span style={{ fontFamily: font.mono, fontSize: 19, color: color.fg2 }}>density {x.rho.toFixed(2)}×</span>
            </div>
          ))}
          {rows.map((x, k) => (
            <div
              key={`o${x.name}`}
              style={{ position: 'absolute', left: px(x.onset) + 12, top: ry(k) + 17, fontFamily: font.mono, fontSize: 21, color: x === early || x === late ? color.fg0 : color.fg1, opacity: onsetLabels }}
            >
              lost at {x.onset.toFixed(1)} s
            </div>
          ))}
          {rows.map((x, k) => (
            <div
              key={`d${x.name}`}
              style={{ position: 'absolute', left: (px(x.onset) + px(x.exit)) / 2 - 80, top: ry(k) + 17, width: 160, textAlign: 'center', fontFamily: font.mono, fontSize: 21, color: color.fg0, opacity: dwellLabels }}
            >
              {x.dwell.toFixed(1)} s
            </div>
          ))}
          {ticks.map((t) => (
            <div key={`t${t}`} style={{ position: 'absolute', left: px(t) - 40, top: P.y + H + 12, width: 80, textAlign: 'center', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
              {t}
            </div>
          ))}
          <div style={{ position: 'absolute', left: P.x + P.w - 300, top: P.y + H + 44, width: 300, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg1 }}>
            flight time, s →
          </div>
          <div style={{ position: 'absolute', left: P.x, top: P.y - 50, fontFamily: font.mono, fontSize: 19, color: color.warn }}>GPS lost, from onset to exit</div>
        </div>
        <div style={{ position: 'absolute', left: 130, top: P.y + H + 90, display: 'flex', gap: 28, opacity: verdict }}>
          {[
            { label: 'onset moves', value: `${w.spreads.onset.toFixed(1)} s`, tone: color.warn },
            { label: 'length moves', value: `${w.spreads.dwell.toFixed(1)} s`, tone: color.fg0 },
          ].map((b) => (
            <div key={b.label} style={{ padding: '12px 22px', border: `1px solid ${color.line2}`, borderRadius: 8, background: 'rgba(7, 11, 16, 0.72)', fontFamily: font.mono, fontSize: 22, color: color.fg1 }}>
              {b.label} <span style={{ color: b.tone, fontSize: 26 }}>{b.value}</span>
            </div>
          ))}
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
