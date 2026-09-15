# QCL-2 chain: code switching through a gadget

Two logical qubits are encoded into the hand-built `[[4,2,2]]` code, decoded, and encoded into the
`[[8,2,2]]` toric code, which then runs `Z̄`. The decode-and-re-encode is the switching gadget.

```bash
cargo run -p quantum_examples --example qcl_code_switching
```

| Link | Low level | High level | `τ` |
|---|---|---|---|
| first | encode A, decode A, an optional depolarising channel on one logical wire, encode B, run `Z̄_B` | code B's model: encode B, run `Z̄_B` | the physical identity on eight qubits |
| second | code B's model | `Z̄` on two logical qubits | code B's ideal recovery |

With a noiseless gadget both links are exact and so is the composite. With depolarising noise of
probability `0.1` inside the gadget, the first link's residual is the noise, the second link's
stays zero, and the composite's measured residual lies under the recorded bound
`‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂`, whose first constant is the Frobenius-induced norm of code B's
recovery. The program runs the chain at `f32`, `f64` and `Float106`.

This is an example with checks. It claims the residuals it measures and the bound the law records.
