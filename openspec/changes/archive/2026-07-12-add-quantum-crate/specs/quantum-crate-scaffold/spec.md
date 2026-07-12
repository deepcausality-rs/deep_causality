## ADDED Requirements

### Requirement: A dedicated `deep_causality_quantum` crate under workspace policy

The change SHALL add a new crate `deep_causality_quantum` as a workspace member that adopts the
repo-wide policy: `[lints] workspace = true`, `unsafe_code = "forbid"`, MSRV `rust-version = 1.93.0`,
no `dyn`, and no crate-defined macros. The crate SHALL depend only on `deep_causality` (the engine
crate — required by the freeze-hook integration: `CausableGraph::freeze_verified_with_check` and
`CausalityGraphError` live there, and the orphan-legal `impl From<QuantumError> for
CausalityGraphError` must therefore sit in this crate; the engine does not and must not depend back),
`deep_causality_core`, `deep_causality_haft`, `deep_causality_algebra`, `deep_causality_num`,
`deep_causality_num_complex`, `deep_causality_multivector`, `deep_causality_tensor`,
`deep_causality_uncertain` (optional, activated by the `qpu` feature), and
`deep_causality_metric` (the metric-signature single source of truth), and SHALL NOT depend on
`deep_causality_physics` (no dependency cycle). It SHALL define no metric-signature type of its own,
using `deep_causality_metric::Metric` for all Clifford/physics signatures.

#### Scenario: The crate builds under the workspace lint gate

- **WHEN** `bazel test //...` and `cargo clippy -p deep_causality_quantum --all-targets` run
- **THEN** the crate compiles with `unsafe_code = "forbid"` and clippy `-D warnings`, and it appears
  in the workspace `members` list

#### Scenario: No dependency cycle through physics

- **WHEN** the dependency graph is inspected
- **THEN** `deep_causality_quantum` does not depend on `deep_causality_physics`, and the migrated
  kernels reference the crate-local `QuantumError` rather than `PhysicsError`

### Requirement: Errors are a typed `QuantumError`

The crate SHALL expose a `QuantumError` error type as an outer newtype struct wrapping a
`QuantumErrorEnum` whose variants name the exact failure (mirroring the repo's
`CausalityError(CausalityErrorEnum::…)` convention), and the migrated kernels SHALL return it in place
of `PhysicsError`. `QuantumError` SHALL implement `core::error::Error` and `Display`, and typed
variants SHALL be preferred over a `String`-only catch-all wherever a specific variant fits.

#### Scenario: A failure surfaces as a typed variant

- **WHEN** an operation fails (e.g. a non-positive operator, a Clifford-metric mismatch, or a freeze
  commutativity failure)
- **THEN** it returns `QuantumError(QuantumErrorEnum::<Variant>{ … })` naming the specific cause (for a
  commutativity failure, the offending operator pair), not an untyped string

### Requirement: Quantum-information kernels migrate out of `deep_causality_physics`

The quantum-information kernels SHALL move from `deep_causality_physics/src/kernels/quantum/`
(`QuantumGates`/`QuantumOps` traits, the Haruna logical gates, the `Operator`/`Gate` aliases and the
gate/born/expectation/commutator/fidelity kernels, and the `PropagatingEffect` wrappers) into
`deep_causality_quantum`, while `HilbertState` SHALL remain in `deep_causality_multivector` as the
foundational ket carrier. Every dependent (the `deep_causality_physics` re-exports and the four
affected example files) SHALL be updated so the workspace builds.

#### Scenario: Physics loses the quantum-information layer but still builds

- **WHEN** the migration lands and `bazel test //...` runs
- **THEN** `deep_causality_physics` no longer exports the moved kernels, the moved items are exported
  from `deep_causality_quantum`, and `deep_causality_physics`, `deep_causality_multivector`, and the
  quantum examples all compile and pass

#### Scenario: `klein_gordon` disposition is explicit

- **WHEN** the migration scope is fixed in Phase 0
- **THEN** the `klein_gordon` PDE kernel's location is decided against its `multi_physics_pipeline`
  usage (default: it stays in `deep_causality_physics`), and the decision is recorded in `design.md`

### Requirement: Verifiable and emergent modalities are kept type-distinct

The crate SHALL separate the verifiable modality (deterministic simulated Choi–Jamiołkowski
operators, the default build) from the emergent modality (a physical cloud-QPU call), exposing the
emergent path only as a `QpuSampler`-style seam behind an off-by-default `qpu` feature, and SHALL NOT
add any network or async dependency in this change.

#### Scenario: Default build is verifiable-only

- **WHEN** `deep_causality_quantum` is built with default features
- **THEN** only the verifiable simulated-CJ path compiles, the `qpu` seam is absent, and no
  network/async dependency is pulled in

#### Scenario: The emergent seam is a trait, not an adapter

- **WHEN** the `qpu` feature is enabled
- **THEN** a `QpuSampler` seam trait is available whose implementations return measurement shots as
  classical/`Uncertain` data at the Kleisli cut, and no concrete vendor adapter is shipped by this
  change

### Requirement: A reified circuit and a generic classical-shot `QpuSampler`

The crate SHALL expose a storable `QuantumCircuit` (a `num_qubits` register, an ordered `Vec<GateOp>`
over the migrated gate alphabet, and a computational-basis measurement) as plain data
(`Clone + Debug + PartialEq`) carrying no `HilbertState` and no amplitudes, whose constructor rejects an
out-of-range qubit index with a typed `QuantumError`; and a `QpuSampler` trait used only as a generic
bound `S: QpuSampler` (never `dyn`) with `sample(&self, &QuantumCircuit, shots) -> Result<Self::Shots,
QuantumError>` where the associated `Shots` is bounded by a `ShotHistogram` trait exposing a classical
outcome-count map — never amplitudes. The crate SHALL ship an in-process deterministic simulator
implementing `QpuSampler` over the migrated gate kernels; no network/async/vendor dependency SHALL be
added.

#### Scenario: A deterministic simulator satisfies the seam

- **WHEN** the in-process simulator samples a Bell circuit for `N` shots with a fixed seed
- **THEN** it returns `Ok` with counts summing to `N`, the same seed reproduces the same histogram, no
  amplitudes are exposed, and an out-of-range qubit in `QuantumCircuit::new` returns a `QuantumError`

### Requirement: Shots bridge to `Uncertain` and lift into the causal monad under the `qpu` feature

The crate SHALL provide bridge functions mapping a `ShotHistogram` to `deep_causality_uncertain` values
(a per-qubit `Uncertain<bool>` via `bernoulli`, and an observable `Uncertain<f64>` via `from_samples`),
introducing no new value substance and returning a typed `QuantumError` on an empty histogram. Behind an
off-by-default `qpu` feature the crate SHALL expose `qpu_effect(sampler, circuit, shots)` returning a
`CausalEffectPropagationProcess` that routes the shot summary to the value channel, requested parameters
to state, device calibration to context, job failure to error (value absent), and provenance to log;
the default build SHALL compile without it and pull in no network/async dependency.

#### Scenario: Success and failure route to distinct channels

- **WHEN** the `qpu` feature is on and `qpu_effect` runs against a simulator that succeeds, then returns
  a `QuantumError`
- **THEN** the success carries the shot summary on the value channel with params/calibration/log
  populated, and the failure carries the error on the error channel with the value absent
