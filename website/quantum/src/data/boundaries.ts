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
    text: 'The qpu feature is a typed boundary with an in-process simulator behind it. There is no vendor backend, no pulse layer and no transpiler. The feature adds no network or async dependency.',
    evidence: 'deep_causality_quantum/Cargo.toml, feature `qpu`; src/types/qpu/',
  },
  {
    id: 'no-decoder',
    status: 'nongoal',
    mark: 'not pursued',
    title: 'It contains no decoding algorithm.',
    text: 'A decoder’s reading of the measurement record enters as data. The library validates a decoder’s detector error model against the circuit, and it ships no decoder and runs no real-time loop.',
    evidence: 'src/types/abstraction/decoder_abstraction.rs, module docs',
  },
  {
    id: 'no-distance',
    status: 'nongoal',
    mark: 'not pursued',
    title: 'It claims no code distance and no fault-tolerance threshold.',
    text: 'The code report says "no distance claimed". The fault-tolerance check tests each listed fault: the remainder must be constant, and the propagated Pauli must weigh no more than the fault. It claims no threshold, distance or asymptotic suppression.',
    evidence: 'src/types/abstraction/fault_tolerance.rs, module docs; qcl_geometric_qec output',
  },
  {
    id: 'cycles',
    status: 'nongoal',
    mark: 'by decision',
    title: 'It refuses cyclic causal structures.',
    text: 'build() returns CyclicStructureUnsupported for a cyclic candidate. The refusal is a scope decision: the criterion the library applies does not reject a cycle, so the builder does, and the error says so.',
    evidence: 'src/types/pipeline/config.rs; qcl_crosstalk output, [build]',
  },
  {
    id: 'predictions',
    status: 'partial',
    mark: 'you supply',
    title: 'You supply each candidate’s predicted read-out.',
    text: 'An experiment carries a cost, a shot count and one predicted probability per candidate. The library does not derive those numbers from a device model, so a decision is as good as the predictions it is given.',
    evidence: 'src/types/design/experiment_design.rs, `Experiment::new`; qcl_crosstalk constants.rs',
  },
  {
    id: 'scale',
    status: 'partial',
    mark: 'measured',
    title: 'The planner covers seven candidates by default.',
    text: 'design solves the minimum-cost cover exactly, over subsets of the candidate pairs: 2 to the power of the pair count, or 2^21 at seven. More candidates return HypothesisCountExceeded unless you raise the cap with with_max_hypotheses. The in-process simulator stops at 24 qubits, and the numeric ideal recovery of a code stops at ten.',
    evidence: 'src/types/design/experiment_design.rs; src/types/qpu/sim.rs; src/types/abstraction/ideal_recovery.rs',
  },
  {
    id: 'std',
    status: 'partial',
    mark: 'host only',
    title: 'The QCL builder needs the standard library.',
    text: 'The builder sits behind the qcm feature, which implies std because it reaches the causal graph. A bare-metal build gets the operator and gate layers, and CI builds those for thumbv7em-none-eabihf with a heap.',
    evidence: 'deep_causality_quantum/Cargo.toml; .github/workflows/rust_no_std.yml',
  },
  {
    id: 'speed',
    status: 'nongoal',
    mark: 'not pursued',
    title: 'It is not a simulator for large systems.',
    text: 'The kernels are dense complex matrices sized for the checks. The exact checks on codes decide over supports in F₂ and build no state vector.',
    evidence: 'src/types/qgates/operator_linalg.rs; qcl_geometric_qec output',
  },
];
