/**
 * The crosstalk-attribution run, as data.
 *
 * Every figure on the landing page that describes the demonstration reads from
 * here, so the hero figure, the plan table and the prose cannot disagree.
 *
 * Sources, all in this repository:
 *   - read-outs and costs: `examples/quantum_examples/qcl_examples/qcl_crosstalk/constants.rs`
 *   - observation, verdicts, separation, plan: the output of
 *       cargo run --release -p quantum_examples --example qcl_crosstalk
 *     The run is seeded (`SEED`), and two runs produce identical output.
 *   - run time: release binary, process start included. One measurement gave a
 *     median of 3.0 ms over 20 runs and a second 2.4 ms over 30 runs, each within a
 *     2.2 to 3.6 ms spread, so the pages say 2 to 3 ms.
 *
 * The read-outs are the example's stated modelling assumptions, and the
 * observation is drawn from the Born sampler at H1's predicted read-out. The
 * page says both wherever the numbers appear.
 */

export const RUN = {
  command: 'cargo run --release -p quantum_examples --example qcl_crosstalk',
  shots: 1024,
  seed: 20260821,
  floorBits: 5,
  agreementSigmas: 3,
  /** Wall clock of the release binary, process start included, as the pages quote it. */
  wallClock: '2 to 3 ms',
  /** Observed read-out of the first planned experiment, and its standard error. */
  observed: { estimate: 0.377, standardError: 0.015 },
  /** Separation of the survivor from its nearest rival, in bits. */
  survivorSeparationBits: 100.1,
  /** Separation of the tightest pair of candidates under the chosen plan, in bits. */
  tightestPairBits: 164.8,
  planCost: 2,
  tomographyCost: 200,
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
  { id: 'H3', cause: 'A shared bath drives both', structure: 'Q1 ← B → Q2', predictsE1: 0.1, verdict: 'rejected' },
];

export type Experiment = {
  id: 'E0' | 'E1' | 'E2' | 'E3' | 'E4';
  name: string;
  /** What is done, in words. */
  does: string;
  cost: number;
  /** Predicted read-out under H1, H2 and H3, in that order. */
  predicts: [number, number, number];
  chosen: boolean;
};

export const experiments: Experiment[] = [
  { id: 'E0', name: 'Passive', does: 'Watch both qubits idle', cost: 1, predicts: [0.04, 0.04, 0.04], chosen: false },
  { id: 'E1', name: 'Hold qubit 1', does: 'Hold qubit 1 excited, read qubit 2', cost: 1, predicts: [0.4, 0.1, 0.1], chosen: true },
  { id: 'E2', name: 'Hold qubit 2', does: 'Hold qubit 2 excited, read qubit 1', cost: 1, predicts: [0.1, 0.4, 0.1], chosen: true },
  { id: 'E3', name: 'Echo both', does: 'Refocus a static coupling, read both', cost: 2, predicts: [0.01, 0.01, 0.04], chosen: false },
  { id: 'E4', name: 'Process tomography', does: 'Reconstruct the whole process', cost: 200, predicts: [0.9, 0.5, 0.1], chosen: false },
];
