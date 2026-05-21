## Why

Precision-as-a-parameter is the established convention across the DeepCausality numerical stack. `deep_causality_tensor::CausalTensor<T>` is generic over `T` with per-method trait bounds. `deep_causality_multivector` is generic over `R: RealField`. `deep_causality_num` exposes the `RealField` and `ComplexField<R>` traits precisely so downstream crates can parameterize over arbitrary floating-point precision — `f32`, `f64`, the upcoming stable `f128`, dual numbers for automatic differentiation, interval arithmetic, arbitrary-precision rationals. `deep_causality_topology::types::gauge::gauge_field::GaugeField<G, M, R>` already follows this pattern with a three-parameter shape.

But the rest of `deep_causality_topology` is inconsistent and pervasively hardcoded. A thorough audit of the crate's public API turned up **~65 distinct `f64` usages across 11 modules**:

- **`CubicalReggeGeometry<D>`** — 8 public methods + private `EdgeLengths<D>` enum (3 of 4 variants) hardcode `f64`.
- **`Manifold` API** — `covariance_matrix() -> Vec<Vec<f64>>`, `eigen_covariance() -> Vec<f64>`, `simplex_volume_squared() -> f64`, plus `D: Into<f64>` and `C: From<f64> + Into<f64>` trait-bound crutches that defeat the existing generic parameters.
- **`ReggeGeometry<T>`** — partially generic but uses `T: Float + From<f64> + Into<f64>` so every internal computation round-trips through `f64`. Curvature helpers (`compute_dihedral_angle`, `calculate_ricci_curvature`) return `f64` directly.
- **`CurvatureTensor`** — every operation (flat tensor construction, index raising / lowering, Ricci, Kretschmann) carries a `T: From<f64> + Into<f64>` bound and pervasively constructs constants via `<T as From<f64>>::from(...)`.
- **`DifferentialForm<T>::scale(scalar: f64)`** — forces the coefficient type `T` to implement `Mul<f64, Output = T>`, locking the form algebra to `f64` scaling.
- **`PointCloud::triangulate`** — Gaussian elimination, Hodge dual, volume computation all bounded on `T: Float + From<f64>` with `From<f64>` constants for tolerances.
- **`GaugeGroup` trait** — the `structure_constant(a, b, c) -> f64` method **on the trait itself**, plus four impls (`SU2`, `SU3`, `SE3`, `SO(3,1)`) hardcoding `f64` returns. Gauge group structure constants flow directly into the gauge action, so the trait return type cannot stay `f64` if the rest of the gauge machinery is generic.
- **`Manifold::differential::{laplacian, codifferential}`** — `From<f64>` bounds for internal tolerances (`1e-12` style constants).
- **Gauge field Metropolis** — `metropolis_step() -> Result<f64, _>` returning the acceptance ratio, plus an internal `let rnd: f64 = rng.random()`.
- **Test utilities** — `create_triangle_complex() -> SimplicialComplex<f64>`, `create_line_complex() -> SimplicialComplex<f64>`.

The two root causes are uniform: **(1) hardcoded `f64` in struct fields, return types, and parameters**, and **(2) trait-bound crutches like `From<f64> + Into<f64>` and `Mul<f64, Output = T>` that pretend to be generic but force every numerical step through `f64`**.

This blocks four lines of upcoming work and one quality-of-life concern:

1. **`add-cubical-regge-calculus-core` (R1–R3)** — Regge-action sums over many hinges accumulate `O(num_hinges · ε)` floating-point error; `f64` is inadequate for high-resolution lattices. Users need to opt into `f128` (or `f32` for memory-bound workloads).
2. **`add-cubical-regge-calculus-analytical` (R4–R6)** — needs both `R: RealField` (volumes, gradients) and `C: ComplexField<R>` (Lorentzian Wick-rotated Regge action). With these traits already in `deep_causality_num`, the analytical change set should never need a `Complex<f64>` shim — but only if topology is parameterized over `R` first.
3. **`add-hodge-decomposition`** — builds on a `HasHodgeStar<R>` trait. If shipped against hardcoded `f64`, every downstream Hodge ⋆ derivation is locked to `f64` forever.
4. **Lattice gauge theory at production precision** — `LatticeGaugeField` already lives on `Arc<LatticeComplex<D>>`. Once cubical Regge geometry couples to it, gauge actions need the same precision parameter as the metric. The `GaugeGroup::structure_constant -> f64` trait method becomes a hard blocker.
5. **Consistency with the rest of the workspace.** `tensor`, `multivector`, and the in-crate `GaugeField<G, M, R>` already parameterize. The remaining `f64` islands in `topology` are an outlier.

Breaking changes are acceptable. The topology crate is pre-1.0; every offending surface was shipped with the understanding that further iteration would happen. This change set is the prerequisite refactor that lets every downstream feature roadmap land against a consistent, precision-parametric surface.

## What Changes

**Invariant after R0 ships:** the public API of `deep_causality_topology` contains **zero** hardcoded `f64` or `f32` in any struct field, function/method signature, trait method, error variant, or trait bound. Every floating-point quantity is `R: RealField` for some `R` chosen by the caller. Every complex-valued quantity (forward-looking — none today, but the door is opened) is `C: ComplexField<R>`. The `From<f64> + Into<f64>` and `Mul<f64, Output = T>` crutches are removed in full.

Concretely, the following surfaces are parameterized:

- **`CubicalReggeGeometry<D>` → `CubicalReggeGeometry<D, R: RealField>`.** Private `EdgeLengths<D>` becomes `EdgeLengths<D, R>`; its `Uniform`, `PerAxis`, `PerEdge` variants carry `R` instead of `f64`. All eight constructors / accessors retype.
- **`ReggeGeometry<T>` → `ReggeGeometry<R: RealField>`.** Renames the type parameter, drops `From<f64> + Into<f64>` from every bound, rewires internals to operate on `R` end-to-end. `compute_dihedral_angle -> Result<R, _>`, `calculate_ricci_curvature -> Result<CausalTensor<R>, _>`, every helper (`compute_determinant`, `compute_area`, etc.) returns `R`.
- **`CurvatureTensor<T, ...>` → `CurvatureTensor<R: RealField, ...>`.** Drops `From<f64> + Into<f64>` from every impl block. Replaces every `<T as From<f64>>::from(...)` site with a `RealField`-native expression or a `<R as FromPrimitive>::from_f64(...).expect("...")` call against the existing `FromPrimitive` trait (see Decision 2 in `design.md`).
- **`Manifold::covariance_matrix -> Result<Vec<Vec<R>>, _>` and `Manifold::eigen_covariance -> Result<Vec<R>, _>`.** `D: Into<f64>` bound is removed; results are returned in the manifold's own scalar type.
- **`Manifold::simplex_volume_squared -> Result<R, _>`.** `C: From<f64> + Into<f64>` bound removed.
- **`Manifold::differential::{laplacian, codifferential, hodge, exterior}` impls** — drop the `From<f64>` bound on internal tolerances; use `R::epsilon()` (already on `RealField`).
- **`DifferentialForm<T>::scale(scalar: R)` where `T: Mul<R, Output = T>`, `R: RealField`.** The `Mul<f64, Output = T>` and `From<f64>` bounds are replaced.
- **`PointCloud::triangulate` and friends** — drop `T: Float + From<f64>` for `R: RealField` (also see Decision 6 on whether `T` and `R` are merged or split here).
- **`GaugeGroup::structure_constant<R: RealField>(a, b, c) -> R` on the trait.** The four impls (`SU2`, `SU3`, `SE3`, `SO(3,1)`) retype to return `R`. The trait gains an `R: RealField` parameter on the method (or on the trait — see Decision 5).
- **`metropolis_step -> Result<R, _>`** for the acceptance ratio. Internal `rng.random::<f64>()` becomes `rng.random::<R>()` against a `RealField`-aware `RandomField` impl (a tiny addition discussed in design.md Decision 7).
- **Test utilities** — `create_triangle_complex<R: RealField>() -> SimplicialComplex<R>`, `create_line_complex<R: RealField>() -> SimplicialComplex<R>`. Existing tests retype call sites to explicit `<f64>`.
- **`deep_causality_num` is not modified.** R0 uses the existing `FromPrimitive` trait (already implemented for `f32`, `f64`, `Float106`) for numeric-literal materialization. Topology generic-code sites bound `R: RealField + FromPrimitive` and call `<R as FromPrimitive>::from_f64(literal).expect("invariant")`. See `design.md` Decision 2 for the pivot rationale (an earlier R0 design attempted to add four constructors to `RealField` and collided with the existing `FromPrimitive` trait).

**Architectural correction (Option 2C):** an earlier R0 draft used a GAT `ChainComplex::Metric<R>` and a struct-level `F: RealField` bound on `Manifold<K, F>`. That design broke two of `Manifold`'s defining capabilities: (a) the full HKT trait surface (`Functor`/`Monad`/`CoMonad`/`Applicative` from `deep_causality_haft`) became unimplementable on stable Rust without modifying haft, because the haft `Type<T> where T: Satisfies<Self::Constraint>` GAT cannot carry a stricter `RealField` bound at the impl; (b) cross-algebra cell data (multivectors from `deep_causality_multivector`, tensors from `deep_causality_tensor`, dual numbers for autodiff, complex numbers for Lorentzian QFT) became unrepresentable because none of those satisfy `RealField`. R0 corrects to a plain associated type on `ChainComplex::Metric` with the metric precision living on the *complex* (`SimplicialComplex<R: RealField>`, `LatticeComplex<const D, R: RealField>`), and `Manifold<K, F>` has no struct-level bound on `F`. Per-impl-block `F: RealField` bounds are added where numerical operations actually need them. This restores HKT and cross-algebra composition without modifying haft. See `design.md` Decision 1 for the full rationale.

**Hard rip-and-replace. No bridge code, no legacy compatibility shims, no type aliases, no parallel `f64`-flavored methods.** Every hardcoded-`f64` surface in `deep_causality_topology` is replaced in place. There is no transitional API surface to maintain.

**Scope is strictly `deep_causality_topology`.** Two follow-up change sets cover the dependent library crates:

- **`generalize-physics-over-realfield`** — `deep_causality_physics` is a major refactor in its own right. A preliminary scan shows the crate carries "`f64` in disguise" — encapsulated wrapper types around concrete `f64` storage — and so deserves a dedicated audit-and-rewrite pass with the same hard-rip-and-replace policy. Drafted as a sibling change set, **priority parity with R0**: R0 must land first (physics depends on topology); physics lands immediately after.
- **`generalize-effects-over-realfield`** — `deep_causality_effects` consumes topology types deeply (4 of 12 `EffectData` variants bake `f64`; `NumericValue` enum has explicit `F32`/`F64` variants; eight bespoke `From<...>` impls form an adapter layer). The enum redesign required is its own scope. **Lower priority** than R0 and physics — can wait.

**Propagation policy (general):** the `R: RealField` parameter flows upward through every dependent **library** crate that touches the generalized type. Library code does **not** pin `R = f64` to absorb the migration; it propagates the parameter further upstream. Only **end-consumer call sites** — binaries (`main.rs`), examples (`examples/`), benchmarks (`benches/`), and workspace integration tests at the consumer edge — bind a concrete `R`. This policy is the same for all three change sets; this proposal only ratifies it for topology.

**During the R0-only gap (R0 shipped, physics and effects not yet migrated):** `deep_causality_physics` and `deep_causality_effects` are temporarily allowed to pin their topology consumption to `::<f64>` at their own internal call sites. This is a documented temporary exception, not a precedent — it expires when each library's own change set lands. R0's task list includes the mechanical turbofish updates to keep the workspace compiling against the new topology surface; the deeper "f64 in disguise" cleanup happens in each library's dedicated change set.

**Downstream proposals** (`add-cubical-regge-calculus-core`, `add-cubical-regge-calculus-analytical`) are updated as a single editorial pass once R0's design is approved, **not** in this change set. The cubical Regge proposals are unshipped, so this is a proposal-doc revision, not a code change.

## Capabilities

### New Capabilities

- `topology-realfield-generic`: The contract that every public-API surface in `deep_causality_topology` is parameterized over `R: RealField` from `deep_causality_num`, with zero hardcoded `f64` or `f32` in any struct field, function signature, trait method, error variant, or trait bound. Covers `CubicalReggeGeometry<D, R>`, `ReggeGeometry<R>`, `CurvatureTensor<R, ...>`, `Manifold<K, F>` and its `covariance` / `geometry` / `differential` APIs, `DifferentialForm<T>::scale`, `PointCloud::triangulate`, the `GaugeGroup` trait and its four impls, the Metropolis step, and the test utilities. Also covers the `from_f64` constructor added to `RealField` in `deep_causality_num`.

### Modified Capabilities

<!-- None at the spec-folder level. The cubical Regge calculus and Hodge decomposition proposals are unshipped (no entry in openspec/specs/), so they cannot be modified specs here. They are updated as a follow-up editorial pass on their proposal docs once R0's design is approved. -->

## Impact

- **Crates affected:**
  - **`deep_causality_topology`** — the entire crate. ~65 public-API surfaces retype.
  - **`deep_causality_num`** — one additive method on the `RealField` trait (`fn from_f64(value: f64) -> Self`). Backwards-compatible at the trait level if shipped with a default impl that panics; preferable to ship without a default and require all impls to provide it (`f32`, `f64` are one-liners; downstream impls in the workspace are equally trivial).
- **Breaking changes (deliberate):** every call site that names a topology type or method with a hardcoded `f64` parameter / return / bound must update. Migration is mechanical (one-line fixes). The workspace audit shows the consumers are entirely in-crate tests, the topology crate's own examples, and a handful of internal helpers in `deep_causality_physics` (lattice gauge theory) and `deep_causality_effects` (effect propagation over manifolds). Each downstream call site adds an explicit `::<f64>` turbofish or migrates to `R = f64`.
- **Source-compatibility migration path:** every breaking-change call site has a one-line fix. The audit's two root causes (hardcoded `f64`, `From<f64>` trait-bound crutches) map to two mechanical rewrites: (a) replace `f64` with `R` and propagate the `<R>` parameter, (b) replace `From<f64>` bounds with `R: RealField + FromPrimitive` and use `<R as FromPrimitive>::from_f64(literal).expect(...)`, `R::epsilon()`, `R::pi()`, etc., in place of `<T as From<f64>>::from(0.5)` style sites.
- **No new public traits within `deep_causality_topology`.** `RealField` and `ComplexField<R>` already live in `deep_causality_num`. R0 introduces zero new topology traits; it threads existing trait bounds.
- **No trait surface change.** R0 uses the existing `FromPrimitive` trait from `deep_causality_num`. Documented in detail in `design.md` Decision 2.
- **`GaugeGroup::structure_constant<R: RealField>(...) -> R`** is a method-level generic added to the existing trait. Implementors must update; the four in-crate impls are mechanical.
- **Tests:** every `SimplicialComplex<f64>` / `CubicalReggeGeometry<D>` test in the crate retypes to `…<f64>` (turbofish-explicit). New parameterized tests exercising at least one second precision (recommendation: `f32` to make precision loss visible; `f128` slots in for free once stable) are added per module group under `tests/`.
- **BUILD.bazel:** no new test folder modules — all changes are in existing locations.
- **Effort estimate:** ~600–900 LOC of refactor on the production surface (mostly trait-bound and signature changes; the bodies stay structurally identical after the `From<f64>` round-trip removal); ~200 LOC of test-call-site updates; ~25–40 property tests at `R = f32` to gate the parameterization is real. Total ~1000–1200 LOC touched, ~30 new tests, ~15–20 hours of focused work spread across the seven module groups.
- **Sequencing:** **R0 is the precondition for the entire cubical Regge roadmap and for any future precision-sensitive work in topology.** `add-cubical-regge-calculus-core`, `add-cubical-regge-calculus-analytical`, `add-hodge-decomposition`, and any precision-aware lattice-gauge-theory work in `deep_causality_physics` all depend on this change set.
- **What is NOT in scope:** introducing new functionality (everything is a refactor of existing surfaces); performance tuning (the refactor must match `R = f64` performance, but no new optimizations); changing the algorithmic content of any method (Cayley-Menger, Hodge ⋆, deficit angles, etc. compute the same values bit-identically at `R = f64`); generalizing over `ComplexField<R>` where complex values do not yet appear in the public API (Lorentzian Wick rotation, eigenvalues of asymmetric matrices, etc. land in their respective downstream change sets — R0 only prepares the door).
- **Reference:** workspace-wide audit on `deep_causality_topology` public API (`f64` surfaces categorized P0–P3, ~65 locations across 11 modules), conversation context 2026-05-21. The trim "cubical-Regge-path only" scope considered earlier in the conversation is rejected — the full-crate sweep is the correct scope per user direction "zero hard-coded dependency on any specific float types".
