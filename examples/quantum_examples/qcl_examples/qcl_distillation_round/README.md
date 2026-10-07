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

Each probability is a noise world of its own, a context of one `Data` node holding the exact
fraction as a `Rational<i64>` in lowest terms (`model_config.rs`), so `0.05` is held and printed as
`1/20`. The sweep runs the chain once per world, and the cross-precision rows read the `1/20` world
and divide its numerator by its denominator at their own precision, rather than widening an `f64`
approximation.

This is an example with checks. It claims the residuals it measures and the bound the law records.
