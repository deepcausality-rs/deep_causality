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
  - SURD: `load_surd_input → clean_data → feature_select → surd_discover → surd_analyze → finalize`
  - BRCD: `load_brcd_input → brcd_discover → brcd_analyze → finalize`
- **BREAKING**: Rename the SURD entry/discover/analyze methods (`load_data` →
  `load_surd_input`, `causal_discovery` → `surd_discover`, the analyze step →
  `surd_analyze`) and brand the SURD intermediate states (`SurdData`,
  `SurdCleaned`, `SurdFeatures`, `SurdResults`). SURD numeric output is preserved
  exactly, guarded by a regression test.
- Add a `BrcdDataLoader` that reads a `BrcdLoaderConfig<T>` (two dataset paths, an
  optional CPDAG path, shared CSV options, and the reused algorithm
  `BrcdConfig<T>`) and produces a single `BrcdInput<T>` bundle. When no CPDAG path
  is given, `cpdag` is left `None`, which is what makes `brcd_run` learn the CPDAG
  via BOSS. No bootstrap variant.
- Add CPDAG CSV serialization in `deep_causality_discovery` (`load_cpdag_csv` /
  `save_cpdag_csv`) faithful to the `MixedGraph` typed-endpoint model
  (`Mark` ∈ Tail/Arrow/Circle per endpoint), reusing the existing `csv`
  dependency. `deep_causality_topology` is untouched.
- **BREAKING**: Generalize the discovery result carrier to a closed
  `DiscoveryOutcome<T>` enum (`Surd(SurdResult<T>)`, `Brcd(BrcdResult<T>)`), and
  generalize `CdlReport<T>`, the `ProcessResultAnalyzer` contract, and
  `CausalDiscoveryError` (add a `Brcd(BrcdError)` variant) accordingly.
- Replace the single `AnalyzeConfig` with two typed configs
  (`SurdAnalyzeConfig`, `BrcdAnalyzeConfig`), each read by its own `*_analyze`
  method; the per-algorithm typestate removes the need for a runtime-dispatched
  enum config.

## Capabilities

### New Capabilities
- `cdl-pipeline`: the dual-algorithm CDL typestate — two isolated SURD/BRCD
  lineages, the compile-time isolation guarantees, the shared
  `DiscoveryOutcome → analyze → finalize` tail, the generalized report, and the
  preserved SURD behavior.
- `brcd-data-loading`: BRCD input ingestion — `BrcdDataLoader`,
  `BrcdLoaderConfig<T>`, the `BrcdInput<T>` bundle, the BOSS-fallback contract,
  and the CPDAG CSV save/load format.

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
  diagnostic stack; seams are left compatible but nothing is built.
