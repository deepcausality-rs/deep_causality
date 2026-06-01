## 0. Numeric foundations (Option A — added during apply)

The original plan assumed `RealField` already exposed every op SURD/tensor math needs and
that `CausalTensorMathExt` was generic. Neither held: the trait lacked `log2`/`log10`, and the
tensor math extension had hand-written `f32`/`f64` impls. These had to land first.

- [x] 0.1 Add `log2` and `log10` to the `RealField` trait with `f32`/`f64` impls (native std / `libm` no_std) and a `Float106` impl.
- [x] 0.2 Add 100% test coverage for the new `log2`/`log10` across `f32`, `f64`, and `Float106`.
- [x] 0.3 Replace the `f32`/`f64`-specific `CausalTensorMathExt` impls with a single generic `impl<T: RealField + FromPrimitive> CausalTensorMathExt<T> for CausalTensor<T>`; delete the two concrete impl files (owner-authorized).

## 1. Algorithms — SURD (`generic-precision-algorithms`)

- [x] 1.1 Record golden `f64` outputs for SURD on the existing test inputs before any change.
- [x] 1.2 Make `SurdResult` generic as `SurdResult<T>` (**no default type parameter** — precision is always chosen explicitly at the pipeline entry); update its fields and accessors to `T`.
- [x] 1.3 Make the SURD core and `surd_states_cdl` generic over `T: RealField + FromPrimitive + Default + Send + Sync`; replace `f64` literals with `T::zero()`/`T::one()`/`T::from_f64(..)` and use `RealField` ops for the entropy/log math.
- [x] 1.4 Add `FromPrimitive` alongside `RealField` only where integer-to-real conversion is needed (counts, orders). `Default` is required by `sum_axes`.
- [x] 1.5 Update SURD tests to pin `f64` via annotations (the now-generic fns no longer infer it) and confirm the `f64` path matches the golden output; `Float106` exercised end-to-end via the example.

## 2. Algorithms — MRMR (`generic-precision-algorithms`)

- [x] 2.1 MRMR was **already generic** over `T: Float, Option<T>: FloatOption<T>` — no rewrite needed.
- [x] 2.2 Verified `mrmr_features_selector` composes under the new pipeline bounds (`T: Precision + Float`, `Option<T>: FloatOption<T>`) at `Float106` end-to-end.
- [x] 2.3 No literal/intermediate changes required.
- [x] 2.4 Existing MRMR tests remain green; precision switch verified via the discovery example.

## 3. Discovery — data layer (`generic-precision-discovery`)

- [x] 3.1 Loaders read files as `f64`, then `load_data`/`load_tensor` cast once into `T` via `cast_loaded_tensor<T: Precision>` (NaN preserved as `T::nan()`); native high-precision input accepted directly through `load_tensor(CausalTensor<T>)`.
- [x] 3.2 Make the `DataLoader`, `DataCleaner`, and `DataPreprocessor` traits and their impls (`OptionNoneDataCleaner`, `DataDiscretizer`, `MissingValueImputer`) generic over `T`, no `dyn`.
- [x] 3.3 Discovery tests load CSV at `f64`; `Float106` exercised through the example pipeline.

## 4. Discovery — pipeline and stages (`generic-precision-discovery`)

- [x] 4.1 Thread precision through the typestate per-state (`WithData<T>`, `WithCleanedData<T>`, `WithFeatures<T>`, `WithCausalResults<T>`, `WithAnalysis<T>`) — **no default `T`**, and no phantom `T` on `NoData`/`Finalized` where there are no reals. Introduced the `Precision` marker trait (`RealField + FromPrimitive + Default + Send + Sync` + blanket impl) in `traits/precision.rs`.
- [x] 4.2 Make the `FeatureSelector`, `CausalDiscovery`, and `ProcessResult` traits and the config types generic over `T`; SURD discovery returns `SurdResult<T>`.
- [x] 4.3 Make the analyzer (`SurdResultAnalyzer`) and the `ConsoleFormatter` generic over `T`; the analyzer compares thresholds in `T`-space and only renders via `to_f64()`.
- [x] 4.4 Precision is named once at the call site (`load_data::<FloatType>`); the example uses a `FloatType` alias so the precision switch is a one-line edit.

## 5. Verification and hygiene

- [x] 5.1 The discovery example runs the full SURD pipeline at `Float106` (load → clean → MRMR → SURD → analyze → finalize); 305 discovery tests pin the `f64` behavior.
- [x] 5.2 `cargo build` / `cargo test` green for `deep_causality_num`, `deep_causality_tensor`, `deep_causality_algorithms`, `deep_causality_discovery`; new code covered.
- [x] 5.3 No external numeric crate added, `unsafe_code = "forbid"` intact, no `dyn` introduced.
- [x] 5.4 Test files registered in their module trees.
- [x] 5.5 Full workspace `cargo build --workspace --all-targets`, `cargo test --workspace`, `cargo fmt --all --check`, and `cargo clippy --workspace --all-targets` all clean.

## 6. Sequencing with the BRCD prep

- [ ] 6.1 Update `brcd-prep-foundations` to declare this change as a prerequisite; restate its discovery-pipeline result as `DiscoveryOutcome<T>` (algorithm-specific enum over the now-generic precision) rather than `SurdResult<f64> → DiscoveryOutcome`.
- [ ] 6.2 Prepare a commit message and request the owner commit.
