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

type Row = Record<string, number>;

/** Parse the two-row-header CSV `write_rows` emits: names, then `#units`, then data. */
function parseRows(csv: string): Row[] {
  const lines = csv.trim().split('\n');
  const names = lines[0].split(',');
  return lines
    .slice(1)
    .filter((l) => !l.startsWith('#units'))
    .map((l) => {
      const cells = l.split(',').map(Number);
      return Object.fromEntries(names.map((n, i) => [n, cells[i]]));
    });
}

const traceRows = parseRows(traceCsv);
const drawRows = parseRows(drawsCsv);
const tableRows = parseRows(tableCsv);

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
  if (draws.length < 2 || !close(mean(drifts), r.drift_mean) || !close(sd(drifts), r.drift_sd)) {
    throw new Error(`weather_draws.csv does not reproduce the ${name} row of weather_table.csv`);
  }
  const steps = traceRows.filter((s) => s.d_temp === r.d_temp);
  const denied = steps.filter((s) => s.gnss_denied === 1);
  const traceDrift = Math.max(...denied.map((s) => s.nav_err));
  if (!close(traceDrift, drifts[0])) {
    throw new Error(`weather_trace.csv does not reproduce draw 0 of ${name}`);
  }
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
export const gates = outputTxt
  .split('\n')
  .map((l) => l.match(/\[(PASS|FAIL)\] \[\w+\] (\([0-9a-z]+\) [^:]+):/))
  .filter((m): m is RegExpMatchArray => m !== null)
  .map((m) => ({ passed: m[1] === 'PASS', label: m[2] }));
