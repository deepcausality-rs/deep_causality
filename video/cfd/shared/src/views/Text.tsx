/**
 * Type on the stage: the eyebrow, the narration caption, and the state card that lists what the
 * pause holds. All sizes in stage pixels.
 */
import { interpolate } from 'remotion';
import { color, font } from '../tokens';

export const Eyebrow: React.FC<{ text: string; opacity?: number }> = ({ text, opacity = 1 }) => (
  <div
    style={{
      position: 'absolute',
      left: 96,
      top: 72,
      fontFamily: font.mono,
      fontSize: 20,
      letterSpacing: '0.14em',
      textTransform: 'uppercase',
      color: color.fg2,
      opacity,
      display: 'flex',
      alignItems: 'center',
      gap: 16,
    }}
  >
    <span style={{ width: 28, height: 1, background: color.accent, display: 'inline-block' }} />
    {text}
  </div>
);

/** Caption text with each `^n` set as a superscript exponent: `10^19` reads 10¹⁹. */
export const Rich: React.FC<{ text: string }> = ({ text }) => (
  <>
    {text.split(/\^(-?\d+)/).map((part, i) =>
      i % 2 ? (
        <sup key={i} style={{ fontSize: '0.62em', lineHeight: 0 }}>
          {part}
        </sup>
      ) : (
        part
      )
    )}
  </>
);

/** One narration phrase, fading in and out over its own frames. */
export const Caption: React.FC<{ text: string; local: number; length: number }> = ({ text, local, length }) => {
  const opacity = interpolate(local, [0, 8, length - 8, length], [0, 1, 1, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });
  const rise = interpolate(local, [0, 10], [10, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });
  return (
    <div
      style={{
        position: 'absolute',
        left: 240,
        right: 240,
        bottom: 84,
        textAlign: 'center',
        fontFamily: font.sans,
        fontSize: 38,
        lineHeight: 1.35,
        fontWeight: 400,
        color: color.fg0,
        opacity,
        transform: `translateY(${rise}px)`,
        textShadow: '0 2px 18px rgba(0,0,0,0.85)',
      }}
    >
      <Rich text={text} />
    </div>
  );
};

export interface StateRow {
  label: string;
  value: React.ReactNode;
}

/** A hairline card listing values, each row arriving in turn as `progress` runs 0 → 1. */
export const StateCard: React.FC<{ title: string; rows: StateRow[]; note?: string; progress: number; x: number; y: number }> = ({
  title,
  rows,
  note,
  progress,
  x,
  y,
}) => (
  <div
    style={{
      position: 'absolute',
      left: x,
      top: y,
      width: 560,
      padding: '28px 32px',
      background: 'rgba(11, 17, 24, 0.82)',
      border: `1px solid ${color.line2}`,
      borderRadius: 10,
      backdropFilter: 'blur(6px)',
      opacity: interpolate(progress, [0, 0.15], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }),
    }}
  >
    <div style={{ fontFamily: font.mono, fontSize: 18, letterSpacing: '0.12em', textTransform: 'uppercase', color: color.accent, marginBottom: 18 }}>
      {title}
    </div>
    {rows.map((r, i) => {
      const at = 0.15 + (i / rows.length) * 0.7;
      const o = interpolate(progress, [at, at + 0.12], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });
      return (
        <div
          key={r.label}
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            padding: '12px 0',
            borderTop: i === 0 ? 'none' : `1px solid ${color.line1}`,
            opacity: o,
            transform: `translateX(${(1 - o) * 12}px)`,
          }}
        >
          <span style={{ fontFamily: font.sans, fontSize: 24, color: color.fg1 }}>{r.label}</span>
          <span style={{ fontFamily: font.mono, fontSize: 24, color: color.fg0 }}>{r.value}</span>
        </div>
      );
    })}
    {note && (
      <div
        style={{
          marginTop: 16,
          fontFamily: font.sans,
          fontSize: 21,
          lineHeight: 1.4,
          color: color.fg2,
          opacity: interpolate(progress, [0.85, 1], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }),
        }}
      >
        {note}
      </div>
    )}
  </div>
);

/** Scientific notation with a superscript exponent: 3.3 × 10¹⁶. */
export const Sci: React.FC<{ value: number; digits?: number }> = ({ value, digits = 1 }) => {
  const e = Math.floor(Math.log10(value));
  return (
    <span>
      {(value / 10 ** e).toFixed(digits)} × 10<sup style={{ fontSize: '0.65em' }}>{e}</sup>
    </span>
  );
};
