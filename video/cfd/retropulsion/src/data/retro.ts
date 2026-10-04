/**
 * The retropulsion run, read from the files the example commits (synced into public/traces/ by
 * `pnpm sync`), with every on-screen number checked against them: `retropulsion_trace.csv` (the
 * informed descent across its four legs), `retropulsion_branch_trace.csv` (the five throttle
 * branches after the fork), `retropulsion_uninformed_trace.csv` (the landing leg flown with the
 * standard-day margin), `retropulsion_branches.csv` (the roster table), `output.txt`, and the weather
 * example's `weather_table.csv`, which the run reads in flight.
 *
 * Nothing here is typed by hand. A mismatch throws, which fails `calculateMetadata` and so the
 * render.
 */
import { staticFile } from 'remotion';
import { parseRows, type Row } from '@cfd-video/shared/csv';

const fail = (msg: string): never => {
  throw new Error(`retropulsion data: ${msg}`);
};
const near = (a: number, b: number, tol: number) => Math.abs(a - b) <= tol;

export interface Sample {
  leg: number;
  t: number;
  altitudeKm: number;
  mach: number;
  denied: boolean;
  throttle: number;
  propellant: number;
  descentRate: number;
  speed: number;
  ne: number;
  heatFlux: number;
  /** Flight-path angle below the horizontal, rad: asin(descent rate / speed). */
  gamma: number;
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

export interface Landing {
  leg: Sample[];
  /** The burn lights on the step after the last coasting row; its altitude is that row's, m. */
  lightAltitudeM: number;
  lightT: number;
  touchdownT: number;
  contactMs: number;
  propellantKg: number;
  printed: { margin: number; lightM: number; contactMs: number; reserveKg: number };
}

export interface Retro {
  belief: {
    measuredDT: number;
    k: number;
    rows: { dTemp: number; mean: number; sd: number }[];
    bracket: [number, number];
    informed: { mean: number; sd: number; margin: number };
    uninformed: { mean: number; sd: number; margin: number };
  };
  samples: Sample[];
  events: {
    entry: { mach: number; altitudeKm: number };
    blackout: { from: number; to: number; dwell: number };
    ignition: { t: number; altitudeKm: number; mach: number };
    /** Engine off from blackout exit to the ignition commit, s. */
    coastS: number;
    fork: { t: number; altitudeKm: number; mach: number; throttle: number; q: number; propellant: number };
    subsonic: { t: number; altitudeKm: number; mach: number };
    end: { t: number };
  };
  branches: Branch[];
  branchSeconds: number;
  landings: { informed: Landing; uninformed: Landing };
  /** Height of the touchdown plane: the altitude at the last recorded step, m. */
  groundM: number;
  propellantLeftKg: number;
  totalSteps: number;
  runSeconds: number;
  gates: { passed: boolean; label: string }[];
}

const fetchText = async (file: string) => {
  const res = await fetch(staticFile(`traces/${file}`));
  if (!res.ok) fail(`cannot read traces/${file}; run pnpm sync`);
  return res.text();
};

export async function loadRetro(): Promise<Retro> {
  const [descentCsv, branchCsv, uninformedCsv, rosterCsv, outputTxt, weatherCsv] = await Promise.all([
    fetchText('retropulsion_trace.csv'),
    fetchText('retropulsion_branch_trace.csv'),
    fetchText('retropulsion_uninformed_trace.csv'),
    fetchText('retropulsion_branches.csv'),
    fetchText('output.txt'),
    fetchText('weather_table.csv'),
  ]);
  const printed = (re: RegExp, what: string) => outputTxt.match(re) ?? fail(`output.txt has no ${what}`);

  // ── Plan: the measured day reads the weather table ─────────────────────────────────────────
  const table = parseRows(weatherCsv, 'weather_table.csv').sort((a, b) => a.d_temp - b.d_temp);
  const plan = printed(/measured dT = (-?[0-9.]+) K -> drift ([0-9.]+) \+- ([0-9.]+) m, ignition margin ([0-9.]+) m \(k = ([0-9.]+)\)/, 'Act 0 plan line');
  const standardPlan = printed(/standard-day belief \(the uninformed world\) -> margin ([0-9.]+) m/, 'standard-day belief line');
  const measuredDT = Number(plan[1]);
  const k = Number(plan[5]);
  const lo = [...table].reverse().find((r) => r.d_temp <= measuredDT) ?? fail('measured day below the table');
  const hi = table.find((r) => r.d_temp >= measuredDT) ?? fail('measured day above the table');
  const w = hi.d_temp === lo.d_temp ? 0 : (measuredDT - lo.d_temp) / (hi.d_temp - lo.d_temp);
  const lerp = (key: string) => lo[key] + w * (hi[key] - lo[key]);
  const standardRow = table.find((r) => r.d_temp === 0) ?? fail('no standard-day row');
  const belief: Retro['belief'] = {
    measuredDT,
    k,
    rows: table.map((r) => ({ dTemp: r.d_temp, mean: r.drift_mean, sd: r.drift_sd })),
    bracket: [lo.d_temp, hi.d_temp],
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
  const toSample = (r: Row): Sample => ({
    leg: r.leg,
    t: r.t,
    altitudeKm: r.altitude / 1000,
    mach: r.mach,
    denied: r.gnss_denied === 1,
    throttle: r.throttle,
    propellant: r.propellant,
    descentRate: r.descent_rate,
    speed: r.speed,
    ne: r.ne_peak,
    heatFlux: r.heat_flux,
    gamma: r.speed > 0 ? Math.asin(Math.min(1, Math.max(-1, r.descent_rate / r.speed))) : Math.PI / 2,
  });
  const samples = parseRows(descentCsv, 'retropulsion_trace.csv').map(toSample);
  if (samples.some((s) => !Number.isFinite(s.speed) || !Number.isFinite(s.heatFlux) || !Number.isFinite(s.ne))) {
    fail('retropulsion_trace.csv has no speed, ne_peak or heat_flux column; re-run the example');
  }
  const uninformedSamples = parseRows(uninformedCsv, 'retropulsion_uninformed_trace.csv').map(toSample);

  const legEnd = (leg: number) => samples.findLastIndex((s) => s.leg === leg);
  const deniedFrom = samples.findIndex((s) => s.denied);
  const deniedTo = samples.findLastIndex((s) => s.denied);
  if (deniedFrom < 0) fail('no blackout in retropulsion_trace.csv');
  // The throttle flown lags the command by one step, so the corridor commits on the step before the
  // first lit row; output.txt prints that step, counted from 1 within leg 2.
  const THROTTLE_LAG_STEPS = 1;
  const firstLitIx = samples.findIndex((s, i) => s.leg === 2 && s.throttle > 0 && i > deniedTo);
  if (firstLitIx < 0) fail('no ignition in retropulsion_trace.csv');
  const commitIx = firstLitIx - THROTTLE_LAG_STEPS;
  const commitPrinted = printed(/ignition corridor committed at step (\d+): Mach ([0-9.]+)/, 'ignition commit line');
  const printedCommitIx = samples.findIndex((s) => s.leg === 2) + Number(commitPrinted[1]) - 1;
  if (commitIx !== printedCommitIx) {
    fail(`the first lit row is ${firstLitIx - printedCommitIx} step(s) after the commit output.txt prints at step ${commitPrinted[1]}; the throttle lag is ${THROTTLE_LAG_STEPS}`);
  }
  const window = printed(/blackout onset at ([0-9.]+) s against the table's [0-9.]+ s for this day \(error [0-9.]+ s\) and dwell ([0-9.]+) s/, 'gate (1) window');
  const blackout = { from: samples[deniedFrom].t, to: samples[deniedTo].t, dwell: Number(window[2]) };
  if (!near(blackout.from, Number(window[1]), 0.051)) fail(`the trace loses GPS at ${blackout.from} s; gate (1) says ${window[1]} s`);
  if (!near(blackout.to + 0.1 - blackout.from, blackout.dwell, 0.15)) fail(`the trace's blackout lasts ${blackout.to + 0.1 - blackout.from} s; gate (1) says ${blackout.dwell} s`);
  const ignition = { t: samples[commitIx].t, altitudeKm: samples[commitIx].altitudeKm, mach: samples[commitIx].mach };
  if (!near(ignition.mach, Number(commitPrinted[2]), 1e-6)) fail(`the trace commits at Mach ${ignition.mach}; output.txt says ${commitPrinted[2]}`);

  // The fork: the end of the coast-and-burn leg, as the act line prints it. `title` is the act's
  // printed title; a block in any other layout fails the load.
  const act = (title: string) =>
    printed(
      new RegExp(`--- Act: ${title.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}[^\\n]*\\n\\s+steps\\s+(\\d+) \\| altitude\\s+([0-9.]+) km \\| Mach\\s+([0-9.]+)[^\\n]*\\n\\s+mass\\s+[0-9.]+ kg \\| propellant\\s+([0-9.]+) kg \\| throttle ([0-9.]+) \\| q\\s+([0-9.]+) Pa`),
      `'--- Act: ${title}' block of the form 'steps N | altitude A km | Mach M ...' then 'mass X kg | propellant P kg | throttle T | q Q Pa'`
    );
  const forkAct = act('COAST + BURN');
  const f = samples[legEnd(2)];
  const fork = { t: f.t, altitudeKm: Number(forkAct[2]), mach: Number(forkAct[3]), throttle: Number(forkAct[5]), q: Number(forkAct[6]), propellant: Number(forkAct[4]) };
  if (!near(f.altitudeKm, fork.altitudeKm, 0.006) || !near(f.mach, fork.mach, 0.006) || !near(f.throttle, fork.throttle, 0.006)) {
    fail('the end of leg 2 in retropulsion_trace.csv does not match the COAST + BURN act in output.txt');
  }
  const burnAct = act('BURN —');
  const s3 = samples[legEnd(3)];
  const subsonic = { t: s3.t, altitudeKm: s3.altitudeKm, mach: s3.mach };
  if (!near(subsonic.altitudeKm, Number(burnAct[2]), 0.006) || !near(subsonic.mach, Number(burnAct[3]), 0.006)) {
    fail('the end of leg 3 in retropulsion_trace.csv does not match the BURN act in output.txt');
  }
  const totalSteps = [...outputTxt.matchAll(/^\s+steps\s+(\d+) \|/gm)].reduce((a, m) => a + Number(m[1]), 0);
  if (totalSteps !== samples.length) fail(`output.txt's legs add to ${totalSteps} steps; the trace has ${samples.length} rows`);

  // ── The landing, flown twice ─────────────────────────────────────────────────────────────────
  const beliefRow = (name: string) => {
    const m = printed(new RegExp(`^ {2}${name}\\s+([0-9.]+)\\s+([0-9.]+)\\s+([0-9.]+)\\s+([0-9.]+)`, 'm'), `${name} belief row`);
    return { margin: Number(m[1]), lightM: Number(m[2]), contactMs: Number(m[3]), reserveKg: Number(m[4]) };
  };
  const landing = (trace: Sample[], name: string): Landing => {
    const leg = trace.filter((s) => s.leg === 4);
    // Bang-bang: the leg coasts with the engine off, then lights once and burns to the ground.
    const coastIx = leg.findIndex((s) => s.throttle === 0);
    const lightIx = leg.findIndex((s, i) => i > coastIx && s.throttle > 0);
    if (coastIx < 0 || lightIx < 0) fail(`no coast-then-burn in the ${name} landing`);
    const last = leg[leg.length - 1];
    const row = beliefRow(name);
    const l = {
      leg,
      lightAltitudeM: leg[lightIx - 1].altitudeKm * 1000,
      lightT: leg[lightIx].t,
      touchdownT: last.t,
      contactMs: last.descentRate,
      propellantKg: last.propellant,
      printed: row,
    };
    if (!near(l.lightAltitudeM, row.lightM, 0.5) || !near(l.contactMs, row.contactMs, 0.005)) {
      fail(`the ${name} landing trace lights at ${l.lightAltitudeM} m and lands at ${l.contactMs} m/s; output.txt says ${row.lightM} m and ${row.contactMs} m/s`);
    }
    return l;
  };
  const landings = { informed: landing(samples, 'informed'), uninformed: landing(uninformedSamples, 'uninformed') };
  const touchdown = printed(/touchdown: altitude floor reached at ([0-9.]+) km at ([0-9.]+) m\/s .*?with ([0-9.]+) kg propellant remaining/, 'gate (6) touchdown');
  const last = samples[samples.length - 1];
  if (!near(last.altitudeKm, Number(touchdown[1]), 0.0006) || !near(last.propellant, Number(touchdown[3]), 0.06)) {
    fail('the last step of retropulsion_trace.csv does not match gate (6)');
  }

  // ── The mid-burn fork ────────────────────────────────────────────────────────────────────────
  const roster = parseRows(rosterCsv, 'retropulsion_branches.csv');
  const branchRows = parseRows(branchCsv, 'retropulsion_branch_trace.csv');
  const names = new Map<number, string>();
  for (const m of outputTxt.matchAll(/^ {2}([a-z]+)\s+([0-9.]+)\s+([0-9.]+)\s+(—|-?[0-9.]+)\s+([0-9.]+)/gm)) {
    names.set(Number(m[2]), m[1]);
  }
  const branches: Branch[] = roster.map((r) => {
    const rows = branchRows.filter((b) => b.commanded_throttle === r.commanded_throttle);
    const name = names.get(r.commanded_throttle) ?? fail(`output.txt names no branch at throttle ${r.commanded_throttle}`);
    if (rows.length < 2) fail(`no branch trace for ${name}`);
    const lastRow = rows[rows.length - 1];
    if (!near(lastRow.axial_accel, r.net_deceleration, 1e-6) || !near(lastRow.throttle, r.realized_throttle, 1e-9)) {
      fail(`the ${name} branch trace does not end on its retropulsion_branches.csv row`);
    }
    // The fork's accumulated value: the branch's final value less what the table says it shed.
    const dv0 = lastRow.dv_actual - r.dv_actual;
    return {
      name,
      commanded: r.commanded_throttle,
      flown: r.realized_throttle,
      preserved: Number.isNaN(r.preserved_fraction) ? null : r.preserved_fraction,
      deceleration: r.net_deceleration,
      dvActual: r.dv_actual,
      dvFrozen: r.dv_frozen,
      t: rows.map((b) => b.t - fork.t),
      dv: rows.map((b) => b.dv_actual - dv0),
    };
  });
  // Each branch's printed roster row: commanded, flown, axial deceleration.
  for (const b of branches) {
    const m = printed(new RegExp(`^ {2}${b.name}\\s+([0-9.]+)\\s+([0-9.]+)\\s+(?:—|-?[0-9.]+)\\s+([0-9.]+)`, 'm'), `${b.name} roster row`);
    if (!near(b.flown, Number(m[2]), 5e-5) || !near(b.deceleration, Number(m[3]), 5e-5)) fail(`the ${b.name} branch does not match its printed roster row`);
  }

  const clock = printed(/wall-clock budget: ([0-9.]+) s elapsed/, 'wall-clock gate');
  const gates = outputTxt
    .split('\n')
    .map((l) => l.match(/\[(PASS|FAIL)\] \[\w+\] (\([0-9a-z]+\) [^:]+):/))
    .filter((m): m is RegExpMatchArray => m !== null)
    .map((m) => ({ passed: m[1] === 'PASS', label: m[2] }));
  const gateLines = outputTxt.match(/\[(PASS|FAIL)\]/g)?.length ?? 0;
  if (gates.length === 0 || gates.length !== gateLines) {
    fail(`output.txt has ${gateLines} gate lines; ${gates.length} read as '[PASS|FAIL] [kind] (id) label:'`);
  }

  return {
    belief,
    samples,
    events: {
      entry: { mach: samples[0].mach, altitudeKm: samples[0].altitudeKm },
      blackout,
      ignition,
      coastS: ignition.t - (blackout.to + 0.1),
      fork,
      subsonic,
      end: { t: last.t },
    },
    branches,
    branchSeconds: Math.max(...branches.map((b) => b.t[b.t.length - 1])),
    landings,
    groundM: last.altitudeKm * 1000,
    propellantLeftKg: Number(touchdown[3]),
    totalSteps,
    runSeconds: Number(clock[1]),
    gates,
  };
}

/** The recorded step nearest flight time `t`. */
export function sampleAt(trace: Sample[], t: number): Sample {
  let lo = 0;
  let hi = trace.length - 1;
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1;
    if (trace[mid].t <= t) lo = mid;
    else hi = mid;
  }
  return Math.abs(trace[lo].t - t) <= Math.abs(trace[hi].t - t) ? trace[lo] : trace[hi];
}
