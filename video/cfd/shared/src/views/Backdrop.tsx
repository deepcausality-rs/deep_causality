/**
 * Space and the Earth limb seen from a reentry altitude: a seeded star field, the night-side Earth,
 * and the thin airglow line along the limb. Drawn in 2D behind the transparent 3D canvas.
 * `limbY` places the horizon in stage pixels.
 */
import { useMemo } from 'react';
import { useCurrentFrame } from 'remotion';
import { color, stage } from '../tokens';

/** A small deterministic generator, so every render places the stars identically. */
const seeded = (seed: number) => () => {
  seed = (seed * 1664525 + 1013904223) % 4294967296;
  return seed / 4294967296;
};

export const Backdrop: React.FC<{ limbY: number; drift?: number }> = ({ limbY, drift = 0 }) => {
  const frame = useCurrentFrame();
  const stars = useMemo(() => {
    const r = seeded(7);
    return Array.from({ length: 260 }, () => ({
      x: r() * stage.width,
      y: r() * stage.height,
      size: 0.4 + r() ** 3 * 1.6,
      base: 0.25 + r() * 0.6,
      phase: r() * Math.PI * 2,
    }));
  }, []);
  const R = 3400;
  const cx = stage.width / 2 - drift;
  const cy = limbY + R;
  return (
    <svg width={stage.width} height={stage.height} viewBox={`0 0 ${stage.width} ${stage.height}`} style={{ position: 'absolute', inset: 0 }}>
      <defs>
        <linearGradient id="space" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#010307" />
          <stop offset="0.75" stopColor="#040a16" />
          <stop offset="1" stopColor="#0a1a33" />
        </linearGradient>
        <radialGradient id="earth" cx="0.5" cy="0" r="0.25">
          <stop offset="0" stopColor="#0d2240" />
          <stop offset="0.35" stopColor="#081428" />
          <stop offset="1" stopColor={color.bg0} />
        </radialGradient>
        <filter id="glow" x="-10%" y="-10%" width="120%" height="120%">
          <feGaussianBlur stdDeviation="14" />
        </filter>
        <filter id="glow-soft" x="-10%" y="-10%" width="120%" height="120%">
          <feGaussianBlur stdDeviation="40" />
        </filter>
        <linearGradient id="dawn" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0" stopColor="#3b7bd6" stopOpacity="0.25" />
          <stop offset="0.6" stopColor="#5cd4e1" stopOpacity="0.55" />
          <stop offset="1" stopColor="#f2b56b" stopOpacity="0.7" />
        </linearGradient>
      </defs>
      <rect width={stage.width} height={stage.height} fill="url(#space)" />
      {stars.map((s, i) => (
        <circle
          key={i}
          cx={s.x}
          cy={s.y}
          r={s.size}
          fill="#dfe9f5"
          opacity={s.y < limbY ? s.base * (0.85 + 0.15 * Math.sin(frame / 18 + s.phase)) : 0}
        />
      ))}
      {/* The atmosphere above the limb: a wide soft band, then the bright airglow line. */}
      <circle cx={cx} cy={cy} r={R + 70} fill="none" stroke="#1d4f8f" strokeWidth={120} opacity={0.35} filter="url(#glow-soft)" />
      <circle cx={cx} cy={cy} r={R} fill="url(#earth)" />
      <circle cx={cx} cy={cy} r={R + 6} fill="none" stroke="url(#dawn)" strokeWidth={10} filter="url(#glow)" />
      <circle cx={cx} cy={cy} r={R + 3} fill="none" stroke="url(#dawn)" strokeWidth={2} opacity={0.9} />
    </svg>
  );
};
