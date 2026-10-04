/**
 * Scene 2, the question: seen from above, the descent so far, the no-bank path ahead, and an aim
 * point 20 m to the side. A dial shows what the bank angle is. Downrange is compressed so the 20 m
 * offset is visible beside kilometres of flight; the sideways scale is stated on screen. The
 * candidate paths are schematic, and labelled so: their ends carry no data, and the fork scene
 * draws the traced branches.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Corridor } from '../data/corridor';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Question: React.FC<{ corridor: Corridor; timing: SceneTiming }> = ({ corridor: c, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const y0 = 600;
  const pauseX = 560;
  const aimX = 1560;
  const PX_PER_M = 8;
  const aimY = y0 - c.aimOffsetM * PX_PER_M;
  const past = interpolate(frame, [0, 24], [0, 1], { ...clamp, easing: ease });
  const ahead = interpolate(frame, [at(ph, 1, 0), at(ph, 1, 0.4)], [0, 1], { ...clamp, easing: ease });
  const aim = fade(frame, at(ph, 1, 0.35), at(ph, 1, 0.55));
  const options = interpolate(frame, [at(ph, 2, 0), at(ph, 2, 0.6)], [0, 1], { ...clamp, easing: ease });
  // The dial swings through a few bank angles while the first line names it.
  const bank = interpolate(frame, [at(ph, 0, 0.1), at(ph, 0, 0.5), at(ph, 0, 0.9), at(ph, 1, 0.3)], [0, 28, -12, 12], { ...clamp, easing: ease });
  const navErr = c.pause.navErr;

  // Schematic sideways ends of the candidate paths, m.
  const candidates = [6, 14, 20, 28];
  const candidate = (k: number) => {
    const endY = y0 - k * PX_PER_M;
    return `M${pauseX},${y0} C${pauseX + 380},${y0} ${aimX - 420},${endY} ${aimX},${endY}`;
  };

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 40% 55%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text="The question · seen from above" />
        <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
          {/* The descent so far, the pause, the no-bank path ahead. */}
          <line x1={160} y1={y0} x2={pauseX} y2={y0} stroke={color.accent} strokeWidth={3} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - past} />
          <line x1={pauseX} y1={y0} x2={aimX + 140} y2={y0} stroke={color.fg2} strokeWidth={2} strokeDasharray="10 10" opacity={ahead} />
          {candidates.map((k, i) => (
            <path key={k} d={candidate(k)} fill="none" stroke={color.fg1} strokeWidth={1.6} strokeDasharray="4 8" opacity={0.6 * options * (i === 2 ? 1 : 0.7)} />
          ))}
          {/* The aim and the offset that defines it. */}
          <g opacity={aim}>
            <line x1={aimX} y1={y0} x2={aimX} y2={aimY} stroke={color.fg1} strokeWidth={1.4} />
            <line x1={aimX - 10} y1={y0} x2={aimX + 10} y2={y0} stroke={color.fg1} strokeWidth={1.4} />
            <circle cx={aimX} cy={aimY} r={14} fill="none" stroke={color.fg0} strokeWidth={3} />
            <circle cx={aimX} cy={aimY} r={4} fill={color.fg0} />
          </g>
          {/* The vehicle at the pause. */}
          <g transform={`translate(${pauseX} ${y0})`}>
            <circle r={26} fill="none" stroke={color.accent} strokeWidth={1.5} opacity={0.35 + 0.25 * Math.sin(frame / 8)} />
            <path d="M14,0 L-12,-11 L-6,0 L-12,11 Z" fill={color.accent} />
          </g>
        </svg>

        <div style={{ position: 'absolute', left: 160, top: y0 + 26, fontFamily: font.mono, fontSize: 19, color: color.accent, opacity: past }}>
          the descent so far
        </div>
        <div style={{ position: 'absolute', left: pauseX - 70, top: y0 - 72, whiteSpace: 'nowrap', fontFamily: font.mono, fontSize: 19, color: color.warn, opacity: past }}>
          GPS lost · nav error {navErr.toFixed(2)} m
        </div>
        <div style={{ position: 'absolute', left: aimX - 380, top: y0 + 22, fontFamily: font.mono, fontSize: 19, color: color.fg2, opacity: ahead }}>
          no bank
        </div>
        <div style={{ position: 'absolute', left: aimX + 30, top: aimY - 30, fontFamily: font.mono, fontSize: 22, color: color.fg0, opacity: aim }}>
          aim
        </div>
        <div style={{ position: 'absolute', left: aimX + 18, top: (y0 + aimY) / 2 - 14, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: aim }}>
          {c.aimOffsetM} m to the side
          <div style={{ fontSize: 16, color: color.fg2 }}>set by the example</div>
        </div>
        <div style={{ position: 'absolute', left: (pauseX + aimX) / 2 - 20, top: y0 - 230, fontFamily: font.sans, fontSize: 64, color: color.fg0, opacity: options }}>?</div>
        <div style={{ position: 'absolute', left: aimX + 18, top: y0 - Math.max(...candidates) * PX_PER_M - 40, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: options }}>
          candidate paths
          <div style={{ fontSize: 16, color: color.fg2 }}>schematic</div>
        </div>
        <div style={{ position: 'absolute', right: 96, bottom: 168, fontFamily: font.mono, fontSize: 16, color: color.fg2, opacity: ahead }}>
          downrange compressed · sideways {PX_PER_M} px per metre
        </div>

        {/* The bank dial: the vehicle seen from behind, its lift tilted by the bank angle. */}
        <div style={{ position: 'absolute', left: 120, top: 652, opacity: fade(frame, at(ph, 0, 0), at(ph, 0, 0.2)) }}>
          <svg width={240} height={200}>
            <path d="M30,150 A90,90 0 0 1 210,150" fill="none" stroke={color.line2} strokeWidth={2} />
            {[-30, -15, 0, 15, 30].map((d) => {
              const r = ((d - 90) * Math.PI) / 180;
              return <line key={d} x1={120 + 82 * Math.cos(r)} y1={150 + 82 * Math.sin(r)} x2={120 + 92 * Math.cos(r)} y2={150 + 92 * Math.sin(r)} stroke={color.fg2} strokeWidth={1.5} />;
            })}
            <g transform={`rotate(${bank} 120 150)`}>
              <line x1={120} y1={150} x2={120} y2={76} stroke={color.accent} strokeWidth={4} strokeLinecap="round" />
              <path d="M120,62 L111,80 L129,80 Z" fill={color.accent} />
              <ellipse cx={120} cy={150} rx={30} ry={12} fill={color.bg1} stroke={color.fg1} strokeWidth={2} />
            </g>
          </svg>
          <div style={{ marginTop: -8, fontFamily: font.mono, fontSize: 19, color: color.fg1, whiteSpace: 'nowrap' }}>
            bank angle <span style={{ color: color.fg0 }}>{Math.abs(bank).toFixed(0)}°</span>
            <div style={{ fontSize: 16, color: color.fg2 }}>the roll that tilts the lift sideways</div>
          </div>
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
