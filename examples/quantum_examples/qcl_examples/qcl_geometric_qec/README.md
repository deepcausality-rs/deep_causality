# QCL code path: the toric code, verified exactly

`validate` takes a chain complex. On the `4 × 4` square torus, Kitaev's `[[32, 2]]` toric code, it
runs four exact checks and none of them simulates anything.

```bash
cargo run -p quantum_examples --example qcl_geometric_qec
```

| Stage | What it decides | How |
|---|---|---|
| `derive_code` | `n = 32` from the 1-cells, `k = 2` from `β₁` over 𝔽₂, 16 Z checks of weight 4, 16 X checks of weight 4 | the shipped counts and columns |
| `check_ldpc_weights` | both weights of both check matrices against a bound of 4; a bound of 3 rejects with margin `4/3` naming the first offending check | one record per row and column |
| `check_class_invariance` | `Z̄`, `S̄` and `T̄` on each of the two classes act on the class, not the representative | Haruna Eq. (3.20), decided over the code space with an exact rational phase |
| `check_clifford_action` | `H̄` on each class swaps `Z̄(γ)` and `X̄(γ̃)` | a symplectic tableau over 𝔽₂ |

The in-process simulator caps at 24 qubits and this code has 32. The checks do not need it: every
verdict is a computation over supports, bounded by the weights of the chains rather than by the
register, which is what lets a code whose Hilbert space no simulator holds be decided at all.
