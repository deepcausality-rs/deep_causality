## Context

CDL (`deep_causality_discovery`) is a typestate pipeline that loads one dataset,
cleans it, selects features with MRMR, runs SURD, analyzes the result, and emits
a report. Every state and method is specialized to SURD and to `SurdResult<T>`:

- `CDL<NoData> → WithData<T> → WithCleanedData<T> → WithFeatures<T> → WithCausalResults<T> → WithAnalysis<T> → CdlReport<T>`
- `CausalDiscovery::discover` returns `SurdResult<T>`; `WithCausalResults`,
  `CdlReport`, and `ProcessResultAnalyzer` all name `SurdResult<T>` directly.

BRCD now exists and is verified in `deep_causality_algorithms`. Its driver
`brcd_run<T, N>(normal, anomalous, cpdag: Option<&MixedGraph<N>>, &BrcdConfig<T>)`
consumes two aligned `n × num_vars` matrices and an optional CPDAG; when the
CPDAG is `None` it learns one from the normal data via `boss_learn` (BOSS) before
ranking. BRCD output is a `BrcdResult<T>` (ranked candidate root-cause sets with
posterior weights), indexed by raw column position.

Three facts about BRCD shape this design:

1. It needs two datasets, not one.
2. MRMR feature selection (and the `Option<T>` cleaning that feeds it) would drop
   and reorder columns, desyncing the variable indices that both the CPDAG and
   the BRCD ranks refer to. BRCD must not pass through those stages.
3. The CPDAG-`None` → BOSS fallback is internal to `brcd_run`. Wiring it requires
   nothing beyond leaving `cpdag = None`.

Constraints from the repo: static dispatch only (no `dyn`); one type per module;
no external crates beyond what is already present (`csv` is a direct dependency,
`tempfile` is a dev-dependency); `deep_causality_algorithms`'s `BrcdConfig`,
`FamilyKind`, `BrcdResult`, `BrcdError`, and `boss_learn` are reused as-is.

## Goals / Non-Goals

**Goals:**

- Host SURD and BRCD as peer algorithms in CDL with parallel, explicit method
  names (`load_<algo>_input`, `<algo>_discover`, `<algo>_analyze`).
- Make crossing the two lineages a compile error, not a convention.
- Preserve SURD's numeric output and rendered report exactly.
- Provide a `BrcdDataLoader` that turns a config (two paths + optional CPDAG path
  + CSV options + reused `BrcdConfig<T>`) into one `BrcdInput<T>` bundle.
- Serialize/deserialize a CPDAG to a CSV file faithful to the `MixedGraph`
  typed-endpoint model, housed in `deep_causality_discovery`.

**Non-Goals:**

- The bootstrap CPDAG-uncertainty variant (`brcd_run_bootstrap`). Single learned
  or supplied CPDAG only.
- The composite differential-SURD-on-a-localized-node diagnostic stack. The
  pipeline becomes able to host both algorithms; it does not chain them.
- Any change to `deep_causality_algorithms` or `deep_causality_topology`.
- A `SurdInput` bundle mirroring `BrcdInput`. SURD's feature-selection step takes
  a user closure mid-pipeline, so its inputs are not fully determined up front.

## Decisions

### D1. Two isolated typestate lineages, converging at `finalize`

```
NoData
  ├ load_surd_input ► SurdData<T> ─clean_data► SurdCleaned<T> ─feature_select► SurdFeatures<T> ─surd_discover► SurdResults<T> ─surd_analyze─┐
  │                    (preprocess / filter_cohort on SurdData)                                                                            ├► WithAnalysis<T> ─finalize► CdlReport<T>
  └ load_brcd_input ► BrcdLoaded<T> ──────────────────────────────────────────────────────────────────brcd_discover► BrcdResults<T> ─brcd_analyze─┘
```

Isolation is achieved purely by which state each method is implemented on. A
method named `brcd_discover` exists only in `impl<T> CDL<BrcdLoaded<T>>`, and
`surd_discover` only in `impl<T> CDL<SurdFeatures<T>>`; likewise `surd_analyze`
is only on `SurdResults<T>` and `brcd_analyze` only on `BrcdResults<T>`. Calling
the wrong one on the wrong lineage is `error[E0599]: no method named …`. The
lineages share no state until `WithAnalysis<T>`, and that state exposes only
`finalize`, so discovery cannot be re-run or crossed.

`finalize` stays shared (one impl on `WithAnalysis<T>`) because it is pure report
packaging with nothing algorithm-specific left to decide.

**Alternatives considered.** (a) Thread an optional second dataset and optional
graph through the existing single chain (the older `cdl-integration` note's D5).
Rejected: it routes BRCD through MRMR/cleaning, which corrupts it, and the
single `load_data` only reads one file. (b) A single shared `causal_discovery`
plus a single `analyze` that match a `DiscoveryOutcome` at runtime. Rejected:
the user can then call the wrong analyze and fail at runtime; the typestate makes
it a compile error instead. (c) A fully separate `BrcdCdl` type sharing nothing.
Rejected: duplicates the report/format/print tail for no isolation gain over (D1).

### D2. `DiscoveryOutcome<T>` closed enum as the convergence carrier

```rust
pub enum DiscoveryOutcome<T> { Surd(SurdResult<T>), Brcd(BrcdResult<T>) }
```

Each `*_analyze` wraps its concrete result into this enum as it transitions to
the shared `WithAnalysis<T>`. `CdlReport<T>` carries `DiscoveryOutcome<T>` plus
`Option<MrmrResult>` (`None` for BRCD), and its `Display` matches the variant.
This is the single point of polymorphism, it mirrors the existing
`CausalDiscoveryConfig` enum pattern, and it uses no `dyn`. Adding a third
algorithm later is a compile-checked exhaustive-match change.

**Alternatives considered.** A second generic result parameter threaded through
every state (rejected for the type churn it spreads); a boxed trait object
(rejected: violates the static-dispatch rule).

### D3. Per-algorithm analyzers and typed analyze configs

`surd_analyze` runs the existing `SurdResultAnalyzer` with a `SurdAnalyzeConfig`
(today's three thresholds). `brcd_analyze` runs a new `BrcdResultAnalyzer` with a
`BrcdAnalyzeConfig` (e.g. top-k candidates to report). Both produce the existing
`ProcessAnalysis(Vec<String>)`, so the formatter is unchanged. The
`ProcessResultAnalyzer` contract is generalized (an associated input/config type)
so each analyzer stays typed to its own result.

Because analyze is split at the type level, the single `AnalyzeConfig` is
replaced by two typed configs stored as separate `Option<..>` fields in
`CdlConfig`. This supersedes an earlier plan to make `AnalyzeConfig` an enum: an
enum would let a `Surd` config reach a BRCD pipeline and fail at runtime, exactly
the coupling the typestate removes.

### D4. `BrcdDataLoader` produces one `BrcdInput<T>` bundle; reuses `BrcdConfig`

```rust
pub struct BrcdLoaderConfig<T> {
    normal_path: String, anomalous_path: String,
    cpdag_path: Option<String>,   // None → brcd_run learns the CPDAG via BOSS
    csv: CsvConfig,               // defaults to CsvConfig::default()
    brcd_config: BrcdConfig<T>,   // the reused algorithm config; defaults to BrcdConfig::default()
}
pub struct BrcdInput<T> {
    normal: CausalTensor<T>, anomalous: CausalTensor<T>,
    cpdag: Option<MixedGraph<()>>, brcd_config: BrcdConfig<T>,
}
```

`BrcdDataLoader::load(&BrcdLoaderConfig<T>) -> Result<BrcdInput<T>, BrcdLoadError>`:
loads both CSVs through the existing `CsvDataLoader` (each → `CausalTensor<f64>`),
casts `f64 → T` with the same logic as `cast_loaded_tensor`, validates equal
2-D `num_vars`, parses the CPDAG file when `cpdag_path` is `Some` (validating
`num_vertices == num_vars`), and bundles the result. `brcd_config` is the
algorithm's own `BrcdConfig<T>`, embedded whole and passed straight into
`brcd_run`; there is no parallel config struct. `load_brcd_input(input)` seeds the
`BrcdLoaded<T>` state from the bundle.

BOSS is never invoked by the loader. Leaving `cpdag = None` and handing that to
`brcd_run` is the entire BOSS-fallback wiring, by the algorithm's own contract.

### D5. CPDAG CSV format faithful to the `MixedGraph` typed-endpoint model

`MixedGraph` stores edges as a canonical-pair map
`BTreeMap<(lo, hi), Edge { lo: Mark, hi: Mark }>`, with `Mark ∈ {Tail, Arrow,
Circle}`. The file is a 1:1 dump of that map, so it round-trips any mixed graph
(DAG/CPDAG/MAG/PAG), not only arc/undirected CPDAGs:

```
# deep_causality MixedGraph v1; vertices=5
src,dst,mark_src,mark_dst
0,1,Tail,Arrow      # arc 0 -> 1
1,2,Tail,Tail       # undirected 1 -- 2
```

`load_cpdag_csv(path) -> Result<MixedGraph<()>, CpdagError>` reads `vertices=N`
from the comment header, builds `MixedGraph::<()>::new(N, N units, 0)`, and
applies each row with `add_edge(src, dst, mark_src, mark_dst)`. `save_cpdag_csv`
writes the header then `lo,hi,edge.lo,edge.hi` for each entry in `edges()`. The
`csv` reader is configured with `.comment(Some(b'#'))` so the metadata/comment
lines are skipped during the edge pass; a cheap first-line scan extracts `N`.
Marks are spelled as full words (`Tail`/`Arrow`/`Circle`) for hand-editability,
mapped to `Mark` by a small match in the discovery serializer, so topology needs
no change. Storing `N` preserves isolated vertices that max-index inference would
drop.

**Alternatives considered.** A 0/1 adjacency matrix (rejected: lossy — cannot
express `Circle` marks, and the arc direction is a guessed convention not present
in the model); housing the IO in `deep_causality_topology` (rejected: keeps the
data-structure crate free of filesystem code, and discovery already has `csv`).

### D6. SURD preservation

The SURD front half keeps its cleaning, MRMR feature selection, and
`surd_states_cdl` call unchanged in behavior; only method/state names change and
the result is wrapped into `DiscoveryOutcome::Surd` at `surd_analyze`. A
regression test runs the SURD chain on a fixed dataset and asserts the rankings,
the SURD decomposition, and the rendered report are identical to the pre-change
output.

## Risks / Trade-offs

- **Wide breaking diff across the discovery crate.** → Confined to one crate;
  only in-repo examples/tests consume the chain. A regression test pins SURD
  output so the rename cannot silently alter behavior.
- **CPDAG file desync (vertex count ≠ dataset `num_vars`, or stale indices).** →
  The loader validates `cpdag.num_vertices() == num_vars` and fails loud with a
  `CpdagError` / `BrcdLoadError`, rather than letting `brcd_run` mismatch.
- **BRCD needs dense matrices; missing values would propagate as NaN.** → BRCD
  bypasses the `Option<T>` cleaning path by design; the loader produces dense
  `CausalTensor<T>` and documents that the two inputs must be numeric/complete.
  Imputation, if needed, is a pre-load concern, not part of the BRCD lineage.
- **New `deep_causality_topology` dependency on `deep_causality_discovery`.** →
  Single internal path dependency; topology is already a transitive dependency
  via `deep_causality_algorithms`, so no new external surface.
- **Marks as words make slightly larger files than single letters.** → Files are
  small (one line per edge) and hand-edited; readability wins.

## Migration Plan

1. Land the generalized tail (`DiscoveryOutcome`, `CdlReport<T>`, analyzer
   contract, `CausalDiscoveryError::Brcd`) and the branded SURD states/methods in
   one pass; update SURD examples/tests; confirm the SURD regression test passes.
2. Add `deep_causality_topology` to `deep_causality_discovery`'s dependencies.
3. Add CPDAG CSV serialization (`load_cpdag_csv` / `save_cpdag_csv`) with
   round-trip tests.
4. Add `BrcdLoaderConfig`, `BrcdInput`, `BrcdDataLoader`, the `BrcdLoaded`/
   `BrcdResults` states, and `load_brcd_input` / `brcd_discover` / `brcd_analyze`.
5. Add a BRCD example (supplied CPDAG and BOSS-fallback paths) and tests.
6. Re-export the new and reused public types from `lib.rs`.

Rollback is `git revert` of the branch; no data migration is involved.

## Open Questions

None outstanding. All design decisions were resolved during proposal review
(no bootstrap; CPDAG IO in discovery; full-word marks; per-algorithm analyze
with typed configs; `finalize` shared).
