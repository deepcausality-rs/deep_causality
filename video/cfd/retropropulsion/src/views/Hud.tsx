/**
 * The heads-up layer the flight scenes share: the timeline with the run's events, and the live
 * readout of the step on screen.
 */
import type { Retro, Sample } from '../data/retro';
import { Readout, Timeline } from '@cfd-video/shared';

/** Altitude in km down to 1 km, in metres below it. */
export const altitudeText = (km: number) => (km >= 1 ? `${km.toFixed(1)} km` : `${(km * 1000).toFixed(0)} m`);

export const Hud: React.FC<{ retro: Retro; s: Sample; frozen?: number; opacity?: number }> = ({ retro: r, s, frozen = 0, opacity = 1 }) => (
  <div style={{ opacity }}>
    <Timeline
      t={s.t}
      tMax={r.events.end.t}
      blackout={r.events.blackout}
      marks={[
        { t: r.events.ignition.t, label: `ignition · ${r.events.ignition.t.toFixed(0)} s`, tone: 'accent' },
        { t: r.events.subsonic.t, label: 'subsonic', tone: 'fg' },
      ]}
      frozen={frozen}
      opacity={1}
    />
    <Readout
      rows={[
        { label: 'altitude', value: altitudeText(s.altitudeKm) },
        { label: 'Mach', value: s.mach.toFixed(2) },
        { label: 'GPS', value: s.denied ? 'lost' : 'linked', tone: s.denied ? 'warn' : 'accent' },
        { label: 'throttle', value: s.throttle.toFixed(2), tone: s.throttle > 0 ? 'accent' : undefined },
      ]}
    />
  </div>
);
