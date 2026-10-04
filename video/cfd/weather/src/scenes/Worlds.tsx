/**
 * Scene 2, the worlds: the baseline on the left, and the five atmospheres alternated from it, each
 * with its temperature offset and density scale from the table. Then the alternation line each
 * world's reference draw wrote to its audit log, verbatim, and one pip per receiver-noise draw.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Weather, World } from '../data/weather';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

/** Signed kelvin, with a true minus sign. */
export const kelvin = (dT: number) => `${dT > 0 ? '+' : dT < 0 ? '−' : ''}${Math.abs(dT)} K`;

const Pips: React.FC<{ n: number; shown: number; tone: string }> = ({ n, shown, tone }) => (
  <svg width={n * 20} height={16}>
    {Array.from({ length: n }, (_, k) => (
      <circle key={k} cx={8 + k * 20} cy={8} r={6} fill={tone} opacity={interpolate(shown * n - k, [0, 1], [0, 0.9], clamp)} />
    ))}
  </svg>
);

export const Worlds: React.FC<{ weather: Weather; timing: SceneTiming }> = ({ weather: w, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const span = (i: number, a = 0, b = 1) => interpolate(frame, [at(ph, i, a), at(ph, i, b)], [0, 1], { ...clamp, easing: ease });
  const base = w.worlds[0];
  const others = w.worlds.slice(1);

  const B = { x: 130, y: 330, w: 500, h: 300 };
  const R = { x: 800, y: 214, w: 990, h: 112, gap: 18 };
  const rowY = (k: number) => R.y + k * (R.h + R.gap);
  const shownAt = (k: number) => span(1, 0.12 + 0.12 * k, 0.3 + 0.12 * k);
  const markers = (k: number) => span(2, 0.1 + 0.08 * k, 0.35 + 0.08 * k);
  const pips = span(3, 0.05, 0.6);
  const params = (x: World) => `${kelvin(x.dTemp)} · density ${x.rho.toFixed(2)}×`;

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 30% 45%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text="DeepCausality · one baseline, five alternated worlds" />
        <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
          {others.map((x, k) => {
            const y1 = rowY(k) + R.h / 2;
            const y0 = B.y + B.h / 2;
            return (
              <path
                key={x.name}
                d={`M${B.x + B.w},${y0} C${B.x + B.w + 120},${y0} ${R.x - 120},${y1} ${R.x},${y1}`}
                fill="none"
                stroke={color.accent}
                strokeOpacity={0.55}
                strokeWidth={1.6}
                pathLength={1}
                strokeDasharray={1}
                strokeDashoffset={1 - shownAt(k)}
              />
            );
          })}
        </svg>

        <div
          style={{
            position: 'absolute',
            left: B.x,
            top: B.y,
            width: B.w,
            height: B.h,
            boxSizing: 'border-box',
            padding: '30px 34px',
            background: 'rgba(11, 17, 24, 0.82)',
            border: `1px solid ${color.accent}`,
            borderRadius: 10,
            opacity: span(0, 0.05, 0.3),
          }}
        >
          <div style={{ fontFamily: font.mono, fontSize: 18, letterSpacing: '0.12em', textTransform: 'uppercase', color: color.accent }}>baseline</div>
          <div style={{ marginTop: 12, fontFamily: font.sans, fontSize: 46, fontWeight: 500, color: color.fg0 }}>{base.label}</div>
          <div style={{ marginTop: 8, fontFamily: font.sans, fontSize: 24, color: color.fg1 }}>the corridor’s validated descent</div>
          <div style={{ marginTop: 18, fontFamily: font.mono, fontSize: 22, color: color.fg1 }}>{params(base)}</div>
          <div style={{ position: 'absolute', left: 34, bottom: 30 }}>
            <Pips n={base.draws.length} shown={pips} tone={color.accent} />
          </div>
        </div>

        {others.map((x, k) => (
          <div
            key={x.name}
            style={{
              position: 'absolute',
              left: R.x,
              top: rowY(k),
              width: R.w,
              height: R.h,
              boxSizing: 'border-box',
              padding: '18px 26px',
              display: 'grid',
              gridTemplateColumns: '250px 1fr 160px',
              columnGap: 24,
              alignItems: 'center',
              background: 'rgba(11, 17, 24, 0.82)',
              border: `1px solid ${color.line2}`,
              borderRadius: 10,
              opacity: shownAt(k),
              transform: `translateX(${(1 - shownAt(k)) * 16}px)`,
            }}
          >
            <div>
              <div style={{ fontFamily: font.sans, fontSize: 32, fontWeight: 500, color: color.fg0 }}>{x.label}</div>
              <div style={{ marginTop: 6, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>{params(x)}</div>
            </div>
            <div style={{ fontFamily: font.mono, fontSize: 16, lineHeight: 1.45, color: color.fg2, opacity: markers(k) }}>{x.marker}</div>
            <Pips n={x.draws.length} shown={pips} tone={color.fg1} />
          </div>
        ))}

        <div style={{ position: 'absolute', left: B.x, top: B.y + B.h + 40, width: 480, fontFamily: font.mono, fontSize: 22, lineHeight: 1.5, color: color.fg1, opacity: span(3, 0.5, 0.8) }}>
          {w.worlds.length} worlds × {w.drawsPerWorld} draws
          <br />= <span style={{ color: color.accent }}>{w.worlds.length * w.drawsPerWorld} descents</span>, flown in parallel
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
