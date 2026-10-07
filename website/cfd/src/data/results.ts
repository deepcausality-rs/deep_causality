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
  /** `output.txt` line 10: "marched 106 steps to 76.8 km, Mach 26.8". */
  onsetStep: 106,
  onsetKm: 76.8,
  /** `constants.rs` BRANCH_STEPS: every branch continues this many steps. */
  branchSteps: 100,
  /** Coarse sweep, `output.txt` lines 16-21, column "miss (traj)". */
  coarse: [
    { bank: 0, miss: 30.0 },
    { bank: 5, miss: 16.463 },
    { bank: 10, miss: 3.818 },
    { bank: 15, miss: 11.516 },
    { bank: 20, miss: 24.988 },
    { bank: 40, miss: 48.426 },
  ] satisfies BankBranch[],
  /** Fine sweep around the coarse winner, `output.txt` lines 26-36. */
  fine: [
    { bank: 7.5, miss: 9.8 },
    { bank: 8, miss: 8.501 },
    { bank: 8.5, miss: 7.227 },
    { bank: 9, miss: 5.994 },
    { bank: 9.5, miss: 4.835 },
    { bank: 10, miss: 3.818 },
    { bank: 10.5, miss: 3.085 },
    { bank: 11, miss: 2.865 },
    { bank: 11.5, miss: 3.264 },
    { bank: 12, miss: 4.103 },
    { bank: 12.5, miss: 5.174 },
  ] satisfies BankBranch[],
  /** The committed branch, marked `<- committed` on line 33. */
  committed: { bank: 11, miss: 2.865 } satisfies BankBranch,
  /** `output.txt` line 10, same line: Mach 26.8 at the onset. */
  onsetMach: 26.8,
  /**
   * `constants.rs` AIM_CROSS_RANGE_M: the aim point sits this far to the side of
   * the unsteered (0 degree) landing point, so the zero-bank miss of 30.0 m is a
   * property of the setup, not a measured outcome.
   */
  aimOffsetM: 30,
  /** `output.txt` line 5: "32x32 compressible tensor-train layer". */
  gridCells: '32×32',
  /** Gate (5b): "45.9 s elapsed (budget 600 s)". `clock` in corridor/main.rs spans the whole
   * example: descent, both fork rounds and the three legs after them, not the study alone. */
  wallClockS: 45.9,
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
 * The mid-burn throttle what-if, `retropropulsion/output.txt` lines 26-30, columns
 * `flown`, `preserved` and `axial m/s2`.
 */
export const retropropulsion = {
  roster: [
    { name: 'coast', flown: 0.0, preserved: null, axial: 13.9101 },
    { name: 'low', flown: 0.2, preserved: 0.4335, axial: 11.6754 },
    { name: 'mid', flown: 0.4, preserved: 0.18, axial: 11.6531 },
    { name: 'high', flown: 0.6, preserved: 0.0171, axial: 13.3226 },
    { name: 'hard', flown: 0.85, preserved: -0.0329, axial: 18.5463 },
  ] satisfies ThrottleBranch[],
  /** Gate (4c): the largest departure from the frozen-drag prediction, m/s. */
  frozenDragSeparation: 179.9207,
} as const;
