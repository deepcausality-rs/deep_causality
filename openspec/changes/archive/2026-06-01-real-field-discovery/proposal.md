## Why

`deep_causality_algorithms` and `deep_causality_discovery` hardwire `f64` (≈213 and ≈96 source references). The rest of the numerical stack — `deep_causality_tensor`, `deep_causality_multivector`, `deep_causality_topology`, `deep_causality_physics` — already generalizes over the `RealField` abstraction in `deep_causality_num`, so a user picks precision at the call site with a single type alias (see `examples/mathematics_examples/composable_multi_math/effect_tensor_algebra_roundtrip/main.rs`, which switches `f32`/`f64`/`Float106` by changing one line).

Two reasons to close the gap now. First, cohesion: these are the last two crates outside the shared `RealField` paradigm. Second, practicality: Rust is adding `f16` and `f128`; together with `f32`, `f64`, and the project's `Float106` that is five real types, which makes precision-as-a-parameter concretely useful rather than theoretical.

This change is the prerequisite for the BRCD work. Doing it first means the BRCD preparatory foundations (`brcd-prep-foundations`) are generic over precision from the start, instead of being written against `f64` and re-generified later.

## What Changes

- **Algorithms** (`deep_causality_algorithms`). Make SURD and MRMR generic over `T: RealField`. `SurdResult<f64>` becomes `SurdResult<T>`; the MRMR result becomes generic; public functions (`surd_states_cdl`, `mrmr_features_selector`, and the rest) gain the precision parameter. The `f64` numerical results are preserved exactly.
- **Discovery** (`deep_causality_discovery`). Make the whole CDL pipeline generic over `T: RealField`: the data loaders (CSV, Parquet) produce `CausalTensor<Option<T>>`; the data cleaners, preprocessors, feature selector, causal-discovery stage, analyzer, and formatter all carry `T`; the typestate states and every stage trait (`DataLoader`, `DataCleaner`, `DataPreprocessor`, `FeatureSelector`, `CausalDiscovery`, `ProcessResult`) and the configs gain the precision parameter. Precision is chosen at the call site, as in the math stack. SURD and MRMR results at `T = f64` stay identical.
- **BREAKING** at the source-API level of both crates: public types and functions gain a precision type parameter. Behavior at `f64` is unchanged.

Non-goals: no new algorithms, no behavior change at `f64`, and none of the BRCD preparatory capabilities (those stay in `brcd-prep-foundations`, which sequences after this change).

## Capabilities

### New Capabilities
- `generic-precision-algorithms`: SURD and MRMR generic over `T: RealField`, with `f64` results preserved.
- `generic-precision-discovery`: the CDL discovery pipeline — loaders, cleaners, preprocessors, feature selection, discovery, analysis, formatting, typestate, traits, and configs — generic over `T: RealField`, with precision chosen at the call site and `f64` behavior preserved.

### Modified Capabilities
<!-- None. No existing capability spec in openspec/specs/ governs the algorithms or discovery crates; this is a generification of crates not previously spec'd at the requirements level. -->

## Impact

- **Crates touched.** `deep_causality_algorithms` (SURD, MRMR) and `deep_causality_discovery` (entire pipeline). Both depend already on `deep_causality_num::RealField` and the generic `CausalTensor<T>`; no new dependency is added.
- **Consumers.** Examples and CDL users gain a precision parameter; a `T = f64` default on generic types keeps existing call sites compiling (see design).
- **Sequencing.** This change lands **before** `brcd-prep-foundations`. That change's pipeline generalization (two datasets, an algorithm-specific result enum) then builds on the now-generic crates as `DiscoveryOutcome<T>`. `brcd-prep-foundations` must be updated to depend on this change.
- **Risk.** A large, mostly mechanical refactor (~300 `f64` sites). The non-mechanical parts are the `f64`-literal to `T`-constant conversion and parsing source data into `T`; both are addressed in design.
- **Constraints preserved.** `unsafe_code = "forbid"`, static dispatch (no `dyn`), and no external numeric crates.
