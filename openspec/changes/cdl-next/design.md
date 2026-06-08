## Context

CDL (`deep_causality_discovery`) was a typestate pipeline specialized to SURD and
to `SurdResult<T>`: one dataset, one result type, a master `CdlConfig` object, and
a `NoData` entry. BRCD now exists and is verified in `deep_causality_algorithms`.
Its driver `brcd_run<T, N>(normal, anomalous, cpdag: Option<&MixedGraph<N>>,
&BrcdConfig<T>)` consumes two aligned `n × num_vars` matrices and an optional
CPDAG; when the CPDAG is `None` it learns one from the normal data via `boss_learn`
(BOSS) before ranking.

Three facts about BRCD shape this design: it needs two datasets, not one; MRMR
feature selection (and the `Option<T>` cleaning that feeds it) would drop and
reorder columns, desyncing the variable indices both the CPDAG and the BRCD ranks
refer to, so BRCD must not pass through those stages; and the CPDAG-`None` → BOSS
fallback is internal to `brcd_run`, so wiring it requires nothing beyond leaving
`cpdag = None`.

Constraints: static dispatch only; one type per module; no external crates beyond
those present (`csv` direct, `tempfile` dev); the algorithm crate's `BrcdConfig`,
`FamilyKind`, `BrcdResult`, `BrcdError`, and `boss_learn` are reused as-is.

## Goals / Non-Goals

**Goals:**

- Host SURD and BRCD as peer algorithms with parallel, explicit, config-driven
  stage names (`<algo>_load_input`, `<algo>_discover`, `<algo>_analyze`).
- Make crossing the two lineages a compile error, not a convention.
- Make a single, explicit, compile-checked config the source of truth for a run.
- Preserve SURD's numeric output and rendered report.
- Read CPDAGs from a file faithful to the `MixedGraph` typed-endpoint model.

**Non-Goals:**

- The bootstrap CPDAG-uncertainty variant (`brcd_run_bootstrap`).
- The composite differential-SURD-on-a-localized-node stack. The pipeline becomes
  able to host both algorithms; it does not chain them. The `deep_causality_haft`
  Arrow combinators (`first`/`split`/`fanout`) are the future fit for that
  parallel fan-out, ideally via an effectful/Kleisli arrow added later.
- Any change to `deep_causality_algorithms` or `deep_causality_topology`.

## Decisions

### D1. Two isolated typestate lineages, converging at `finalize`

```
build_surd(&cfg) ► SurdConfigured ─load─► SurdData ─clean─► SurdCleaned ─select─► SurdFeatures ─discover─► SurdResults ─analyze─┐
                                                                                                                               ├► WithAnalysis ─finalize► CdlReport
build_brcd(&cfg) ► BrcdConfigured ───────────────────────────────────────────────load─► BrcdLoaded ─discover─► BrcdResults ─analyze─┘
```

Isolation is achieved by which state each method is implemented on:
`surd_*` methods exist only on SURD states, `brcd_*` only on BRCD states, and
`feature_select`/`clean_data` only on SURD states. Crossing is `error[E0599]`,
guarded by `compile_fail` doctests. The lineages share no state until
`WithAnalysis<T>`, which exposes only `finalize`. `CDL<State>` carries no separate
config object; each lineage threads its own run config through its states.

### D2. `CdlConfigBuilder` is the single source of truth (staged typestate builder)

The product configs (`SurdLoaderConfig<T>`, `BrcdLoaderConfig<T>`) have only
`pub(crate)` constructors. `CdlConfigBuilder::build_surd_config::<T>()` /
`build_brcd_config()` start staged builders where each required field is its own
stage, so `build()` is reachable only once every required field is set — omitting
one is a compile error (make-invalid-states-unrepresentable). `build()` then
verifies the referenced files exist, returning `CdlError::ReadDataError` if not.
SURD requires `path`, `target_index`, `num_features`, `max_order`, `analyze`;
BRCD requires `normal_path`, `anomalous_path`, `brcd_config` (the reused
`BrcdConfig<T>`, mandatory — no hidden algorithm default). The precision `T` is
pinned at config-build time (`build_surd_config::<T>()`; `BrcdConfig::<T>` for
BRCD), so the pipeline needs no turbofish.

**Alternatives considered.** A loose `Config::new(..).with_*()` builder
(rejected: a second construction path undermines "single source of truth", and
a non-typestate builder can't enforce required fields at compile time). Embedding
the run config in a master `CdlConfig` (rejected: `CdlConfig` was non-generic and
the run configs are generic over `T`; carrying the run config in the states keeps
one source of truth and let `CdlConfig`/`NoData` be removed).

### D3. `build_surd`/`build_brcd` entries; config-driven, parameterless DSL

`CdlBuilder::build_surd(&SurdLoaderConfig<T>)` / `build_brcd(&BrcdLoaderConfig<T>)`
seed the `*Configured` state, carrying the config into the pipeline. The stages
then take no algorithm parameters and read them from the carried config:
`feature_select()` runs MRMR with the config's `num_features`/`target_index`;
`surd_discover()` runs `surd_states_cdl` with the config's `max_order`;
`surd_analyze()` uses the config's thresholds; BRCD's loading/discovery use the
bundle. This trades the previous mid-pipeline closures (custom selector / custom
discovery) for an explicit, config-driven pipeline — CDL's SURD lineage *is*
MRMR + SURD-states, so the loss is acceptable and a power-user closure variant
can be added later.

### D4. Fluent effect surface

Each stage is a method on `CdlEffect<CDL<State>>` that delegates to the
`CDL<State>` method through an `FnOnce` `and_then` (the `FnOnce` form lets owned
values move in and threads/merges the warning log like `bind`). The pipeline reads
`build_surd(&cfg).surd_load_input().clean_data(..)..` with no `.bind(|c| c. …)`
plumbing. `bind` stays public.

### D5. BRCD loading is an in-pipeline stage

`brcd_load_input` is a stage on `CDL<BrcdConfigured<T>>` that invokes the
`pub(crate)` `BrcdDataLoader` to read the two CSVs (+ optional CPDAG) into a
`BrcdInput<T>` bundle, inside the monad. Loading is no longer a separate
user-facing call. `cpdag = None` (no CPDAG path) is the entire BOSS-fallback
wiring, by `brcd_run`'s own contract.

### D6. `CdlDiscoveryOutcome<T>` closed enum carries the convergence

`CdlDiscoveryOutcome<T> { Surd(Box<SurdResult<T>>), Brcd(BrcdResult<T>) }` — the
single point of polymorphism, no `dyn`. `Box` on `Surd` keeps the variants
size-balanced (`SurdResult` is far larger than `BrcdResult`). `CdlReport<T>`
carries it plus `Option<MrmrResult>` (`None` for BRCD); `Display` matches the
variant. `CausalDiscoveryError` gains `Brcd(BrcdError)`. Each `*_analyze` runs its
own analyzer (`SurdResultAnalyzer` / `BrcdResultAnalyzer`) via a
`ProcessResultAnalyzer` trait generalized with associated `Input`/`Config` types.

### D7. CPDAG CSV faithful to the typed-endpoint model

`MixedGraph` stores edges as a canonical-pair map `BTreeMap<(lo,hi), Edge { lo:
Mark, hi: Mark }>`, `Mark ∈ {Tail, Arrow, Circle}`. The file is a 1:1 dump
(`vertices=N` comment header + `src,dst,mark_src,mark_dst` rows), so it round-trips
any DAG/CPDAG/MAG/PAG and preserves isolated vertices. `load_cpdag_csv` /
`save_cpdag_csv` live in the discovery crate (which has `csv`), keeping topology
free of filesystem code.

### D8. SURD preservation; `Precision` left clean

The SURD chain keeps its cleaning, MRMR, and `surd_states_cdl` behavior; only the
surface (config-driven, renamed) changes. A regression test runs the full SURD
chain and asserts it produces a SURD-variant report. `feature_select` adds a local
`T: Float + Debug + 'static` bound (MRMR's `F: Float`, which `RealField` does not
imply — the blanket runs `Float ⇒ RealField`); putting `Float` on `Precision`
itself was rejected because `Real` and `Float` share method names (`nan`/`is_nan`)
and would make every existing call ambiguous.

## Risks / Trade-offs

- **Wide breaking diff.** → Confined to one crate; only in-repo examples/tests
  consume the chain. A SURD regression test pins behavior.
- **Compile-time covers fields, not files.** → `build()` adds a runtime
  file-exists check (fail-fast); parse/dimension errors surface through the effect
  at the load stage.
- **Config-driven DSL drops the stage closures.** → CDL's SURD lineage is fixed to
  MRMR + SURD-states by design; a closure variant can return if needed.
- **BRCD needs dense matrices.** → BRCD bypasses `Option<T>` cleaning; the loader
  produces dense `CausalTensor<T>` and validates 2-D + equal `num_vars`.

## Migration Plan

Landed in one branch: generalized tail + `CdlDiscoveryOutcome` + analyzer contract
+ `CausalDiscoveryError::Brcd`; the staged `CdlConfigBuilder` + product configs;
`build_surd`/`build_brcd` + `*Configured` states + config-driven stages + fluent
wrappers; in-pipeline `BrcdDataLoader`; CPDAG CSV IO; removal of `CdlConfig` /
`NoData`; examples rewritten; full test migration. Rollback is `git revert`.

## Open Questions

None outstanding. All decisions were resolved during implementation review.
