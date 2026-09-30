# DeepCausality, distilled: causal discovery and the Causal Discovery Language

Source: every file in `deep_causality_algorithms/` (0.5.4) and `deep_causality_discovery/`
(0.7.3), read in full on 2026-09-30: `src/`, `tests/`, `benches/`, `examples/`, `verification/`,
`Cargo.toml`, `BUILD.bazel` and the READMEs. Bundled CSV data and PDFs were checked by header and
row count, not read row by row. The relation to the causal engine is checked against
`deep_causality/` (see `dc.md`).

## The finding

Causal discovery here is two different questions with two different algorithms. SURD asks how
much each set of sources tells about one target, and splits that information into redundant,
unique and synergistic parts. BRCD asks which variable's mechanism changed between a normal and an
anomalous regime, and ranks every candidate by posterior probability. `deep_causality_algorithms`
implements both, plus the MRMR feature selector, the BOSS structure learner and a polynomial-time
sampler for Markov equivalence classes. The Causal Discovery Language (CDL) in
`deep_causality_discovery` is a typestate pipeline that runs either algorithm from CSV files to a
printed report inside an error-and-warning monad.

The BRCD half is the strong half. The Rust port reproduces the reference Python ranking position
for position on four real microservice incidents (45/45, 45/45, 44/44, 45/45 variables), fixes a
sign bug in the reference BOSS score and a floating-point underflow in the reference ranking, and
replaces an exponential class-size enumeration with the Clique-Picking counter. The CDL's BRCD
pipeline adds a keyed CPDAG cache whose correctness is tested with a poisoned cache.

The CDL's SURD pipeline computes a number with no meaning. `surd_states_cdl` expects a joint
probability table with the target on axis 0 and one discrete source per further axis. The pipeline
hands it the raw `rows × selected_features` data matrix, from which MRMR has already removed the
target column. SURD then treats each observation as a target state and the selected columns as the
states of a single source. No stage builds a histogram. The pipeline tests assert only that the run
succeeds and that the report has its section headers, so nothing catches it.

Discovery does not connect to the causal engine in code. Neither crate depends on
`deep_causality`; the discovery README describes how a SURD result should guide the design of a
`CausaloidGraph`, and a human does that translation.

## 1. The two questions

| | SURD | BRCD |
|---|---|---|
| Question | How does information about a target split across sources? | Which variable's mechanism shifted? |
| Input | Joint probability tensor `p(T, S1, …, Sn)` | Two data matrices (normal, anomalous), optional CPDAG |
| Output | `SurdResult`: redundant, unique, synergistic info per source set, leak, state maps | `BrcdResult`: candidate sets ranked by log-posterior |
| Source | Martínez-Sánchez, Lozano-Durán, arXiv:2505.10878 (2025) | Lee, Zhou, Kocaoglu, ICML 2026 |
| Needs structure | No | A CPDAG, supplied or learned by BOSS |
| Feature gate | always | `topology` (default on) |

MRMR is a preprocessing step for SURD, not a discovery algorithm: it picks the `k` columns most
relevant to a target and least redundant with each other.

## 2. `deep_causality_algorithms`

Features: `default = ["std", "topology"]`; `topology` gates `brcd` and `dag_sampling`; `parallel`
turns on rayon. SURD and MRMR build without `std` given a heap. All numeric code is generic over
the algebra tower (`RealField + FromPrimitive`); tests instantiate `f64` and `Float106`.

### 2.1 SURD

```rust
pub fn surd_states<T>(p_raw: &CausalTensor<T>, max_order: MaxOrder) -> Result<SurdResult<T>, CausalTensorError>;
pub fn surd_states_cdl<T>(p_raw: &CausalTensor<Option<T>>, max_order: MaxOrder) -> Result<SurdResult<T>, CausalTensorError>;
pub enum MaxOrder { Min, Some(usize), Max }   // Min = pairs; Some(k), 2 <= k <= n; Max = all sources
```

The algorithm normalizes the table, computes the information leak `H(T | S) / H(T)` (zero when
`H(T)` is zero), and for each target state computes the specific mutual information of every
source combination up to `max_order`. It sorts those values stably (rank tolerance `1e-9`), raises
each order's values to at least the previous order's maximum, and takes differences. A singleton
increment is redundant, except the last singleton, which is unique; a multi-source increment is
synergistic. Aggregate redundant and unique information share one map, `redundant_info`: a key with
one source is unique information, a longer key is redundancy among those sources. Each increment
also yields a state map, split into causal (positive specific information) and non-causal
(negative) slices. Source indices in the keys are 1-based; axis 0 is the target.

`surd_states_cdl` skips `None` cells (pairwise deletion) and completes where the dense variant
returns `DivisionByZero`, for example on a constant target. `parallel` spreads the per-target-state
loop.

### 2.2 MRMR

`mrmr_features_selector::<V, T>(tensor, num_features, target_col)` implements the FCQ variant. The
first feature has the highest F-statistic against the target; each later one maximizes relevance
divided by mean absolute correlation with the already selected set. The target is never a
candidate. Scores are divided by the maximum score, so the first entry (an F-statistic) and the
rest (ratios) share one scale by fiat. `None` and NaN cells are dropped pairwise. Ties go to the
lowest index. A zero-over-zero or infinite score is an error (`FeatureScoreError`), as is any
non-finite observation. `MrmrError::NotEnoughFeatures` is never returned; the corresponding test is
commented out.

### 2.3 BRCD

```rust
pub fn brcd_run<T, N>(normal: &CausalTensor<T>, anomalous: &CausalTensor<T>,
                      cpdag: Option<&MixedGraph<N>>, config: &BrcdConfig<T>) -> Result<BrcdResult<T>, BrcdError>;
```

A `None` CPDAG is learned by `boss_learn` from the normal data. The run stacks both regimes and
adds an indicator column `FNODE` (0 normal, 1 anomalous). Candidates are all `k`-subsets of the
variables, `k = num_root_causes`.

1. **Structure, per candidate.** Enumerate the orientations of every undirected edge incident on
   the candidate (`ConfigStrategy::Full`, `2^du`, refused above `MAX_CONFIG_EDGES = 16`) or climb
   greedily to the most probable ones (`ConfigStrategy::MapPrune`, `du + 1` evaluations). Keep the
   configurations that are acyclic after Meek closure and add no unshielded collider at a target.
   For each kept configuration, point `FNODE` at the candidate, count the equivalence class with
   `dag_sampling::mec_size`, and draw one member with `sample_dag`. The configuration's weight is
   its class size over the candidate's total. Each candidate gets its own SplitMix-derived seed,
   so the result does not depend on evaluation order; under `parallel` candidates run on rayon.
2. **Families, once each.** Score every distinct (child, parents) family that any sampled DAG
   contains, in parallel under `parallel`. Continuous: ridge-Gaussian regression, per regime when
   `FNODE` is a parent, otherwise a logistic-gated two-component mixture. Discrete: prequential
   Dirichlet with `alpha_star = 5`.
3. **Posterior.** One configuration: sum the family scores. Several: per row, log-sum-exp over the
   weighted DAGs, summed over rows. Add the log prior and rank on the log-posterior. A candidate
   with no valid configuration scores `−∞`. The reference exponentiates before ranking and
   underflows to ties.

`BrcdConfig` fields: `seed`, `family`, `node_transform`, `transform_parents`, `num_root_causes`,
`ridge` (default `1e-4`), `alpha_star`, `gate`, `config_strategy`. With a fully directed CPDAG the
class size is 1 and the run is deterministic; the seed matters only when undirected edges remain.

`brcd_run_bootstrap` resamples the normal data `B` times, learns a CPDAG per resample, keeps the
top `k` distinct CPDAGs weighted by `log p(D | C) + log(1/k) − log q`, and marginalizes the ranking
over them.

**BOSS.** `boss_learn` searches permutations with a grow-shrink tree over a BIC score (higher is
better; the vendored reference has the sign inverted and learns the empty graph), shuffles with a
seeded Xoshiro, stops at tolerance `1e-8` or 2000 rounds, and converts the best DAG to its CPDAG
(`dag_to_cpdag`: v-structures, then Meek R1–R4).

### 2.4 `dag_sampling`

A port of Clique-Picking (Wienöbst, Bannach, Liśkiewicz). `mec_size` counts the DAGs in a CPDAG's
class in polynomial time as a product over chordal components; `sample_dag` draws one uniformly;
`representative_dag` returns a fixed member. Input is validated: an edge that is neither directed
nor undirected is `NotACpdag`, a directed cycle is `NotAcyclic`, a non-chordal component is
`NotACpdag`. The count is generic over the weight type, so `Float106` holds classes that overflow
`f64` precision.

The older `brcd::brcd_mec::mec_size` enumerates orientations exactly up to
`MEC_ENUM_BOUND = 100_000`. Production no longer calls it; the tests use it as the oracle.

### 2.5 How the algorithms are verified

- Clique-Picking agrees with the enumeration oracle on the author's anchors (54 and 108 members),
  on `K4`, `K5`, a closed-form case (32), and on several hundred random connected chordal graphs.
  The sampler is checked for validity (every draw is fully directed, acyclic, keeps compelled arcs,
  and maps back to the input CPDAG), full support, and a chi-square uniformity test at `p = 0.001`,
  all with fixed seeds.
- `verification_meek_r4` enumerates every labelled DAG up to 5 vertices and every configuration
  Algorithm 1 builds, and compares the shipped Meek closures against the definition of a compelled
  edge (enumerated extensions). It has a positive control for R4, a negative control (the identity
  closure must under-orient somewhere), a soundness check, and a check that the F-augmented graph
  is already closed.
- `verification_base` recovers the shifted mechanism in a synthetic `X → Y → Z` chain.
- `verification_online_boutique` and `verification_sockshop` replay four RCAEval incidents with the
  supplied service-map CPDAG and require the full Rust ranking to equal the reference ranking.
- `brcd_eval_accuracy_compute` compares `MapPrune` with `Full`: identical top-1 where `Full` is
  feasible, `du + 1` against `2^du` configuration evaluations, and completion at 1000 variables
  on bounded-degree graphs. Its own printout states that wall-clock growth there is roughly cubic,
  and that `MapPrune` is slower than `Full` when `du` is small.

## 3. The Causal Discovery Language

### 3.1 The carrier

```rust
pub struct CdlEffect<T> { pub inner: Result<T, CdlError>, pub warnings: CdlWarningLog }
pub struct CDL<State> { pub state: State }
```

`CdlEffect` has the same shape as the core causal monad (value, error, log) but is its own type,
with an HKT witness `CdlEffectWitness` and Functor, Applicative and Monad instances from
`deep_causality_haft`. `and_then` takes an `FnOnce`, short-circuits on error, and concatenates the
warning logs; every stage method is defined on both `CDL<State>` and `CdlEffect<CDL<State>>`, so a
pipeline reads as a plain method chain. `CdlBuilder::pure` lifts a value.

### 3.2 Configuration

Each pipeline takes one configuration value built by a staged builder. Required fields are separate
types, so a missing one does not compile; `build()` checks that every named file exists.

```rust
let config = CdlConfigBuilder::build_surd_config::<FloatType>()
    .with_path(&file_path).with_target_index(3).with_num_features(3)
    .with_max_order(MaxOrder::Max).with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
    .build()?;                                   // optional: with_exclude_indices, with_csv

let config = CdlConfigBuilder::build_brcd_config()
    .with_normal_path(n).with_anomalous_path(a).with_brcd_config(BrcdConfig::continuous(seed))
    .build()?;                                   // optional: with_cpdag_path, with_cpdag_cache_path, with_csv
```

`Precision` is `RealField + FromPrimitive + Default + Send + Sync`; one type alias sets the
precision of the whole run.

### 3.3 The two pipelines

```text
SURD  build_surd(&cfg) → surd_load_input → [preprocess, filter_cohort] → clean_data(cleaner)
      → feature_select → surd_discover → surd_analyze → finalize → CdlReport
      states: SurdConfigured → SurdData → SurdCleaned → SurdFeatures → SurdResults → WithAnalysis

BRCD  build_brcd(&cfg) → brcd_load_input → brcd_discover → brcd_analyze → finalize → CdlReport
      states: BrcdConfigured → BrcdLoaded → BrcdResults → WithAnalysis
```

A stage exists only on its state, so the two lineages cannot be mixed; `compile_fail` doctests in
`lib.rs` prove it. `WithAnalysis` is shared and holds `CdlDiscoveryOutcome::{Surd(Box<SurdResult>),
Brcd(BrcdResult)}`. `CdlReport` carries the dataset path, record count, the MRMR result (SURD
only) and the outcome; `print_results` prints the report and warnings, or the error.

**BRCD loading.** `brcd_load_input` reads both CSVs, requires equal column counts, and resolves the
CPDAG in priority order: a supplied CPDAG file; a cache file whose sidecar `<path>.key` matches an
FNV-1a hash of the format version, shape, the bit patterns of every value, and the seed; otherwise
BOSS learns the CPDAG and the loader writes it and its key. The CPDAG file format is a
`# deep_causality MixedGraph v1; vertices=N` header followed by `src,dst,mark_src,mark_dst` rows
with `Tail`, `Arrow` or `Circle` marks. `brcd_cache_cold_vs_warm` shows a warm run returning the
same ranking as a cold one without re-learning.

## 4. How discovery relates to the causal engine

The dependency graph is one-way and ends before the engine: `deep_causality_discovery` depends on
`deep_causality_algorithms`, the math crates and `csv`/`parquet`, and neither depends on
`deep_causality`. The only consumer of either crate in the workspace is
`examples/causal_discovery_examples`.

The discovery README states the intended hand-off. A strong unique influence suggests a direct
`Causaloid(Source) → Causaloid(Target)` link; a strong synergy suggests a many-to-one collection
with `AggregateLogic::All`; unique or redundant influences suggest `AggregateLogic::Any`; SURD's
state maps suggest the conditions inside a causaloid's function. No code performs any step of that
mapping.

BRCD's output has no counterpart in the engine at all. It ranks the variables most likely to have
changed mechanism, which is a diagnosis for an operator, not a model to evaluate. The composite
root-cause-analysis vision (SURD × BRCD) lives in `openspec/notes/composite_rca.md`, not in code.

## 5. Where the code and the claims part

1. **The CDL SURD pipeline is wrong** (the finding above). `feature_select` returns the selected
   columns without the target (`types/cdl/surd_cleaned.rs`); `surd_discover` passes that matrix
   to `surd_states_cdl` (`types/cdl/surd_features.rs`). A correct pipeline would keep the target
   column, discretize each column, and build the joint histogram with the target on axis 0.
   `MrmrConfig`'s doc says the target "is always included in the selection"; the selector excludes
   it. `DataDiscretizer` bins columns independently and builds no joint table.
2. **The SURD analyzer mislabels its sections.** "Unique influences" reads single-source entries of
   `mutual_info`, which is total mutual information. The actual unique information is the
   single-source entries of `redundant_info`, which the analyzer prints under "Redundant
   influences" (`types/analysis/surd_result_analyzer.rs`).
3. **Unwired components.** `SurdCausalDiscovery`, `MrmrFeatureSelector`, `ParquetDataLoader`,
   `ConsoleFormatter`, the `CausalDiscovery` and `FeatureSelector` traits and their configs are
   public and tested but no pipeline stage or example uses them. `surd_load_input` always reads
   CSV. `MissingValueImputer` is a static function, not a `DataPreprocessor`. `ConsoleFormatter`
   prints from inside the library.
4. **Warnings never occur.** No stage adds a `CdlWarning`; the channel is plumbed and empty. The
   `CdlError::Missing*Config` variants are never constructed, since the builders make missing
   configuration a compile error.
5. **Ignored configuration.** `CsvConfig::columns`, `ParquetConfig::columns` and `batch_size` have
   no effect. `brcd_analyze` uses `BrcdAnalyzeConfig::default()` (top 5); the builder offers no way
   to set it. CSV parse errors surface as `OsError`; `skip_rows` counts after the header; a ragged
   row is caught only when the tensor is built; `target_index` indexes the columns left after
   `exclude_indices`, which the docs do not say.
6. **The verification README's "fault rank" is the reference top-1, not the injected fault.**
   `compare_case` takes `expected.first()` as the fault (`verification/brcd/common.rs`). On Online
   Boutique that is `adservice_cpu`, the injected fault. On Sock Shop it is `shipping_latency`
   (case 1) and `carts_latency` (case 2); the injected `carts_cpu` (column 0) sits at reference
   rank 6 and 4. The table's "1 / 1" and "1 / 4" therefore measure agreement with the reference,
   and the prose "the learned CPDAG pushes the fault from rank 1 to rank 4" describes
   `carts_latency`. In case 2 the BOSS-learned ranking places `carts_cpu` second, ahead of the
   supplied ranking. The ranking-equality checks in `verify_case` are unaffected.
7. **Stale or contradictory comments.** `dag_sampling/mod.rs` has a section "Counting only (for
   now)" and says chordality "is not checked"; the module samples and refuses non-chordal input.
   `brcd/mod.rs` calls Phase 1 sequential and says it samples "one representative DAG"; it runs in
   parallel under `parallel` and samples one DAG per configuration. `csv_exclude_bug_repro_tests` says an
   assertion "will FAIL"; it passes. SURD consistency tests carry "(Buggy)" notes on fixed code.
   The test `the_mixture_lies_above_both_of_its_components` asserts the mixture is at most the
   own-regime density.
8. **Minor.** `RidgeFit::predict` truncates rows of the wrong width instead of refusing them.
   The Yeo-Johnson node transform is not supported. `brcd_mec` stays public beside `dag_sampling`.
   `CdlEffect`, `CDL` and `CdlReport` expose public fields, against the repository's field rule.
   The discovery `Cargo.toml` documentation link points at `docs.rs/deep_causality`.

## 6. File map

| Concern | Path |
|---|---|
| SURD | `deep_causality_algorithms/src/causal_discovery/surd/` (`surd_algo.rs`, `surd_algo_cdl.rs`, `surd_max_order.rs`, `surd_result.rs`, `surd_utils/`) |
| BRCD run and phases | `deep_causality_algorithms/src/causal_discovery/brcd/` (`brcd_algo.rs`, `brcd_config.rs`, `brcd_augment.rs`, `brcd_mapconfig.rs`, `brcd_validity.rs`, `brcd_result.rs`) |
| BRCD family scores | `…/brcd/brcd_gaussian.rs`, `brcd_gate.rs`, `brcd_dirichlet.rs`, `brcd_project.rs` |
| BOSS and bootstrap | `…/brcd/brcd_boss_{score,gst,search,learn,cpdag,config,bootstrap}.rs` |
| Exact enumeration oracle | `…/brcd/brcd_mec.rs` |
| Class counting and sampling | `deep_causality_algorithms/src/dag_sampling/` (`count.rs`, `sample.rs`, `chordal.rs`, `clique_tree.rs`, `index_set.rs`) |
| MRMR | `deep_causality_algorithms/src/feature_selection/mrmr/` |
| Verification harnesses, data | `deep_causality_algorithms/verification/brcd/` (`common.rs`, `verification_*.rs`, `data/`) |
| Benchmarks | `deep_causality_algorithms/benches/` (`brcd_benchmark`, `brcd_benchmark_wide`, `mrmr_benchmark`) |
| CDL carrier | `deep_causality_discovery/src/types/cdl_effect/`, `cdl_builder/`, `cdl_warning/` |
| Typestates and stages | `deep_causality_discovery/src/types/cdl/` |
| Configuration | `deep_causality_discovery/src/types/config/`, `cdl_config_builder/` |
| Loaders, CPDAG cache and file format | `deep_causality_discovery/src/types/data_loader/` (`brcd.rs`, `cpdag_cache.rs`, `cpdag_csv.rs`, `csv.rs`, `parquet.rs`) |
| Cleaning, preprocessing, selection | `…/data_cleaner/`, `…/data_preprocessor/`, `…/feature_selector/` |
| Analysis and report | `…/analysis/`, `…/cdl_report/`, `…/cdl_discovery_outcome/`, `…/formatter/` |
| Examples | `deep_causality_discovery/examples/brcd_cache_cold_vs_warm.rs`, `examples/causal_discovery_examples/` (`algos/`, `cdl/`) |
