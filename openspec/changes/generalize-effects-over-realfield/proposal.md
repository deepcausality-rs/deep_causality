## Why

`deep_causality_effects` is the heterogeneous-container layer that bundles topology, multivector, and tensor data for effect propagation through causal graphs. Its current public surface bakes `f64` into the load-bearing types in three ways:

1. **`EffectData` enum** (`src/types/effect_data/mod.rs`) hardcodes `f64` in 4 of its 12 variants: `Float(f64)`, `MultiVector(CausalMultiVector<f64>)`, `Tensor(CausalTensor<f64>)`, `PointCloud(PointCloud<f64, f64>)`, `SimplicialComplex(SimplicialComplex<f64>)`, `Manifold(SimplicialManifold<f64, f64>)`. These are first-class variant payloads; every pattern-match site sees concrete `f64`.
2. **`NumericValue` enum** (`src/types/numeric_value/mod.rs`) is an explicit polymorphic numeric wrapper with separate `F32(f32)` and `F64(f64)` variants alongside the integer cases (`U8` through `U128`, `I8` through `I128`). The pattern collides directly with `R: RealField` — generalizing means deciding whether to keep both variants (run-time dispatch), collapse to a single `Real(R)` (compile-time dispatch via generic), or split the type entirely.
3. **Eight bespoke `From<...>` impls** (`From<f64>`, `From<f32>`, `From<NumericValue>`, `From<CausalMultiVector<f64>>`, `From<CausalTensor<f64>>`, `From<PointCloud<f64, f64>>`, `From<SimplicialComplex<f64>>`, `From<SimplicialManifold<f64, f64>>`) form an adapter layer between producers and `EffectData`. Each impl is shape-specific; the change set must re-derive each one against the generic surface.

This is unlike the wrapper-newtype pattern in `deep_causality_physics`. There the migration is mechanical (rename `T` to `R`, change bounds, retype constants). Here it requires **enum redesign**: the `EffectData` and `NumericValue` variants are part of the type's contract, and the right precision-parametric shape is a design question, not a refactor.

Three concrete blockers without this work:

1. **R0 (`generalize-topology-over-realfield`) installs temporary `::<f64>` pins** at every effects→topology call site, tagged `// TEMP: removed by generalize-effects-over-realfield`. Those pins exist for one purpose — keeping the workspace compiling — and stay until this change set ships.
2. **`generalize-physics-over-realfield`** installs similar pins at every effects→physics consumption site (the audit found these are mediated through `EffectData::Custom(Arc<dyn Any>)`, but any downcast site that names a physics type pins to `::<f64>`).
3. **Effect propagation at non-`f64` precision is impossible today.** A causal graph that integrates `f32` voxel-grid topology data with `f64` simplicial data should be expressible; the current `EffectData` enum forces every effect to be `f64` regardless of its actual provenance.

The user has flagged this work as **lower priority** than R0 and physics. It can wait until the upstream change sets land. This proposal documents the intended shape of the migration so the temporary pins introduced by R0 and physics are removable on a known schedule.

## What Changes

The shape of the migration is not yet fully decided — see `design.md` for the three candidate enum-redesign strategies (collapse to single `Real(R)`, keep both `F32`/`F64` variants for run-time dispatch, split the type). The change set is scoped at the proposal level; specs and tasks are deferred until the enum-design decision is committed.

The following are settled regardless of the enum-redesign choice:

- **`EffectData` becomes `EffectData<R: RealField>`.** The four `f64`-baked variants retype: `Float(R)`, `MultiVector(CausalMultiVector<R>)`, `Tensor(CausalTensor<R>)`, `PointCloud(PointCloud<R, R>)`, `SimplicialComplex(SimplicialComplex<R>)`, `Manifold(SimplicialManifold<R, R>)`. The remaining variants (`Bool`, `Int(i64)`, `String`, `Vector`, `Custom(Arc<dyn Any>)`) are unchanged.
- **The eight `From<...>` impls retype** to consume `R`-parameterized inputs. Each impl is rewritten in place.
- **`NumericValue` is redesigned.** The exact shape is a Decision in `design.md` — likely to collapse the `F32` and `F64` variants into a single `Real(R)` variant under a new `NumericValue<R: RealField>` generic, with the integer variants preserved.
- **Cross-crate consumption propagates `R: RealField`.** Every R0 temp pin (`// TEMP: removed by generalize-effects-over-realfield`) and every physics-change-set temp pin (`// TEMP: removed by generalize-effects-over-realfield`) is removed; `R` flows through naturally.
- **Hard rip-and-replace policy applies.** No bridge code, no type aliases, no parallel `f64`-flavored variants, no `pub const` survivors. Same as R0 and physics.
- **Propagation policy applies.** Library-level effects code stays generic over `R: RealField`; only end-consumer call sites (binaries, examples, benchmarks) bind a concrete `R`.

## Capabilities

### New Capabilities

- `effects-realfield-generic`: The contract that every public-API surface in `deep_causality_effects` is parameterized over `R: RealField` from `deep_causality_num`, with zero hardcoded `f64` or `f32` in any struct field, function signature, trait method, error variant, trait bound, or `pub const` declaration. Covers `EffectData<R>`, `NumericValue<R>` (final shape determined by `design.md`), and the eight cross-crate `From<...>` adapter impls.

### Modified Capabilities

<!-- None at the spec-folder level. -->

## Impact

- **Crate affected:** `deep_causality_effects` only. R0 and `generalize-physics-over-realfield` must have shipped first.
- **Cross-crate dependencies:** consumes the now-generic `deep_causality_topology`, `deep_causality_physics`, `deep_causality_tensor`, `deep_causality_multivector`.
- **Breaking changes (deliberate):** every consumer that pattern-matches on `EffectData` or `NumericValue` updates. The shape of the update depends on the `NumericValue` redesign decision (see design.md).
- **Effort estimate:** depends on the enum-design choice. Two of three candidates are ~4–6 hours; the third (split type) is ~10 hours.
- **Sequencing:** **R0 and `generalize-physics-over-realfield` must ship first.** This change set is the lowest-priority of the three; it can wait until the upstream change sets land and stabilize.
- **What is NOT in scope:** new effect-propagation functionality; redesigning the `EffectData::Custom(Arc<dyn Any>)` boundary (out of scope — `dyn Any` is the documented escape hatch); changing the algorithmic content of effect propagation.
- **Reference:** `deep_causality_effects` audit (conversation context 2026-05-21): 2 enum types with f64-baked variants, 8 cross-crate `From` impls, ~30 `f64`/`f32` public-API locations.
