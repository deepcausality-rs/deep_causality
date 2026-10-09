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
    title: 'You supply the physics of each experiment.',
    text: 'A ConfiguredExperiment holds a configuration, and a ResponseModel you write says what that configuration does in each candidate’s world; the library computes every predicted read-out from it. The crosstalk example writes its candidates as conditional tables and its model in model.rs. Experiment::new still takes typed predictions for a caller without a model. The plan and the verdict are only as good as the model.',
    evidence: 'src/types/pipeline/configured_experiment.rs, src/types/pipeline/response.rs; qcl_crosstalk model.rs',
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

/**
 * Where the sensing examples stop. Printed in their own section of `/boundaries/`, which
 * `/sensing/` links to. Sources: the two examples under
 * `examples/quantum_examples/qcl_examples/` and their output, and the verifications
 * under `deep_causality_quantum/verification/sensing/`.
 */
export const sensingBoundaries: Boundary[] = [
  {
    id: 'sensing-simulated',
    status: 'partial',
    mark: 'simulated',
    title: 'The sensing examples run on simulated observations.',
    text: 'Each example draws its observations from the cause under test, and exits nonzero when a campaign finds a different cause while the true one is on the list. The run with clipping left off the list expects the misattribution and checks for it. QCL also takes counts a lab measured and published values with their standard errors, and the sensing verifications use published values and data regenerated from published models.',
    evidence: 'src/types/pipeline/evidence_source.rs; verification/sensing/common/attribution.rs',
  },
  {
    id: 'sensing-placeholders',
    status: 'partial',
    mark: 'you supply',
    title: 'A lab supplies the placeholders.',
    text: 'The gravimeter’s setup times and tide, and the gradiometer’s leak, dark background and setup times, have no published source, and constants.rs marks each one. The 20,000 s temperature scan follows from the ultracold operation of Karcher et al.',
    evidence: 'qcl_gravimeter_systematics/constants.rs; qcl_gradiometer_crosstalk/constants.rs',
  },
  {
    id: 'sensing-list',
    status: 'partial',
    mark: 'shown',
    title: 'A cause off the list is blamed on another.',
    text: 'A campaign stops at the first candidate that holds and separates. A leak in both directions ends the gradiometer’s campaign at the one-way leak after one experiment, and with clipping off the list the gravimeter’s campaign blames Coriolis. The full plan runs every planned experiment and reports the two-way leak outside the model.',
    evidence: 'qcl_gradiometer_crosstalk output, [checks]; qcl_gravimeter_systematics output, [clipping left off the list]',
  },
  {
    id: 'sensing-white-noise',
    status: 'partial',
    mark: 'measured',
    title: 'Integration time follows white noise inside a range.',
    text: 'The planner sizes draws on σ(τ) = S/√τ and reports a pair uncovered when it needs more averaging than the instrument’s white-noise range holds. The AQG’s one-day scatter, 9.4 nm/s² against 2.55 predicted, lies past that range.',
    evidence: 'src/types/design/instrument_time.rs; verification/sensing/verification_v6_time_model.rs',
  },
  {
    id: 'sensing-drift',
    status: 'partial',
    mark: 'tide only',
    title: 'The examples model one drift, the tide.',
    text: 'Each gravimeter experiment carries the Earth tide at the time it runs, and the campaign plans again when the tide moves a planned prediction. Atom temperature and field hold still in both examples.',
    evidence: 'qcl_gravimeter_systematics/model_config.rs; src/types/pipeline/campaign.rs',
  },
  {
    id: 'sensing-correction',
    status: 'partial',
    mark: 'not built',
    title: 'The gravimeter finds the cause and stops.',
    text: 'The correction each survivor implies, and the run that checks it, are not built. The gradiometer prints the corrective action for its survivor and the bias the mechanism adds.',
    evidence: 'qcl_gravimeter_systematics/README.md, “Not built”; qcl_gradiometer_crosstalk/main.rs, consequence',
  },
  {
    id: 'sensing-screens',
    status: 'partial',
    mark: 'by construction',
    title: 'The quantum checks pass every sensing candidate.',
    text: 'The gravimeter’s candidates are mechanisms, which skip the validate stage. The gradiometer’s factors are diagonal tables, which the normalisation, Markov and decomposability checks pass. The experiments alone separate the candidates.',
    evidence: 'qcl_gravimeter_systematics/main.rs; qcl_gradiometer_crosstalk/main.rs, screen',
  },
];
