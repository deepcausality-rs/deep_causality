/**
 * Where each branch ends, at one scale on both axes so a distance on screen is a distance flown:
 * sideways from the no-bank end, and lower within the descent plane. The end points fall on one
 * arc set by the bank angle flown; the aim sits just off it. All lengths in stage pixels.
 */
import { interpolate } from 'remotion';
import type { Corridor } from '../data/corridor';
import { color, font } from '@cfd-video/shared';

export interface ArcProgress {
  points: number;
  arc: number;
  miss: number;
  result: number;
}

const clamp01 = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;

export const EndArc: React.FC<{ corridor: Corridor; p: ArcProgress; x: number; y: number; w: number }> = ({ corridor, p, x, y, w }) => {
  const crMax = Math.ceil(Math.max(...corridor.branches.map((b) => b.end.cross)) / 10) * 10;
  const s = (w - 40) / crMax;
  const top = 104;
  const ex = (m: number) => 20 + m * s;
  const ey = (m: number) => top + m * s;
  const R = corridor.arc.radius;
  const capPhi = Math.max(...corridor.branches.map((b) => (b.flown * Math.PI) / 180));
  const arcPath = Array.from({ length: 81 }, (_, i) => {
    const phi = (i / 80) * capPhi;
    return `${i === 0 ? 'M' : 'L'}${ex(R * Math.sin(phi)).toFixed(1)},${ey(R * (1 - Math.cos(phi))).toFixed(1)}`;
  }).join(' ');
  const ends = [...corridor.branches]
    .filter((b) => !(b.round === 2 && b.bank === 10))
    .sort((a, b) => a.bank - b.bank);
  const committed = corridor.branches.find((b) => b.round === 2 && b.bank === corridor.committed.bank)!;
  const aim = { x: ex(corridor.aimOffsetM), y: ey(0) };
  const win = { x: ex(committed.end.cross), y: ey(committed.end.drop) };
  const dropMax = Math.ceil(Math.max(...corridor.branches.map((b) => b.end.drop)) / 2) * 2;
  const h = top + dropMax * s + 60;

  return (
    <div style={{ position: 'absolute', left: x, top: y, width: w, height: h + 200 }}>
      <svg width={w} height={h} style={{ position: 'absolute', inset: 0, overflow: 'visible' }}>
        <line x1={ex(0)} y1={ey(0)} x2={ex(crMax)} y2={ey(0)} stroke={color.line2} strokeWidth={1.5} />
        <path d={arcPath} pathLength={1} fill="none" stroke={color.fg2} strokeWidth={2} strokeDasharray={1} strokeDashoffset={1 - p.arc} />
        {ends.map((b, i) => {
          const o = interpolate(p.points, [i / ends.length, i / ends.length + 0.12], [0, 1], clamp01);
          const isWin = b === committed;
          return (
            <circle
              key={`${b.round}-${b.bank}`}
              cx={ex(b.end.cross)}
              cy={ey(b.end.drop)}
              r={isWin ? 9 : 6}
              fill={isWin ? color.accent : b.round === 2 ? 'rgba(92,212,225,0.5)' : color.fg1}
              opacity={o}
              transform={`translate(${ex(b.end.cross)} ${ey(b.end.drop)}) scale(${0.6 + 0.4 * o}) translate(${-ex(b.end.cross)} ${-ey(b.end.drop)})`}
            />
          );
        })}
        <circle cx={aim.x} cy={aim.y} r={11} fill="none" stroke={color.fg0} strokeWidth={2.5} opacity={interpolate(p.points, [0, 0.1], [0, 1], clamp01)} />
        <line x1={aim.x} y1={aim.y} x2={win.x} y2={win.y} stroke={color.accent} strokeWidth={3} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - p.miss} />
      </svg>

      <div style={{ position: 'absolute', left: 0, top: 0, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>
        where each branch ends · same scale both ways
      </div>
      <div style={{ position: 'absolute', left: aim.x - 18, top: aim.y - 52, fontFamily: font.mono, fontSize: 19, color: color.fg0, opacity: interpolate(p.points, [0, 0.1], [0, 1], clamp01) }}>
        aim
      </div>
      <div style={{ position: 'absolute', left: ex(0) - 6, top: ey(0) + 14, fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>0°</div>
      <div
        style={{
          position: 'absolute',
          left: ex(R * Math.sin(capPhi)) - 180,
          top: ey(R * (1 - Math.cos(capPhi))) + 18,
          width: 200,
          textAlign: 'right',
          fontFamily: font.mono,
          fontSize: 18,
          color: color.fg2,
          opacity: p.arc,
        }}
      >
        envelope cap {((capPhi * 180) / Math.PI).toFixed(1)}°
      </div>
      <div
        style={{
          position: 'absolute',
          left: win.x - 250,
          top: win.y + 18,
          width: 230,
          textAlign: 'right',
          fontFamily: font.mono,
          fontSize: 22,
          color: color.accent,
          opacity: interpolate(p.miss, [0.6, 1], [0, 1], clamp01),
        }}
      >
        {corridor.committed.bank}° · {corridor.committed.miss.toFixed(2)} m
      </div>

      {/* The result, under the plot. */}
      <div style={{ position: 'absolute', left: 0, top: h + 24, opacity: p.result }}>
        <div style={{ fontFamily: font.sans, fontSize: 46, fontWeight: 500, color: color.fg0, letterSpacing: '-0.01em' }}>
          {corridor.committed.bank}° lands <span style={{ color: color.accent }}>{corridor.committed.miss.toFixed(2)} m</span> from the aim
        </div>
        <div style={{ marginTop: 10, fontFamily: font.mono, fontSize: 22, color: color.fg1 }}>
          no bank: {corridor.noBankMiss.toFixed(1)} m · {corridor.improvement.toFixed(1)}× closer · {corridor.branches.length} branches, one past
        </div>
      </div>
    </div>
  );
};
