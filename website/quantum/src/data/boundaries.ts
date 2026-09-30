/**
 * What the library does not do, and where it stops. One list, read by the
 * landing page (the first six) and by `/boundaries/` (all of them).
 *
 * Each row cites its evidence. `status`:
 *   nongoal   a decision, stated with its reason
 *   partial   a limit a user meets, with its measured or coded value
 *
 * Sources: the crate under `deep_causality_quantum/src/`, `Cargo.toml`, and the
 * `add-qcl` proposal's out-of-scope list under `openspec/changes/archive/`.
 */

export type Boundary = {
  id: string;
  status: 'nongoal' | 'partial';
  mark: string;
  title: string;
  text: string;
  /** Where a reader can verify the row. */
  evidence: string;
};

export const boundaries: Boundary[] = [
  {
    id: 'no-hardware',
    status: 'nongoal',
    mark: 'not pursued',
    title: 'It does not drive quantum hardware.',
    text: 'The qpu feature defines a typed sampler interface, QpuSampler, and ships an in-process simulator, SimQpu, behind it. The crate has no vendor backend, no pulse layer and no transpiler, and the feature adds no network or async dependency.',
    evidence: 'deep_causality_quantum/Cargo.toml, feature `qpu`; src/types/qpu/',
  },
  {
    id: 'no-decoder',
    status: 'nongoal',
    mark: 'not pursued',
    title: 'It contains no decoding algorithm.',
    text: 'You supply the decoder’s reading of the measurement record as a matrix, and the library checks the decoder’s detector error model against the circuit. The crate has no decoder trait and runs no real-time loop.',
    evidence: 'src/types/abstraction/decoder_abstraction.rs, module docs',
  },
  {
    id: 'no-distance',
    status: 'nongoal',
    mark: 'not pursued',
    title: 'It claims no code distance and no fault-tolerance threshold.',
    text: 'The code report prints "no distance claimed". The fault-tolerance check tests each fault you list: the remainder must be constant, and the propagated Pauli must weigh no more than the fault. A report gives one result per fault and states no threshold, distance or asymptotic suppression.',
    evidence: 'src/types/abstraction/fault_tolerance.rs, module docs; qcl_geometric_qec output',
  },
  {
    id: 'cycles',
    status: 'nongoal',
    mark: 'by decision',
    title: 'It refuses cyclic causal structures.',
    text: 'build() returns CyclicStructureUnsupported for a cyclic candidate, before any check runs. This is a scope decision, and the error message says so: a cycle does not fail the C₃ criterion.',
    evidence: 'src/types/pipeline/config.rs; qcl_crosstalk output, [build]',
  },
  {
    id: 'predictions',
    status: 'partial',
    mark: 'you supply',
    title: 'You supply each candidate’s predicted read-out.',
    text: 'Each experiment carries a cost, a shot count and one predicted probability per candidate, and Experiment::new takes them as data. The crate can compute a prediction from a candidate’s factors (Hypothesis::evaluate); the crosstalk example does not call it and types its predictions into constants.rs. The plan and the verdict are only as good as those numbers.',
    evidence: 'src/types/design/experiment_design.rs, `Experiment::new`; qcl_crosstalk constants.rs',
  },
  {
    id: 'scale',
    status: 'partial',
    mark: 'measured',
    title: 'The planner covers seven candidates by default.',
    text: 'design solves the minimum-cost cover exactly over subsets of the candidate pairs, which number 2 to the power of the pair count, or 2^21 at seven. An eighth candidate returns HypothesisCountExceeded unless you raise the cap with with_max_hypotheses. SimQpu stops at 24 qubits, and the numeric ideal recovery of a code stops at ten.',
    evidence: 'src/types/design/experiment_design.rs; src/types/qpu/sim.rs; src/types/abstraction/ideal_recovery.rs',
  },
  {
    id: 'std',
    status: 'partial',
    mark: 'host only',
    title: 'The QCL builder needs the standard library.',
    text: 'The builder sits behind the qcm feature, which implies std because the causal graph it uses needs std. A bare-metal build gets the operator and gate layers; CI builds those for thumbv7em-none-eabihf with a heap.',
    evidence: 'deep_causality_quantum/Cargo.toml; .github/workflows/rust_no_std.yml',
  },
  {
    id: 'speed',
    status: 'nongoal',
    mark: 'not pursued',
    title: 'It is not a simulator for large systems.',
    text: 'The kernels are dense complex matrices sized for the checks. The exact code checks decide over supports in F₂ and build no state vector.',
    evidence: 'src/types/qgates/operator_linalg.rs; qcl_geometric_qec output',
  },
];
