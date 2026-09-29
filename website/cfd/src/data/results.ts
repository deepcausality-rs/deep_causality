/**
 * Run results the landing page draws as figures.
 *
 * Every number is copied from a committed `output.txt` under
 * `examples/avionics_examples/cfd/plasma_blackout/`. Nothing is rounded beyond
 * what the run printed and nothing is estimated. All figures are `f64`, release
 * build, on the machine named by `MACHINE` in `src/consts.ts`.
 */

export interface BankBranch {
  /** Commanded bank angle, degrees. */
  bank: number;
  /** Trajectory-derived miss distance from the shared aim point, metres. */
  miss: number;
}

/**
 * The bank-angle what-if, `corridor/output.txt`.
 *
 * Both rounds fork the same paused onset. The 10.0° branch flew in both rounds
 * and returned the same miss, so it appears in both lists.
 */
export const corridor = {
  /** `output.txt` line 10: "marched 131 steps to 73.2 km, Mach 27.2". */
  onsetStep: 131,
  onsetKm: 73.2,
  /** `constants.rs` BRANCH_STEPS: every branch continues this many steps. */
  branchSteps: 100,
  /** Coarse sweep, `output.txt` lines 16-21, column "miss (traj)". */
  coarse: [
    { bank: 0, miss: 20.0 },
    { bank: 5, miss: 11.581 },
    { bank: 10, miss: 3.533 },
    { bank: 15, miss: 6.002 },
    { bank: 20, miss: 14.303 },
    { bank: 40, miss: 28.881 },
  ] satisfies BankBranch[],
  /** Fine sweep around the coarse winner, `output.txt` lines 26-36. */
  fine: [
    { bank: 7.5, miss: 7.429 },
    { bank: 8, miss: 6.614 },
    { bank: 8.5, miss: 5.809 },
    { bank: 9, miss: 5.021 },
    { bank: 9.5, miss: 4.256 },
    { bank: 10, miss: 3.533 },
    { bank: 10.5, miss: 2.88 },
    { bank: 11, miss: 2.359 },
    { bank: 11.5, miss: 2.07 },
    { bank: 12, miss: 2.112 },
    { bank: 12.5, miss: 2.467 },
  ] satisfies BankBranch[],
  /** The committed branch, marked `<- committed` on line 34. */
  committed: { bank: 11.5, miss: 2.07 } satisfies BankBranch,
  /** `output.txt` line 10, same line: Mach 27.2 at the onset. */
  onsetMach: 27.2,
  /**
   * `constants.rs` AIM_CROSS_RANGE_M: the aim point sits this far to the side of
   * the unsteered (0 degree) landing point, so the zero-bank miss of 20.0 m is a
   * property of the setup, not a measured outcome.
   */
  aimOffsetM: 20,
  /** `output.txt` line 3: "32x32 compressible tensor-train layer". */
  gridCells: '32×32',
  /** Gate (5b): "44.4 s elapsed (budget 600 s)". `clock` in corridor/main.rs spans the whole
   * example: descent, both fork rounds and the three legs after them, not the study alone. */
  wallClockS: 44.4,
} as const;

export interface ThrottleBranch {
  name: string;
  /** Throttle the branch actually flew, after the envelope's clamp. */
  flown: number;
  /** Fraction of the aerodynamic drag the plume left in place; null for the coast branch. */
  preserved: number | null;
  /** Deceleration each branch flew, m/s². */
  axial: number;
}

/**
 * The mid-burn throttle what-if, `retropulsion/output.txt` lines 24-29, columns
 * `flown`, `preserved` and `axial m/s2`.
 */
export const retropulsion = {
  roster: [
    { name: 'coast', flown: 0.0, preserved: null, axial: 10.595 },
    { name: 'low', flown: 0.2, preserved: 0.251, axial: 7.4718 },
    { name: 'mid', flown: 0.4, preserved: 0.1238, axial: 9.9667 },
    { name: 'high', flown: 0.6, preserved: -0.0161, axial: 12.8932 },
    { name: 'hard', flown: 0.7931, preserved: -0.0611, axial: 17.1977 },
  ] satisfies ThrottleBranch[],
  /** Gate (4c): the largest departure from the frozen-drag prediction, m/s. */
  frozenDragSeparation: 139.3755,
} as const;
