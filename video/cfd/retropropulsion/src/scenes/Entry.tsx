/**
 * Scene 1, entry: the title over the capsule's approach, then the cold day it flies. The flight
 * clock runs from entry toward blackout; altitude, Mach, heating, sheath and flight-path angle are
 * read off the trace at that clock.
 */
import { AbsoluteFill, interpolate, useCurrentFrame } from 'remotion';
import { sampleAt, type Retro } from '../data/retro';
import type { SceneTiming } from '../timeline';
import { Agenda, at, Backdrop, Captions, color, Eyebrow, fade, Flight, font, sheathFromDensity, Stage, stagnationTemperature } from '@cfd-video/shared';
import type { Shot } from '@cfd-video/shared';
import { Hud } from '../views/Hud';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;

export const Entry: React.FC<{ retro: Retro; timing: SceneTiming }> = ({ retro: r, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const t = interpolate(frame, [0, timing.duration], [r.samples[0].t, r.events.blackout.from - 1.5], clamp);
  const s = sampleAt(r.samples, t);
  const shot: Shot = { dolly: interpolate(frame, [0, timing.duration], [0, 0.4], clamp), azimuth: 0.05, lift: 0.34 };
  const title = interpolate(frame, [12, 30, ph[0].start - 10, ph[0].start + 6], [0, 1, 1, 0], clamp);
  const hud = fade(frame, ph[0].start - 6, ph[0].start + 14);

  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <Stage>
        <Backdrop limbY={800 - (s.altitudeKm - 70) * 0.6} drift={frame * 0.15} />
      </Stage>
      <Flight tStag={stagnationTemperature(s.heatFlux)} sheath={sheathFromDensity(s.ne, s.mach)} shot={shot} gamma={s.gamma} mach={s.mach} />
      <Stage>
        <div style={{ opacity: hud }}>
          <Eyebrow text="DeepCausality · plasma retropropulsion" />
          <Hud retro={r} s={s} />
        </div>
        <div style={{ position: 'absolute', left: 1024, top: 290, width: 800, opacity: title }}>
          <div style={{ fontFamily: font.mono, fontSize: 22, letterSpacing: '0.16em', textTransform: 'uppercase', color: color.accent }}>
            DeepCausality · counterfactual dynamics
          </div>
          <div style={{ marginTop: 18, fontFamily: font.sans, fontSize: 88, fontWeight: 500, letterSpacing: '-0.02em', color: color.fg0, lineHeight: 1.05 }}>
            Five
            <br />
            counterfactual
            <br />
            burns
          </div>
          <div style={{ marginTop: 22, fontFamily: font.sans, fontSize: 30, color: color.fg1 }}>From plasma blackout to touchdown, under a retro burn</div>
        </div>
        {/* The two questions the video answers. */}
        <Agenda
          questions={['How hard to burn at supersonic speed?', 'When to light the final burn?']}
          x={1024}
          y={430}
          titleOpacity={fade(frame, at(ph, 1, 0.05), at(ph, 1, 0.2))}
          lineOpacity={(i) => fade(frame, at(ph, 1, 0.2 + 0.35 * i), at(ph, 1, 0.35 + 0.35 * i))}
        />
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
