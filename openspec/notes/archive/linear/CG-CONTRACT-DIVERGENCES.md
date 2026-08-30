<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The CG contract diverged, and phase 5 is what exposed it

Found while scouting task 5.8, before any consumer was repointed.

| | divergence | status |
|---|---|---|
| A | `tolerance` was absolute where the sparse crate scales it by `‖b‖` | **fixed** |
| B | the curvature guard rejects any non-positive `pᵀAp`, not only an exact zero | kept, documented |
| C | the operator's returned length went unchecked, and a short one panicked | **fixed** |
| D | `CgFailure` is an enum where the sparse crate had a struct | kept, one consumer repointed |

`deep_causality_linear/src/algorithms/cg.rs:8` opened with:

> Moves here from `deep_causality_sparse` with its signatures, **convergence behaviour** and
> iteration counts unchanged.

The signatures were unchanged. The convergence behaviour was not.

## Where the record went wrong

`PORTING-FINDINGS.md` lists seven divergences and says the CG signature was "restored to the original
signature and semantics". The semantics were not restored. `ported_cg_tests.rs` carried its own list
of four, of which two had already been fixed without the list being updated:

| item | the header claimed | actually |
|---|---|---|
| 1. argument order | diverges | stale — restored, header not updated |
| 2. reciprocal vs diagonal | diverges | stale — restored, header not updated |
| 3. tolerance is absolute | diverges | **live, unrestored** |
| 4. `CgFailure` is an enum | diverges | live, deliberate |

Two documents describing one seam, neither current, and the live numeric divergence sitting in a list
where half the entries were already obsolete — so the entry that still mattered read like the ones
that did not.

## A — the tolerance was absolute where it was relative

**Was blocking. Fixed.**

```
sparse  (solver/cg.rs:70)       abs_tol = if ‖b‖ == 0 { tolerance } else { tolerance * ‖b‖ }
linear  (algorithms/cg.rs:166)  if residual <= tolerance
```

`deep_causality_sparse` documents `tolerance` as a "relative residual threshold" (`cg.rs:45`).
`deep_causality_topology` documents its default as a "tight **relative** residual (1e-10)"
(`hodge_decomposition_impl.rs:391`). Both describe the sparse behaviour, and the linear crate did not
implement it.

Repointing a caller would have changed the convergence criterion by a factor of `‖b‖`.

### Measured, before the fix

Both crates on the same operator — `tridiagonal(-1, 2, -1)` at `n = 400`, SPD with condition number
growing like `n²` — with `tolerance = 1e-10` and `‖b‖ = 8.9336e7`:

```
  sparse threshold (relative) = 8.9336e-3
  linear threshold (absolute) = 1.0000e-10

 maxit        sparse        linear
   400     CONVERGED        failed
   600     CONVERGED        failed
   800     CONVERGED     CONVERGED
  1000     CONVERGED     CONVERGED
```

A caller with a fixed iteration budget in the 400–600 window succeeded before repointing and would
have returned `NotConverged` after. Where both converged the answers agreed to 9.7e-15 relative — the
divergence was entirely in *when they stop*, never in what they compute, which is what made it
invisible to a test that checks the answer.

Nothing caught it because every CG test in both suites uses `‖b‖ ≈ 1` with `tolerance` at 1e-12 or
below, where relative and absolute agree well inside the assertion margins.

### Measured, after

Same sweep: the convergence decision agrees at every budget and `max|sparse − linear| = 0`.

Pinned by `cg_tests.rs::test_the_tolerance_is_relative_to_the_norm_of_the_right_hand_side`, which
isolates the threshold by asking for zero iterations — the only comparison left is `‖b‖` against the
threshold itself. Removing the scaling fails it.

## B — the curvature guard is stricter

**Kept. The code differs; no case was found where it matters.**

```
sparse  (cg.rs:99)   if pap == R::zero()          -> failure
linear  (cg.rs:171)  if denominator <= R::zero()  -> NotPositiveDefinite
```

The sparse crate documents the exact-zero test as deliberate (`cg.rs:93-98`): a threshold "would
either fire spuriously on the very small `pap` values that arise at Float106 / f32 precision near
convergence, or miss true breakdowns at large magnitudes."

The concern was that the Hodge Laplacian is positive **semi**-definite — it has a kernel, which is
why `subtract_mean_in_place` exists in `topology/src/utils/cg_solver.rs` — so a small negative `pᵀAp`
from rounding near convergence would stop the replacement where the original continued.

### Measured — no divergence found

The singular path-graph Laplacian `L = D − A`, PSD with the constants in its kernel, at
`n ∈ {50, 200, 800}` with a consistent (mean-zero) right-hand side and a `5n` iteration budget: both
crates converge in every case. The negative `pᵀAp` did not arise.

Kept as written. A negative `pᵀAp` means the operator is not positive definite, and dividing by it
steps in the wrong direction; rejecting it is worth the documented difference.

## C — an operator returning the wrong length panicked instead of failing

**Was blocking. Fixed.** Worse than reading the code suggested.

The sparse solver checked `ap.len() != n` on every iteration (`cg.rs:86`) and returned a
`CgFailure`. The linear solver checked neither application. At `cg.rs:159`:

```rust
let mut r: Vec<R> = b.iter().zip(&ax).map(|(bb, a)| *bb - *a).collect();
```

`zip` stops at the shorter side, so `r` came out short — and the truncation did not stay contained,
because the loop then indexes `r` and `ap` by `i` in `0..n`, where `n` came from `b`.

### Measured, before the fix

Same operator, wrapped to return a vector one element short:

```
  sparse: Err(CgFailure { iterations: 0, residual: 3.7416573867739413 })
  linear: panicked at deep_causality_linear/src/algorithms/cg.rs:177:
          index out of bounds: the len is 1 but the index is 1
```

A typed error became a panic — the exact failure mode the crate's error-handling rule exists to
prevent. The `LengthMismatch` variant that should have carried it already existed, guarding `initial`
(`cg.rs:124`) and `diag_a` (`cg.rs:131`): the two arguments the caller passes directly, never the one
the operator produces.

This was reachable without repointing anything. It was a live defect, not a migration risk.

### The tests named this case and asserted the wrong thing

Four tests in `ported_cg_tests.rs` were called `..._rejects_a_wrong_length_operator`. Each asserted
`NotPositiveDefinite` — the curvature guard's incidental rejection of the truncated system, not a
length check — and each used a *longer* vector, the direction that truncates without panicking. None
used a shorter one.

The test bodies even documented it: "the sparse solver compared the returned length against `b.len()`
and reported the mismatch; this one truncates against `b` and the rejection arrives from the
curvature guard on the first iteration instead." A divergence written down, accepted, and named after
the behaviour it did not have.

All four now assert `LengthMismatch` with its fields, and cover both directions.

## D — `CgFailure` changed from a struct to an enum

**Kept, and the one consumer is repointed by hand.**

```
sparse  pub struct CgFailure<R: RealField> { pub iterations: usize, pub residual: R }
linear  pub enum CgFailure<R> { NotConverged {..}, NotPositiveDefinite {..}, LengthMismatch {..} }
```

The enum is the better type — three failure modes that need different responses should not share one
shape. It is also the only one of the four that a re-export shim cannot hide, and it breaks one
consumer. `topology/src/types/manifold/differential/hodge_decomposition_impl.rs:363`:

```rust
Err(CgFailure { iterations, residual }) => Err(HodgeFailReason::Nonconvergence { iterations, residual })
```

Under the enum this stops compiling — which is the good case. It is the only divergence of the four
that could not pass silently, and it is why task 5.6's "matches the last independent release item for
item" has one stated exception rather than none.

## What was done

A and C closed in `../../../../deep_causality_linear/src/algorithms/cg.rs`, both verified by injection: reverting
A fails one test, reverting C fails four. Verified against the crate being replaced by running both
on the same operator, where the convergence decision now agrees at every iteration budget.

B and D stand, both documented in the module header rather than left for the next reader to
rediscover.

The stale headers in `cg.rs` and `ported_cg_tests.rs` are corrected. Both described a state that no
longer held.

## The lesson

A record of known divergences is only load-bearing if closing one updates it. Half of this list was
obsolete, which is what let the live entry hide — and the defect that was never on the list at all
was sitting behind four tests that named it and checked something else.
