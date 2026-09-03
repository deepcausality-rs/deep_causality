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
- **WHEN** the overflow and underflow cases are run at `f32`, `f64` and `Float106`
- **THEN** each returns a finite, correct norm at its own extremes

### Requirement: The Hermitian eigendecomposition does not silently densify a sparse matrix

`eigen_hermitian` SHALL either avoid densifying a sparse input or state in its documentation that it does so and at what cost, and SHALL NOT convert through the generic element accessor.

The dense cyclic Jacobi path reaches a `CsrMatrix` through the `MatrixView` default, which has no
row-major override, so densification runs the element accessor `n²` times — each call a search
within a compressed row. A caller passing a sparse matrix to a function that accepts one has no
signal that the cost is quadratic in accessor calls rather than linear in stored entries.

The minimum acceptable outcome is that the cost is stated. The better outcome is a row-major
conversion that walks the stored entries once.

#### Scenario: Densification walks the stored entries
- **WHEN** a `CsrMatrix` is offered to the eigendecomposition
- **THEN** conversion visits each stored entry once rather than probing every position

#### Scenario: The result is unchanged by the conversion path
- **WHEN** the same matrix is decomposed in sparse and dense form
- **THEN** the eigenvalues agree to the precision in use

#### Scenario: The cost is documented where it remains
- **WHEN** the function's documentation is read
- **THEN** it states the conversion behaviour and its cost for a sparse input

### Requirement: Each hand-rolled site is classified before it is replaced

Every hand-rolled linear-algebra site SHALL be classified as replace, replace-with-care, or keep, with the reason recorded, and only the first class SHALL be replaced without a further decision.

`deep_causality_linear` landed after most of its consumers were written, so the hand-rolled code is
not carelessness — it is what existed at the time. That also means a blanket replacement is wrong.
Three cases genuinely differ:

- **Replace.** A duplicate of something the crate does at least as well: the Frobenius norm in
  quantum, the open-coded complex modulus, the cofactor inverses in general relativity where a
  general path is no slower.
- **Replace-with-care.** A change in performance or in behaviour. A fixed-size stack-allocated
  17×17 filter kit moved onto a heap matrix type may be slower, and that must be measured rather
  than assumed. A CSR matrix-vector product that silently skips out-of-range columns, replaced by one
  that returns an error, is a behaviour change on a shipped path.
- **Keep.** The hand-rolled version is right. A closed-form symmetric 3×3 eigensolver is faster than
  a general Hermitian decomposition, and a 3×3 product written out beats a general one.

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
accumulation and modulus across ten files. The modulus is the one with consequences: every
hand-rolled `(dr*dr + di*di).sqrt()` uses the direct form, while the crate's `Normed::modulus` uses
the scaled form for the reason its doc comment states.

The entrywise max-modulus residual idiom appears five times and collapses to one helper once the
modulus is shared.

#### Scenario: The residual idiom exists once
- **WHEN** the quantum sources are searched for the entrywise max-modulus residual
- **THEN** one implementation is found and the five call sites reach it

#### Scenario: The Frobenius norm is not duplicated
- **WHEN** quantum's Frobenius norm is read
- **THEN** it delegates to `deep_causality_linear`'s implementation through the existing `MatrixView` bridge

#### Scenario: Modulus results are unchanged in the ordinary range
- **WHEN** the migrated sites run against their existing tests
- **THEN** every result matches to the precision in use

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
