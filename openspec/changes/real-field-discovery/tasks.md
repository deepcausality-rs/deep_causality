## 1. Algorithms — SURD (`generic-precision-algorithms`)

- [ ] 1.1 Record golden `f64` outputs for SURD on the existing test inputs before any change.
- [ ] 1.2 Make `SurdResult` generic as `SurdResult<T = f64>`; update its fields and accessors to `T`.
- [ ] 1.3 Make the SURD core and `surd_states_cdl` generic over `T: RealField`; replace `f64` literals with `T::zero()`/`T::one()`/`T::from(..)` and use `RealField` ops for the entropy/log math.
- [ ] 1.4 Add `FromPrimitive` alongside `RealField` only where integer-to-real conversion is needed (counts, orders).
- [ ] 1.5 Update SURD tests to run at `f64` (default) and add a case at `Float106`; assert the `f64` path matches the golden output.

## 2. Algorithms — MRMR (`generic-precision-algorithms`)

- [ ] 2.1 Record golden `f64` MRMR rankings on the existing test inputs.
- [ ] 2.2 Make the MRMR result and `mrmr_features_selector` generic over `T: RealField` (+ `FromPrimitive` for sample counts / F-statistic).
- [ ] 2.3 Replace `f64` literals and intermediates with `RealField` constructors and ops.
- [ ] 2.4 Update MRMR tests to run at `f64` (default) and add a case at `f32`; assert the `f64` ranking matches the golden output.

## 3. Discovery — data layer (`generic-precision-discovery`)

- [ ] 3.1 Make the CSV and Parquet loaders produce `CausalTensor<Option<T>>`, converting parsed numbers into `T`; keep `None` for missing/non-numeric cells.
- [ ] 3.2 Make the `DataLoader`, `DataCleaner`, and `DataPreprocessor` traits and their implementations (`OptionNoneDataCleaner`, discretizer, imputer) generic over `T: RealField`, no `dyn`.
- [ ] 3.3 Test loading one CSV at `f64` and at `Float106`; confirm values and `None` placements match.

## 4. Discovery — pipeline and stages (`generic-precision-discovery`)

- [ ] 4.1 Add the precision parameter to the typestate (`CDL<S, T = f64>` or per-state `T`), threading it through `NoData → WithData → WithCleanedData → WithFeatures → WithCausalResults → Finalized`.
- [ ] 4.2 Make the `FeatureSelector`, `CausalDiscovery`, and `ProcessResult` traits and the config types generic over `T`; SURD discovery returns `SurdResult<T>`.
- [ ] 4.3 Make the analyzer (`SurdResultAnalyzer`) and the `ConsoleFormatter` generic over `T`.
- [ ] 4.4 Confirm `T = f64` defaults keep existing pipeline call sites and examples compiling without naming `T`.

## 5. Verification and hygiene

- [ ] 5.1 Add an end-to-end CDL golden test: the SURD pipeline at `f64` produces an identical report before and after the change; add a smoke run at `Float106`.
- [ ] 5.2 `cargo build -p deep_causality_algorithms` and `-p deep_causality_discovery`; `cargo test -p` each; full coverage of changed code.
- [ ] 5.3 Confirm no external numeric crate added, `unsafe_code = "forbid"` intact, no `dyn` introduced.
- [ ] 5.4 Register any new test files in their module trees and `tests/BUILD.bazel`.
- [ ] 5.5 Run `make format && make fix`, then `make build` and `make test` (two crates plus consumers changed).

## 6. Sequencing with the BRCD prep

- [ ] 6.1 Update `brcd-prep-foundations` to declare this change as a prerequisite; restate its discovery-pipeline result as `DiscoveryOutcome<T>` (algorithm-specific enum over the now-generic precision) rather than `SurdResult<f64> → DiscoveryOutcome`.
- [ ] 6.2 Prepare a commit message and request the owner commit.
