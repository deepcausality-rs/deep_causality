<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Renovate the uncertain crate

## Why

`deep_causality_uncertain` is the last crate in unified math that has not taken precision as a
parameter, and the reason it has not is a global `static`. `SampledValue` carries precision as a
variant — `Float(f64) | DoubleFloat(Float106) | Bool(bool)` — and its own docstring says why: so
that "the computation graph, the global sample cache (a `static`), and the sampler all stay
non-generic". Only one clause of that is a Rust constraint. The other two follow from choosing a
global cache, and that choice predates unified math.

The cost is measured, not argued:

- **The crate names three scalars and serves three scalars.** 72 `SampledValue` variant arms across
  `src/`, 61 of them inside the two samplers. `DistributionEnum<f64>::sample` and
  `DistributionEnum<Float106>::sample` have byte-identical bodies. `f32` and `BFloat16` cannot be
  used at all.
- **It pins a downstream generic crate.** `deep_causality_cfd` bounds its uncertain march on
  `ProbabilisticType` at sixteen bound occurrences across eight files, so a CFD run at `f32` with
  uncertain inflow does not compile — at the one crate that models uncertainty.
- **The cache does not do what its shape suggests.** It is consulted only at the root
  (`types/sampler/` contains zero references to it); within a draw the samplers memoize by node
  identity in a per-call map, and that map is what keeps `x + x` on one draw.
- **It leaks by design.** `sample()` draws a random `u64` index, `sample_with_index` inserts one
  entry under it, and nothing in `src/` ever calls `clear`. `expected_value` over ten thousand
  samples adds ten thousand entries for the life of the process.
- **The shipped branch is the untested one.** Under `cfg(test)` the `OnceLock` static becomes a
  `thread_local!`, so the global that ships is the one branch the suite never executes.
- **The globals are visible in the test suite.** 24 `rusty_fork_test!` invocations across 20 files,
  and a whole dev-dependency on `rusty-fork`, exist to isolate state the crate should not have.
- **The suite is flaky today.** Five tests failed across ~360 runs, and 41 sampled-decision call
  sites were exposed with not one of them seeded. They gate unseeded sequential probability ratio
  tests — p = 0.9 against a 0.8 threshold on a 100-sample budget — against the OS entropy source.

Separately, the crate carries four HKT node arms — `PureOp`, `FmapOp`, `ApplyOp`, `BindOp` — that no
public constructor produces. They were an attempt at categorical composition that could not be
finished, and they are four of the crate's six `Arc<dyn ...>` sites, which AGENTS.md forbids in
library code.

The `sampling-hkt-composition` capability already named this work: "The crate is next in line for
its own renovation ... `SampledValue`, the global sample cache and the lazy graph belong to that
renovation." This is that renovation.

## What Changes

**Goal 1 — modernize: the globals go.**

- **BREAKING** `GlobalSampleCache`, `with_global_cache`, `SamplerKind`, `seed_sampler` and
  `clear_sampler_seed` leave the public API.
- A `SampleSession` value the caller owns replaces them: it holds the seed, the sample counter
  and the mode. `SampleSession::seeded(seed)` and `SampleSession::qmc(seed)` replace the two seed
  functions. It carries no scalar parameter, so one session can drive draws at several scalars and
  correlate them by index.
- Every leaf draw becomes a pure function of (session seed, sample index, leaf ordinal). The ordinal
  is assigned by one deterministic pre-pass, which is exactly what the QMC sampler already does for
  Sobol dimensions. Reproducibility stops being a cache property and becomes a construction
  property — and it becomes stronger: the same index gives the same draw for every tree sharing a
  leaf, not only for the same root.
- `uncertain` owns no global state afterwards. `rusty-fork` leaves the dev-dependencies and the 24
  fork-isolated tests become ordinary tests.
- The four dead HKT arms and both `FunctionOp` arms stop using `Arc<dyn ...>`; the crate reaches
  zero `dyn` sites.

**Goal 2 — precision as a parameter.**

- **BREAKING** `SampledValue`, `ProbabilisticType`, `IntoSampledValue`, `FromSampledValue` and
  `UncertainReal` leave the public API.
- The computation graph becomes generic: `ConstTree<Node<R>>` with `Sample<R> { Real(R), Bool(bool) }`
  and one `Distribution(DistributionEnum<R>)` arm in place of three. The scalar bound is
  `R: RandScalar` — blanket-implemented in `deep_causality_rand`, re-exported by
  `deep_causality_stats`.
- **Zero concrete scalars named in `src/`.** The acceptance test is instantiation at a scalar the
  crate never mentions.
- **BREAKING** Dimensionless probabilities take the caller's scalar: `bernoulli(p: R)`, the
  comparison thresholds, `probability_exceeds`, `estimate_probability -> R`, and both
  `MaybeUncertain` probabilities. `f64` survives only at the display boundary.
- `MaybeUncertain<R>` keeps its name and its twelve CFD call sites, and is reimplemented over the
  generic tree rather than as four per-type files.
- `deep_causality_cfd` drops `+ ProbabilisticType` from sixteen bound occurrences and compiles at `f32`.

**Goal 3 — categorical composition.**

- A materialised ensemble is written into an **existing witnessed container**, and `materialize` is
  generic over the witness, so the same call produces a `DenseVector<R>` or a rank-1
  `CausalTensor<R>` at the caller's choice.
- This needs one capability `deep_causality_haft` does not have: building a rank-1 container of a
  witness from a sequence. All sixty-odd haft traits were checked; `Foldable` consumes and nothing
  produces. The new trait is the counterpart to `Foldable` and lives beside it, implemented by
  `linear` and `tensor` — so `uncertain` gains a dependency on `haft` alone and stays at tier 5.
- Composition after materialisation uses what already exists: `ZipDenseVectorWitness` /
  `ZipTensorWitness` for correlation by index, and `DiagonalTraversable::sequence_zip` for turning a
  structure of ensembles inside out. The cartesian `Traversable::sequence` is not used for sampling.
- The lazy graph gets haft's `Arrow` instance — `run`, `compose`, `first`, `split` — which is
  possible precisely because goal 1 makes evaluation at an index a pure function. The container
  traits still get no witness, for the reason already ratified: they carry no `'static` and a stored
  closure cannot satisfy them.

## Capabilities

### New Capabilities

- `uncertain-sample-session`: the caller-owned session, index-addressed leaf draws, the removal of
  all five globals, and reproducibility as a construction property rather than a cache property.
- `uncertain-ensemble-carrier`: carrier-generic materialisation into any witnessed rank-1 container,
  and the haft construction capability that makes one signature serve both containers.
- `uncertain-arrow-graph`: the lazy computation graph as a value-level `Arrow`, and the removal of
  the four unreachable HKT node arms it replaces.

### Modified Capabilities

- `uncertain-realfield-generic`: today's requirement already describes a generic graph
  (`SampledValue<R>`, `UncertainNodeContent<R>`) while its own Purpose describes the closed
  dispatcher that is actually in the tree; the requirement was never met. It is restated as zero
  concrete scalars with a blanket bound, and the "f64 behavior preserved bit-for-bit, source
  compatible" requirement is withdrawn — this change is breaking by intent.
- `qmc-sampler`: the "Sampler-discriminated sample cache" requirement is withdrawn. With no cache
  there is no key to discriminate; a session is Monte-Carlo or QMC by construction.
- `sampling-hkt-composition`: the carrier requirement gains its `uncertain`-side realisation, and
  "a lazy sampler carrier is not given a witness" gains the positive half it implied — the graph is
  Arrow-shaped, and now actually is an `Arrow`.
- `qcl-evidence`: its scenario "The `Uncertain` boundary states its own restriction" records that the
  bound gains `ProbabilisticType`, "which `deep_causality_uncertain` implements for `f64` and
  `Float106` and not for `f32`". After this change there is no such bound and no such restriction.

## Impact

**Crates changed.** `deep_causality_uncertain` (the renovation), `deep_causality_haft` (one new
capability trait), `deep_causality_linear` and `deep_causality_tensor` (one impl each),
`deep_causality_cfd` (ten bounds lose `+ ProbabilisticType`), `deep_causality` (45 alias sites, held
by keeping the aliases), `deep_causality_quantum` (two files).

**Dependencies.** `deep_causality_uncertain` gains `deep_causality_haft`; it gains no dependency on
`linear` or `tensor` and stays at tier 5. It loses the `rusty-fork` dev-dependency.

**Public API.** Breaking. `SampledValue`, `ProbabilisticType`, `IntoSampledValue`,
`FromSampledValue`, `UncertainReal`, `GlobalSampleCache`, `with_global_cache`, `SamplerKind`,
`seed_sampler` and `clear_sampler_seed` are removed; probability parameters change type. The
`UncertainF64`, `UncertainBool`, `MaybeUncertainF64` and `UncertainF106` aliases are kept, so the
45 core call sites and the 12 CFD sites need no edit.

**Tests.** 247 today, of which 24 are fork-isolated and at least one is flaky. The flake is fixed by
seeding the gate through a session rather than by relaxing the assertion.
