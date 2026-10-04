/**
 * Scene 4, the pause: the lit, plume-coupled capsule at the fork step. The first lines name the
 * problem on the picture: the plume holding the bow shock off. Then time stops, the camera circles
 * the frozen vehicle, a card lists the state the pause holds (from the act line of `output.txt`), and
 * five branches leave the capsule, one per throttle in the roster. The fan is schematic: its spread
 * carries no data, and the next scene draws the real branches.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig, Easing } from 'remotion';
import { sampleAt, type Retro } from '../data/retro';
import type { SceneTiming } from '../timeline';
import { at, Backdrop, Captions, color, Eyebrow, fade, Flight, font, noseOnStage, plumeLength, QuestionTitle, sheathFromDensity, Stage, stagnationTemperature, StateCard, VIEW_SCALE } from '@cfd-video/shared';
import type { Shot } from '@cfd-video/shared';
import { Hud } from '../views/Hud';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.45, 0, 0.55, 1);

export const Pause: React.FC<{ retro: Retro; timing: SceneTiming }> = ({ retro: r, timing }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const ph = timing.phrases;
  const f = r.events.fork;
  const s = sampleAt(r.samples, f.t);
  const N = r.branches.length;

  const stopAt = at(ph, 2, 0.25);
  const freeze = fade(frame, stopAt, stopAt + 12);
  const orbit = interpolate(frame, [stopAt, timing.duration], [0, 1], { ...clamp, easing: ease });
  const shot: Shot = { dolly: 0.6 + 0.1 * orbit, azimuth: -0.3 + 0.15 * orbit, lift: 0.42, pan: 0.9 };

  // Labels on the picture: the plume, and the bow shock it holds off.
  const plume = plumeLength(s.throttle, s.mach) * VIEW_SCALE;
  const label = fade(frame, at(ph, 1, 0), at(ph, 1, 0.2)) * (1 - fade(frame, at(ph, 2, 0), at(ph, 2, 0.2)));
  const question = fade(frame, 4, 20) * (1 - fade(frame, at(ph, 2, 0), at(ph, 2, 0.2)));
  const plumeAt = noseOnStage(shot, frame, 0.5 * plume, s.gamma, 0);
  const shockAt = noseOnStage(shot, frame, 1.2 * plume, s.gamma, 0);

  const card = interpolate(frame, [at(ph, 2, 0.3), at(ph, 3, 0.9)], [0, 1], clamp);
  // The branches leave the capsule along the flight path and spread across it.
  const fan = interpolate(frame, [at(ph, 3, 0.0), at(ph, 3, 0.7)], [0, 1], { ...clamp, easing: Easing.bezier(0.16, 1, 0.3, 1) });
  const nose = noseOnStage(shot, frame, 1.1 * plume, s.gamma, 0);
  const ahead = noseOnStage(shot, frame, 1.1 * plume + 0.32, s.gamma, 0);
  const dx = ahead.x - nose.x;
  const dy = ahead.y - nose.y;
  const len = Math.hypot(dx, dy);
  const across = { x: -dy / len, y: dx / len };
  const end = (k: number) => {
    const spread = (k - (N - 1) / 2) * 38;
    return { x: nose.x + dx + across.x * spread, y: nose.y + dy + across.y * spread };
  };
  const path = (k: number) => {
    const e = end(k);
    return `M${nose.x},${nose.y} C${nose.x + dx * 0.45},${nose.y + dy * 0.45} ${e.x - dx * 0.35},${e.y - dy * 0.35} ${e.x},${e.y}`;
  };
  const names = [...r.branches].sort((a, b) => a.commanded - b.commanded).map((b) => (b.commanded === 0 ? 'coast' : b.commanded.toFixed(2)));

  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <Stage>
        <Backdrop limbY={810 - (s.altitudeKm - 30) * 0.5} drift={Math.min(frame, stopAt) * 0.2} />
      </Stage>
      <AbsoluteFill style={{ filter: `brightness(${1 - 0.3 * freeze}) saturate(${1 - 0.35 * freeze})` }}>
        <Flight
          tStag={stagnationTemperature(s.heatFlux)}
          sheath={sheathFromDensity(s.ne, s.mach)}
          shot={shot}
          gamma={s.gamma}
          alpha={0}
          throttle={s.throttle}
          mach={s.mach}
          wakeTime={Math.min(frame, stopAt) / fps}
        />
      </AbsoluteFill>
      <Stage>
        <Eyebrow text="Question 1 · the pause, mid-burn" />
        <QuestionTitle n={1} text="How hard to burn?" x={96} y={230} opacity={question} />
        <Hud retro={r} s={s} frozen={freeze} />
        <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
          {Array.from({ length: N }, (_, k) => (
            <path
              key={k}
              d={path(k)}
              fill="none"
              stroke={color.accent}
              strokeOpacity={0.7}
              strokeWidth={2}
              pathLength={1}
              strokeDasharray={1}
              strokeDashoffset={1 - interpolate(fan, [k * 0.04, 0.7 + k * 0.04], [0, 1], clamp)}
            />
          ))}
        </svg>
        {names.map((n, k) => {
          const e = end(k);
          return (
            <div
              key={n}
              style={{
                position: 'absolute',
                left: e.x - 130,
                top: e.y - 12,
                width: 116,
                textAlign: 'right',
                fontFamily: font.mono,
                fontSize: 19,
                color: color.accent,
                opacity: interpolate(fan, [0.5 + k * 0.08, 0.8 + k * 0.04], [0, 1], clamp),
              }}
            >
              {n}
            </div>
          );
        })}
        <div style={{ position: 'absolute', left: Math.max(96, Math.min(end(0).x, end(N - 1).x) - 130), top: Math.max(end(0).y, end(N - 1).y) + 34, fontFamily: font.mono, fontSize: 16, color: color.fg2, opacity: fan, whiteSpace: 'nowrap' }}>
          {N} branches, one starting state · schematic
        </div>
        <div style={{ position: 'absolute', left: plumeAt.x - 60, top: plumeAt.y + 70, fontFamily: font.mono, fontSize: 20, color: color.fg0, opacity: label, whiteSpace: 'nowrap' }}>
          retro plume, firing forward
        </div>
        <div style={{ position: 'absolute', left: shockAt.x - 190, top: shockAt.y - 110, fontFamily: font.mono, fontSize: 20, color: color.fg0, opacity: label, whiteSpace: 'nowrap' }}>
          bow shock, held off by the plume
        </div>
        <StateCard
          x={1240}
          y={430}
          progress={card}
          title="What the pause holds"
          rows={[
            { label: 'altitude', value: `${f.altitudeKm.toFixed(2)} km` },
            { label: 'Mach', value: f.mach.toFixed(2) },
            { label: 'dynamic pressure', value: `${f.q.toFixed(0)} Pa` },
            { label: 'throttle', value: f.throttle.toFixed(2) },
            { label: 'propellant', value: `${f.propellant.toFixed(1)} kg` },
          ]}
          note={`The marched flow with the plume on it, shared by all ${N} branches.`}
        />
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
