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
| [`quantum_counterfactual`](quantum_counterfactual/README.md) | Quantum computing | Quantum error correction via "time-travel debugging": a `QuantumHistory` of `HilbertState` vectors rides the state channel; syndrome measurement detects a bit-flip and rewinds the qubit to \|0⟩. | `cargo run -p quantum_examples --example quantum_counterfactual` |
| [`quantum_geometric_tensor`](quantum_geometric_tensor/README.md) | Condensed matter | The Quantum Geometric Tensor (QGT), quantum metric, and Berry curvature; geometric (flat-band) transport in Twisted Bilayer Graphene. | `cargo run -p quantum_examples --example quantum_geometric_tensor` |
| [`gauge_electroweak`](gauge_electroweak/README.md) | Quantum field theory | The W boson mass and ρ parameter computed with one-loop quantum corrections (including the top-quark loop). | `cargo run -p quantum_examples --example gauge_electroweak` |
| [`topological_insulator`](topological_insulator/README.md) | Quantum materials | Chern-number calculation for topological phase classification: Berry connection from the overlap of neighbouring electron wavefunctions \|ψ(k)⟩ across the Brillouin zone. | `cargo run -p quantum_examples --example topological_insulator` |
| `hopf_fibration_multivector` | Quantum state geometry | Encodes a qubit state (spinor) in a multivector and projects it onto the Bloch sphere via the Hopf fibration; useful for topological data analysis and quantum-control debugging. | `cargo run -p quantum_examples --example hopf_fibration_multivector` |
| [`ikkt_matrix_model`](ikkt_matrix_model/README.md) | Quantum gravity | The IKKT matrix model: emergent spacetime from matrix dynamics, a non-perturbative formulation of quantum gravity / superstring theory over a non-commutative (quantum) spacetime. | `cargo run -p quantum_examples --example ikkt_matrix_model` |

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
