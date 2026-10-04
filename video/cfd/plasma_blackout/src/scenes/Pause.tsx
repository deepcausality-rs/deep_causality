/**
 * Scene 3, the pause: the descent stops the moment GPS is lost. Time freezes (the wake stops, the
 * frame dims, the timeline reads paused) and the camera circles the frozen vehicle while a card
 * lists the state the pause holds, read off the trace, each row as the narration names it. On the
 * last line the branches leave the nose, one per candidate bank angle. The fan is schematic: its
 * spread carries no data, and the next scene draws the real one.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig, Easing } from 'remotion';
import { sampleAt, type Corridor } from '../data/corridor';
import type { SceneTiming } from '../timeline';
import { at, Backdrop, Captions, color, Eyebrow, fade, Flight, font, noseOnStage, Sci, sheathFromDensity, Stage, stagnationTemperature, StateCard } from '@cfd-video/shared';
import type { Shot } from '@cfd-video/shared';
import { Timeline } from '../views/Timeline';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.45, 0, 0.55, 1);

export const Pause: React.FC<{ corridor: Corridor; timing: SceneTiming }> = ({ corridor: c, timing }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const ph = timing.phrases;
  const N = c.branches.length;

  // Time stops on "pauses the flight"; from then on the camera circles the frozen vehicle.
  const stopAt = at(ph, 0, 0.3);
  const freeze = fade(frame, stopAt, stopAt + 12);
  const orbit = interpolate(frame, [stopAt, timing.duration], [0, 1], { ...clamp, easing: ease });
  const shot: Shot = { dolly: 0.55 + 0.12 * orbit, azimuth: 0.05 - 0.3 * orbit, lift: 0.34 };
  const s = sampleAt(c, c.pause.t);

  // The card fills in as the narration names the flow, the sheath and the navigation filter.
  const card = interpolate(
    frame,
    [at(ph, 1, 0.2), at(ph, 2, 0.02), at(ph, 2, 0.2), at(ph, 2, 0.55), at(ph, 3, 0.1), at(ph, 4, 0.1), at(ph, 4, 0.4)],
    [0, 0.15, 0.325, 0.5, 0.675, 0.85, 1],
    clamp
  );

  // The branches leave the nose along the flight path and spread across it.
  const fan = interpolate(frame, [at(ph, 4, 0.05), at(ph, 4, 0.6)], [0, 1], { ...clamp, easing: Easing.bezier(0.16, 1, 0.3, 1) });
  const nose = noseOnStage(shot, frame);
  const ahead = noseOnStage(shot, frame, 0.4);
  const dx = ahead.x - nose.x;
  const dy = ahead.y - nose.y;
  const len = Math.hypot(dx, dy);
  const across = { x: -dy / len, y: dx / len };
  const branch = (k: number) => {
    const spread = (k - (N - 1) / 2) * 16;
    const end = { x: nose.x + dx + across.x * spread, y: nose.y + dy + across.y * spread };
    return `M${nose.x},${nose.y} C${nose.x + dx * 0.45},${nose.y + dy * 0.45} ${end.x - dx * 0.35},${end.y - dy * 0.35} ${end.x},${end.y}`;
  };

  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <Stage>
        <Backdrop limbY={800 - (c.pause.altitudeKm - 70) * 0.6} drift={Math.min(frame, stopAt) * 0.15} />
      </Stage>
      <AbsoluteFill style={{ filter: `brightness(${1 - 0.38 * freeze}) saturate(${1 - 0.4 * freeze})` }}>
        <Flight
          tStag={stagnationTemperature(s.heatFlux)}
          sheath={sheathFromDensity(s.ne)}
          shot={shot}
          wakeTime={Math.min(frame, stopAt) / fps}
        />
      </AbsoluteFill>
      <Stage>
        <Eyebrow text="DeepCausality · the pause" />
        <Timeline t={c.pause.t} tMax={c.endT} pauseT={c.pause.t} frozen={freeze} opacity={1} />
        <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
          {Array.from({ length: N }, (_, k) => (
            <path
              key={k}
              d={branch(k)}
              fill="none"
              stroke={color.accent}
              strokeOpacity={0.6}
              strokeWidth={1.8}
              pathLength={1}
              strokeDasharray={1}
              strokeDashoffset={1 - interpolate(fan, [k * 0.02, 0.7 + k * 0.018], [0, 1], clamp)}
            />
          ))}
          <circle cx={nose.x} cy={nose.y} r={7} fill={color.accent} opacity={fan > 0 ? 1 : 0} />
        </svg>
        <div
          style={{
            position: 'absolute',
            left: Math.max(96, nose.x + dx - 260),
            top: nose.y + dy + Math.abs(across.y) * ((N - 1) / 2) * 16 + 24,
            fontFamily: font.mono,
            fontSize: 19,
            color: color.accent,
            whiteSpace: 'nowrap',
            opacity: interpolate(fan, [0.5, 1], [0, 1], clamp),
          }}
        >
          {N} branches, one starting state
          <div style={{ fontSize: 16, color: color.fg2 }}>schematic</div>
        </div>
        <StateCard
          x={1240}
          y={220}
          progress={card}
          title="What the pause holds"
          rows={[
            { label: 'altitude', value: `${c.pause.altitudeKm.toFixed(1)} km` },
            { label: 'Mach', value: c.pause.mach.toFixed(1) },
            { label: 'electron density', value: <><Sci value={c.pause.electronDensity} /> m⁻³</> },
            { label: 'navigation error', value: `${c.pause.navErr.toFixed(2)} m` },
          ]}
          note={`Shared by all ${N} branches.`}
        />
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
