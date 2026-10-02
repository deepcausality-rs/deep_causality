/**
 * The corridor run, step by step, for the walk 1 animation.
 *
 * Read at build time from the files the corridor example commits beside its `output.txt`:
 * `corridor_trace.csv` (one row per coupled step of the flown descent, 0.1 s of flight each) and
 * `corridor_branch_trace.csv` (every branch of both rounds, from the pause to the end of its
 * continuation). Nothing here is typed by hand. Every event the animation names is located in the
 * trace, and the branch misses recomputed from the positions are checked against the ones
 * `results.ts` copies from `output.txt`, so the build fails if the two ever disagree.
 */
import descentCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/corridor/corridor_trace.csv?raw';
import branchCsv from '../../../../examples/avionics_examples/cfd/plasma_blackout/corridor/corridor_branch_trace.csv?raw';
import outputTxt from '../../../../examples/avionics_examples/cfd/plasma_blackout/corridor/output.txt?raw';
import { corridor } from './results';

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

const descent = parseRows(descentCsv);
const branchRows = parseRows(branchCsv);

/** One sample of the flown descent. */
export interface DescentSample {
  t: number;
  altitudeKm: number;
  plasmaFreq: number;
  denied: boolean;
  navErr: number;
}

export const samples: DescentSample[] = descent.map((r) => ({
  t: r.t,
  altitudeKm: r.altitude / 1000,
  plasmaFreq: r.plasma_freq,
  denied: r.gnss_denied === 1,
  navErr: r.nav_err,
}));

const pauseIx = samples.findIndex((s) => s.denied);
const exitIx = samples.findIndex((s, i) => i > pauseIx && !s.denied);
const legTwoEnd = descent.findLastIndex((r) => r.leg === 2);
const darkPeakIx = samples.reduce(
  (best, s, i) => (s.denied && s.navErr > samples[best].navErr ? i : best),
  pauseIx
);

if (pauseIx < 0 || exitIx < 0 || legTwoEnd < 0) {
  throw new Error('corridor_trace.csv: no blackout window found');
}

const band = outputTxt.match(/GPS L1 \(w = ([0-9.e+-]+) rad\/s\)/);
if (!band) throw new Error('output.txt: no GPS L1 band line');

/** The events the animation names, each read off the trace. */
export const events = {
  /** GPS L1 band the classifier compares the plasma frequency against, rad/s (`output.txt`). */
  bandRadS: Number(band[1]),
  pause: { t: samples[pauseIx].t, altitudeKm: samples[pauseIx].altitudeKm },
  /** End of the peak-passage leg: the RAM-C II comparison station. */
  peakPassage: {
    t: samples[legTwoEnd].t,
    altitudeKm: samples[legTwoEnd].altitudeKm,
    electronDensity: descent[legTwoEnd].ne_peak,
  },
  /** Largest navigation error while GPS is lost: the last step before a fix folds. */
  darkNavPeak: { t: samples[darkPeakIx].t, navErr: samples[darkPeakIx].navErr },
  exit: {
    t: samples[exitIx].t,
    altitudeKm: samples[exitIx].altitudeKm,
    navErr: samples[exitIx].navErr,
  },
  end: { t: samples[samples.length - 1].t, navErr: samples[samples.length - 1].navErr },
} as const;

if (Math.abs(events.pause.altitudeKm - corridor.onsetKm) > 0.05) {
  throw new Error(
    `corridor_trace.csv pauses at ${events.pause.altitudeKm} km; results.ts says ${corridor.onsetKm} km`
  );
}

/** One branch of the bank-angle what-if, after the pause. */
export interface Branch {
  round: 1 | 2;
  bank: number;
  /** Seconds since the pause. */
  t: number[];
  /** Sideways distance from the no-bank branch at the same step, metres (toward the aim). */
  crossRange: number[];
  /** Distance from the no-bank end point within the descent plane, at the end, metres. */
  inPlaneEnd: number;
  /** Miss distance from the aim point, recomputed from the end positions, metres. */
  miss: number;
}

const byBranch = new Map<string, Row[]>();
for (const r of branchRows) {
  const key = `${r.round}:${r.bank_cmd}`;
  byBranch.set(key, [...(byBranch.get(key) ?? []), r]);
}
const ballistic = byBranch.get('1:0');
if (!ballistic) throw new Error('corridor_branch_trace.csv: no zero-bank branch');
const ballisticEnd = ballistic[ballistic.length - 1];
const aim = { x: ballisticEnd.x, y: ballisticEnd.y, z: ballisticEnd.z - corridor.aimOffsetM };
const t0 = ballistic[0].t - (ballistic[1].t - ballistic[0].t);

export const branches: Branch[] = [...byBranch.values()].map((rows) => {
  const end = rows[rows.length - 1];
  const branch: Branch = {
    round: rows[0].round === 1 ? 1 : 2,
    bank: rows[0].bank_cmd,
    t: rows.map((r) => r.t - t0),
    crossRange: rows.map((r, i) => -(r.z - ballistic[i].z)),
    inPlaneEnd: Math.hypot(end.x - ballisticEnd.x, end.y - ballisticEnd.y),
    miss: Math.hypot(end.x - aim.x, end.y - aim.y, end.z - aim.z),
  };
  const copied = [...corridor.coarse, ...corridor.fine].find((b) => b.bank === branch.bank);
  if (!copied || Math.abs(copied.miss - branch.miss) > 1e-3) {
    throw new Error(
      `branch ${branch.bank} deg misses ${branch.miss} m in the trace; results.ts says ${copied?.miss}`
    );
  }
  return branch;
});

/** The gate lines of the committed run, as printed: label and pass flag. */
export const gates = outputTxt
  .split('\n')
  .map((l) => l.match(/\[(PASS|FAIL)\] \[\w+\] (\([0-9a-z]+\) [^:]+):/))
  .filter((m): m is RegExpMatchArray => m !== null)
  .map((m) => ({ passed: m[1] === 'PASS', label: m[2] }));
