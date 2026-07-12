# deep_causality_quantum

Quantum causal models (QCM) based on the causal monad for
[DeepCausality](https://deepcausality.com).

## Overview

A **quantum causal model** replaces the classical conditional-probability tables of a
causal graph with quantum channels. Each node `Aᵢ` carries a Choi–Jamiołkowski operator
`ρ_{Aᵢ|Pa(Aᵢ)}`, and the whole model is their product `σ = ∏ᵢ ρ_{Aᵢ|Pa(Aᵢ)}` — the
*process operator*. This crate is the quantum-information layer of the workspace: the
operator algebra that builds and validates those factors, the freeze-time checks that
decide whether a factorization is a legal QCM, and the gate kernels that lift quantum
mechanics into the DeepCausality causal monad (`PropagatingEffect`).

The pure-state ket (`HilbertState`, a minimal-left-ideal element of a Clifford algebra)
stays in `deep_causality_multivector` as the foundational carrier; all metric signatures
come from `deep_causality_metric`, the workspace's single source of truth. This crate
defines no metric type of its own.

### Key features

- **Operator layer** — a validated `DensityMatrix`, the Choi–Jamiołkowski isomorphism
  (`choi_from_kraus` / `kraus_from_choi`), channel application (`apply_kraus` /
  `apply_choi`), CPTP checks (`check_completely_positive` / `check_trace_preserving`), and
  the dense linear-algebra kernels (`partial_trace`, `embed_on_legs`, `matrix_commutator`,
  …) they are built from.
- **Freeze-time QCM validation** — the quantum Markov condition enforced as a
  commutativity check at the graph freeze boundary, plus C₃-exclusion faithfulness.
- **Orthomodular Verdict** — `Projection<R, D>`, the Birkhoff–von Neumann projection
  lattice, as a `Verdict` carrier, with Born-rule read-out to `Prob`.
- **Gate kernels on the causal monad** — Born probability, expectation value, gate
  application, commutator, fidelity, and the Haruna gauge-field logical gates, each with a
  `PropagatingEffect` wrapper that routes failure to the error channel.
- **Typed errors** — `QuantumError` names the exact failure: dimension / metric mismatch,
  non-finite values, normalization, non-positive operators, non-CPTP channels,
  non-convergence, freeze-time commutativity, or an unfaithful structure.
- **Two modalities, kept apart** — a *verifiable* default path backed by Lean proofs and
  an *emergent* physical-QPU seam behind the `qpu` feature.

## Quantum causal models

A classical causal model factorizes a joint distribution over its graph. A **quantum
causal model** (R. Lorenz, 2022) does the same for a *process operator* `σ` on the
composite Hilbert space, factorized into per-node Choi–Jamiołkowski operators. Not every
product of operators is a valid QCM: the factors must satisfy the **quantum Markov
condition** — factors whose Hilbert supports intersect must **pairwise commute** (Lorenz
2022, Def. 3.3).

This crate makes that condition a **freeze-time gate**. The factors ride an external
`ProcessFactors` decoration (the operator analogue of the engine's edge-keyed lambda
store); they are *static freeze-time data*, never carried on the runtime state channel.
`freeze_quantum` embeds each intersecting-support pair onto its common support, forms the
commutator, and compares `‖[ρⱼ, ρₖ]‖_F` against a depth-aware forward-error tolerance
(`CommutatorTolerance`). The check is **sound** — it never accepts a non-commuting model —
and may be incomplete. A failure names the exact offending pair and rolls the graph back to
its dynamic state.

```rust
// Condensed from the `qcm_freeze_check` example.
use deep_causality_quantum::{
    CommutatorTolerance, FactorSupports, ProcessFactors, freeze_quantum,
};

// σx and σz share leg 0 but do not commute → the freeze aborts, naming the pair.
let mut factors = ProcessFactors::<f64>::new();
factors.insert(0, sigma_x()); // a 2×2 complex CausalTensor
factors.insert(1, sigma_z());
let mut supports = FactorSupports::new();
supports.declare(0, &[0]);
supports.declare(1, &[0]);

let outcome = freeze_quantum(
    &mut graph, &[], &factors, &supports, &CommutatorTolerance::default(), None,
);
assert!(outcome.is_err()); // QuantumError::CommutatorNonZero { node_j: 0, node_k: 1, .. }
```

### Faithfulness (C₃-exclusion)

When the declared input/output systems are supplied (the last `freeze_quantum` argument),
the freeze additionally enforces **C₃-exclusion faithfulness** (van der Lugt & Lorenz,
arXiv:2508.11762): a causal structure that contains a `C₃` sub-relation (canonically, two
commuting CNOTs) has no traditional-circuit causally-faithful decomposition and is rejected
at freeze. The structure is derived from the frozen graph's reachability over the declared
systems by `CausalStructure`.

## Two modalities

The crate keeps two senses of "quantum" strictly apart by construction:

| Modality | What it is | Build |
|---|---|---|
| **Verifiable** | Deterministic simulated Choi–Jamiołkowski operators, checked at the freeze boundary and backed by Lean proofs. | Default |
| **Emergent** | A physical-QPU call lifted into the causal monad as a typed effect (`qpu_effect`), with an in-process `SimQpu` fallback. | `qpu` feature |

The emergent seam adds no network or async dependency; it is a typed boundary, not a driver.

## Core types

| Type | Role |
|---|---|
| `QuantumError` / `QuantumErrorEnum` | Typed failure (outer newtype over the enum) |
| `DensityMatrix<R>` | Validated mixed state: Hermitian, PSD, unit trace |
| `ProcessFactors<R>` / `FactorSupports` | External Choi-factor store and their Hilbert supports |
| `CommutatorTolerance<R>` / `QuantumMarkovReport<R>` | Depth-aware freeze tolerance and the instrumented report |
| `CausalStructure` | Declared input/output influence relation; C₃ detection |
| `Projection<R, D>` | Orthomodular projection-lattice `Verdict` carrier |
| `QuantumOps<R>` / `QuantumGates` | Dirac-notation state operations and the standard gate interface |
| `Operator<R>` / `Gate<R>` | Aliases for `HilbertState<R>` (from `deep_causality_multivector`) |
| `QuantumCircuit` / `GateOp` / `SimQpu` / `QpuSampler` | The `qpu`-feature emergent seam |

## The operator layer

The Choi–Jamiołkowski isomorphism represents a channel `E: L(H_in) → L(H_out)` as an
operator `J(E) = Σ_{ik} |i⟩⟨k| ⊗ E(|i⟩⟨k|)`. `E` is completely positive iff `J ⪰ 0` and
trace-preserving iff `Tr_out(J) = I_in`. The crate provides both directions and the
validation:

- `choi_from_kraus` / `kraus_from_choi` — the isomorphism, gated on finiteness and
  Hermiticity (PSD requires both).
- `apply_kraus` / `apply_choi` — the two channel-application routes.
- `check_completely_positive` / `check_trace_preserving` — the CPTP checks.
- `partial_trace`, `embed_on_legs`, `matrix_commutator`, `matrix_trace`, `frobenius_norm`,
  `hermiticity_defect`, `identity_matrix` — the dense complex linear algebra (Hermitian
  eigendecomposition from `deep_causality_tensor`) they compose.

## Verdicts at the measurement boundary

A quantum causaloid does not carry a `Verdict` over its operators — general effects
`0 ≤ E ≤ I` form only an effect algebra with *partial* meet/join. Instead a verdict is
*read out* at the measurement boundary: the Born rule `Tr(Pρ)` becomes a `Prob` MV-algebra
verdict (`born_projective_prob`), or the measurement projection itself is a proposition in
the orthomodular lattice `Projection<R, D>` — a bounded lattice with orthocomplement
`I − P` and meet/join on subspace ranges, which satisfies the orthomodular law but *fails*
distributivity (the way `Prob` fails excluded middle).

## Gate kernels on the causal monad

Every kernel over `HilbertState<R>` has a `PropagatingEffect` wrapper that lifts it into
the arity-5 causal monad:

| Kernel | Wrapper | Computes |
|---|---|---|
| `born_probability_kernel` | `born_probability` | `\|⟨basis\|state⟩\|²` |
| `expectation_value_kernel` | `expectation_value` | `⟨ψ\|A\|ψ⟩` for a Hermitian `A` |
| `apply_gate_kernel` | `apply_gate` | `U\|ψ⟩` |
| `commutator_kernel` | `commutator` | `[A, B]` |
| `fidelity_kernel` | `fidelity` | state fidelity |

Because `HilbertState` is a Clifford minimal-left-ideal ket, the inner product is
signature-dependent: the reversion adjoint on a positive-signature (Euclidean) metric, and
the Clifford (Dirac) conjugation on a negative-signature `Cl(0,n)` metric
(`dirac_bracket_kernel`, `clifford_conjugation`). The **Haruna logical gates**
(`logical_s` / `z` / `x` / `hadamard` / `cz` / `t`, after Haruna 2025, arXiv:2511.15224)
realize gauge-field-formalism gates on those fields; their matrix exponential surfaces
overflow and non-convergence as typed errors rather than a silent identity.

## Usage

The crate is unpublished; add it as a git dependency:

```toml
[dependencies]
deep_causality_quantum = { git = "https://github.com/deepcausality-rs/deep_causality.git", branch = "main" }
```

Enable the emergent QPU seam with the `qpu` feature:

```toml
deep_causality_quantum = { git = "…", branch = "main", features = ["qpu"] }
```

## Examples

Runnable examples live in [`examples/quantum_examples/`](../examples/quantum_examples):

| Example | Demonstrates |
|---|---|
| `qcm_freeze_check` | The freeze-time Markov commutativity check: a commuting model freezes, a non-commuting one aborts naming the pair. |
| `quantum_counterfactual` | Quantum error correction via history-aware state rewind ("time travel") on the causal monad. |
| `gauge_electroweak` | Electroweak `SU(2)×U(1)` unification and Higgs symmetry breaking. |
| `topological_insulator` | Chern number of the Qi-Wu-Zhang model via the tangent functor and nested Brillouin-zone quadrature. |
| `quantum_geometric_tensor` | The quantum geometric tensor (quantum metric + Berry curvature) for flat-band systems. |
| `ikkt_matrix_model` | The IKKT matrix model: spacetime emerging from `S = -Tr([X_μ, X_ν]²)`. |
| `hopf_fibration_multivector` | Projecting a spinor state through the Hopf fibration onto the Bloch sphere. |

Run one with, for example:

```bash
cargo run --release -p quantum_examples --example qcm_freeze_check
```

## Feature flags

| Feature | Default | Description |
|---|:---:|---|
| `qpu` | | The emergent QPU seam: `QpuSampler` / `ShotHistogram` / the shots→`Uncertain` bridges / `qpu_effect` / the in-process `SimQpu`. Adds no network or async dependency. |

## Formalization

The verifiable path is backed by Lean 4 proofs under
[`lean/DeepCausalityFormal/Quantum/`](../lean/DeepCausalityFormal/Quantum): the
partial-trace identities (`PartialTrace.lean`), the Choi application (`Choi.lean`), and the
counterexample showing that the partial trace does **not** preserve commutation
(`PartialTraceCounterexample.lean`) — the theorem that makes the freeze commutativity check
a genuine obligation. Each proved theorem is tied to an executable Rust witness under
`tests/formalization_lean/`.

## Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).
