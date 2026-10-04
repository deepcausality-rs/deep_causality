/**
 * The heads-up layer of the flight scene: the flight clock with the world's blackout window, and the
 * live readout of the step on screen.
 */
import type { Sample, Weather, World } from '../data/weather';
import { Readout, Timeline } from '@cfd-video/shared';

export const Hud: React.FC<{ weather: Weather; world: World; s: Sample; opacity?: number }> = ({ weather: w, world, s, opacity = 1 }) => (
  <div style={{ opacity }}>
    <Timeline t={s.t} tMax={w.flightS} blackout={{ from: world.onset, to: world.exit }} marks={[]} frozen={0} opacity={1} />
    <Readout
      rows={[
        { label: 'world', value: world.label, size: 21 },
        { label: 'altitude', value: `${s.altitudeKm.toFixed(1)} km` },
        { label: 'Mach', value: s.mach.toFixed(2) },
        { label: 'GPS', value: s.denied ? 'lost' : 'linked', tone: s.denied ? 'warn' : 'accent' },
        { label: 'nav error', value: `${s.navErr.toFixed(1)} m`, tone: s.denied ? 'warn' : undefined },
      ]}
    />
  </div>
);
