/**
 * Scene 6, burn-out and the landing: the flight resumes from the fork step on its own guidance,
 * burns to the subsonic handover, cuts off, falls, lights its stopping burn and touches down. The
 * flight clock is keyed to the narration; altitude, Mach, throttle, descent rate and flight-path
 * angle are read off the trace. The last line cuts to a fixed camera on the ground, which the
 * capsule descends into.
 */
import { AbsoluteFill, interpolate, useCurrentFrame } from 'remotion';
import { sampleAt, type Retro } from '../data/retro';
import type { SceneTiming } from '../timeline';
import { at, Backdrop, Captions, color, Eyebrow, fade, Flight, sheathFromDensity, Stage, stagnationTemperature, Strip, VIEW_SCALE } from '@cfd-video/shared';
import type { FixedCamera, Shot } from '@cfd-video/shared';
import { Hud } from '../views/Hud';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
/**
 * The ground camera, scene units: it aims `track` below the capsule while following it, and
 * `above` the ground once it holds, looking up by `rise` over its 9.75 distance.
 */
const GROUND_CAM = { track: -0.25, above: 0.9, rise: 0.35, fov: 22 };
const GROUND_HORIZON = 540 + (Math.tan(Math.atan2(GROUND_CAM.rise, Math.hypot(2.6, 9.4))) / Math.tan((GROUND_CAM.fov * Math.PI) / 360)) * 540;

/**
 * The night sky behind the view from the ground, its horizon at the ground camera's eye level:
 * `horizon` is the stage height where the level line of sight lands.
 */
const GroundSky: React.FC<{ horizon: number }> = ({ horizon }) => (
  <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
    <defs>
      <linearGradient id="night" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stopColor="#010307" />
        <stop offset={(horizon / 1080) * 0.85} stopColor="#06101f" />
        <stop offset={horizon / 1080} stopColor="#0f2240" />
        <stop offset={horizon / 1080 + 0.001} stopColor="#05080c" />
        <stop offset="1" stopColor="#030508" />
      </linearGradient>
    </defs>
    <rect width={1920} height={1080} fill="url(#night)" />
  </svg>
);

export const Landing: React.FC<{ retro: Retro; timing: SceneTiming }> = ({ retro: r, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const e = r.events;
  const land = r.landings.informed;
  const t = interpolate(
    frame,
    [0, at(ph, 0, 0.9), at(ph, 1, 0.1), at(ph, 2, 0), at(ph, 2, 0.85), at(ph, 3, 0.85)],
    [e.fork.t, e.subsonic.t, e.subsonic.t + 3, land.lightT - 6, land.lightT, land.touchdownT],
    clamp
  );
  const s = sampleAt(r.samples, t);
  const heightM = s.altitudeKm * 1000 - r.groundM;

  // Shot A follows the capsule; shot B waits on the ground for it.
  const cut = at(ph, 3, 0);
  const toGround = fade(frame, cut - 12, cut + 6);
  const shot: Shot = {
    dolly: interpolate(frame, [0, cut], [0.6, 0.75], clamp),
    azimuth: interpolate(frame, [0, cut], [-0.3, -0.05], clamp),
    lift: 0.42,
    pan: interpolate(frame, [at(ph, 0, 0.9), at(ph, 1, 0.3), at(ph, 2, 0.6), at(ph, 2, 1)], [0.7, 0, 0, 0.5], clamp),
  };
  // The ground camera rides down with the capsule, then holds on the touchdown spot for the last
  // metres, so the capsule settles onto the ground in frame.
  const hs = heightM * VIEW_SCALE;
  const groundY = -hs;
  const targetY = Math.max(groundY + GROUND_CAM.above, GROUND_CAM.track);
  const ground: FixedCamera = { position: [2.6, targetY - GROUND_CAM.rise, 9.4], target: [0, targetY, 0], fov: GROUND_CAM.fov };
  const flight = {
    tStag: stagnationTemperature(s.heatFlux),
    sheath: sheathFromDensity(s.ne, s.mach),
    gamma: s.gamma,
    alpha: 0,
    throttle: s.throttle,
    mach: s.mach,
  };
  // The strips wait for cutoff, so they do not cover the plume during the burn.
  const strips = fade(frame, at(ph, 1, 0), at(ph, 1, 0.3)) * (1 - toGround);
  const times = r.samples.map((x) => x.t);
  const tFrom = e.fork.t - 10;

  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <AbsoluteFill style={{ opacity: 1 - toGround }}>
        <Stage>
          <Backdrop limbY={810 - (s.altitudeKm - 30) * 0.5} drift={frame * 0.2} />
        </Stage>
        {toGround < 1 && <Flight {...flight} shot={shot} />}
      </AbsoluteFill>
      <AbsoluteFill style={{ opacity: toGround }}>
        <Stage>
          <GroundSky horizon={GROUND_HORIZON} />
        </Stage>
        {toGround > 0 && <Flight {...flight} shot={shot} camera={ground} groundM={heightM} />}
      </AbsoluteFill>
      <Stage>
        <Eyebrow text="DeepCausality · burn-out and the landing" />
        <Hud retro={r} s={s} />
        <div style={{ opacity: strips }}>
          <Strip
            x={96}
            y={636}
            w={820}
            h={232}
            title="throttle flown"
            t={times}
            v={r.samples.map((x) => x.throttle)}
            tFrom={tFrom}
            tTo={e.end.t}
            tNow={t}
            lo={0}
            hi={1}
            marks={[
              { t: e.subsonic.t, value: sampleAt(r.samples, e.subsonic.t).throttle, label: `cutoff · Mach ${e.subsonic.mach.toFixed(1)}`, tone: 'fg' },
              { t: land.lightT, value: sampleAt(r.samples, land.lightT).throttle, label: `stopping burn · ${land.printed.lightM.toFixed(2)} m`, tone: 'accent', left: true },
            ]}
          />
          <Strip
            x={1004}
            y={636}
            w={820}
            h={232}
            title="descent rate, m/s"
            t={times}
            v={r.samples.map((x) => x.descentRate)}
            tFrom={tFrom}
            tTo={e.end.t}
            tNow={t}
            lo={0}
            hi={Math.ceil(Math.max(...r.samples.filter((x) => x.t >= tFrom).map((x) => x.descentRate)) / 50) * 50}
            marks={[{ t: land.touchdownT, value: land.contactMs, label: `touchdown · ${land.contactMs.toFixed(2)} m/s`, tone: 'accent', left: true }]}
          />
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
