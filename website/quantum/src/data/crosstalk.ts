/**
 * The crosstalk-attribution run, as data.
 *
 * Every figure on the landing page that describes the demonstration reads from
 * here, so the hero figure, the plan table and the prose cannot disagree.
 *
 * Sources, all in this repository:
 *   - costs and the qubit factors: `examples/quantum_examples/qcl_examples/qcl_crosstalk/constants.rs`
 *     and `model.rs`
 *   - predicted read-outs, observation, verdicts, separation, plan, campaign: the output of
 *       cargo run --release -p quantum_examples --example qcl_crosstalk
 *     The run is seeded (`SEED`), and two runs produce identical output.
 *   - run time: release binary, process start included. Two measurements of 30 runs each gave
 *     medians of 4.0 and 3.8 ms, within a 3.6 to 4.9 ms spread, so the pages say about 4 ms.
 *
 * The costs and the qubit factors are inputs the example types into `constants.rs` and
 * `model.rs`. The predicted read-outs are computed from those factors by the example's response
 * model, and the observation is drawn from the Born sampler at H1's predicted read-out. The
 * campaign runs E1 and stops, because H1 separates. The page says each of these wherever the
 * numbers appear.
 */

export const RUN = {
  command: 'cargo run --release -p quantum_examples --example qcl_crosstalk',
  shots: 1024,
  seed: 20260821,
  floorBits: 5,
  agreementSigmas: 3,
  /** Wall clock of the release binary, process start included, as the pages quote it. */
  wallClock: 'about 4 ms',
  /** Observed read-out of the first planned experiment, and its standard error. */
  observed: { estimate: 0.396, standardError: 0.015 },
  /**
   * Separation of the survivor from its nearest rival, in bits, as the run prints it: the distance
   * between the predictions 0.40 and 0.10 over 1024 shots.
   */
  survivorSeparationBits: 99.5,
  /**
   * Separation of the tightest pair of candidates, as the run prints it ("tightest pair separates
   * at ... bits"): the smallest, over the three pairs, of the best separation any offered
   * experiment reaches. E1 and E2 reach it, so it is the chosen plan's as well.
   */
  tightestPairBits: 99.5,
  planCost: 2,
  /** What all four experiments cost together, as the run prints it. */
  allExperimentsCost: 5,
  /** Experiments the campaign ran before it stopped, and what they cost. */
  campaignRan: 1,
  campaignSpent: 1,
  precision: 'Float106',
} as const;

export type Candidate = {
  id: 'H1' | 'H2' | 'H3';
  /** Plain-language name of the cause. */
  cause: string;
  /** The structure, as the example names it. */
  structure: string;
  /** Read-out predicted by experiment E1, `do(Q1 = |1>) P(e2)`. */
  predictsE1: number;
  verdict: 'consistent' | 'rejected';
};

export const candidates: Candidate[] = [
  { id: 'H1', cause: 'Qubit 1 drives qubit 2', structure: 'Q1 → Q2', predictsE1: 0.4, verdict: 'consistent' },
  { id: 'H2', cause: 'Qubit 2 drives qubit 1', structure: 'Q2 → Q1', predictsE1: 0.1, verdict: 'rejected' },
  { id: 'H3', cause: 'A shared bath drives both qubits', structure: 'Q1 ← B → Q2', predictsE1: 0.1, verdict: 'rejected' },
];

export type Experiment = {
  id: 'E0' | 'E1' | 'E2' | 'E3';
  name: string;
  /** What is done, in words. */
  does: string;
  cost: number;
  /** Predicted read-out under H1, H2 and H3, in that order, as the run prints it. */
  predicts: [number, number, number];
  chosen: boolean;
};

export const experiments: Experiment[] = [
  { id: 'E0', name: 'Passive', does: 'Watch both qubits idle, read both excited', cost: 1, predicts: [0.04, 0.04, 0.04], chosen: false },
  { id: 'E1', name: 'Hold qubit 1', does: 'Hold qubit 1 excited, read qubit 2', cost: 1, predicts: [0.4, 0.1, 0.1], chosen: true },
  { id: 'E2', name: 'Hold qubit 2', does: 'Hold qubit 2 excited, read qubit 1', cost: 1, predicts: [0.1, 0.4, 0.1], chosen: true },
  { id: 'E3', name: 'Echo both', does: 'Refocus a quasi-static coupling, read both qubits excited', cost: 2, predicts: [0.01, 0.01, 0.04], chosen: false },
];
