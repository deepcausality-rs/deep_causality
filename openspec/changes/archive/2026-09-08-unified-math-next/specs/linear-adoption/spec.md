<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: The Euclidean norm does not overflow for a representable result

`vector_norm_l2` SHALL be computed in the scaled form, so that a vector whose norm is representable returns that norm rather than an infinity.

The current implementation is `vector_norm_sq(v).sqrt()`, a naive sum of `modulus_squared`. A vector
containing a component near the type's maximum squares to an infinity and returns `inf` for a norm
that is perfectly representable; a vector of components near the minimum positive value underflows to
zero.

The crate next door already solved this and documented why. `Normed::modulus` in `num_complex`
factors the larger component out — `max · sqrt(1 + (min/max)²)`, where the ratio is in `[0, 1]` and
cannot overflow — and its doc comment states exactly the failure being avoided. That reasoning
applies unchanged to a vector of any length.

It also needs the non-finite cases guarded explicitly, at both lengths. Left to the arithmetic, a
`NaN` beside a zero sorts into `min` — `NaN > 0` is false — and the zero-maximum guard returns a
finite zero, swallowing it; and two infinities give `∞/∞`, returning `NaN` for a modulus that is
infinite. Both disagree with what summing the squares says, which is the oracle the two members of
`Normed` must agree on.

#### Scenario: A large component does not overflow
- **WHEN** the norm is taken of a vector containing a component near the type's maximum
- **THEN** the result is finite and equals the true norm to the precision in use

#### Scenario: A small component does not underflow
- **WHEN** the norm is taken of a vector whose components are all near the minimum positive value
- **THEN** the result is non-zero and equals the true norm to the precision in use

#### Scenario: The ordinary range is unchanged
- **WHEN** the norm is taken of a vector in the ordinary range
- **THEN** the result matches the previous implementation to the precision in use

#### Scenario: The same guarantee holds at every precision
- **WHEN** the overflow and underflow cases are run at `f32`, `f64`, `Float106` and `BFloat16`
- **THEN** each returns a finite, correct norm at its own extremes

### Requirement: A sparse matrix converts to row-major by walking its stored entries

`CsrMatrix` SHALL override `MatrixView::to_row_major` so that conversion visits each stored entry once, rather than probing every position through `get`.

The defect is not in any one algorithm. `MatrixView::to_row_major` has a default that calls
`get(i, j)` for every position in the shape, and `CsrMatrix::get` answers by scanning that row's
stored range. `CsrMatrix` does not override the default, so conversion costs a scan per position
rather than one pass over the stored entries.

Every dense kernel taking a `MatrixView` goes through that door — `eigen_hermitian` (which is simply
`m.to_row_major()` followed by a symmetric eigensolve), `qr`, `svd` and `cholesky`. One override fixes
the class. `matrix_norm_frobenius` is **not** in it: it reads entries through `get` and never asks for
a row-major buffer, so the override does not reach it and it was listed here in error.

An earlier draft of this change specified this as a flaw in `eigen_hermitian`. That was the wrong
place: the eigensolver does nothing but ask for a row-major copy.

**Latent, and fixed anyway.** No caller passes a `CsrMatrix` to any of those algorithms today. The
reasoning is the same one the Meek stage used for R4: the type is general-purpose, its callers are
not fixed, and a bound is not a proof.

#### Scenario: Conversion walks the stored entries
- **WHEN** a `CsrMatrix` is converted to row-major
- **THEN** the cost is proportional to its stored entries and its shape, not to a scan per position

#### Scenario: The result is unchanged
- **WHEN** the same sparse matrix is converted before and after the override
- **THEN** the row-major buffer is identical, including the zeros at unstored positions

#### Scenario: Every MatrixView algorithm benefits
- **WHEN** a `CsrMatrix` is offered to `eigen_hermitian`, `qr`, `svd` or `cholesky`
- **THEN** each reaches its dense path through the override

#### Scenario: The latency is recorded, not overstated
- **WHEN** the stage's notes are read
- **THEN** they record that no caller passes a sparse matrix to these algorithms today

### Requirement: Each hand-rolled site is classified before it is replaced

Every hand-rolled linear-algebra site SHALL be classified as replace, move-in, replace-with-care, or keep, with the reason recorded, and only the first two classes SHALL be actioned without a further decision.

`deep_causality_linear` landed after most of its consumers were written, so the hand-rolled code is
not carelessness — it is what existed at the time. That also means a blanket replacement is wrong.
Four cases genuinely differ:

- **Replace.** A duplicate of something the crate does at least as well: the open-coded complex
  modulus and multiplication, the five-copy residual idiom, the CSR matrix-vector products.
- **Move-in.** Linear algebra the crate does *not* have. The mandate is two-way: a site like this is
  moved into `deep_causality_linear` and the call site dispatches to it, so the operation ends up
  defined once. The closed-form symmetric 3×3 eigensolver, the written-out 3×3 determinant and
  double-dot, the slice inner product and the mixed-scalar `CsrMatrix<i8>` matvec are all this case.
- **Replace-with-care.** A change in performance or in behaviour. A fixed-size stack-allocated
  17×17 filter kit moved onto a heap matrix type may be slower, and that must be measured rather
  than assumed. A CSR matrix-vector product that silently skips out-of-range columns, replaced by one
  that returns an error, is a behaviour change on a shipped path.
- **Keep.** Not linear algebra, or the same arithmetic the crate already reaches another way. This is
  a small class. "The hand-rolled version is right and the crate has nothing like it" belongs under
  **move-in**, not here — an earlier draft placed the 3×3 eigensolver and the written-out 3×3
  products under keep on exactly that reasoning, and both are move-in.

#### Scenario: Every site carries a classification
- **WHEN** the inventory is reviewed
- **THEN** each site names its class and the reason, and no site is unclassified

#### Scenario: A performance-sensitive replacement is measured
- **WHEN** a site classified replace-with-care on performance grounds is replaced
- **THEN** a benchmark before and after is recorded, and a regression reverts the replacement

#### Scenario: A kept site records why it is kept
- **WHEN** a site is classified keep
- **THEN** its reason is recorded in a comment at the site, so the next reader does not re-litigate it

### Requirement: A behaviour change in a numerical kernel is stated, not absorbed

Replacing a hand-rolled operation whose error behaviour differs from the crate's SHALL be recorded as a behaviour change with its own test, and SHALL NOT be presented as a refactor.

The clearest instance is the ideal-MHD matrix-vector product, which silently skips columns outside
its range where `CsrMatrix::vec_mult` returns an error. Adopting the crate's version turns a silent
wrong answer into a typed failure. That is the right direction, and it is still a change in what a
shipped solver does when it meets malformed input.

#### Scenario: The changed path is tested at its old behaviour
- **WHEN** the input that previously triggered the silent skip is offered after replacement
- **THEN** a typed error is returned, and a test pins that outcome

#### Scenario: The change is recorded
- **WHEN** the stage's notes are read
- **THEN** each behaviour change names the site, the old behaviour, the new behaviour and the reason

### Requirement: Quantum's complex arithmetic goes through the complex tower

`deep_causality_quantum` SHALL use `deep_causality_num_complex`'s operators and `Normed::modulus` rather than open-coding complex arithmetic on the real and imaginary components.

The crate depends on `num_complex` already and open-codes complex multiplication, conjugation, trace
accumulation and modulus across ten files. The entrywise max-modulus residual idiom appears five
times and collapses to one helper.

One correction to an earlier draft narrows this: `QubitOperator::unitarity_defect` already uses
`Normed::modulus`, with a doc comment giving exactly the overflow reason — so the crate is not
unaware of the scaled form, and one site is already migrated.

A second correction *widens* it, and reverses what an earlier draft of this requirement said. That
draft excluded the Frobenius norm, on the ground that delegating buys nothing because
`modulus_squared` is `re² + im²`, the same direct form quantum already folds over a slice, and only
`modulus` is scaled. That was true of `deep_causality_linear` as it stood when the draft was written.
It stopped being true when `vector_norm_l2` and `matrix_norm_frobenius` were both put into the scaled
form. The Frobenius norm is the two-norm of the entries read as one vector — an identity the crate
asserts in its own suite — so delegating it now removes an overflow rather than buying nothing.

`frobenius_norm` folds `re² + im²` and reaches infinity above about `1.34e154`. The place that
changes an answer is `markov_pairs`, where the norm of a caller-supplied factor feeds
`CommutatorTolerance::threshold` unguarded, sending the threshold to infinity with it. Delegating
fixes that at its source, which is better than guarding one consumer of a norm that overflows.

The delegation costs quantum nothing. The norms were bounded on `NormedScalar`, which adds `Field`,
`FromPrimitive` and `Copy` on top of `Normed` — none of which any norm body calls for. Relaxing them
to `Normed` is a weakening, so no existing caller breaks, and it keeps the delegation from forcing a
`FromPrimitive` bound onto `frobenius_norm`: `Complex<T>: Normed` holds for every `T: RealField`,
where `Complex<T>: FromPrimitive` does not.

#### Scenario: The residual idiom exists once
- **WHEN** the quantum sources are searched for the entrywise max-modulus residual
- **THEN** one implementation is found and the five call sites reach it

#### Scenario: The Frobenius norm delegates to the scaled form
- **WHEN** quantum's `frobenius_norm` is read after this change
- **THEN** it calls `deep_causality_linear`'s scaled Euclidean norm over the operator's entries rather than folding `re² + im²` itself

#### Scenario: The delegation does not widen a public bound
- **WHEN** `frobenius_norm`'s signature is read after the delegation
- **THEN** it still asks only for `R: RealField`, because the crate's norms are bounded on `Normed` rather than on `NormedScalar`

#### Scenario: Modulus results are unchanged in the ordinary range
- **WHEN** the migrated sites run against their existing tests
- **THEN** every result matches to the precision in use

#### Scenario: The threshold path survives a large factor
- **WHEN** `markov_pairs` is given a factor whose entries exceed the square root of the type's maximum
- **THEN** the Frobenius norm is finite and the threshold derived from it is finite

### Requirement: The three reachability pre-passes in the causality engine collapse onto one

`deep_causality` SHALL compute a backward reachable set through one implementation, and SHALL NOT retain three open-coded copies.

The engine open-codes transitive reachability in three places, of which the inclusive backward
reachable set is the expensive one. Collapsing them is small, has a caller today, and removes the
divergence risk that three copies of a traversal carry.

Where the underlying graph engine returns a not-frozen error for a dynamic graph, the collapsed
implementation preserves that behaviour rather than papering over it.

#### Scenario: One implementation serves all three sites
- **WHEN** the engine's sources are searched for a reachability traversal
- **THEN** one implementation is found and the three former sites call it

#### Scenario: The collapse preserves each site's behaviour
- **WHEN** each site's existing tests run after the collapse
- **THEN** they pass unchanged, including the not-frozen error path

#### Scenario: An out-of-range vertex behaves as before
- **WHEN** a traversal is started from a vertex outside the graph
- **THEN** the result matches the pre-collapse behaviour rather than a newly introduced error

### Requirement: An addition to a published error enum or trait is treated as breaking

Adding a variant to `LinearErrorEnum` or a method to `ultragraph`'s pathfinding trait SHALL be recorded as a breaking change with its dependents enumerated.

Neither is `#[non_exhaustive]`, so a downstream `match` on the enum breaks on a new variant, and the
pathfinding trait has two implementors inside this workspace and is public API of a published crate.
Both are ordinary changes to make; neither is a change to make silently.

#### Scenario: The break is recorded before the addition lands
- **WHEN** a variant or a method is added
- **THEN** the stage's notes name it as breaking and list the affected implementors and matchers

#### Scenario: Every implementor is updated in the same change
- **WHEN** a trait method is added
- **THEN** both implementors are updated, and the workspace builds
