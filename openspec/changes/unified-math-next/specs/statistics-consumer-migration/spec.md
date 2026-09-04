<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Migration begins only after the crate's own suite is green

No consumer SHALL be repointed at `deep_causality_stats` until the crate's own suite passes at the repository's coverage requirement.

Migrating call sites against an implementation that is still moving mixes two failure sources: a
broken consumer and a broken library. Sequencing them makes a post-migration failure unambiguously a
migration failure. This is the rule `linear-consumer-migration` already established for the linear
crate, applied here for the same reason.

#### Scenario: Migration is gated
- **WHEN** the first consumer call site is repointed
- **THEN** the crate's own suite is already green at full coverage

#### Scenario: A post-migration failure is attributable
- **WHEN** a consumer fails to build or its tests fail after migration
- **THEN** the crate's suite is known to have been passing beforehand

### Requirement: Each migrated call site keeps its existing numerical semantics

Every migrated call site SHALL select the parameters that reproduce its current behaviour, and any change in its result SHALL be recorded as a deliberate decision rather than absorbed as a side effect.

The three entropy implementations disagree on base, on normalisation and on zero policy. The
parameterised replacement can reproduce any of them, so migration is a choice of parameters, not a
convergence. A caller whose result changes is a caller whose behaviour changed, and that is a
decision about that caller's mathematics — for physics, whose kernel is decided here to move
from nats to bits — not something to be settled by whichever implementation was absorbed first.

The physics change is the one deliberate result change in this stage. It alters a published kernel's
value by a factor of `ln 2`, it brings all three call sites into agreement on what they compute, and
it makes the name accurate. Its one internal caller is its own causal wrapper.

#### Scenario: SURD's results are unchanged
- **WHEN** the SURD entropy and conditional entropy paths run after migration
- **THEN** their outputs are identical to the pre-migration outputs on the existing test corpus

#### Scenario: The CDL variant keeps its normalisation
- **WHEN** the `Option`-carrying SURD variant runs after migration
- **THEN** it still normalises by the sum of its present entries and still returns zero when that sum is below epsilon

#### Scenario: A changed result is a recorded decision
- **WHEN** any migrated call site produces a different value than before
- **THEN** the change is recorded with the reason, and the caller's own tests are updated deliberately rather than to make a failure disappear

#### Scenario: The physics kernel computes in bits after migration
- **WHEN** the physics entropy kernel is migrated
- **THEN** it computes in bits, so a uniform distribution on `n` outcomes yields `log2 n`
- **AND** its name states its base, so the base is readable at the call site
- **AND** its existing tests, which pin the nats result, are changed deliberately alongside it

### Requirement: The two SURD implementations converge on one code path

The `T` and `Option<T>` SURD paths SHALL share one entropy and one conditional-entropy implementation after migration, differing only in how they present their input.

The two files carry near-identical implementations of the same mathematics, differing in their
element type. That duplication is the largest single block this stage retires, and leaving both
pointed at the new crate without collapsing them would keep the divergence risk while adding a
dependency.

#### Scenario: One implementation serves both paths
- **WHEN** the SURD sources are read after migration
- **THEN** both paths reach the same entropy implementation, with the `Option` path supplying its presence policy as a parameter

#### Scenario: The collapse preserves both behaviours
- **WHEN** each path's existing tests run after the collapse
- **THEN** both pass unchanged

### Requirement: Absorbed duplicates are removed, not left in place

Every call site the crate absorbs SHALL have its local implementation removed, and the stage SHALL NOT be complete while a superseded copy remains.

Adding a crate that nothing uses is worse than not adding it: the duplication survives and the
workspace grows a dependency.

The absorbed set is bounded by D7, which keeps `tensor`'s own copies where they are. So it is: two of
the three entropy implementations plus physics's, **three** of the four log-sum-exp copies (the
fourth is `tensor`'s `CausalTensorStatsExt::logsumexp`), **two** of the three Gaussian log-density
sites (the third is `tensor`'s), the two ridge forms, the logistic IRLS, Pearson, the descriptive
statistics on slices, and the two binning routines.

An earlier draft listed all four log-sum-exp and all three Gaussian sites and then required that "no
second implementation remains", which D7 makes unsatisfiable. The carve-out is stated here rather
than discovered at verification time.

Where a call site's bound cannot be met by the new crate — the physics kernel carries a
`MaybeParallel` bound the crate deliberately does not — the local wrapper stays and only its
mathematics is delegated.

#### Scenario: No superseded copy survives outside tensor
- **WHEN** the workspace is searched for the absorbed computations after migration
- **THEN** each resolves to `deep_causality_stats`, except `tensor`'s log-sum-exp and Gaussian log-density, which D7 keeps

#### Scenario: The dense solve the absorbed fits depend on is given a home
- **WHEN** the ridge fits and the logistic gate move to the crate
- **THEN** `brcd_linalg`'s dense LU either moves with them or is recorded as retained, and the choice states whether the crate's solve is LU or Cholesky
- **AND** the reason is recorded, because `brcd_linalg`'s own doc says partial pivoting rather than Cholesky is deliberate for parity with the reference's `numpy.linalg.solve`, and a Cholesky that floors a non-positive pivot drifts the rankings

#### Scenario: A retained wrapper delegates its mathematics
- **WHEN** a call site keeps a local wrapper for a bound the crate does not carry
- **THEN** the wrapper contains no arithmetic of its own beyond the parallelism it exists for

#### Scenario: The dependency edges are added where needed
- **WHEN** a consumer is migrated
- **THEN** its manifest declares `deep_causality_stats` through the workspace table, and its Bazel target lists it

### Requirement: Migration makes the previously f64-internal paths precision-generic

Call sites that computed in `f64` behind a generic signature SHALL compute in their caller's scalar after migration.

Two of the absorbed implementations take a generic parameter and compute internally in `f64`. That is
the stack's precision thesis broken quietly: a caller working at `Float106` receives a result
computed at half its precision, with nothing in the signature to say so. Migration to a
scalar-generic crate fixes it as a consequence, and the suite pins the fix rather than assuming it.

#### Scenario: A wide-precision caller gets wide-precision arithmetic
- **WHEN** a migrated path is called at `Float106` on an input whose exact result is known
- **THEN** the result carries `Float106` accuracy, materially better than the same computation at `f64`

#### Scenario: The narrow paths are unchanged
- **WHEN** a migrated path is called at `f64` after migration
- **THEN** its result matches the pre-migration result to the precision in use
