# QCL-2 chain: a concatenated code

The hand-built `[[4,2,2]]` code concatenated with itself, as two abstractions composed. The inner
code's four physical qubits form two blocks of two, each encoded by the outer code; eight physical
qubits stand under four middle qubits under two logical ones.

```bash
cargo run -p quantum_examples --example qcl_concatenated_code
```

| Link | Low level | High level | `τ` |
|---|---|---|---|
| first | the eight-qubit model: inner encoder, two outer encoders, the outer-encoded program | the inner code's four-qubit model | the outer recovery on each block |
| second | the inner code's four-qubit model | the logical gate on two qubits | the inner recovery |

`Abstraction::compose` pastes the two squares and records, per query, `ε₁`, `ε₂`, `‖τ₁‖_pre`,
`‖τ₂‖_post`, the bound `‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂` and the composite's measured residual. For
`Z̄` and `X̄` both links are exact and the composite is exact, the Rust witness of Lorenz & Tull's
Proposition 17 whose Lean statement is `lean/DeepCausalityFormal/Quantum/Abstraction.lean`. `CZ̄`
pairs a qubit of each block and is refused by name: no transversal gadget between code blocks is
part of this construction. The program runs the chain at `f32`, `f64` and `Float106`.

This is an example with checks. It claims the residuals it measures and the bound the law records.
