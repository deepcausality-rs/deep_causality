/**
 * The descent's clock along the top: entry to the end of the corridor run, with the pause
 * marked. `t` is flight time in seconds; `frozen` dims the head to show that time has stopped.
 */
import { interpolate } from 'remotion';
import { color, font } from '@cfd-video/shared';

export const Timeline: React.FC<{ t: number; tMax: number; pauseT: number; frozen: number; opacity: number }> = ({
  t,
  tMax,
  pauseT,
  frozen,
  opacity,
}) => {
  const x0 = 96;
  const x1 = 1824;
  const y = 150;
  const px = (s: number) => x0 + (s / tMax) * (x1 - x0);
  const head = px(Math.min(t, tMax));
  return (
    <div style={{ position: 'absolute', inset: 0, opacity }}>
      <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
        <line x1={x0} y1={y} x2={x1} y2={y} stroke={color.line2} strokeWidth={2} />
        <line x1={x0} y1={y} x2={head} y2={y} stroke={color.accent} strokeWidth={3} />
        <line x1={px(pauseT)} y1={y - 14} x2={px(pauseT)} y2={y + 14} stroke={color.warn} strokeWidth={2} />
        <circle cx={head} cy={y} r={7} fill={color.accent} opacity={1 - 0.5 * frozen} />
        {frozen > 0 && (
          <g opacity={frozen}>
            <rect x={head + 16} y={y - 11} width={5} height={22} fill={color.fg0} />
            <rect x={head + 26} y={y - 11} width={5} height={22} fill={color.fg0} />
          </g>
        )}
      </svg>
      <div style={{ position: 'absolute', left: x0, top: y + 20, fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>entry</div>
      <div style={{ position: 'absolute', left: px(pauseT) - 60, top: y + 20, width: 240, fontFamily: font.mono, fontSize: 18, color: color.warn }}>
        GPS lost · {pauseT.toFixed(1)} s
      </div>
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
