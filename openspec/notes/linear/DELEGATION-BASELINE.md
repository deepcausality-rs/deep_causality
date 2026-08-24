<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Delegation baseline

Task 2.12. What `CausalTensor`'s decomposition methods return **today**, captured before any body
moved. Phase 5 reduces each to a call into `deep_causality_linear`, and "the returned factors are
unchanged" is checkable only against a record taken first.

Machine: M3 Max, 16 cores, 128 GB. `cargo run` on the workspace at the commit that begins the linear
change.

## Captured output

```
svd(I3)            = Ok(shapes U[3, 3] S[3] Vt[3, 3], S=[1.0, 1.0000000000000002, 1.0000000000000002])
svd(diag 1,3)      = Ok(shapes U[2, 2] S[2] Vt[2, 2], S=[2.99999998257707, 1.0000000006452936])
svd(rank1)         = Ok(shapes U[2, 2] S[2] Vt[2, 2], S=[5.0, 1.1102230246251565e-15])
svd(nonsquare 2x3) = Ok(shapes U[2, 2] S[2] Vt[2, 3], S=[0.0, 0.0])
svd(empty 0x0)     = Ok(shapes U[0, 0] S[0] Vt[0, 0], S=[])
qr(gen2)           = Ok(([2, 2], [2, 2]))
eigen(sym)         = Ok([2.0, 5.0])
eigen(nonsquare)   = Some(ShapeMismatch)
inverse(rank1)     = Some(SingularMatrix)
inverse(I3).is_ok  = true
```

## Three things this corrected in the phase-1 declarations

**`S` is rank-1.** `svd` returns `(U, S, Vᵀ)` where `S` has shape `[k]`, not `[k, k]`. A reading of
the trait signature alone — `Result<(Self, Self, Self), _>` — suggests three matrices, because
`Self` is `CausalTensor` in all three positions and a `CausalTensor` can hold any rank. Running it
settles what the rank actually is. `SvdFactors<T>` is therefore
`(DenseMatrix<T>, DenseVector<T>, DenseMatrix<T>)`.

**The empty matrix is decomposed, not rejected.** `svd` on a `0x0` returns `Ok` with three empty
factors. The delegating method must keep doing that, so `LinearError::EmptyMatrix` is the wrong
answer there.

**The existing SVD is iterative and accurate to about 1e-8, not 1e-15.** `svd(diag(1, 3))` returns
`2.99999998257707` for a singular value that is exactly `3`. Tests written at a `1e-9` tolerance
would fail against the implementation being moved, and tightening the tolerance later is a change to
the numerics rather than to the delegation. The delegation tests assert `1e-6`, and phase 5 diffs
the two implementations against each other rather than against an ideal.

That last one is the reason to capture a baseline rather than assert what the answer ought to be. An
assertion at `1e-9` would have looked like a delegation regression when it was the existing power
iteration's convergence all along.

## Error mapping

| tensor | linear |
|---|---|
| `CausalTensorError::ShapeMismatch` on a non-square eigen | `LinearError::NotSquare` |
| `CausalTensorError::SingularMatrix` | `LinearError::Singular` |
| `CausalTensorError::DimensionMismatch` on a non-2-D input | unreachable here: rank is type-level |

The third row is the point of the dense matrix type. A rank-3 tensor cannot be offered to these
functions at all, so the error variant guarding against it has nothing left to guard.
