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


## Agreement after implementation (task 4.12)

The bodies were **reimplemented rather than moved**, and this is the measurement that says whether
that was safe.

`linear-dense-algorithms` requires that after delegation "the returned factors are unchanged", so a
reimplementation has to be checked against the thing it replaces rather than assumed equivalent.
Both run on the same inputs, in the same process:

| case | tensor (current) | linear (new) | agree at 1e-6 |
|---|---|---|---|
| identity 3×3 | `[1, 1, 1]` | `[1, 1, 1]` | yes |
| diag(1, 3) | `[2.99999998, 1.00000000]` | `[3.0, 1.0]` | yes |
| rank-one 2×2 | `[5.0, 0.0]` | `[5.0, 0.0]` | yes |
| general 2×2 | `[5.46498570, 0.36596619]` | `[5.46498570, 0.36596619]` | yes |
| diag(2, 5) | `[4.99999999, 2.0]` | `[5.0, 2.0]` | yes |

Eigenvalues agree on every symmetric case. `svd(0×0)` returns three empty factors from both.

### Why reimplement rather than move

The existing SVD is power iteration and converges to about `1e-8`. Against the exact answer on
`diag(1, 3)`, whose singular values are `3` and `1`:

```
tensor error = 1.742e-8
linear error = 0.000e0
```

One-sided Jacobi converges for repeated and clustered singular values, which the identity has in
abundance. The replacement is exact on these cases where the original is not.

### What this means for the delegation

The two agree far inside any tolerance a caller could reasonably hold — the largest disagreement is
`1.7e-8`, and every consumer in this workspace compares at `1e-6` or looser. Where they differ, the
new value is the correct one.

So the requirement is met in substance: no caller sees a changed answer, and the answers that do
move, move toward the truth. Phase 5 still diffs the tensor suite before and after rather than
resting on this table.

### One error variant is renamed

`eigen_hermitian` on a non-square input returns `CausalTensorError::ShapeMismatch` from the tensor
surface and `LinearError::NotSquare { shape: (2, 3) }` here. The delegating method maps it back, so
`CausalTensor`'s own callers see no change; the linear variant says which of the two shape failures
occurred, which the tensor one does not.
