/**
 * Scene 5, commit and fly through: the committed branch from the pause to the end of the run. The
 * flight clock is keyed to the narration: the 61 km station on its line, the peak drift as the
 * blackout line ends, GPS return and the first fix on the last two. Every number is read off the
 * trace.
 */
import { AbsoluteFill, interpolate, useCurrentFrame } from 'remotion';
import { sampleAt, type Corridor } from '../data/corridor';
import type { SceneTiming } from '../timeline';
import { at, Backdrop, Captions, color, Eyebrow, fade, Flight, font, noseOnStage, Readout, SatLinks, Sci, sheathFromDensity, Stage, stagnationTemperature, Strip } from '@cfd-video/shared';
import type { Shot } from '@cfd-video/shared';
import { Timeline } from '../views/Timeline';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;

export const FlyThrough: React.FC<{ corridor: Corridor; timing: SceneTiming }> = ({ corridor: c, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const t = interpolate(
    frame,
    [at(ph, 0, 0.25), at(ph, 1, 0.75), at(ph, 3, 0), at(ph, 4, 0.9), at(ph, 5, 0.35), at(ph, 6, 0.6)],
    [c.pause.t, c.peakPassage.t, c.peakPassage.t + 14, c.darkNavPeak.t, c.exit.t, c.endT],
    clamp
  );
  const s = sampleAt(c, t);
  const link = s.denied ? 0 : fade(frame, at(ph, 5, 0.35), at(ph, 5, 0.35) + 10);
  const shot: Shot = { dolly: interpolate(frame, [0, timing.duration], [0.3, 0.6], clamp), azimuth: -0.42, lift: 0.42 };
  const hud = fade(frame, 0, 18);
  const times = c.descent.map((d) => d.t);

  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <Stage>
        <Backdrop limbY={810 - (s.altitudeKm - 45) * 0.8} drift={frame * 0.2} />
      </Stage>
      <Flight tStag={stagnationTemperature(s.heatFlux)} sheath={sheathFromDensity(s.ne)} shot={shot} bank={s.bank} />
      <Stage>
        <SatLinks to={noseOnStage(shot, frame)} link={link} />
        <div style={{ opacity: hud }}>
          <Eyebrow text={`DeepCausality · the committed branch, ${c.committed.bank}°`} />
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
            y={636}
            w={820}
            h={232}
            title="electron density, log scale, m⁻³"
            t={times}
            v={c.descent.map((d) => d.ne)}
            tFrom={0}
            tTo={c.endT}
            tNow={t}
            lo={14}
            hi={21}
            log
            threshold={{ value: c.ramcReference, label: 'RAM-C II flight, 1970: 10¹⁹', tone: 'fg' }}
            marks={[
              {
                t: c.peakPassage.t,
                value: c.peakPassage.electronDensity,
                label: (
                  <>
                    {c.peakPassage.altitudeKm.toFixed(1)} km: <Sci value={c.peakPassage.electronDensity} />
                  </>
                ),
                tone: 'accent',
                below: true,
              },
            ]}
          />
          <Strip
            x={1004}
            y={636}
            w={820}
            h={232}
            title="navigation error, m"
            t={times}
            v={c.descent.map((d) => d.navErr)}
            tFrom={0}
            tTo={c.endT}
            tNow={t}
            lo={0}
            hi={Math.ceil(c.darkNavPeak.navErr / 10) * 10}
            marks={[
              { t: c.darkNavPeak.t, value: c.darkNavPeak.navErr, label: `${c.darkNavPeak.navErr.toFixed(1)} m in the dark`, tone: 'warn', left: true },
              { t: c.exit.t, value: c.exit.navErr, label: `first fix: ${c.exit.navErr.toFixed(2)} m`, tone: 'accent', left: true },
            ]}
          />
          <div
            style={{
              position: 'absolute',
              right: 114,
              top: 372,
              width: 300,
              fontFamily: font.mono,
              fontSize: 19,
              color: color.fg1,
              opacity: fade(frame, at(ph, 6, 0.6), at(ph, 6, 0.9)),
            }}
          >
            at the end: <span style={{ color: color.accent }}>{c.endNavErr.toFixed(2)} m</span>
          </div>
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
