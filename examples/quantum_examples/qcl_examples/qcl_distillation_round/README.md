# QCL-2 chain: a distillation round

This example builds the case Lorenz & Tull defer (§7.1) as two abstractions on the hand-built
`[[4,2,2]]` code, and claims only what it measures.

```bash
cargo run -p quantum_examples --example qcl_distillation_round
```

| Link | Low level | High level | `τ` |
|---|---|---|---|
| first | encode, depolarise every physical qubit with probability `p`, run the encoded `T̄ H̄` on both logical qubits | the same model without the noise | the physical identity |
| second | the noiseless code model | `T H` on two qubits | the ideal recovery |

The first link's residual is the noise; the second link is exact. The composite's measured residual
is the part of the noise the ideal recovery does not remove, the part a distillation round exists to
drive down, and the law bounds it by `‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂`. The program sweeps `p` over
`0`, `0.01` and `0.05` and runs the chain at `f32`, `f64` and `Float106`.

This is an example with checks. It claims the residuals it measures and the bound the law records.
