/**
 * The retropulsion descent, step by step, for the walk 3 animation.
 *
 * Read at build time from the files the retropulsion example commits beside its `output.txt`:
 * `retropulsion_trace.csv` (the informed descent across its four legs), `retropulsion_branch_trace.csv`
 * (the five throttle branches after the fork), `retropulsion_uninformed_trace.csv` (the landing leg
 * flown with the standard-day margin) and `retropulsion_branches.csv` (the roster table). The plan
 * reads walk 2's `weather_table.csv`, the same file the program reads in flight.
 *
 * Nothing is typed by hand. The build fails if interpolating the table does not reproduce the
 * margins `output.txt` prints, if a branch trace does not end on its table row, or if the landing
 * traces do not reproduce the printed burn-light altitudes and contact speeds.
 */
import descentCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/retropulsion/retropulsion_trace.csv?raw';
import branchCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/retropulsion/retropulsion_branch_trace.csv?raw';
import uninformedCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/retropulsion/retropulsion_uninformed_trace.csv?raw';
import rosterCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/retropulsion/retropulsion_branches.csv?raw';
import outputTxt from '../../../../examples/avionics_examples/cfd/plasma_blackout/retropulsion/output.txt?raw';
import weatherCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/weather/weather_table.csv?raw';
import { parseGates, parseRows, type Row } from './traceCsv';

const fail = (msg: string): never => {
  throw new Error(`retropulsion data: ${msg}`);
};
const near = (a: number, b: number, tol: number) => Math.abs(a - b) <= tol;
const printed = (re: RegExp, what: string) => {
  const m = outputTxt.match(re);
  return m ?? fail(`output.txt has no ${what}`);
};

// ── Plan: the measured day reads walk 2's table ──────────────────────────────────────────────
const table = parseRows(weatherCsv, 'weather_table.csv').sort((a, b) => a.d_temp - b.d_temp);
const plan = printed(
  /measured dT = (-?[0-9.]+) K -> drift ([0-9.]+) \+- ([0-9.]+) m, ignition margin ([0-9.]+) m \(k = ([0-9.]+)\)/,
  'Act 0 plan line'
);
const standardPlan = printed(/standard-day belief \(the uninformed world\) -> margin ([0-9.]+) m/, 'standard-day belief line');
const measuredDT = Number(plan[1]);
const k = Number(plan[5]);
const lo = [...table].reverse().find((r) => r.d_temp <= measuredDT) ?? fail('measured day below the table');
const hi = table.find((r) => r.d_temp >= measuredDT) ?? fail('measured day above the table');
const w = hi.d_temp === lo.d_temp ? 0 : (measuredDT - lo.d_temp) / (hi.d_temp - lo.d_temp);
const lerp = (key: string) => lo[key] + w * (hi[key] - lo[key]);
const standardRow = table.find((r) => r.d_temp === 0) ?? fail('no standard-day row');

export const belief = {
  measuredDT,
  k,
  rows: table.map((r) => ({ dTemp: r.d_temp, mean: r.drift_mean, sd: r.drift_sd })),
  bracket: [lo.d_temp, hi.d_temp] as const,
  informed: { mean: lerp('drift_mean'), sd: lerp('drift_sd'), margin: lerp('drift_mean') + k * lerp('drift_sd') },
  uninformed: { mean: standardRow.drift_mean, sd: standardRow.drift_sd, margin: standardRow.drift_mean + k * standardRow.drift_sd },
};
if (
  !near(belief.informed.mean, Number(plan[2]), 0.005) ||
  !near(belief.informed.sd, Number(plan[3]), 0.005) ||
  !near(belief.informed.margin, Number(plan[4]), 0.005) ||
  !near(belief.uninformed.margin, Number(standardPlan[1]), 0.005)
) {
  fail('interpolating weather_table.csv does not reproduce the margins output.txt prints');
}

// ── The informed descent ─────────────────────────────────────────────────────────────────────
const descent = parseRows(descentCsv, 'retropulsion_trace.csv');
export interface Sample {
  leg: number;
  t: number;
  altitudeKm: number;
  mach: number;
  denied: boolean;
  throttle: number;
  propellant: number;
  descentRate: number;
}
const toSample = (r: Row): Sample => ({
  leg: r.leg,
  t: r.t,
  altitudeKm: r.altitude / 1000,
  mach: r.mach,
  denied: r.gnss_denied === 1,
  throttle: r.throttle,
  propellant: r.propellant,
  descentRate: r.descent_rate,
});
export const samples: Sample[] = descent.map(toSample);
const uninformedSamples: Sample[] = parseRows(uninformedCsv, 'retropulsion_uninformed_trace.csv').map(toSample);

const legEnd = (leg: number) => samples.findLastIndex((s) => s.leg === leg);
const deniedFrom = samples.findIndex((s) => s.denied);
const deniedTo = samples.findLastIndex((s) => s.denied);
// The throttle flown lags the command by one step, so the corridor commits on the step before the
// first lit row.
const commitIx = samples.findIndex((s, i) => s.leg === 2 && s.throttle > 0 && i > deniedTo) - 1;

/** The informed descent's events, each read off the trace. */
export const events = {
  blackout: { from: samples[deniedFrom].t, to: samples[deniedTo].t },
  ignition: {
    t: samples[commitIx].t,
    altitudeKm: samples[commitIx].altitudeKm,
    mach: samples[commitIx].mach,
  },
  fork: { t: samples[legEnd(2)].t },
  subsonic: { t: samples[legEnd(3)].t, altitudeKm: samples[legEnd(3)].altitudeKm, mach: samples[legEnd(3)].mach },
  end: { t: samples[samples.length - 1].t },
};
if (deniedFrom < 0 || commitIx < 0) fail('no blackout or ignition in retropulsion_trace.csv');
const commitPrinted = printed(/ignition corridor committed at step \d+: Mach ([0-9.]+)/, 'ignition commit line');
if (!near(events.ignition.mach, Number(commitPrinted[1]), 1e-6)) {
  fail(`the trace commits at Mach ${events.ignition.mach}; output.txt says ${commitPrinted[1]}`);
}

// ── The landing, flown twice ─────────────────────────────────────────────────────────────────
/** The landing leg of one belief: where the stopping burn lights and how the vehicle arrives. */
function landing(trace: Sample[]) {
  const leg = trace.filter((s) => s.leg === 4);
  // Bang-bang: the leg coasts with the engine off, then lights once and burns to the ground.
  const coastIx = leg.findIndex((s) => s.throttle === 0);
  const lightIx = leg.findIndex((s, i) => i > coastIx && s.throttle > 0);
  if (coastIx < 0 || lightIx < 0) fail('no coast-then-burn in a landing leg');
  const last = leg[leg.length - 1];
  return {
    leg,
    // The burn lights on the step after the last coasting row; its start altitude is that row's.
    lightAltitudeM: leg[lightIx - 1].altitudeKm * 1000,
    lightT: leg[lightIx].t,
    touchdownT: last.t,
    contactMs: last.descentRate,
    propellantKg: last.propellant,
  };
}
const informedLanding = landing(samples);
const uninformedLanding = landing(uninformedSamples);
const beliefRow = (name: string) => {
  const m = printed(new RegExp(`^ {2}${name}\\s+([0-9.]+)\\s+([0-9.]+)\\s+([0-9.]+)\\s+([0-9.]+)`, 'm'), `${name} belief row`);
  return { margin: Number(m[1]), lightM: Number(m[2]), contactMs: Number(m[3]), reserveKg: Number(m[4]) };
};
const informedPrinted = beliefRow('informed');
const uninformedPrinted = beliefRow('uninformed');
for (const [trace, row, name] of [
  [informedLanding, informedPrinted, 'informed'],
  [uninformedLanding, uninformedPrinted, 'uninformed'],
] as const) {
  if (!near(trace.lightAltitudeM, row.lightM, 0.5) || !near(trace.contactMs, row.contactMs, 0.005)) {
    fail(`the ${name} landing trace lights at ${trace.lightAltitudeM} m and lands at ${trace.contactMs} m/s; output.txt says ${row.lightM} m and ${row.contactMs} m/s`);
  }
}
export const landings = {
  informed: { ...informedLanding, printed: informedPrinted },
  uninformed: { ...uninformedLanding, printed: uninformedPrinted },
};

// ── The mid-burn fork ────────────────────────────────────────────────────────────────────────
const roster = parseRows(rosterCsv, 'retropulsion_branches.csv');
const branchRows = parseRows(branchCsv, 'retropulsion_branch_trace.csv');
const names = new Map<number, string>();
for (const m of outputTxt.matchAll(/^ {2}([a-z]+)\s+([0-9.]+)\s+([0-9.]+)\s+(—|-?[0-9.]+)\s+([0-9.]+)/gm)) {
  names.set(Number(m[2]), m[1]);
}
export interface Branch {
  name: string;
  commanded: number;
  flown: number;
  preserved: number | null;
  deceleration: number;
  dvActual: number;
  dvFrozen: number;
  /** Seconds since the fork, and the velocity shed since the fork, m/s. */
  t: number[];
  dv: number[];
}
const forkT = events.fork.t;
export const branches: Branch[] = roster.map((r) => {
  const rows = branchRows.filter((b) => b.commanded_throttle === r.commanded_throttle);
  const name = names.get(r.commanded_throttle) ?? fail(`output.txt names no branch at throttle ${r.commanded_throttle}`);
  if (rows.length < 2) fail(`no branch trace for ${name}`);
  const last = rows[rows.length - 1];
  if (!near(last.axial_accel, r.net_deceleration, 1e-6) || !near(last.throttle, r.realized_throttle, 1e-9)) {
    fail(`the ${name} branch trace does not end on its retropulsion_branches.csv row`);
  }
  // The fork's accumulated value: the branch's final value less what the table says it shed.
  const dv0 = last.dv_actual - r.dv_actual;
  return {
    name,
    commanded: r.commanded_throttle,
    flown: r.realized_throttle,
    preserved: Number.isNaN(r.preserved_fraction) ? null : r.preserved_fraction,
    deceleration: r.net_deceleration,
    dvActual: r.dv_actual,
    dvFrozen: r.dv_frozen,
    t: rows.map((b) => b.t - forkT),
    dv: rows.map((b) => b.dv_actual - dv0),
  };
});

/** The gate lines of the committed run, as printed: label and pass flag. */
export const gates = parseGates(outputTxt);
