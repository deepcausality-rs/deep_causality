/**
 * Scene 1, the problem: entry to the moment GPS is lost. The flight clock runs from entry to the
 * pause over the narration; altitude, Mach, sheath and nose temperature are read off the trace at
 * that clock, and the plasma-frequency strip crosses the GPS L1 band on the traced step.
 */
import { AbsoluteFill, interpolate, useCurrentFrame } from 'remotion';
import type { Corridor } from '../data/corridor';
import type { SceneTiming } from '../timeline';
import { at, Backdrop, Captions, color, Eyebrow, fade, Flight, font, noseOnStage, Readout, SatLinks, sheathFromDensity, Stage, stagnationTemperature, Strip } from '@cfd-video/shared';
import type { Shot } from '@cfd-video/shared';
import { Timeline } from '../views/Timeline';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;

export const sampleAt = (c: Corridor, t: number) => c.descent.reduce((a, s) => (Math.abs(s.t - t) < Math.abs(a.t - t) ? s : a));

export const Problem: React.FC<{ corridor: Corridor; timing: SceneTiming }> = ({ corridor: c, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  // The clock reaches the pause as the third phrase says so.
  const t = interpolate(frame, [ph[0].start, at(ph, 2, 0.8)], [c.descent[0].t, c.pause.t], clamp);
  const s = sampleAt(c, t);
  const link = s.denied ? 0 : 1 - fade(frame, at(ph, 2, 0.8) - 8, at(ph, 2, 0.8));
  const shot: Shot = { dolly: interpolate(frame, [0, timing.duration], [0, 0.55], clamp), azimuth: 0.05, lift: 0.34 };
  const title = interpolate(frame, [12, 30, ph[0].start - 10, ph[0].start + 6], [0, 1, 1, 0], clamp);
  const hud = fade(frame, ph[0].start - 6, ph[0].start + 14);

  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <Stage>
        <Backdrop limbY={800 - (s.altitudeKm - 70) * 0.6} drift={frame * 0.15} />
      </Stage>
      <Flight tStag={stagnationTemperature(s.heatFlux)} sheath={sheathFromDensity(s.ne)} shot={shot} />
      <Stage>
        <SatLinks to={noseOnStage(shot, frame)} link={link * hud} />
        <div style={{ opacity: hud }}>
          <Eyebrow text="DeepCausality · plasma-blackout corridor" />
          <Timeline t={t} tMax={c.endT} pauseT={c.pause.t} frozen={0} opacity={1} />
          <Readout
            rows={[
              { label: 'altitude', value: `${s.altitudeKm.toFixed(1)} km`, size: 24 },
              { label: 'Mach', value: s.mach.toFixed(1), size: 24 },
              { label: 'GPS', value: link > 0.5 ? 'linked' : 'lost', tone: link > 0.5 ? 'accent' : 'warn', size: 22 },
            ]}
          />
          <Strip
            x={96}
            y={640}
            w={640}
            h={220}
            title="plasma frequency, log scale"
            t={c.descent.map((d) => d.t)}
            v={c.descent.map((d) => d.plasmaFreq)}
            tFrom={0}
            tTo={c.pause.t + 1.5}
            tNow={t}
            lo={4}
            hi={12}
            log
            threshold={{ value: c.bandRadS, label: 'GPS L1', tone: 'warn' }}
            marks={[{ t: c.pause.t, value: c.bandRadS, label: `GPS lost · ${c.pause.altitudeKm.toFixed(1)} km`, tone: 'warn', left: true }]}
          />
        </div>
        {/* The title card, over the approach before the first line. */}
        <div style={{ position: 'absolute', left: 1024, top: 290, width: 800, opacity: title }}>
          <div style={{ fontFamily: font.mono, fontSize: 22, letterSpacing: '0.16em', textTransform: 'uppercase', color: color.accent }}>
            DeepCausality · counterfactual dynamics
          </div>
          <div style={{ marginTop: 18, fontFamily: font.sans, fontSize: 88, fontWeight: 500, letterSpacing: '-0.02em', color: color.fg0, lineHeight: 1.05 }}>
            Seventeen
            <br />
            counterfactual
            <br />
            trajectories
          </div>
          <div style={{ marginTop: 22, fontFamily: font.sans, fontSize: 30, color: color.fg1 }}>A reentry through plasma blackout, forked mid-flight</div>
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
