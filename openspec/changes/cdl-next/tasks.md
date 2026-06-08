## 1. Dependencies and scaffolding

- [ ] 1.1 Add `deep_causality_topology` (path dependency) to `deep_causality_discovery/Cargo.toml` under internal dependencies.
- [ ] 1.2 Mirror the new dependency in `deep_causality_discovery/BUILD.bazel` (deps), and prepare `tests/BUILD.bazel` for the new test modules.
- [ ] 1.3 Confirm `csv` (direct dep) and `tempfile` (dev-dep) are present; no new external crates are added.

## 2. Generalized discovery tail

- [ ] 2.1 Add `DiscoveryOutcome<T>` closed enum (`Surd(SurdResult<T>)`, `Brcd(BrcdResult<T>)`) in `src/types/causal_discovery/` with no `dyn`; register the module.
- [ ] 2.2 Add `Brcd(BrcdError)` variant to `CausalDiscoveryError` in `src/errors/causal_discovery_error.rs`, with `Display`, `Error::source`, and a `From<BrcdError>` impl; update the existing `Display` text that names SURD.
- [ ] 2.3 Generalize `ProcessResultAnalyzer` in `src/traits/process_result.rs` so each analyzer is typed to its own result and config (associated input/config type), keeping `ProcessAnalysis` and `ProcessResultFormatter` unchanged.
- [ ] 2.4 Generalize `CdlReport<T>` in `src/types/cdl_report/mod.rs` to carry `causal_analysis: DiscoveryOutcome<T>` and `feature_selection: Option<MrmrResult>`; update its `Display` to match the variant (SURD section vs BRCD ranking, no feature-selection section for BRCD).
- [ ] 2.5 Generalize the converged `WithAnalysis<T>` state to carry `DiscoveryOutcome<T>` and `Option<MrmrResult>`; implement `finalize` once on `WithAnalysis<T>` producing `CdlReport<T>`.

## 3. SURD lineage rename and preservation

- [ ] 3.1 Rename SURD typestates to `SurdData<T>`, `SurdCleaned<T>`, `SurdFeatures<T>`, `SurdResults<T>` in `src/types/cdl/`; keep field contents.
- [ ] 3.2 Rename `load_data` → `load_surd_input` on `CDL<NoData>` (keep `load_tensor`/`load_data_with_config` equivalents under SURD naming); keep `preprocess`/`filter_cohort` on `SurdData<T>`.
- [ ] 3.3 Move `clean_data` to `SurdData<T> → SurdCleaned<T>` and `feature_select` to `SurdCleaned<T> → SurdFeatures<T>`.
- [ ] 3.4 Rename `causal_discovery` → `surd_discover` on `SurdFeatures<T> → SurdResults<T>`, wrapping the `SurdResult<T>` for later outcome construction.
- [ ] 3.5 Add `surd_analyze` on `SurdResults<T> → WithAnalysis<T>`, running `SurdResultAnalyzer` with a `SurdAnalyzeConfig`, defaulting when absent, and wrapping into `DiscoveryOutcome::Surd` with `Some(MrmrResult)`.
- [ ] 3.6 Rename `AnalyzeConfig` to `SurdAnalyzeConfig` (the three thresholds) in `src/types/config/`; store it as its own `Option<..>` in `CdlConfig`; remove the single `analyze_config` enum field.

## 4. CPDAG CSV serialization

- [ ] 4.1 Add `src/types/data_loader/cpdag_csv.rs` with `save_cpdag_csv<T>(&MixedGraph<T>, path)` and `load_cpdag_csv(path) -> Result<MixedGraph<()>, CpdagError>`; register in `src/types/data_loader/mod.rs`.
- [ ] 4.2 Implement the writer: `# … vertices=N` header, then one `lo,hi,mark_lo,mark_hi` row per entry of `graph.edges()`, marks as full words.
- [ ] 4.3 Implement the reader: extract `N` from the header line, configure the `csv` reader with `.comment(Some(b'#'))`, build `MixedGraph::<()>::new(N, N units, 0)`, apply each row via `add_edge(src, dst, mark_src, mark_dst)`; map `Mark ↔ "Tail"/"Arrow"/"Circle"` with a small local match.
- [ ] 4.4 Add `CpdagError` in `src/errors/` (file-not-found, CSV parse, unknown mark token, non-numeric/out-of-range vertex, `TopologyError` from edge insertion); register and fold into `CdlError`.

## 5. BRCD input loading

- [ ] 5.1 Add `BrcdLoaderConfig<T>` in `src/types/config/` (`normal_path`, `anomalous_path`, `cpdag_path: Option<String>`, `csv: CsvConfig` defaulting to `CsvConfig::default()`, `brcd_config: BrcdConfig<T>` defaulting to `BrcdConfig::default()`), with constructors/getters.
- [ ] 5.2 Add `BrcdInput<T>` bundle in `src/types/` (`normal`, `anomalous`, `cpdag: Option<MixedGraph<()>>`, `brcd_config`), with getters.
- [ ] 5.3 Add `BrcdDataLoader` in `src/types/data_loader/` with `load(&BrcdLoaderConfig<T>) -> Result<BrcdInput<T>, BrcdLoadError>`: load both CSVs via `CsvDataLoader`, cast `f64 → T` (reuse the `cast_loaded_tensor` logic), validate 2-D and equal `num_vars`, load+validate the CPDAG when a path is given (`num_vertices == num_vars`), else `cpdag = None`.
- [ ] 5.4 Add `BrcdLoadError` in `src/errors/` covering file/dimension/CPDAG failures (wrapping `CpdagError` and `DataLoadingError`); register and fold into `CdlError`.

## 6. BRCD lineage

- [ ] 6.1 Add `BrcdLoaded<T>` and `BrcdResults<T>` typestates in `src/types/cdl/`; register modules.
- [ ] 6.2 Add `load_brcd_input(input: BrcdInput<T>)` on `CDL<NoData> → CDL<BrcdLoaded<T>>`.
- [ ] 6.3 Add `brcd_discover` on `CDL<BrcdLoaded<T>> → CDL<BrcdResults<T>>`, calling `brcd_run(normal, anomalous, cpdag.as_ref(), &brcd_config)` and mapping `BrcdError` to `CdlError` via `CausalDiscoveryError::Brcd`.
- [ ] 6.4 Add `BrcdResultAnalyzer` in `src/types/analysis/` and `BrcdAnalyzeConfig` in `src/types/config/` (top-k); register both; store the config as its own `Option<..>` in `CdlConfig`.
- [ ] 6.5 Add `brcd_analyze` on `CDL<BrcdResults<T>> → CDL<WithAnalysis<T>>`, running `BrcdResultAnalyzer` with `BrcdAnalyzeConfig` (default when absent) and wrapping into `DiscoveryOutcome::Brcd` with `None` feature selection.

## 7. Public surface

- [ ] 7.1 Re-export from `lib.rs`: the new pipeline/state/outcome types, `BrcdDataLoader`, `BrcdLoaderConfig`, `BrcdInput`, `BrcdResultAnalyzer`, `BrcdAnalyzeConfig`, `SurdAnalyzeConfig`, `save_cpdag_csv`/`load_cpdag_csv`, `CpdagError`, `BrcdLoadError`.
- [ ] 7.2 Re-export the reused algorithm types: `BrcdConfig`, `FamilyKind`, `BrcdResult`, `BrcdError` (and `MixedGraph` for CPDAG construction), mirroring the existing SURD re-exports.

## 8. Examples

- [ ] 8.1 Update the SURD example chain to the new method names (`load_surd_input`/`surd_discover`/`surd_analyze`).
- [ ] 8.2 Add a BRCD example: build a `BrcdLoaderConfig`, `BrcdDataLoader::load`, then `load_brcd_input → brcd_discover → brcd_analyze → finalize`, with one supplied-CPDAG case and one BOSS-fallback (`cpdag_path = None`) case.

## 9. Tests and coverage

- [ ] 9.1 Mirror the new src modules under `tests/` (`_tests` suffix), register each in its `mod.rs`, and declare them in `tests/BUILD.bazel`.
- [ ] 9.2 SURD regression test: assert rankings, decomposition, and rendered report are identical to the pre-change output on a fixed dataset.
- [ ] 9.3 CPDAG round-trip tests (using `tempfile`): directed/undirected/bidirected/circle edges and an isolated vertex; malformed-content rejection (`CpdagError`).
- [ ] 9.4 `BrcdDataLoader` tests: aligned load, mismatched `num_vars` rejection, non-2-D rejection, `cpdag_path = None` yields `None`, CPDAG size-mismatch rejection.
- [ ] 9.5 BRCD pipeline tests: supplied-CPDAG end to end, BOSS-fallback end to end, `BrcdError → CdlError` propagation, report renders the BRCD ranking.
- [ ] 9.6 Compile-fail tests (trybuild or documented `compile_fail` doctests): `brcd_discover` on a SURD state, `surd_analyze` on `BrcdResults`, `feature_select` on a BRCD state.

## 10. Verification

- [ ] 10.1 `cargo build -p deep_causality_discovery` and `cargo test -p deep_causality_discovery` pass.
- [ ] 10.2 `make format && make fix` clean (no clippy suppressions added).
- [ ] 10.3 Confirm 100% coverage of added/edited files, excluding only genuinely unreachable code.
