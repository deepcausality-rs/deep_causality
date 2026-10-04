/**
 * The weather campaign, read from the files the example commits (synced into public/traces/ by
 * `pnpm sync`), with every on-screen number checked against them: `weather_table.csv` (one row per
 * world), `weather_draws.csv` (every receiver-noise draw), `weather_trace.csv` (each world's
 * reference draw, step by step), the reference draws' audit logs, and `output.txt`.
 *
 * Nothing here is typed by hand. A mismatch throws, which fails `calculateMetadata` and so the
 * render.
 */
import { staticFile } from 'remotion';

type Row = Record<string, number>;

/** Parse the two-row-header CSV `write_rows` emits: names, then `#units`, then data. */
const parseRows = (csv: string): Row[] => {
  const lines = csv.trim().split('\n');
  const names = lines[0].split(',');
  return lines
    .slice(1)
    .filter((l) => !l.startsWith('#units'))
    .map((l) => {
      const cells = l.split(',').map(Number);
      return Object.fromEntries(names.map((n, i) => [n, cells[i]]));
    });
};

const fail = (msg: string): never => {
  throw new Error(`weather data: ${msg}`);
};
const near = (a: number, b: number, tol: number) => Math.abs(a - b) <= tol;

export interface Sample {
  t: number;
  altitudeKm: number;
  mach: number;
  speed: number;
  ne: number;
  heatFlux: number;
  denied: boolean;
  navErr: number;
  /** Flight-path angle below the horizontal, rad, from the altitude lost per step and the speed. */
  gamma: number;
}

export interface Draw {
  draw: number;
  /** Largest navigation error while GPS is lost, m. */
  drift: number;
  /** Navigation error at the end of the run, m. */
  terminal: number;
}

export interface World {
  /** As the run names it: `polar_winter`. */
  name: string;
  /** As the video says it: `polar winter`. */
  label: string;
  dTemp: number;
  rho: number;
  /** The accelerometer bias the world flies, relative to the calibrated bias. */
  departure: number;
  onset: number;
  exit: number;
  dwell: number;
  driftMean: number;
  driftSd: number;
  terminalMax: number;
  draws: Draw[];
  /** The reference draw, one sample per coupled step. */
  trace: Sample[];
  /** The alternation line of the reference draw's audit log; none for the baseline. */
  marker: string | null;
  /** The baseline's mean drift scaled by this world's departure and squared dwell, m. */
  predicted: number;
}

export interface Weather {
  /**
   * In table order; the first is the baseline. The props are serialized on their way to the
   * compositions, so scenes compare worlds by name or index, never by reference.
   */
  worlds: World[];
  /** Bias departure per kelvin from the calibration point. */
  imuPerK: number;
  drawsPerWorld: number;
  steps: number;
  flightS: number;
  spreads: { onset: number; dwell: number };
  /** Gate (4): polar winter's mean drift over the standard day's. */
  coldFactor: number;
  /** Gate (4b): the polar-standard separation, the combined sigma, and their ratio. */
  resolved: { separation: number; sigma: number; ratio: number };
  /** The worst relative error of the prediction over the worlds. */
  predictionError: number;
  /** Gate (5): the ceiling every draw's terminal navigation error stays under, m. */
  reacquireM: number;
  runSeconds: number;
  gates: { passed: boolean; label: string }[];
}

const fetchText = async (file: string) => {
  const res = await fetch(staticFile(`traces/${file}`));
  if (!res.ok) fail(`cannot read traces/${file}; run pnpm sync`);
  return res.text();
};

export async function loadWeather(): Promise<Weather> {
  const [tableCsv, drawsCsv, traceCsv, outputTxt] = await Promise.all([
    fetchText('weather_table.csv'),
    fetchText('weather_draws.csv'),
    fetchText('weather_trace.csv'),
    fetchText('output.txt'),
  ]);
  const printed = (re: RegExp, what: string) => outputTxt.match(re) ?? fail(`output.txt has no ${what}`);

  const imuPerK = Number(printed(/bias departure 1 \+ ([0-9.]+)\/K away from the calibration point/, 'IMU thermal model line')[1]);
  const mc = printed(/Monte Carlo: (\d+) deterministic receiver-noise draws per condition \((\d+) descents total\)/, 'Monte Carlo line');
  const drawsPerWorld = Number(mc[1]);
  const baselineLine = printed(/(\d+) coupled steps per world, (\d+) s of flight each/, 'baseline line');
  const steps = Number(baselineLine[1]);
  const flightS = Number(baselineLine[2]);

  // ── The table, row by row against the printed rows ───────────────────────────────────────────
  const table = parseRows(tableCsv);
  const drawRows = parseRows(drawsCsv);
  const traceRows = parseRows(traceCsv);
  if (traceRows.some((r) => !Number.isFinite(r.speed) || !Number.isFinite(r.heat_flux) || !Number.isFinite(r.ne_peak))) {
    fail('weather_trace.csv has no mach, speed, ne_peak or heat_flux column; re-run the example');
  }
  const rowRe = /^ {2}([a-z_]+)\s+(-?\d+)\s+(\d\.\d\d)\s+(\d\.\d\d)\s+([0-9.]+)\s+([0-9.]+)\s+([0-9.]+)\s+\S+\s+\S+\s+([0-9.]+) \+- ([0-9.]+)\s+[0-9.]+ \(max ([0-9.]+)\)$/gm;
  const printedRows = [...outputTxt.matchAll(rowRe)];
  if (printedRows.length !== table.length) fail(`output.txt prints ${printedRows.length} world rows; weather_table.csv has ${table.length}`);
  if (Number(mc[2]) !== drawsPerWorld * table.length) fail('the Monte Carlo line does not multiply out');

  const worlds: World[] = await Promise.all(
    printedRows.map(async (m, i) => {
      const name = m[1];
      const dTemp = Number(m[2]);
      const row = table.find((r) => r.d_temp === dTemp) ?? fail(`no table row at ${dTemp} K for ${name}`);
      const p = (k: number) => Number(m[k]);
      const shown: [number, number, number, string][] = [
        [row.rho_scale, p(3), 0.005, 'density'],
        [row.bias_departure, p(4), 0.005, 'IMU departure'],
        [row.onset, p(5), 0.05, 'onset'],
        [row.exit, p(6), 0.05, 'exit'],
        [row.dwell, p(7), 0.05, 'dwell'],
        [row.drift_mean, p(8), 0.005, 'drift mean'],
        [row.drift_sd, p(9), 0.005, 'drift sd'],
        [row.terminal_max, p(10), 0.0005, 'worst terminal error'],
      ];
      for (const [v, s, tol, what] of shown) if (!near(v, s, tol)) fail(`${name}: the table's ${what} ${v} does not print as ${s}`);
      if (!near(row.bias_departure, 1 + imuPerK * Math.abs(dTemp), 1e-12)) fail(`${name}: departure is not 1 + ${imuPerK}/K × |dT|`);

      // The draws reproduce the row's mean, sample standard deviation and worst terminal error.
      const draws: Draw[] = drawRows
        .filter((d) => d.d_temp === dTemp)
        .map((d) => ({ draw: d.draw, drift: d.drift_max, terminal: d.terminal }));
      if (draws.length !== drawsPerWorld) fail(`${name} has ${draws.length} draws, not ${drawsPerWorld}`);
      const mean = draws.reduce((a, d) => a + d.drift, 0) / draws.length;
      const sd = Math.sqrt(draws.reduce((a, d) => a + (d.drift - mean) ** 2, 0) / (draws.length - 1));
      if (!near(mean, row.drift_mean, 1e-9) || !near(sd, row.drift_sd, 1e-9)) fail(`${name}: the draws do not reproduce the drift mean and sd`);
      if (!near(Math.max(...draws.map((d) => d.terminal)), row.terminal_max, 1e-12)) fail(`${name}: the draws do not reproduce the worst terminal error`);

      // The reference draw's trace: its blackout is the row's window, its worst dark error draw 0's drift.
      const rows = traceRows.filter((r) => r.d_temp === dTemp);
      if (rows.length !== steps) fail(`${name} traces ${rows.length} steps, not ${steps}`);
      const trace: Sample[] = rows.map((r, k) => {
        // The altitude lost over the step into this one; the first step uses the step after it.
        const [a, b] = k === 0 ? [rows[0], rows[1]] : [rows[k - 1], r];
        const rate = -(b.altitude - a.altitude) / (b.t - a.t);
        return {
          t: r.t,
          altitudeKm: r.altitude / 1000,
          mach: r.mach,
          speed: r.speed,
          ne: r.ne_peak,
          heatFlux: r.heat_flux,
          denied: r.gnss_denied === 1,
          navErr: r.nav_err,
          gamma: Math.asin(Math.min(1, Math.max(-1, rate / r.speed))),
        };
      });
      const dark = trace.filter((s) => s.denied);
      if (dark.length === 0) fail(`${name} never loses GPS`);
      if (!near(dark[0].t, row.onset, 1e-9) || !near(dark[dark.length - 1].t, row.exit, 1e-9)) fail(`${name}: the trace's blackout is not the row's window`);
      if (!near(Math.max(...dark.map((s) => s.navErr)), draws[0].drift, 1e-9)) fail(`${name}: the trace's worst dark error is not draw 0's drift`);
      if (!near(trace[trace.length - 1].t, flightS, 1e-9)) fail(`${name}: the trace ends at ${trace[trace.length - 1].t} s, not ${flightS} s`);

      // Gate (1): every alternated world's audit log names the world that replaced the baseline.
      const log = await fetchText(`audit/weather.audit.sweep-1.case-${String(i).padStart(2, '0')}-${name}.draw-0.log`);
      const marker = log.split('\n').find((l) => l.startsWith('!!ContextAlternation!!')) ?? null;

      return {
        name,
        label: name.replace(/_/g, ' '),
        dTemp,
        rho: row.rho_scale,
        departure: row.bias_departure,
        onset: row.onset,
        exit: row.exit,
        dwell: row.dwell,
        driftMean: row.drift_mean,
        driftSd: row.drift_sd,
        terminalMax: row.terminal_max,
        draws,
        trace,
        marker,
        predicted: 0,
      };
    })
  );
  const baseline = worlds[0];
  if (baseline.dTemp !== 0 || baseline.rho !== 1 || baseline.marker !== null) fail('the first world is not the unmarked standard-day baseline');
  for (const w of worlds.slice(1)) {
    if (w.marker !== `!!ContextAlternation!!: world '${baseline.name}' replaced with '${w.name}'`) fail(`${w.name}'s audit log does not name its alternation`);
  }

  // ── The prediction: the baseline's drift scaled by the departure and the squared dwell ──────
  for (const w of worlds) w.predicted = baseline.driftMean * w.departure * (w.dwell / baseline.dwell) ** 2;
  const predictionError = Math.max(...worlds.map((w) => Math.abs(w.predicted / w.driftMean - 1)));

  // ── The gates, rebuilt from the table ────────────────────────────────────────────────────────
  const onsets = worlds.map((w) => w.onset);
  const dwells = worlds.map((w) => w.dwell);
  const spreads = { onset: Math.max(...onsets) - Math.min(...onsets), dwell: Math.max(...dwells) - Math.min(...dwells) };
  const g3 = printed(/onset spread ([0-9.]+) s across the table .*?; dwell spread ([0-9.]+) s/, 'gate (3)');
  if (spreads.onset.toFixed(1) !== g3[1] || spreads.dwell.toFixed(1) !== g3[2]) fail('the table does not reproduce gate (3) spreads');

  const polar = worlds.find((w) => w.name === 'polar_winter') ?? fail('no polar_winter world');
  const coldFactor = polar.driftMean / baseline.driftMean;
  const g4 = printed(/polar-winter mean blackout drift ([0-9.]+) m vs standard-day ([0-9.]+) m \(([0-9.]+)x/, 'gate (4)');
  if (polar.driftMean.toFixed(2) !== g4[1] || baseline.driftMean.toFixed(2) !== g4[2] || coldFactor.toFixed(2) !== g4[3]) fail('the table does not reproduce gate (4)');

  const separation = polar.driftMean - baseline.driftMean;
  const sigma = Math.hypot(polar.driftSd, baseline.driftSd);
  const g4b = printed(/polar-standard separation ([0-9.]+) m vs combined sigma ([0-9.]+) m \(([0-9.]+) sigma/, 'gate (4b)');
  if (separation.toFixed(2) !== g4b[1] || sigma.toFixed(2) !== g4b[2] || (separation / sigma).toFixed(1) !== g4b[3]) fail('the table does not reproduce gate (4b)');

  const g5 = printed(/worst-draw terminal navigation error under ([0-9.]+) m across all (\d+) descents/, 'gate (5)');
  const reacquireM = Number(g5[1]);
  if (Number(g5[2]) !== worlds.length * drawsPerWorld) fail('gate (5) counts a different number of descents');
  if (worlds.some((x) => x.draws.some((d) => !(d.terminal < reacquireM)))) fail(`a draw ends above gate (5)'s ${reacquireM} m`);

  const clock = printed(/wall-clock budget: ([0-9.]+) s for the whole six-world table/, 'wall-clock gate');
  const gates = outputTxt
    .split('\n')
    .map((l) => l.match(/\[(PASS|FAIL)\] \[\w+\] (\([0-9a-z]+\) [^:]+):/))
    .filter((m): m is RegExpMatchArray => m !== null)
    .map((m) => ({ passed: m[1] === 'PASS', label: m[2] }));

  return {
    worlds,
    imuPerK,
    drawsPerWorld,
    steps,
    flightS,
    spreads,
    coldFactor,
    resolved: { separation, sigma, ratio: separation / sigma },
    predictionError,
    reacquireM,
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
