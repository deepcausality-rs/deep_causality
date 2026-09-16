# Quantum Examples

Worked examples whose subject matter is directly quantum: quantum
computing, quantum geometry of electronic bands, topological quantum
matter, electroweak loop corrections, and the spinor/Bloch-sphere
structure of a qubit state.

These were consolidated here from `physics_examples`,
`material_examples`, and `mathematics_examples` so that the quantum
material lives in one place. Each example is self-contained; there is no
shared library code.

This crate sits alongside
[`physics_examples`](../physics_examples),
[`material_examples`](../material_examples), and
[`mathematics_examples`](../mathematics_examples).

## Examples

| Example | Field | What it shows | Command |
|---|---|---|---|
| [`quantum_counterfactual`](quantum_counterfactual/README.md) | Quantum computing | The three-qubit repetition code: a real `X` gate, two parity measurements whose values are `±1` whatever the amplitudes are, and a decoder that restores the state exactly. `alternate_value_if` then forces a different syndrome, and the same decoder destroys it. | `cargo run -p quantum_examples --example quantum_counterfactual` |
| [`quantum_geometric_tensor`](quantum_geometric_tensor/README.md) | Condensed matter | The quantum geometric tensor as one object with two readings: a symmetric metric and an antisymmetric Berry curvature. The metric computed here is what feeds the flat-band Drude weight, so the geometry-to-transport chain is made rather than asserted. | `cargo run -p quantum_examples --example quantum_geometric_tensor` |
| [`gauge_electroweak`](gauge_electroweak/README.md) | Quantum field theory | The W mass predicted from three measured numbers: tree level misses by 1.5 GeV, one loop lands within 8 MeV of the measurement. The tolerance the run checks against is the accuracy the summary claims. | `cargo run -p quantum_examples --example gauge_electroweak` |
| [`topological_insulator`](topological_insulator/README.md) | Quantum materials | The Chern number two independent ways: exact tangent-functor derivatives under nested quadrature, against a Fukui-Hatsugai-Suzuki lattice sum. One differentiates and never forms a spinor; the other forms spinors and never differentiates. | `cargo run -p quantum_examples --example topological_insulator` |
| [`hopf_fibration_multivector`](hopf_fibration_multivector/README.md) | Quantum state geometry | Why a qubit's global phase is unobservable: the state as a rotor in `Cl(3)`, the Bloch vector as a sandwich product, and a full circuit of the fiber showing the state travel while its shadow holds still. The spinor double cover is visible in the walk. | `cargo run -p quantum_examples --example hopf_fibration_multivector` |
| [`qcm_freeze_check`](qcm_freeze_check/README.md) | Quantum causal models | The condition that decides whether a graph of Choi-Jamiolkowski factors is a quantum causal model at all: factors sharing a Hilbert leg must pairwise commute. Enforced at the freeze boundary, and a failure rolls the graph back to dynamic. | `cargo run -p quantum_examples --example qcm_freeze_check` |
| [`qcl_qcm_freeze`](qcl_examples/qcl_qcm_freeze/README.md) | Quantum causal models | The QCL model path: one configuration origin, `validate` running the Markov and C₃ checks the shipped freeze runs, a screen on success and the structured error on failure, and the shipped freeze's rollback on a dynamic graph. | `cargo run -p quantum_examples --example qcl_qcm_freeze` |
| [`qcl_geometric_qec`](qcl_examples/qcl_geometric_qec/README.md) | Quantum error correction | The QCL code path on the [[32, 2]] toric code: `derive_code`, `check_ldpc_weights`, class invariance of Z̄, S̄ and T̄ over the code space, and the Clifford check of H̄, all exact 𝔽₂ predicates and none simulated. | `cargo run -p quantum_examples --example qcl_geometric_qec` |
| [`qcl_concatenated_code`](qcl_examples/qcl_concatenated_code/README.md) | Quantum error correction | QCL-2: the `[[4,2,2]]` code concatenated with itself as two abstractions composed; `Z̄` and `X̄` compose exactly (Proposition 17), the law `ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂` is recorded with both constants, and a gate across code blocks is refused by name. Runs at `f32`, `f64` and `Float106`. | `cargo run -p quantum_examples --example qcl_concatenated_code` |
| [`qcl_code_switching`](qcl_examples/qcl_code_switching/README.md) | Quantum error correction | QCL-2: switching two logical qubits from `[[4,2,2]]` into `[[8,2,2]]` through a decode-and-re-encode gadget as the first link; a noiseless gadget composes exactly and a depolarised one sits under the recorded bound. | `cargo run -p quantum_examples --example qcl_code_switching` |
| [`qcl_distillation_round`](qcl_examples/qcl_distillation_round/README.md) | Quantum error correction | QCL-2: a distillation round on `[[4,2,2]]`, noisy encoded `T̄ H̄` under the ideal recovery, the case the paper defers; claims the residual it measures and the bound the law records. | `cargo run -p quantum_examples --example qcl_distillation_round` |
| [`qcl_crosstalk`](qcl_examples/qcl_crosstalk/README.md) | Quantum causal discovery | The keystone: four structural candidates, the cyclic one refused at `build()`, three screened by Markov and C₃, the hand-off into `control`, a two-intervention plan at cost 2 against tomography at 200, and a Boolean adjudication naming the direct cause. | `cargo run -p quantum_examples --example qcl_crosstalk` |
| [`qcl_crosstalk_circuits`](qcl_examples/qcl_crosstalk_circuits/README.md) | Quantum causal discovery | QCL-2: the crosstalk decision reproduced over circuit-derived candidates; `H₁`, `H₂` and the cyclic `H₄` as circuits whose wiring carries the structure, the cycle refused at `build()`, the dilations screened and forked, the same plan and survivor. | `cargo run -p quantum_examples --example qcl_crosstalk_circuits` |
| [`ikkt_matrix_model`](ikkt_matrix_model/README.md) | Quantum gravity | The IKKT matrix model relaxed along its equation of motion at **fixed norm**, so the action falls because the matrices come to commute rather than because they shrank. Commuting matrices have a joint spectrum, and that spectrum is the emergent spacetime. | `cargo run -p quantum_examples --example ikkt_matrix_model` |

## Adding New Examples

1. Create directory: `<your_example>/`
2. Add `main.rs` with doc comments (`//!` module docs)
3. Add `README.md` following the [standard template](../physics_examples/README.md)
4. Register in `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example"
   path = "your_example/main.rs"
   ```
