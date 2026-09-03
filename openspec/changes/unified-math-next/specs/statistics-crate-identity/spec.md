<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: The statistics crate sits at tier 4 over num, algebra and linear

`deep_causality_stats` SHALL be a workspace member at `deep_causality_unified_math/deep_causality_stats`, depending on `deep_causality_num`, `deep_causality_algebra` and `deep_causality_linear`, and SHALL NOT depend on `deep_causality_rand`, `deep_causality_tensor` or `deep_causality_par`.

Tier 4 is forced, not chosen. `deep_causality_linear` is itself tier 3 and a crate depends only on
crates in lower tiers, so a crate over `linear` is tier 4 at the earliest. The assessment's "tier 3"
is arithmetically impossible.

`rand` is excluded because none of the functions with a consumer uses randomness — descriptive
statistics, Pearson, ridge, logistic IRLS, log-sum-exp, entropy, conditional entropy, Gaussian
log-density and binning are all deterministic. Declaring an unused dependency would also fail the
repository's unused-dependency check.

`tensor` is excluded to keep the crate below it, so `tensor` may later delegate to it without a
cycle. The consequence is that this crate's surface is over slices, and tensor-shaped wrappers stay
where they are.

`par` is excluded because the crate ships no parallel path. The one absorbed function that carries a
`MaybeParallel` bound today keeps its parallel wrapper in its own crate.

#### Scenario: The tier is respected
- **WHEN** the workspace dependency graph is computed
- **THEN** `deep_causality_stats` depends only on crates at tier 3 or below, and no cycle exists

#### Scenario: The excluded dependencies are absent
- **WHEN** the crate's manifest is read
- **THEN** it names neither `deep_causality_rand`, `deep_causality_tensor` nor `deep_causality_par`

#### Scenario: The unused-dependency check passes
- **WHEN** the repository's dependency check runs
- **THEN** it reports no unused dependency for this crate

### Requirement: The crate is registered in every place a crate must be registered

The crate SHALL carry its own manifest, Bazel build file, workspace lint opt-in and README, and SHALL be added to the root dependency table, `AGENTS.md`'s tier block and the unified-math README's crate table and tier diagram.

The workspace glob picks up the directory, which is the easy half. The registrations outside the
crate are the half that fails silently: a crate absent from the root dependency table cannot be
named by a consumer with `workspace = true`, and a crate absent from the documentation tier blocks is
invisible to the next reader deciding where code belongs.

The root dependency table entry uses two-digit version precision, so a local patch release is picked
up without editing the table.

#### Scenario: The crate builds under both systems
- **WHEN** `cargo build -p deep_causality_stats` and `bazel build //deep_causality_unified_math/deep_causality_stats:all` run
- **THEN** both succeed

#### Scenario: The lint opt-in is present
- **WHEN** the manifest is read
- **THEN** it carries `[lints] workspace = true`, so the repository-wide `unsafe_code = "forbid"` applies

#### Scenario: The documentation records the new tier
- **WHEN** `AGENTS.md`'s tier block and the unified-math README's crate table are read after this change
- **THEN** both list the crate at tier 4 with its dependency set

#### Scenario: A consumer can name it through the workspace
- **WHEN** a consumer declares `deep_causality_stats = { workspace = true }`
- **THEN** it resolves at the two-digit constraint recorded in the root table

### Requirement: The crate is generic in its scalar and names no concrete float

Every public function SHALL be generic over the scalar under the algebra tower's bounds, and the crate SHALL NOT name `f32`, `f64` or `Float106` in any public signature.

The stack's thesis is that precision is a parameter, and a statistics crate that computes in `f64`
behind a generic signature — which is what two of the implementations being absorbed do today —
silently discards it. The bound is whatever the function's mathematics actually needs: `Real` for the
analytic operations, `RealField` where division is required, plus `FromPrimitive` where a literal or
a count crosses into the working type.

Where a count must reach the real axis, it crosses through `deep_causality_num`'s lift module rather
than through an ad-hoc conversion, and no function carries a local lift helper.

#### Scenario: The same computation runs at three precisions
- **WHEN** any public function is called at `f32`, at `f64` and at `Float106`
- **THEN** each compiles and returns a result in its own precision

#### Scenario: No concrete float appears in the surface
- **WHEN** the public signatures are enumerated
- **THEN** none names a concrete float type

#### Scenario: Precision is not silently narrowed
- **WHEN** a function is called at `Float106`
- **THEN** no intermediate is computed at `f64`, and the result carries the wider precision

### Requirement: The crate's scope excludes functions with no consumer

The crate SHALL implement only functions with a caller in this workspace at the time of writing, and SHALL NOT implement cross-entropy, mutual information, Kullback–Leibler divergence, Jensen–Shannon divergence, Hellinger distance or the Bhattacharyya coefficient.

The assessment's function list mixes two kinds of entry. Some replace code that exists several times
over. Others were proposed because a statistics crate conventionally has them, or because a future
consumer might: nine of the roughly thirty listed have no library caller today.

`AGENTS.md` is explicit that configurability, extensibility and generalisation are not added unless
requested, and a new crate is the easiest place in a repository to violate that rule at scale. The
excluded functions are excluded on that ground alone; each is straightforward to add when something
calls it.

Mutual information is the one that most invites an exception, because SURD computes information
quantities. It stays out because what SURD computes is specific mutual information and information
leak over marginalised tensor axes, which is not the slice-shaped general function, and building the
general one would leave SURD's version in place beside it.

#### Scenario: The excluded functions are absent
- **WHEN** the crate's public surface is enumerated
- **THEN** it contains none of the six named functions

#### Scenario: Each shipped function names its caller
- **WHEN** the crate's function list is reviewed
- **THEN** each entry names at least one call site in this workspace that it replaces or serves

#### Scenario: A later addition is justified by a caller
- **WHEN** a function is proposed for the crate after this change
- **THEN** it is accepted only with a named consumer
