/**
 * The flight clock along the top: entry to the end of the recorded flight, with the blackout window
 * shaded and the run's events marked. `t` is flight time in seconds; `frozen` dims the head and shows the pause
 * glyph, to show that time has stopped.
 */
import { interpolate } from 'remotion';
import { color, font } from '../tokens';

export interface TimelineMark {
  t: number;
  label: string;
  tone?: 'warn' | 'accent' | 'fg';
}

export const Timeline: React.FC<{
  t: number;
  tMax: number;
  blackout: { from: number; to: number };
  marks: TimelineMark[];
  frozen: number;
  opacity: number;
}> = ({ t, tMax, blackout, marks, frozen, opacity }) => {
  const x0 = 96;
  const x1 = 1824;
  const y = 150;
  const px = (s: number) => x0 + (s / tMax) * (x1 - x0);
  const head = px(Math.min(t, tMax));
  const toneColor = (tone?: string) => (tone === 'warn' ? color.warn : tone === 'accent' ? color.accent : color.fg1);
  return (
    <div style={{ position: 'absolute', inset: 0, opacity }}>
      <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
        <line x1={x0} y1={y} x2={x1} y2={y} stroke={color.line2} strokeWidth={2} />
        <rect x={px(blackout.from)} y={y - 5} width={px(blackout.to) - px(blackout.from)} height={10} fill={color.warn} opacity={0.22} />
        <line x1={x0} y1={y} x2={head} y2={y} stroke={color.accent} strokeWidth={3} />
        {marks.map((m) => (
          <line key={m.label} x1={px(m.t)} y1={y - 12} x2={px(m.t)} y2={y + 12} stroke={toneColor(m.tone)} strokeWidth={2} />
        ))}
        <circle cx={head} cy={y} r={7} fill={color.accent} opacity={1 - 0.5 * frozen} />
        {frozen > 0 && (
          <g opacity={frozen}>
            <rect x={head + 16} y={y - 11} width={5} height={22} fill={color.fg0} />
            <rect x={head + 26} y={y - 11} width={5} height={22} fill={color.fg0} />
          </g>
        )}
      </svg>
      <div style={{ position: 'absolute', left: x0, top: y + 20, fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>entry</div>
      <div style={{ position: 'absolute', left: (px(blackout.from) + px(blackout.to)) / 2 - 60, top: y + 20, width: 120, textAlign: 'center', fontFamily: font.mono, fontSize: 17, color: color.warn }}>
        blackout
      </div>
      {marks.map((m) => (
        <div
          key={m.label}
          style={{ position: 'absolute', left: px(m.t) - 8, top: y + 20, fontFamily: font.mono, fontSize: 17, color: toneColor(m.tone), whiteSpace: 'nowrap' }}
        >
          {m.label}
        </div>
      ))}
      <div
        style={{
          position: 'absolute',
          left: head + 44,
          top: y - 14,
          fontFamily: font.mono,
          fontSize: 20,
          color: color.fg0,
          opacity: interpolate(frozen, [0.4, 1], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }),
        }}
      >
        paused
      </div>
      <div style={{ position: 'absolute', left: x1 - 120, top: y + 20, width: 120, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
        {tMax.toFixed(0)} s
      </div>
    </div>
  );
};
