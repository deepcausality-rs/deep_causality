/**
 * Scene 1, entry: the title over the capsule's descent on the standard day, then the two questions.
 * The flight clock runs from entry to just past the blackout onset; altitude, Mach, heating, sheath,
 * flight-path angle and the navigation error are read off the baseline's reference trace.
 */
import { AbsoluteFill, interpolate, useCurrentFrame } from 'remotion';
import { sampleAt, type Weather } from '../data/weather';
import type { SceneTiming } from '../timeline';
import { Agenda, at, Backdrop, Captions, color, Eyebrow, fade, Flight, font, noseOnStage, SatLinks, sheathFromDensity, Stage, stagnationTemperature } from '@cfd-video/shared';
import type { Shot } from '@cfd-video/shared';
import { Hud } from '../views/Hud';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;

export const Entry: React.FC<{ weather: Weather; timing: SceneTiming }> = ({ weather: w, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const b = w.worlds[0];
  const t = interpolate(frame, [0, timing.duration], [b.trace[0].t, b.onset + 4], clamp);
  const s = sampleAt(b.trace, t);
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
          <SatLinks to={noseOnStage(shot, frame, 0, s.gamma)} link={s.denied ? 0 : 1} />
          <Eyebrow text="DeepCausality · weather dispersion" />
          <Hud weather={w} world={b} s={s} />
        </div>
        <div style={{ position: 'absolute', left: 1024, top: 290, width: 800, opacity: title }}>
          <div style={{ fontFamily: font.mono, fontSize: 22, letterSpacing: '0.16em', textTransform: 'uppercase', color: color.accent }}>
            DeepCausality · counterfactual dynamics
          </div>
          <div style={{ marginTop: 18, fontFamily: font.sans, fontSize: 88, fontWeight: 500, letterSpacing: '-0.02em', color: color.fg0, lineHeight: 1.05 }}>
            Six
            <br />
            counterfactual
            <br />
            atmospheres
          </div>
          <div style={{ marginTop: 22, fontFamily: font.sans, fontSize: 30, color: color.fg1 }}>One descent through six weathers, reduced to one table</div>
        </div>
        {/* The two questions the video answers. */}
        <Agenda
          questions={['Does the weather move the GPS blackout?', 'Does it move the navigation error?']}
          x={940}
          y={470}
          width={880}
          titleOpacity={fade(frame, at(ph, 1, 0.05), at(ph, 1, 0.2))}
          lineOpacity={(i) => fade(frame, at(ph, 1, 0.2 + 0.35 * i), at(ph, 1, 0.35 + 0.35 * i))}
        />
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
