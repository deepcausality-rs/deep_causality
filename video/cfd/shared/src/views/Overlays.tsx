/**
 * Overlays shared by the flight scenes: the scene's narration captions, a data strip, the GPS
 * links to the vehicle, and the live readout. All lengths in stage pixels.
 */
import { interpolate, useCurrentFrame } from 'remotion';
import type { PhraseTiming } from '../timing';
import { color, font } from '../tokens';
import { Caption } from './Text';

const clamp01 = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;

/** The scene's phrase on screen now, if any. */
export const Captions: React.FC<{ phrases: PhraseTiming[] }> = ({ phrases }) => {
  const frame = useCurrentFrame();
  const p = phrases.find((x) => frame >= x.start && frame < x.start + x.length);
  return p ? <Caption text={p.text} local={frame - p.start} length={p.length} /> : null;
};

/** Frame at which a phrase starts, plus a fraction of its length. */
export const at = (phrases: PhraseTiming[], index: number, fraction = 0) =>
  phrases[index].start + Math.round(phrases[index].length * fraction);

export interface StripMark {
  t: number;
  value: number;
  label: React.ReactNode;
  tone?: 'accent' | 'warn' | 'fg';
  /** Place the label left of the point instead of right. */
  left?: boolean;
  /** Place the label below the point instead of above. */
  below?: boolean;
}

/**
 * A time series in a hairline panel, drawn up to `tNow`. `log` plots log₁₀ of the value. A
 * dashed `threshold` line marks a reference such as the GPS L1 band; its label sits in the header,
 * clear of the curve. Mark labels sit above their point unless `below` is set.
 */
export const Strip: React.FC<{
  x: number;
  y: number;
  w: number;
  h: number;
  title: string;
  t: number[];
  v: number[];
  tFrom: number;
  tTo: number;
  tNow: number;
  lo: number;
  hi: number;
  log?: boolean;
  threshold?: { value: number; label: string; tone: 'warn' | 'fg' };
  marks?: StripMark[];
  opacity?: number;
}> = ({ x, y, w, h, title, t, v, tFrom, tTo, tNow, lo, hi, log = false, threshold, marks = [], opacity = 1 }) => {
  const pad = { l: 16, r: 16, t: 40, b: 16 };
  const pw = w - pad.l - pad.r;
  const ph = h - pad.t - pad.b;
  const f = (val: number) => (log ? Math.log10(Math.max(val, 1e-30)) : val);
  const px = (s: number) => pad.l + ((s - tFrom) / (tTo - tFrom)) * pw;
  const py = (val: number) => pad.t + ph - ((Math.min(Math.max(f(val), lo), hi) - lo) / (hi - lo)) * ph;
  const pts: string[] = [];
  for (let i = 0; i < t.length; i++) {
    if (t[i] < tFrom || t[i] > Math.min(tNow, tTo)) continue;
    pts.push(`${pts.length === 0 ? 'M' : 'L'}${px(t[i]).toFixed(1)},${py(v[i]).toFixed(1)}`);
  }
  const toneColor = (tone?: string) => (tone === 'accent' ? color.accent : tone === 'warn' ? color.warn : color.fg0);
  const head = t.reduce((best, s, i) => (s <= tNow && s >= tFrom ? i : best), -1);
  return (
    <div
      style={{
        position: 'absolute',
        left: x,
        top: y,
        width: w,
        height: h,
        background: 'rgba(7, 11, 16, 0.72)',
        border: `1px solid ${color.line1}`,
        borderRadius: 10,
        opacity,
      }}
    >
      <div style={{ position: 'absolute', left: 16, top: 12, fontFamily: font.mono, fontSize: 17, color: color.fg1 }}>{title}</div>
      <svg width={w} height={h} style={{ position: 'absolute', inset: 0, overflow: 'visible' }}>
        {threshold && (
          <line x1={pad.l} x2={pad.l + pw} y1={py(threshold.value)} y2={py(threshold.value)} stroke={toneColor(threshold.tone)} strokeWidth={1.6} strokeDasharray="6 6" opacity={0.85} />
        )}
        <path d={pts.join(' ')} fill="none" stroke={color.accent} strokeWidth={2.6} strokeLinejoin="round" />
        {head >= 0 && <circle cx={px(t[head])} cy={py(v[head])} r={5} fill={color.accent} />}
        {marks
          .filter((m) => m.t <= tNow)
          .map((m) => (
            <circle key={`${m.t}`} cx={px(m.t)} cy={py(m.value)} r={6} fill="none" stroke={toneColor(m.tone)} strokeWidth={2} />
          ))}
      </svg>
      {threshold && (
        <div style={{ position: 'absolute', right: 16, top: 13, display: 'flex', alignItems: 'center', gap: 10, fontFamily: font.mono, fontSize: 16, color: toneColor(threshold.tone) }}>
          <svg width={28} height={4}>
            <line x1={0} x2={28} y1={2} y2={2} stroke={toneColor(threshold.tone)} strokeWidth={1.6} strokeDasharray="6 6" />
          </svg>
          {threshold.label}
        </div>
      )}
      {marks
        .filter((m) => m.t <= tNow)
        .map((m) => (
          <div
            key={`l${m.t}`}
            style={{
              position: 'absolute',
              left: px(m.t) + (m.left ? -10 : 10),
              top: py(m.value) + (m.below ? 16 : -32),
              transform: m.left ? 'translateX(-100%)' : undefined,
              fontFamily: font.mono,
              fontSize: 17,
              color: toneColor(m.tone),
              whiteSpace: 'nowrap',
            }}
          >
            {m.label}
          </div>
        ))}
    </div>
  );
};

/** Thin links from three satellites to the vehicle; `link` 1 is a clear signal, 0 is none. */
export const SatLinks: React.FC<{ to: { x: number; y: number }; link: number }> = ({ to, link }) => {
  const frame = useCurrentFrame();
  const sats = [
    { x: 300, y: -20 },
    { x: 980, y: -20 },
    { x: 1700, y: -20 },
  ];
  return (
    <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
      {sats.map((s, i) => {
        const flicker = 0.75 + 0.25 * Math.sin(frame / 5 + i * 2.1);
        return (
          <line
            key={i}
            x1={s.x}
            y1={s.y}
            x2={to.x}
            y2={to.y}
            stroke={color.accent}
            strokeWidth={1.4}
            strokeDasharray="2 10"
            strokeDashoffset={-frame * 2}
            opacity={0.55 * link * flicker}
          />
        );
      })}
    </svg>
  );
};

/** One row of the readout. */
export interface ReadoutRow {
  label: string;
  value: string;
  tone?: 'accent' | 'warn';
  /** Value font size, stage pixels. */
  size?: number;
}

/** A live readout in the top-right corner, on a panel so the wake behind it stays clear. */
export const Readout: React.FC<{ rows: ReadoutRow[]; opacity?: number }> = ({ rows, opacity = 1 }) => (
  <div
    style={{
      position: 'absolute',
      right: 96,
      top: 210,
      width: 300,
      padding: '4px 18px',
      background: 'rgba(7, 11, 16, 0.72)',
      border: `1px solid ${color.line1}`,
      borderRadius: 10,
      opacity,
      fontFamily: font.mono,
    }}
  >
    {rows.map((r, i) => (
      <div
        key={r.label}
        style={{ display: 'flex', justifyContent: 'space-between', padding: '8px 0', borderBottom: i < rows.length - 1 ? `1px solid ${color.line1}` : 'none' }}
      >
        <span style={{ fontSize: 19, color: color.fg1 }}>{r.label}</span>
        <span style={{ fontSize: r.size ?? 23, color: r.tone === 'accent' ? color.accent : r.tone === 'warn' ? color.warn : color.fg0 }}>{r.value}</span>
      </div>
    ))}
  </div>
);

export const fade = (frame: number, from: number, to: number) => interpolate(frame, [from, to], [0, 1], clamp01);
