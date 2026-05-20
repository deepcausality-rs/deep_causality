## Context

`deep_causality_num` exposes the `RealField` trait (an ordered field with transcendentals — `sin`, `cos`, `exp`, `ln`, `sqrt`, `pi`, `e`, `epsilon`) and `ComplexField<R: Field>` (a field with conjugation and real / imaginary projections). Together they cover every floating-point use case in the numerical stack: real-valued quantities use `R: RealField`; complex-valued quantities use `C: ComplexField<R>`. `f32`, `f64`, and the upcoming stable `f128` all implement `RealField`; the in-house `Complex<R>` and `Quaternion<R>` types layer on top.

The cross-crate convention is to parameterize over these traits at the *struct* level so precision is a deployment-time choice, not a hard-coded library choice:

- `CausalTensor<T>` in `deep_causality_tensor` is fully generic over `T` and adds per-method trait bounds where operations require them.
- `deep_causality_multivector` is generic over `R: RealField`.
- `deep_causality_topology::types::gauge::gauge_field::GaugeField<G, M, R>` is the existing in-crate exemplar — three parameters: gauge group, matrix algebra, real scalar.

The rest of `deep_causality_topology` does not follow this convention. A workspace audit identified ~65 distinct `f64` usages across 11 modules. The audit's two root causes are uniform: **hardcoded `f64` in struct fields, return types, and parameters**, and **trait-bound crutches like `From<f64> + Into<f64>` and `Mul<f64, Output = T>` that pretend to be generic but force every numerical step through `f64`**.

R0 fixes this for `deep_causality_topology` only. The two follow-up change sets (`generalize-physics-over-realfield`, `generalize-effects-over-realfield`) apply the same audit-and-rewrite pattern to the dependent library crates. Each follow-up is scoped to its own surface; R0 does not absorb their migrations.

Stakeholders: anyone implementing the cubical Regge calculus roadmap (R1–R6), the Hodge decomposition follow-up, or any precision-sensitive lattice-gauge-theory work; anyone needing `f32` for memory-bound workloads; anyone waiting for `f128` to stabilize. Today none of those are possible against `deep_causality_topology`; after R0 ships, all of them are one type-parameter away.

## Goals / Non-Goals

**Goals:**

- After R0 ships, the public API of `deep_causality_topology` contains **zero** hardcoded `f64` or `f32` in any struct field, function/method signature, trait method, error variant, or trait bound.
- Every floating-point quantity in the crate's public API is `R: RealField` for some `R` chosen by the caller.
- The `From<f64> + Into<f64>` and `Mul<f64, Output = T>` trait-bound crutches are removed in full; their replacement is `R: RealField` plus a small additive `RealField::from_f64` constructor (Decision 2).
- Internal computation operates on `R` end-to-end with no `f64` round-trips. Every `<T as From<f64>>::from(literal)` site is rewritten to a `RealField`-native expression or `R::from_f64(literal)`.
- The `GaugeGroup` trait's `structure_constant` method generalizes to return `R: RealField`. All four in-crate impls (`SU2`, `SU3`, `SE3`, `SO(3,1)`) retype.
- The internal `let rnd: f64 = rng.random()` in the Metropolis step generalizes to `R`. A small `RandomField` impl path lands so an `R: RealField` random sample is well-defined.
- The behavior at `R = f64` is bit-identical to the pre-R0 baseline. Every existing test passes after retyping call sites to explicit `<f64>`.
- A second-precision property-test pass at `R = f32` exercises the parameterization is real, not nominal. Catches silent `.into()` round-trips the type system misses.

**Non-Goals:**

- New functionality. R0 changes types and trait bounds; method bodies preserve their algorithmic content.
- Touching `deep_causality_physics` or `deep_causality_effects` beyond the minimum turbofish patches needed to keep the workspace compiling. Their deeper "f64 in disguise" cleanup is the scope of their own dedicated change sets.
- Performance optimization beyond preserving the existing `R = f64` baseline. Any speed-ups are coincidental.
- Generalizing over `ComplexField<R>`. No complex-valued quantity exists in the public API today; R0 only opens the door for downstream change sets to add them.
- Type aliases like `CubicalReggeGeometry64<const D: usize>` for ergonomics. Explicitly rejected — bridge code is bridge code.
- Default type parameters like `<R = f64>` on any generalized type. Explicitly rejected — defaults hide the precision choice.
- Performance improvements to `RealField` impls in `deep_causality_num`. The `from_f64` addition is the only cross-crate change.

## Decisions

### Decision 1: One `R: RealField` parameter per generalized type, threaded into every method

Every generalized struct / enum carries exactly one real-field parameter. Examples:

- `CubicalReggeGeometry<D, R: RealField>` — replaces `CubicalReggeGeometry<D>`. The private `EdgeLengths<D, R>` enum's `Uniform`, `PerAxis`, `PerEdge` variants carry `R` in place of `f64`.
- `ReggeGeometry<R: RealField>` — renamed from `ReggeGeometry<T>`. The `Float + From<f64> + Into<f64>` bound is replaced with `R: RealField`.
- `CurvatureTensor<R: RealField, ...>` — renamed from `CurvatureTensor<T, ...>`. Same bound replacement.
- `DifferentialForm<T, R: RealField>` — adds the `R` parameter; `T: Mul<R, Output = T>` replaces `T: Mul<f64, Output = T>`.

**Why one parameter, not separate `R_lengths`, `R_angles`, etc.:** these values are connected by closed forms. Splitting the parameter creates unenforceable consistency constraints; one parameter ensures all derived quantities live in the same field by construction.

**Why no default `R = f64`:** the entire point is to make precision a *choice*. A default of `f64` would let new call sites silently fall back to `f64` and accumulate the same precision loss the parameter is meant to solve.

### Decision 2: Add `RealField::from_f64(value: f64) -> Self` to `deep_causality_num`

The current `RealField` trait exposes `pi`, `e`, `epsilon`, and the transcendentals, but no general-purpose constructor for arbitrary numeric literals. Several internal sites in `deep_causality_topology` materialize constants like `1e-12` (epsilon tolerances), `2.0` (binary half), `8πG` (Einstein-Hilbert prefactor), etc., that aren't reachable through `R::one()`, `R::pi()`, `R::epsilon()` alone.

R0 adds one method to the `RealField` trait:

```rust
pub trait RealField: /* existing bounds */ {
    // ... existing methods ...

    /// Constructs a value from an `f64` literal. Used for numeric constants
    /// that cannot be expressed through `one()`, `pi()`, `e()`, or `epsilon()`.
    fn from_f64(value: f64) -> Self;
}

impl RealField for f32 { fn from_f64(v: f64) -> Self { v as f32 } /* ... */ }
impl RealField for f64 { fn from_f64(v: f64) -> Self { v } /* ... */ }
```

**No default impl.** Every implementor provides the method explicitly. `f32` and `f64` impls are one-line casts. Future impls (`f128`, dual numbers, fixed-point, arbitrary precision) implement the trivial conversion.

**Why this and not `R::from_i64`, `R::two()`, `R::half()`, etc.:** `from_f64` is the minimum sufficient surface. The alternatives are all derivable from it; offering all of them would be API surface inflation. The cost is one method on a trait that already has ~30 methods.

**Why not `R: From<f64>` as a bound where needed?** That's the exact crutch we're removing. `From<f64>` is a marker trait that any wrapper type can claim; `from_f64` lives on `RealField` itself, anchoring the conversion to the precision-parametric abstraction.

**Why not a separate `FromLiteral` trait?** Premature decomposition. One method on `RealField` is sufficient and keeps the dependency footprint minimal.

### Decision 3: Drop `From<f64> + Into<f64>` from every existing generic bound; rewire internals

`ReggeGeometry<T>` bounds `T: Float + Zero + Copy + PartialOrd + From<f64> + Into<f64>`. The `From<f64> + Into<f64>` half exists because internal numerical algorithms were written against `f64` and round-trip through it. `CurvatureTensor`, `PointCloud::triangulate`, and `Manifold` geometry / covariance helpers have the same pattern.

R0 rewrites those internals to operate on `R: RealField` end-to-end:

- `arctan2`, `sqrt`, `powf`, `sin`, `cos`, etc. are all on `RealField`.
- Constants like `R::pi()`, `R::e()`, `R::epsilon()` come from the trait.
- Numeric literals come from `R::from_f64(literal)`.
- Algebraic constants like `R::one() + R::one()` for `2`, or `R::one() / (R::one() + R::one())` for `0.5`, are used where they read more naturally than `R::from_f64(2.0)`.

After the rewrite, every bound becomes `R: RealField` (or `T: Mul<R, Output = T>, R: RealField` for the scale-shaped variants). The `Float + Zero + Copy + PartialOrd` superset is satisfied by `RealField` directly.

**Acceptance test:** grep the crate for `From<f64>`, `Into<f64>`, `as f64`, and `.into()` patterns post-R0. Zero hits in `deep_causality_topology/src/`.

### Decision 4: `GaugeGroup::structure_constant` takes `R: RealField` as a method-level generic

The `GaugeGroup` trait currently declares:

```rust
fn structure_constant(_a: usize, _b: usize, _c: usize) -> f64 { 0.0 }
```

This is a hard `f64` lock on the entire gauge-group abstraction. R0 changes the method signature:

```rust
fn structure_constant<R: RealField>(a: usize, b: usize, c: usize) -> R;
```

**Method-level generic, not trait-level.** A trait-level parameter `trait GaugeGroup<R: RealField>` would force each implementor to pick its precision at trait-impl time; a method-level generic lets `SU2` etc. produce structure constants at whatever precision the caller asks for. The four in-crate impls (`SU2`, `SU3`, `SE3`, `SO(3,1)`) each implement the method via `R::from_f64(literal)` for their hardcoded coefficient values.

**No default impl.** The current `0.0` default disappears — implementors must provide the structure constants explicitly. Compile-time forcing is strictly better than the runtime "zero" silent failure for an Abelian-group misuse.

### Decision 5: `Manifold` API generalizes per-impl-block, not per-struct

`Manifold<K, F>` is already generic over the complex `K` and the field-data type `F`. The `f64` leakage is concentrated in *impl blocks* that bind `K = SimplicialComplex<C>` with `C: From<f64> + Into<f64>` and `D: Into<f64>` bounds:

```rust
// Before:
impl<C, D> Manifold<SimplicialComplex<C>, D>
where C: Float + From<f64> + Into<f64> + ..., D: Into<f64> + Copy
{
    pub fn covariance_matrix(&self) -> Result<Vec<Vec<f64>>, _> { ... }
}

// After:
impl<C, D, R> Manifold<SimplicialComplex<C>, D>
where C: RealField, D: RealField, R: RealField, /* ... */
{
    pub fn covariance_matrix(&self) -> Result<Vec<Vec<R>>, _> { ... }
}
```

**Where does `R` come from?** Two options:
- **(a)** `R` is a method-level generic that the caller binds via turbofish: `manifold.covariance_matrix::<f64>()`.
- **(b)** `R` is bound to one of the existing struct parameters (typically `D`, the field-data type): `R = D`.

R0 picks **(b)** wherever the result is naturally in the same field as the input. Covariance / volume / curvature outputs are in `D`'s field. Where the output is genuinely independent (e.g. a numerical tolerance returned from a check), method-level `R` is used.

**Rationale:** option (b) is invisible to callers (the type flows from existing inputs) and matches `tensor`'s convention. Option (a) is reserved for the small set of methods where the output is precision-independent of the inputs.

### Decision 6: `PointCloud::triangulate` and friends generalize the existing `T` parameter

`PointCloud::triangulate` is bounded `T: Float + Sum + From<f64> + ...`. The `T` parameter is the field-data type of the point cloud. R0 changes the bound to `T: RealField`, removes the `From<f64>` half, and rewrites the internal Gaussian elimination / Hodge dual / volume helpers to use `T::from_f64(...)` for the few literal constants they need (epsilon tolerances, fixed coefficients).

No new type parameter is introduced. The existing `T` is widened from "any `Float` with f64 round-trip" to "any `RealField`".

### Decision 7: Metropolis acceptance ratio and RNG output are `R: RealField`

`metropolis_step -> Result<f64, _>` becomes `metropolis_step -> Result<R, _>` where `R` is the gauge field's real-scalar parameter (already exposed via `GaugeField<G, M, R>`).

The internal `let rnd: f64 = rng.random()` becomes `let rnd: R = R::from_f64(rng.random::<f64>())`. The RNG is consulted at `f64` precision (cheap, fast, and the source of the random bits doesn't need to be `R`-aware) and the result is converted up. **This is the one allowed `f64` literal in the entire crate post-R0** — it is internal, not part of the public API, and reflects the practical reality that `deep_causality_rand`'s RNG primitives produce `f64`.

If a future change set wants RNG output natively at `R` precision (for `f128`-resolution Monte Carlo), the `R::from_f64(...)` site is the single point to revise.

### Decision 8: Test utilities go generic; tests retype call sites to `::<f64>` explicitly

`create_triangle_complex<R: RealField>() -> SimplicialComplex<R>` and `create_line_complex<R: RealField>() -> SimplicialComplex<R>`. Every existing test call site adds an explicit `::<f64>()` turbofish.

**No pre-baked `_f64` aliases.** The turbofish at the call site is the migration's documentation — "this test runs at this precision."

New parameterized tests at `R = f32` are added per module group to exercise the abstraction. `f32`-specific tolerance widening (`R::epsilon() * <factor>`) is used where the `f64` test passes at `1e-12` and the `f32` test reasonably passes only at `1e-6`.

### Decision 9: No bridge code, no parallel APIs, no deprecation paths

R0 is a hard rip-and-replace. Specifically:

- No type aliases (`CubicalReggeGeometry64<D>` etc.) — rejected.
- No default type parameters (`<R = f64>` on any type) — rejected.
- No `#[deprecated]` `f64`-returning methods alongside their generic replacements — rejected.
- No `From<f64>` impls on generalized topology types to ease migration — rejected.
- No "f64 convenience layer" or "old API still works" mode — rejected.

Downstream call sites within `deep_causality_topology` (tests, examples) migrate by setting `::<f64>` at their construction sites. Downstream library crates (`deep_causality_physics`, `deep_causality_effects`) temporarily pin their topology consumption to `::<f64>` until their own change sets land; this is the single permitted exception, documented in the proposal and removed when each follow-up ships.

### Decision 10: Property-test gate at `R = f32`

The `R = f64` test suite must pass bit-identically to pre-R0. Additionally, every algorithmically-meaningful test (cell volume, dihedral angle, Cayley-Menger volume, curvature tensor invariants, Hodge ⋆ identities once R4 ships, Gaussian elimination determinants) is duplicated against `R = f32` with widened tolerances.

If an `f32` duplicate fails where the `f64` original passes by a wide margin, it indicates a hidden `f64` literal or `.into()` round-trip survived the refactor. The `f32` pass is a cheap insurance policy.

**Test layout:** the duplicates live alongside the `f64` originals in the same test file, named `_f32` (e.g. `cell_volume_unit_grid_f64` and `cell_volume_unit_grid_f32`). Macro-generated when the test logic is identical and the only difference is the precision and tolerance.

## Risks / Trade-offs

- **[Risk] Workspace consumers (`deep_causality_physics`, `deep_causality_effects`) won't compile after R0 unless their topology call sites are patched.**
  → **Mitigation:** R0's task list includes a final task group that adds `::<f64>` turbofish at every `physics` / `effects` call site that names a topology type. This is the minimum mechanical patch to keep CI green. Each library's deeper "f64 in disguise" cleanup is its own change set. The temporary `::<f64>` pins are tagged `// TEMP: removed by generalize-{physics,effects}-over-realfield` to be greppable.

- **[Risk] `RealField::from_f64` round-trips lose precision when `R` has higher precision than `f64`.** Calling `f128::from_f64(some_literal)` extends an `f64` to `f128`, but the bits beyond `f64` precision are zero — the value is the `f64` approximation lifted, not the true `f128` value of the source literal.
  → **Mitigation:** documented in the doc comment of `from_f64`. The method is a convenience for materializing constants from `f64` literals (the standard Rust literal type); callers who need true higher-precision constants use the precision's native constructor or `RealField::pi() / R::e() / R::from_f64` of derived expressions. The vast majority of `from_f64` call sites in `deep_causality_topology` are tolerance constants (`1e-12`) where `f64` precision is already overkill.

- **[Risk] Performance regression vs. hand-tuned `f64` paths.** `RealField` method calls go through trait dispatch. The Rust monomorphizer should inline to identical machine code at `R = f64`, but in rare cases (cross-crate generic instantiation behind `#[inline]` gates) it does not.
  → **Mitigation:** R0's verification task runs the existing `f64` benchmarks against the new generic surface at `R = f64` and asserts no measurable regression (>2%). If a regression appears, add `#[inline]` to the hot `RealField` impl methods on `f64` in `deep_causality_num`.

- **[Risk] The `GaugeGroup::structure_constant` method-level generic creates a turbofish burden at every call site.** Callers of `SU3::structure_constant(a, b, c)` must now write `SU3::structure_constant::<R>(a, b, c)` or rely on context-driven inference.
  → **Mitigation:** in practice every call site is inside a generic function or impl block that already has an `R: RealField` parameter in scope; inference resolves `R` automatically. The audit found ~5 internal call sites; all of them are in generic contexts. If a future call site needs the turbofish, it's a one-line addition.

- **[Risk] `Manifold` covariance / eigenvalue routines might depend on `f64`-only numerical libraries internally.** If `Manifold::eigen_covariance` calls a third-party eigenvalue solver that's `f64`-only, R0's generic return type is a lie — the result is always computed at `f64` and converted.
  → **Mitigation:** audit during implementation. If a true `f64`-only dependency exists, the conversion is `R::from_f64(eigenvalue_as_f64)` at the boundary, and a doc-comment flags the internal precision floor. This is acceptable for an internal-precision-limit case (the caller asked for `R`, the algorithm provided what it could). The crate's own algorithms (Cayley-Menger, deficit angles, Hodge ⋆) have no such dependency.

- **[Risk] The breaking changes ripple wider than expected.** A workspace-wide grep may find consumers we haven't audited.
  → **Mitigation:** R0's first task is `cargo build --workspace` after the refactor; every compile error is a missed call site. The compile-error list IS the migration checklist for downstream patches.

- **[Trade-off] R0 is pure overhead from a feature perspective.** No new user-visible capability ships; only the precondition refactor.
  → **Justification:** see proposal. The alternative (ship feature work against the hardcoded `f64` surface and refactor later) costs more in total work because every feature-change breaks again when the parameterization lands.

- **[Trade-off] R0 ships with no type aliases and no defaults.** Every call site that previously wrote `CubicalReggeGeometry<3>` now writes `CubicalReggeGeometry<3, f64>`. Mild ergonomic cost.
  → **Justification:** see Decision 9. Bridge code is bridge code; the cost of explicitness is small and one-time.

## Migration Plan

1. **`deep_causality_num`:** add `RealField::from_f64`. One-line additions to `f32` and `f64` impls. No default impl on the trait. Verify nothing else in `deep_causality_num` needs touching.
2. **`deep_causality_topology` — schema pass:** add `<R: RealField>` parameters and retype signatures. The bodies stay structurally identical at this stage — anywhere they reference an `f64` literal or `.into()`, those lines compile-error and are listed for step 3.
3. **`deep_causality_topology` — body rewrites:** replace every `f64` literal with a `RealField`-native expression (`R::one()`, `R::pi()`, `R::epsilon()`) or a `R::from_f64(literal)` call. Remove every `as f64` and `.into()` round-trip.
4. **`deep_causality_topology` — internal call sites:** update every test file, example, benchmark, and internal helper to thread `R` or set explicit `<f64>`.
5. **`deep_causality_topology` — `f32` property tests:** duplicate the algorithmically-meaningful tests against `R = f32` with widened tolerances.
6. **`deep_causality_topology` — verification:** `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, benchmarks at `R = f64` (no regression > 2%), `bazel test //deep_causality_topology/...`.
7. **`deep_causality_physics` and `deep_causality_effects` — temporary pin:** add `::<f64>` turbofish at every call site that names a topology type. Each pin tagged `// TEMP: removed by generalize-{physics,effects}-over-realfield`. Workspace compiles, `make build` passes.
8. **Workspace-wide verification:** `make build`, `make test`. Address any cross-crate consumer the audit missed.
9. **Update downstream proposals:** revise `add-cubical-regge-calculus-core` and `add-cubical-regge-calculus-analytical` proposal / design / spec docs to reflect the now-generic topology surface. Kill `add-cubical-regge-calculus-analytical`'s Decision 8 (`Complex<f64>` shim is obsolete; use `ComplexField<R>` from `deep_causality_num`).
10. **Rollback:** revert the change set. Behavior at `R = f64` is bit-identical pre- and post-R0; rollback restores the old hardcoded API without observable change.

## Open Questions

1. **Does `RealField` need `from_f64` only, or also `from_i64` for integer literal materialization?** Recommendation: `from_f64` only. Integer-to-`R` paths in topology code can use `R::one()` iteratively (rare; mostly indices, which stay `usize`) or `R::from_f64(i as f64)` (uncommon). If a future change set needs `from_i64`, add it then.
2. **Does `Manifold::eigen_covariance` use an internal `f64`-only eigenvalue solver?** Audit-time check. If yes, document the internal precision floor; the public return type still generalizes to `R`.
3. **Does `deep_causality_rand` need an `R: RealField`-aware random-number primitive?** Recommendation: no in R0. Convert `f64` samples via `R::from_f64` at the boundary (Decision 7). If demand appears, a follow-up change adds native `R`-precision sampling.
4. **Does the project use `proptest` or a hand-rolled multi-precision test harness?** Audit at implementation time; macro-generate the `f32` duplicates if there's no shared harness.
5. **Is there a `cargo-public-api` or similar tool that can verify the "zero hardcoded `f64`" invariant mechanically?** Recommendation: add a CI grep step (`! grep -r 'f64' deep_causality_topology/src/ --include='*.rs' | grep -v -E '(test|comment|doc)'`) to keep the invariant from regressing. Implementation detail; not strictly part of R0.
