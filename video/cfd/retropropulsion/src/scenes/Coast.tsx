/**
 * Scene 3, blackout and the coast: from entry through the blackout window, the engine-off coast,
 * and the ignition commit. The flight clock is keyed to the narration; altitude, Mach, heating,
 * sheath, flight-path angle and throttle are read off the trace. The capsule flies its trim angle of
 * attack until the engine lights, then on axis, as the retro burn is flown.
 */
import { AbsoluteFill, interpolate, useCurrentFrame } from 'remotion';
import { sampleAt, type Retro } from '../data/retro';
import type { SceneTiming } from '../timeline';
import { at, Backdrop, Captions, color, Eyebrow, fade, Flight, noseOnStage, SatLinks, sheathFromDensity, Stage, stagnationTemperature, Strip, TRIM_ALPHA } from '@cfd-video/shared';
import type { Shot } from '@cfd-video/shared';
import { Hud } from '../views/Hud';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
/** The ignition corridor's Mach band, `IGNITION_MACH_MIN` and `IGNITION_MACH_MAX` (`shared/constants.rs`). */
const IGNITION_MACH = [0.4, 2.0] as const;

export const Coast: React.FC<{ retro: Retro; timing: SceneTiming }> = ({ retro: r, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const e = r.events;
  const lit = e.fork.t;
  const t = interpolate(
    frame,
    [0, at(ph, 0, 0.9), at(ph, 1, 0.95), at(ph, 2, 0.35), at(ph, 2, 0.5)],
    [e.blackout.from - 1.5, e.blackout.to + 1, e.ignition.t - 3, e.ignition.t, lit],
    clamp
  );
  const s = sampleAt(r.samples, t);
  // On axis once the engine lights: the burn is flown with no angle of attack.
  const alpha = interpolate(frame, [at(ph, 2, 0.45), at(ph, 2, 0.8)], [TRIM_ALPHA, 0], clamp);
  const shot: Shot = {
    dolly: interpolate(frame, [0, timing.duration], [0.25, 0.6], clamp),
    azimuth: interpolate(frame, [0, timing.duration], [0.05, -0.3], clamp),
    lift: 0.42,
    // Room ahead of the capsule for the plume once the engine lights.
    pan: interpolate(frame, [at(ph, 2, 0.45), at(ph, 2, 0.95)], [0, 0.9], clamp),
  };
  const link = s.denied ? 0 : 1;
  const times = r.samples.map((x) => x.t);

  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <Stage>
        <Backdrop limbY={810 - (s.altitudeKm - 30) * 0.5} drift={frame * 0.2} />
      </Stage>
      <Flight
        tStag={stagnationTemperature(s.heatFlux)}
        sheath={sheathFromDensity(s.ne, s.mach)}
        shot={shot}
        gamma={s.gamma}
        alpha={alpha}
        throttle={s.throttle}
        mach={s.mach}
      />
      <Stage>
        <SatLinks to={noseOnStage(shot, frame, 0, s.gamma, alpha)} link={link} />
        <div style={{ opacity: fade(frame, 0, 18) }}>
          <Eyebrow text="DeepCausality · blackout and the coast" />
          <Hud retro={r} s={s} />
          <Strip
            x={96}
            y={636}
            w={820}
            h={232}
            title="Mach"
            t={times}
            v={r.samples.map((x) => x.mach)}
            tFrom={0}
            tTo={e.end.t}
            tNow={t}
            lo={0}
            hi={Math.ceil(e.entry.mach / 5) * 5}
            threshold={{ value: IGNITION_MACH[1], label: `ignition band: Mach ${IGNITION_MACH[0]} to ${IGNITION_MACH[1].toFixed(1)}`, tone: 'fg' }}
            marks={[{ t: e.ignition.t, value: e.ignition.mach, label: `ignition · Mach ${e.ignition.mach.toFixed(2)}`, tone: 'accent' }]}
          />
          <Strip
            x={1004}
            y={636}
            w={820}
            h={232}
            title="altitude, km"
            t={times}
            v={r.samples.map((x) => x.altitudeKm)}
            tFrom={0}
            tTo={e.end.t}
            tNow={t}
            lo={0}
            hi={Math.ceil(e.entry.altitudeKm / 10) * 10}
            marks={[
              { t: e.blackout.to, value: sampleAt(r.samples, e.blackout.to).altitudeKm, label: 'GPS back', tone: 'warn' },
              { t: e.ignition.t, value: e.ignition.altitudeKm, label: `${e.ignition.altitudeKm.toFixed(1)} km`, tone: 'accent' },
            ]}
          />
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
