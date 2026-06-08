## Why

The CDL pipeline in `deep_causality_discovery` hosts exactly one discovery
algorithm, SURD, and hardcodes its single-dataset flow and `SurdResult<T>`
output through every typestate. The BRCD root-cause algorithm now exists in
`deep_causality_algorithms` and is verified, but it cannot be reached from CDL:
it needs two aligned datasets, an optional CPDAG over the variables, and it must
not pass through MRMR feature selection (which reorders and drops columns and
would desync the graph). This change re-designs CDL to host both algorithms as
peers without compromising SURD.

## What Changes

- **BREAKING**: Replace the single linear pipeline with two compile-time-isolated
  typestate lineages that converge only at `finalize`:
  - SURD: `build_surd → surd_load_input → clean_data → feature_select → surd_discover → surd_analyze → finalize`
  - BRCD: `build_brcd → brcd_load_input → brcd_discover → brcd_analyze → finalize`
- **BREAKING**: A config is the single source of truth. `CdlConfigBuilder` is a
  **staged typestate builder** that constructs the run config:
  - `CdlConfigBuilder::build_surd_config::<T>()` → `SurdLoaderConfig<T>` (path,
    target index, MRMR feature count, max order, analyze thresholds; optional
    exclude indices / CSV options).
  - `CdlConfigBuilder::build_brcd_config()` → `BrcdLoaderConfig<T>` (normal path,
    anomalous path, the reused algorithm `BrcdConfig<T>`; optional CPDAG path /
    CSV options).
  - Required fields are enforced at compile time (`build()` only exists once they
    are set); `build()` additionally verifies the referenced files exist and
    returns a `CdlError` if not. The product config types have no public
    constructor, so the builder is the only way to make one.
- **BREAKING**: `CdlBuilder::build()` is replaced by `CdlBuilder::build_surd(&cfg)`
  and `CdlBuilder::build_brcd(&cfg)`, which carry the run config into the pipeline.
  The DSL stages are then **parameterless and config-driven** (e.g.
  `feature_select()` reads the MRMR feature count and target from the config;
  `surd_discover()` reads the max order; `surd_analyze()` reads the thresholds).
- **Fluent surface**: each stage is a method on the effect, so the pipeline reads
  `build_surd(&cfg).surd_load_input().clean_data(..)..` with no `.bind(|c| c. …)`
  wrapper. An `FnOnce` `and_then` underlies the wrappers; `bind` remains public.
- BRCD data loading is an **in-pipeline stage** (`brcd_load_input`), not a separate
  user call. The loader (`BrcdDataLoader`) is `pub(crate)`. When no CPDAG path is
  given, the bundle's CPDAG is left `None`, which makes `brcd_run` learn it via BOSS.
  No bootstrap variant.
- Add CPDAG CSV serialization (`load_cpdag_csv` / `save_cpdag_csv`) faithful to the
  `MixedGraph` typed-endpoint model (`Mark` ∈ Tail/Arrow/Circle per endpoint),
  reusing the existing `csv` dependency. `deep_causality_topology` is untouched.
- **BREAKING**: Generalize the discovery result carrier to a closed
  `CdlDiscoveryOutcome<T>` enum (`Surd(Box<SurdResult<T>>)`, `Brcd(BrcdResult<T>)`),
  and generalize `CdlReport<T>`, the `ProcessResultAnalyzer` contract (associated
  `Input`/`Config`), and `CausalDiscoveryError` (add a `Brcd(BrcdError)` variant).
- **BREAKING (removals)**: the old master `CdlConfig` object and the `NoData`
  entry state are removed; `CDL<State>` carries only its state, and each lineage
  threads its own run config through its states.

## Capabilities

### New Capabilities
- `cdl-pipeline`: the dual-algorithm CDL typestate — the `build_surd`/`build_brcd`
  entries, the two isolated SURD/BRCD lineages with their config-driven stages,
  the compile-time isolation guarantees, the shared
  `CdlDiscoveryOutcome → analyze → finalize` tail, the generalized report, and the
  preserved SURD behavior.
- `brcd-data-loading`: BRCD input ingestion — the `CdlConfigBuilder` staged
  builders (single source of truth, compile-time required fields + file-exists
  check), the `SurdLoaderConfig` / `BrcdLoaderConfig` product types, the
  in-pipeline `BrcdDataLoader`, the BOSS-fallback contract, and the CPDAG CSV
  save/load format.

### Modified Capabilities
<!-- None: no existing spec covers the CDL pipeline; the only specs in
     openspec/specs/ are causal-arrow and causal-flow. -->

## Impact

- **Crate**: `deep_causality_discovery` (all source and behavior changes live here).
- **New dependency**: `deep_causality_topology` added to
  `deep_causality_discovery` (the loader names `MixedGraph<()>`).
- **Reused as-is**: `deep_causality_algorithms` (`brcd_run`, `BrcdConfig`,
  `FamilyKind`, `BrcdResult`, `BrcdError`, `boss_learn`) and its `MixedGraph`
  dependency; no algorithm-crate changes.
- **Public API**: breaking for `deep_causality_discovery`. Only in-repo examples
  and tests consume the chain; no external consumers depend on the old
  signatures.
- **Out of scope**: the composite differential-SURD-on-a-localized-node
  diagnostic stack (the Arrow combinators in `deep_causality_haft` are the future
  fit for that parallel fan-out); seams are left compatible but nothing is built.
