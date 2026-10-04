/**
 * The corridor run, read from the files the example commits (synced into public/traces/ by
 * `pnpm sync`), with every on-screen number checked against them.
 *
 * Nothing here is typed by hand except the aim offset, which is the example's own constant
 * (`AIM_CROSS_RANGE_M`) and is checked against the zero-bank miss it produces. A mismatch throws,
 * which fails `calculateMetadata` and so the render.
 */
import { staticFile } from 'remotion';

type Row = Record<string, number>;

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
  throw new Error(`corridor data: ${msg}`);
};
const near = (a: number, b: number, tol: number) => Math.abs(a - b) <= tol;

/** The example's `AIM_CROSS_RANGE_M`: the aim sits this far to the side of the no-bank end. */
const AIM_OFFSET_M = 20;

export interface Branch {
  round: 1 | 2;
  /** Commanded bank angle, degrees. */
  bank: number;
  /** Bank angle the lift stage flew after the safety gate, degrees. */
  flown: number;
  /** Seconds since the pause. */
  t: number[];
  /** Sideways distance from the no-bank branch, toward the aim, metres. */
  crossRange: number[];
  /** End position relative to the no-bank end: sideways and lower within the descent plane, metres. */
  end: { cross: number; drop: number };
  /** Distance from the aim at the end, metres. */
  miss: number;
}

export interface Sample {
  t: number;
  altitudeKm: number;
  mach: number;
  ne: number;
  plasmaFreq: number;
  denied: boolean;
  navErr: number;
  heatFlux: number;
  /** Bank angle flown, rad. */
  bank: number;
}

export interface Corridor {
  pause: { t: number; altitudeKm: number; mach: number; electronDensity: number; navErr: number };
  entry: { altitudeKm: number; mach: number };
  /** Flight time at the last recorded step, s. */
  endT: number;
  /** The flown descent, one sample per 0.1 s step: the committed branch after the pause. */
  descent: Sample[];
  /** The GPS L1 band the classifier compares the plasma frequency against, rad/s. */
  bandRadS: number;
  /** The RAM-C II flight reference for the peak electron density, m⁻³. */
  ramcReference: number;
  /** The end of the peak-passage leg: the RAM-C II comparison station. */
  peakPassage: { t: number; altitudeKm: number; electronDensity: number };
  /** Largest navigation error while GPS is lost, and where. */
  darkNavPeak: { t: number; navErr: number };
  /** GPS returns: time, altitude, and the error after the first fix folds. */
  exit: { t: number; altitudeKm: number; navErr: number };
  /** The navigation error at the last recorded step. */
  endNavErr: number;
  /** The whole example's wall-clock, s, from gate (5b). */
  runSeconds: number;
  branchSeconds: number;
  aimOffsetM: number;
  branches: Branch[];
  committed: { bank: number; miss: number };
  /**
   * The navigation when the branches are scored, at the committed branch's last step: the error
   * against truth, and the filter's own 1σ position uncertainty (√ of the position-covariance
   * trace), m.
   */
  scoring: { t: number; navErr: number; navSigma: number };
  noBankMiss: number;
  improvement: number;
  /** One-parameter arc through the end points: cross = R sin φ, drop = R (1 − cos φ). */
  arc: { radius: number; worstResidual: number };
  gates: { passed: boolean; label: string }[];
}

const fetchText = async (file: string) => {
  const res = await fetch(staticFile(`traces/${file}`));
  if (!res.ok) fail(`cannot read traces/${file}; run pnpm sync`);
  return res.text();
};

export async function loadCorridor(): Promise<Corridor> {
  const [descentCsv, branchCsv, outputTxt] = await Promise.all([
    fetchText('corridor_trace.csv'),
    fetchText('corridor_branch_trace.csv'),
    fetchText('output.txt'),
  ]);
  const descent = parseRows(descentCsv);
  const branchRows = parseRows(branchCsv);
  const printed = (re: RegExp, what: string) => outputTxt.match(re) ?? fail(`output.txt has no ${what}`);

  // ── The pause ────────────────────────────────────────────────────────────────────────────
  const pauseIx = descent.findIndex((r) => r.gnss_denied === 1);
  if (pauseIx < 0) fail('no blackout in corridor_trace.csv');
  const p = descent[pauseIx];
  const pause = {
    t: p.t,
    altitudeKm: p.altitude / 1000,
    mach: p.mach,
    electronDensity: p.ne_peak,
    navErr: p.nav_err,
  };
  const leg1 = printed(/marched (\d+) steps to ([0-9.]+) km, Mach ([0-9.]+)/, 'leg 1 line');
  const leg1Ne = printed(/n_e peak = ([0-9.e+-]+) m\^-3/, 'leg 1 n_e');
  const leg1Nav = printed(/nav error vs truth = ([0-9.]+) m/, 'leg 1 nav error');
  if (
    Number(leg1[1]) !== pauseIx + 1 ||
    !near(pause.altitudeKm, Number(leg1[2]), 0.05) ||
    !near(pause.mach, Number(leg1[3]), 0.05) ||
    !near(pause.electronDensity, Number(leg1Ne[1]), 1e-3 * Number(leg1Ne[1])) ||
    !near(pause.navErr, Number(leg1Nav[1]), 5e-5)
  ) {
    fail('the pause in corridor_trace.csv does not match leg 1 of output.txt');
  }

  // ── The branches ─────────────────────────────────────────────────────────────────────────
  const byBranch = new Map<string, Row[]>();
  for (const r of branchRows) {
    const k = `${r.round}:${r.bank_cmd}`;
    byBranch.set(k, [...(byBranch.get(k) ?? []), r]);
  }
  const ballistic = byBranch.get('1:0') ?? fail('no zero-bank branch');
  const b0 = ballistic[ballistic.length - 1];
  const aim = { x: b0.x, y: b0.y, z: b0.z - AIM_OFFSET_M };
  const branches: Branch[] = [...byBranch.values()].map((rows) => {
    const e = rows[rows.length - 1];
    return {
      round: rows[0].round === 1 ? 1 : 2,
      bank: rows[0].bank_cmd,
      flown: (e.bank * 180) / Math.PI,
      t: rows.map((r) => r.t - pause.t),
      crossRange: rows.map((r, i) => -(r.z - ballistic[i].z)),
      end: { cross: -(e.z - b0.z), drop: Math.hypot(e.x - b0.x, e.y - b0.y) },
      miss: Math.hypot(e.x - aim.x, e.y - aim.y, e.z - aim.z),
    };
  });

  // Every miss recomputed from the positions must equal the one the run printed.
  for (const m of outputTxt.matchAll(/^\s+([0-9.]+)deg\s+[0-9.e]+\s+[0-9.e]+\s+[0-9.]+ s\s+([0-9.]+) m/gm)) {
    const bank = Number(m[1]);
    const b = branches.find((x) => x.bank === bank) ?? fail(`no branch at ${bank}°`);
    if (!near(b.miss, Number(m[2]), 5e-4)) fail(`branch ${bank}° misses ${b.miss} m; output.txt says ${m[2]} m`);
  }
  const noBank = branches.find((b) => b.round === 1 && b.bank === 0)!;
  if (!near(noBank.miss, AIM_OFFSET_M, 1e-6)) fail('the no-bank miss is not the aim offset');

  const guidance = printed(/committed ([0-9.]+) deg lands ([0-9.]+) m off the aim vs the ballistic ([0-9.]+) m \(([0-9.]+)x better/, 'gate 4e');
  const committed = { bank: Number(guidance[1]), miss: Number(guidance[2]) };
  const best = branches.filter((b) => b.round === 2).reduce((a, b) => (b.miss < a.miss ? b : a));
  if (best.bank !== committed.bank || !near(best.miss, committed.miss, 5e-3)) fail('the closest fine branch is not the committed one');
  const scored = branchRows.filter((r) => r.round === 2 && r.bank_cmd === committed.bank).at(-1) ?? fail('no committed branch in the branch trace');
  const scoring = { t: scored.t, navErr: scored.nav_err, navSigma: Math.sqrt(scored.nav_var) };

  // The end points on one arc, set by the bank angle flown.
  const phi = (b: Branch) => (b.flown * Math.PI) / 180;
  const num = branches.reduce((a, b) => a + b.end.cross * Math.sin(phi(b)) + b.end.drop * (1 - Math.cos(phi(b))), 0);
  const den = branches.reduce((a, b) => a + Math.sin(phi(b)) ** 2 + (1 - Math.cos(phi(b))) ** 2, 0);
  const radius = num / den;
  const worstResidual = Math.max(
    ...branches.map((b) => Math.hypot(b.end.cross - radius * Math.sin(phi(b)), b.end.drop - radius * (1 - Math.cos(phi(b)))))
  );
  if (worstResidual > 0.05) fail(`the end points are ${worstResidual} m off one arc`);

  // ── After the pause: the committed branch flies on ───────────────────────────────────────
  const samples: Sample[] = descent.map((r) => ({
    t: r.t,
    altitudeKm: r.altitude / 1000,
    mach: r.mach,
    ne: r.ne_peak,
    plasmaFreq: r.plasma_freq,
    denied: r.gnss_denied === 1,
    navErr: r.nav_err,
    heatFlux: r.heat_flux,
    bank: r.bank,
  }));
  const band = printed(/GPS L1 \(w = ([0-9.e+-]+) rad\/s\)/, 'GPS L1 band');
  const bandRadS = Number(band[1]);
  const anchor = printed(/flight anchor ([0-9.e+-]+) m\^-3/, 'RAM-C II anchor');
  const ramcReference = Number(anchor[1]);
  const legTwoEnd = descent.findLastIndex((r) => r.leg === 2);
  const peakPassage = {
    t: samples[legTwoEnd].t,
    altitudeKm: samples[legTwoEnd].altitudeKm,
    electronDensity: samples[legTwoEnd].ne,
  };
  const legSnaps = [...outputTxt.matchAll(/marched \d+ steps to ([0-9.]+) km[\s\S]*?n_e peak = ([0-9.e+-]+) m\^-3[\s\S]*?nav error vs truth = ([0-9.]+) m/g)];
  if (legSnaps.length !== 4) fail(`expected four leg snapshots in output.txt, found ${legSnaps.length}`);
  if (!near(peakPassage.altitudeKm, Number(legSnaps[1][1]), 0.05) || !near(peakPassage.electronDensity, Number(legSnaps[1][2]), 1e-3 * peakPassage.electronDensity)) {
    fail('the peak passage in corridor_trace.csv does not match leg 2 of output.txt');
  }
  const exitIx = samples.findIndex((s, i) => i > pauseIx && !s.denied);
  if (exitIx < 0) fail('GPS never returns in corridor_trace.csv');
  const exit = { t: samples[exitIx].t, altitudeKm: samples[exitIx].altitudeKm, navErr: samples[exitIx].navErr };
  if (!near(exit.altitudeKm, Number(legSnaps[2][1]), 0.05) || !near(exit.navErr, Number(legSnaps[2][3]), 5e-5)) {
    fail('GPS return in corridor_trace.csv does not match leg 3 of output.txt');
  }
  if (!near(samples[samples.length - 1].navErr, Number(legSnaps[3][3]), 5e-5)) fail('the last step does not match leg 4 of output.txt');
  const dark = samples.slice(pauseIx, exitIx);
  const peakIx = dark.reduce((best, s, i) => (s.navErr > dark[best].navErr ? i : best), 0);
  const darkNavPeak = { t: dark[peakIx].t, navErr: dark[peakIx].navErr };
  const clock = printed(/wall-clock budget: ([0-9.]+) s elapsed/, 'wall-clock gate');
  const runSeconds = Number(clock[1]);

  const gates = outputTxt
    .split('\n')
    .map((l) => l.match(/\[(PASS|FAIL)\] \[\w+\] (\([0-9a-z]+\) [^:]+):/))
    .filter((m): m is RegExpMatchArray => m !== null)
    .map((m) => ({ passed: m[1] === 'PASS', label: m[2] }));

  return {
    pause,
    entry: { altitudeKm: descent[0].altitude / 1000, mach: descent[0].mach },
    endT: descent[descent.length - 1].t,
    descent: samples,
    bandRadS,
    ramcReference,
    peakPassage,
    darkNavPeak,
    exit,
    endNavErr: samples[samples.length - 1].navErr,
    runSeconds,
    branchSeconds: Math.max(...branches.map((b) => b.t[b.t.length - 1])),
    aimOffsetM: AIM_OFFSET_M,
    branches,
    committed,
    scoring,
    noBankMiss: noBank.miss,
    improvement: Number(guidance[4]),
    arc: { radius, worstResidual },
    gates,
  };
}
