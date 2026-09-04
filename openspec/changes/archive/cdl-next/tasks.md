## 1. Dependencies and scaffolding

- [x] 1.1 Add `deep_causality_topology` (path dependency) to `deep_causality_discovery/Cargo.toml` and `BUILD.bazel`.

## 2. Generalized discovery tail

- [x] 2.1 Add `CdlDiscoveryOutcome<T>` closed enum (`Surd(Box<SurdResult<T>>)`, `Brcd(BrcdResult<T>)`) in `types/cdl_discovery_outcome/`.
- [x] 2.2 Add `Brcd(BrcdError)` to `CausalDiscoveryError` with `Display`/`source`/`From`.
- [x] 2.3 Generalize `ProcessResultAnalyzer` with associated `Input`/`Config` types.
- [x] 2.4 Generalize `CdlReport<T>` to carry `CdlDiscoveryOutcome<T>` + `Option<MrmrResult>`; variant-matched `Display`.
- [x] 2.5 Converged `WithAnalysis<T>` + shared `finalize`.

## 3. Config types and the single-source-of-truth builder

- [x] 3.1 `SurdLoaderConfig<T>` and `BrcdLoaderConfig<T>` product types with `pub(crate)` constructors and getters.
- [x] 3.2 `CdlConfigBuilder` with staged `build_surd_config::<T>()` / `build_brcd_config()` enforcing required fields at compile time.
- [x] 3.3 `build()` runs a file-exists check, returning `CdlError::ReadDataError` on a missing file.
- [x] 3.4 `SurdAnalyzeConfig` (renamed from `AnalyzeConfig`) and `BrcdAnalyzeConfig`.

## 4. Builder entries and states

- [x] 4.1 `CdlBuilder::build_surd(&cfg)` / `build_brcd(&cfg)`; remove `build()`.
- [x] 4.2 States: `SurdConfigured`/`SurdData`/`SurdCleaned`/`SurdFeatures`/`SurdResults`, `BrcdConfigured`/`BrcdLoaded`/`BrcdResults`, shared `WithAnalysis`. Remove `NoData`/`Finalized` and the `CdlConfig` field; thread each lineage's run config through its states.

## 5. Config-driven, fluent DSL

- [x] 5.1 `and_then` (FnOnce Kleisli) on `CdlEffect`; fluent stage wrappers on each `CdlEffect<CDL<State>>`.
- [x] 5.2 SURD stages: `surd_load_input`, `clean_data`, `feature_select` (MRMR from config), `surd_discover` (SURD-states from config), `surd_analyze` (thresholds from config), `filter_cohort`, `preprocess`.
- [x] 5.3 BRCD stages: `brcd_load_input` (in-pipeline load), `brcd_discover`, `brcd_analyze`.
- [x] 5.4 `feature_select` carries the local `Float + Debug + 'static` bound MRMR needs (not pushed onto `Precision`).

## 6. BRCD loading and CPDAG IO

- [x] 6.1 `BrcdDataLoader` made `pub(crate)`, invoked by `brcd_load_input`; shared `cast_loaded_tensor`.
- [x] 6.2 `BrcdInput<T>` bundle; absent CPDAG path ⇒ `cpdag = None` ⇒ BOSS in `brcd_run`.
- [x] 6.3 `load_cpdag_csv` / `save_cpdag_csv` faithful to the `MixedGraph` typed-endpoint model; `CpdagError`.
- [x] 6.4 `BrcdLoadError`; fold both new errors into `CdlError`.

## 7. Public surface

- [x] 7.1 Re-export the new types and reused algorithm types from `lib.rs`; `BrcdDataLoader` stays `pub(crate)`.
- [x] 7.2 Crate-level `compile_fail` doctests for lineage isolation.

## 8. Examples

- [x] 8.1 SURD example on the config-builder + parameterless DSL surface; shared test-data helpers in `examples/.../shared`.
- [x] 8.2 Two BRCD examples: `brcd_discovery` (supplied CPDAG) and `brcd_boss_discovery` (BOSS fallback).

## 9. Tests

- [x] 9.1 Migrate existing tests to the new surface; rename test files to mirror sources; update `mod.rs` registrations (Bazel globs auto-pick).
- [x] 9.2 SURD regression (full chain → SURD-variant report).
- [x] 9.3 CPDAG round-trip + malformed-content rejection.
- [x] 9.4 Config-builder tests (file-exists error, optional/required fields, getters).
- [x] 9.5 Both lineages end to end (supplied CPDAG, BOSS fallback) + dimension-mismatch error.
- [x] 9.6 `compile_fail` doctests for cross-lineage isolation.

## 10. Verification

- [x] 10.1 `cargo build` / `cargo test -p deep_causality_discovery` pass (327 tests + 3 doctests).
- [x] 10.2 `cargo fmt` + `cargo clippy` clean (no suppressions; `large_enum_variant` fixed by boxing).
- [x] 10.3 Examples build and run; BRCD ranks the injected root cause in both paths.
