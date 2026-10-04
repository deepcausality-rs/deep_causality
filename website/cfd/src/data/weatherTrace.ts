/**
 * The weather campaign, step by step and draw by draw, for the walk 2 animation.
 *
 * Read at build time from the files the weather example commits beside its `output.txt`:
 * `weather_trace.csv` (draw 0 of every world, one row per coupled step), `weather_draws.csv`
 * (every draw of every world: window and navigation metrics) and `weather_table.csv` (the
 * dispersion table walk 3 reads). World names come from the table rows `output.txt` prints.
 * Nothing is typed by hand, and the build fails if the per-draw file does not reproduce the
 * table's statistics or the trace does not reproduce draw 0.
 */
import traceCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/weather/weather_trace.csv?raw';
import drawsCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/weather/weather_draws.csv?raw';
import tableCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/weather/weather_table.csv?raw';
import outputTxt from '../../../../examples/avionics_examples/cfd/plasma_blackout/weather/output.txt?raw';
import { parseGates, parseRows } from './traceCsv';

const traceRows = parseRows(traceCsv, 'weather_trace.csv');
const drawRows = parseRows(drawsCsv, 'weather_draws.csv');
const tableRows = parseRows(tableCsv, 'weather_table.csv');

// "  polar_winter    -40   1.20 ..." : a world name followed by its temperature offset.
const names = new Map<number, string>();
for (const m of outputTxt.matchAll(/^ {2}([a-z_]+)\s+(-?\d+)\s+\d\.\d\d\s/gm)) {
  names.set(Number(m[2]), m[1]);
}

const close = (a: number, b: number) => Math.abs(a - b) <= 1e-6 * Math.max(1, Math.abs(b));
const mean = (xs: number[]) => xs.reduce((a, b) => a + b, 0) / xs.length;
const sd = (xs: number[]) => {
  const m = mean(xs);
  return Math.sqrt(xs.reduce((a, x) => a + (x - m) ** 2, 0) / (xs.length - 1));
};
/** Throws `claim: stat got vs want` unless `got` is close to `want`. */
const reproduce = (claim: string, stat: string, got: number, want: number) => {
  if (!close(got, want)) throw new Error(`${claim}: ${stat} ${got} vs ${want}`);
};

/** One world: its table row, its draws, and draw 0 step by step. */
export interface World {
  name: string;
  dTemp: number;
  rhoScale: number;
  biasDeparture: number;
  onset: number;
  exit: number;
  dwell: number;
  driftMean: number;
  driftSd: number;
  terminalMax: number;
  /** Each draw's largest navigation error while GPS is lost, m, in draw order. */
  drifts: number[];
  /** Draw 0, one sample per coupled step. */
  t: number[];
  navErr: number[];
  denied: boolean[];
}

export const worlds: World[] = tableRows.map((r) => {
  const name = names.get(r.d_temp);
  if (!name) throw new Error(`output.txt: no world printed for d_temp ${r.d_temp}`);
  const draws = drawRows.filter((d) => d.d_temp === r.d_temp).sort((a, b) => a.draw - b.draw);
  const drifts = draws.map((d) => d.drift_max);
  const terminals = draws.map((d) => d.terminal);
  const row = `weather_draws.csv does not reproduce the ${name} row of weather_table.csv`;
  if (draws.length < 2) throw new Error(`${row}: ${draws.length} draw(s), at least 2 needed`);
  const ids = draws.map((d) => d.draw);
  if (ids.some((id, i) => id !== i)) {
    throw new Error(`${row}: draw ids [${ids}] vs [${ids.map((_, i) => i)}]`);
  }
  reproduce(row, 'drift_mean', mean(drifts), r.drift_mean);
  reproduce(row, 'drift_sd', sd(drifts), r.drift_sd);
  reproduce(row, 'draw 0 onset', draws[0].onset, r.onset);
  reproduce(row, 'draw 0 exit', draws[0].exit, r.exit);
  reproduce(row, 'draw 0 dwell', draws[0].dwell, r.dwell);
  reproduce(row, 'terminal_mean', mean(terminals), r.terminal_mean);
  reproduce(row, 'terminal_max', Math.max(...terminals), r.terminal_max);
  const steps = traceRows.filter((s) => s.d_temp === r.d_temp);
  const denied = steps.filter((s) => s.gnss_denied === 1);
  const traceDrift = Math.max(...denied.map((s) => s.nav_err));
  const draw0 = `weather_trace.csv does not reproduce draw 0 of ${name}`;
  // The drift check comes first: with no denied rows it fails, on -Infinity, before `denied[0]`
  // is read.
  reproduce(draw0, 'drift_max', traceDrift, drifts[0]);
  reproduce(draw0, 'onset', denied[0].t, r.onset);
  reproduce(draw0, 'exit', denied[denied.length - 1].t, r.exit);
  reproduce(draw0, 'dwell', denied.length * (steps[1].t - steps[0].t), r.dwell);
  return {
    name,
    dTemp: r.d_temp,
    rhoScale: r.rho_scale,
    biasDeparture: r.bias_departure,
    onset: r.onset,
    exit: r.exit,
    dwell: r.dwell,
    driftMean: r.drift_mean,
    driftSd: r.drift_sd,
    terminalMax: r.terminal_max,
    drifts,
    t: steps.map((s) => s.t),
    navErr: steps.map((s) => s.nav_err),
    denied: steps.map((s) => s.gnss_denied === 1),
  };
});

/** The gate lines of the committed run, as printed: label and pass flag. */
export const gates = parseGates(outputTxt);
