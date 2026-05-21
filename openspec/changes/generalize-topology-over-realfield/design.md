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
- The `From<f64> + Into<f64>` and `Mul<f64, Output = T>` trait-bound crutches are removed in full; their replacement is `R: RealField + FromPrimitive` where the existing `FromPrimitive` trait covers numeric-literal materialization (Decision 2).
- Internal computation operates on `R` end-to-end with no `f64` round-trips. Every `<T as From<f64>>::from(literal)` site is rewritten to a `RealField`-native expression or `<R as FromPrimitive>::from_f64(literal).expect("...")`.
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
- Touching `deep_causality_num`. Under the Decision 2 pivot, R0 uses the existing `FromPrimitive` trait and adds zero methods to `RealField`. No cross-crate change is required.

## Decisions

### Decision 1: Precision lives where precision is used — generalized types carry `R: RealField`; `ChainComplex::Metric` is a plain associated type; `Manifold<K, F>` decouples data precision from metric precision

Every type whose semantics depend on a real-field precision carries an `R: RealField` parameter. Every type whose semantics are purely combinatorial does not. Examples:

**Precision-carrying types** (gain an `R` parameter):
- `CubicalReggeGeometry<D, R: RealField>` — stores `R`-typed edge lengths.
- `ReggeGeometry<R: RealField>` — stores `R`-typed simplicial edge lengths and curvature data.
- `CurvatureTensor<R: RealField, ...>` — stores `R`-typed tensor components.
- `DifferentialForm<T, R: RealField>` — adds an `R` parameter to support `T: Mul<R, Output = T>`.
- `SimplicialComplex<R: RealField>` — already T-parameterized; the `T` is the precision of its Hodge mass matrices.
- `LatticeComplex<const D: usize, R: RealField>` — the combinatorial caches do not depend on `R`, but the parameter exists to anchor the precision of its associated metric (a `PhantomData<R>` field carries the type).

**Trait surface — plain associated type, not GAT:**

```rust
pub trait ChainComplex {
    type CellType: Cell;
    type CellIter<'a>: Iterator<Item = Self::CellType> where Self: 'a;
    type Metric;                       // plain associated type
    // ... combinatorial methods unchanged
}

impl<R: RealField> ChainComplex for SimplicialComplex<R> {
    type Metric = ReggeGeometry<R>;    // precision flows from the complex's R
    // ...
}

impl<const D: usize, R: RealField> ChainComplex for LatticeComplex<D, R> {
    type Metric = CubicalReggeGeometry<D, R>;
    // ...
}

pub struct Manifold<K: ChainComplex, F> {   // ← F is unconstrained at the struct level
    complex: K,
    data: CausalTensor<F>,
    metric: Option<K::Metric>,              // ← K::Metric is a concrete type; no F dependency
    cursor: usize,
}
```

The complex carries its own precision; the metric type follows directly via the plain associated type. The Manifold's data precision `F` is independent of the metric's precision and unconstrained at the struct level.

**Why this shape (the Option 2C decision):**

1. **Restores the full HKT trait surface on `Manifold`.** Under the earlier GAT-on-Metric design, `Manifold<K, F>` needed `F: RealField` at the struct level (so `K::Metric<F>` was nameable). That bound made every `<W as Functor<W>>::fmap` impl require `B: RealField` inside the impl, which the `deep_causality_haft` trait machinery rejects with "impl has stricter requirements than trait" (haft declares `B: Satisfies<F::Constraint>` only). The plain-associated-type design removes the `F: RealField` struct bound entirely: `K::Metric` is a single concrete type independent of `F`, so the struct is well-formed for any `F`, and the haft `Satisfies<NoConstraint>` bounds suffice. `Functor`, `Pure`, `Monad`, `CoMonad`, `Foldable`, and `Applicative` all become implementable on the existing haft trait surface without modifying haft.

2. **Restores cross-algebra composition.** Manifold's defining feature is composing across the multivector and tensor crates through HKT — `Manifold<K, Multivector<f64, ...>>`, `Manifold<K, CausalTensor<f64>>`, `Manifold<K, DualNumber<f64>>`. Multivectors, tensors, and dual numbers are not `RealField` (Clifford algebra isn't totally ordered; tensors are not a field). The earlier GAT design forced `F: RealField` and made these cell-data shapes unrepresentable. Option 2C removes that block.

3. **Preserves the metric across `Functor::fmap`.** Under the GAT design, `fmap A → B` had to drop the metric because `K::Metric<A>` and `K::Metric<B>` were distinct types with no precision-preserving transformation. Under Option 2C, `K::Metric` is a single type independent of the data, so `m_a.metric.clone()` flows through `fmap` naturally. This restores a behavior that the GAT design silently broke.

4. **Matches the mathematical reality more honestly.** The constraint "metric precision equals data precision" is not a mathematical invariant — it was a type-system artifact of the GAT. Real workflows want `f32` metric + `f64` data (memory-bound mesh storage with high-precision field computation), scalar metric + multivector data (Clifford-algebra fields on real geometry), real metric + complex data (Lorentzian QFT), concrete-precision metric + dual-number data (automatic differentiation). The plain-associated-type design supports all of these.

**Why `LatticeComplex` gains an `R` parameter while its combinatorial caches stay `R`-independent:** the parameter exists to anchor the metric type via the `Metric = CubicalReggeGeometry<D, R>` binding. A `PhantomData<R>` field carries the type without adding storage. The "one combinatorial complex hosting metrics at multiple precisions" use case from the earlier GAT draft is rejected — choosing a precision once per complex instance is the common case, and instantiating a second complex at a second precision is trivial because the combinatorial structure is re-derivable from `[shape, periodic]`.

**Why `SimplicialComplex<R>` already had this shape:** the simplicial type carries `hodge_star_operators: Vec<CsrMatrix<T>>` where `T` is the Hodge mass-matrix precision. The pre-R0 design had this T-parameter; Option 2C keeps it and renames T to R to reflect the precision-as-parameter role. The `Manifold<SimplicialComplex<R>, F>` instantiation now allows `F ≠ R` (e.g. multivector cell data on f64 simplicial geometry).

**Why one `R` parameter per precision-carrying type, not separate `R_lengths`, `R_angles`, etc.:** these values are connected by closed forms within a single type. Splitting creates unenforceable consistency constraints; one `R` ensures all derived quantities live in the same field by construction.

**Why no default `R = f64` and no default `F = R`:** the entire point is to make precision a *choice*. A default would let new call sites silently fall back and accumulate the same precision loss the parameter is meant to solve.

**Where `F: RealField` bounds DO live:** only on the impl blocks that perform numerical operations against `F` — covariance, simplex volume, curvature contractions, Cayley-Menger, Laplacian, codifferential. The HKT impls, the constructors, the getters, and any combinatorial / shape-passing operation are bound-free in `F`.

**Rejected alternatives:**
- **GAT on `Metric<R>`** (the earlier R0 draft). Rejected for the reasons enumerated above — broke HKT and cross-algebra composition.
- **Three parameters on `Manifold<K, F, M>`.** Considered; rejected as a deeper API restructuring than necessary. The plain associated type on `ChainComplex::Metric` is the minimum change that achieves the goal.
- **Sibling type `ManifoldWithMetric<K, F, M>` parallel to `Manifold<K, F>`.** Rejected — parallel structures are forbidden by user direction.
- **`PhantomData<R>` on Manifold to anchor metric precision while leaving F unconstrained.** Considered; rejected because it duplicates the precision parameter (the complex already carries it via `K`).

**Stability note:** plain associated types are stable since Rust 1.0. The Option 2C design uses no GATs, no HRTBs, no nightly features — strictly less Rust complexity than the rejected GAT design.

### Decision 2: Use the existing `FromPrimitive` trait for numeric-literal conversions; no new methods on `RealField`

The current `RealField` trait exposes `pi`, `e`, `epsilon`, and the transcendentals, but no general-purpose constructor for arbitrary numeric literals. Several internal sites in `deep_causality_topology` materialize constants like `1e-12` (epsilon tolerances), `2.0` (binary half), `8πG` (Einstein-Hilbert prefactor), etc., that aren't reachable through `R::one()`, `R::pi()`, `R::epsilon()` alone.

`deep_causality_num` already exposes a `FromPrimitive` trait (`deep_causality_num/src/cast/from_primitive/mod.rs`) with fallible `-> Option<Self>` constructors for every primitive numeric type (`from_f64`, `from_f32`, `from_i64`, `from_i32`, plus `i8`/`i16`/`i128`/`u*`/`isize`/`usize`). **R0 uses this existing trait instead of adding methods to `RealField`.**

Topology generic-code sites that materialize literals SHALL bound `R: RealField + FromPrimitive` and call:

```rust
<R as FromPrimitive>::from_f64(0.5).expect("0.5 is representable in every RealField")
```

**Why the existing trait, not new methods on `RealField`:** an early attempt at R0 added four constructors (`from_f64`, `from_f32`, `from_i64`, `from_i32`) directly to `RealField`. The collision with the existing `FromPrimitive` trait — which already declares those exact method names with `Option<Self>` return — created ~50 ambiguous-method-resolution errors across the workspace, all of which the compiler suggested resolving by disambiguating to `<T as FromPrimitive>::from_X(...)`. The compiler was telling us the answer: use the trait that already exists. The pivot is to remove the `RealField` additions and lean on `FromPrimitive` directly.

**Tradeoff: the `.expect(...)` syntactic cost.** `FromPrimitive::from_f64` returns `Option<Self>` because primitive-to-narrower-primitive conversions can fail (e.g. `from_f64(NaN)` to an integer type). For the `R: RealField` case the conversion is always infallible — `f32`, `f64`, and `Float106` never reject a literal like `0.5`. The `.expect("invariant message")` documents this invariant at the call site. Slightly more verbose than a hypothetical `<R as FromPrimitive>::from_f64(0.5).expect("0.5 fits")` infallible call, but it reuses an existing trait surface with established semantics.

**Why not make `RealField: FromPrimitive` a supertrait?** This would tighten the trait surface (every `RealField` implementor must also implement `FromPrimitive`), which is fine in practice (`f32`, `f64`, `Float106` all do). But it isn't necessary — topology call sites can bound `R: RealField + FromPrimitive` explicitly at the use site, leaving `RealField` itself untouched. Keeping the traits orthogonal is a smaller change and leaves room for future precisions that might not want `FromPrimitive` (e.g. a verified-correctness fixed-point type that rejects `f64` inputs).

**Alternatives considered:**
- Add `from_f64`/`from_f32`/`from_i64`/`from_i32` as new methods on `RealField` (the original R0 design). Rejected: collides with existing `FromPrimitive` trait.
- Introduce a new `FromLiteral` trait specific to the topology refactor. Rejected: premature decomposition; `FromPrimitive` already covers the use case.
- Make `RealField: FromPrimitive` a supertrait. Considered; rejected for minimum-change reasons (see preceding paragraph).

### Decision 3: Drop `From<f64> + Into<f64>` from every existing generic bound; rewire internals

`ReggeGeometry<T>` bounds `T: Float + Zero + Copy + PartialOrd + From<f64> + Into<f64>`. The `From<f64> + Into<f64>` half exists because internal numerical algorithms were written against `f64` and round-trip through it. `CurvatureTensor`, `PointCloud::triangulate`, and `Manifold` geometry / covariance helpers have the same pattern.

R0 rewrites those internals to operate on `R: RealField` end-to-end:

- `arctan2`, `sqrt`, `powf`, `sin`, `cos`, etc. are all on `RealField`.
- Constants like `R::pi()`, `R::e()`, `R::epsilon()` come from the trait.
- Numeric literals come from `<R as FromPrimitive>::from_f64(literal).expect("...")`.
- Algebraic constants like `R::one() + R::one()` for `2`, or `R::one() / (R::one() + R::one())` for `0.5`, are used where they read more naturally than `<R as FromPrimitive>::from_f64(2.0).expect("2.0 fits")`.

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

**Method-level generic, not trait-level.** A trait-level parameter `trait GaugeGroup<R: RealField>` would force each implementor to pick its precision at trait-impl time; a method-level generic lets `SU2` etc. produce structure constants at whatever precision the caller asks for. The four in-crate impls (`SU2`, `SU3`, `SE3`, `SO(3,1)`) each implement the method via `<R as FromPrimitive>::from_f64(literal).expect("...")` for their hardcoded coefficient values.

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

`PointCloud::triangulate` is bounded `T: Float + Sum + From<f64> + ...`. The `T` parameter is the field-data type of the point cloud. R0 changes the bound to `T: RealField`, removes the `From<f64>` half, and rewrites the internal Gaussian elimination / Hodge dual / volume helpers to use `<T as FromPrimitive>::from_f64(...).expect("...")` for the few literal constants they need (epsilon tolerances, fixed coefficients).

No new type parameter is introduced. The existing `T` is widened from "any `Float` with f64 round-trip" to "any `RealField`".

### Decision 7: Metropolis acceptance ratio and RNG output are `R: RealField`

`metropolis_step -> Result<f64, _>` becomes `metropolis_step -> Result<R, _>` where `R` is the gauge field's real-scalar parameter (already exposed via `GaugeField<G, M, R>`).

The internal `let rnd: f64 = rng.random()` becomes `let rnd: R = <R as FromPrimitive>::from_f64(rng.random::<f64>()).expect("RNG sample fits in any RealField")`. The RNG is consulted at `f64` precision (cheap, fast, and the source of the random bits doesn't need to be `R`-aware) and the result is converted up. **This is the one allowed `f64` literal in the entire crate post-R0** — it is internal, not part of the public API, and reflects the practical reality that `deep_causality_rand`'s RNG primitives produce `f64`.

If a future change set wants RNG output natively at `R` precision (for `f128`-resolution Monte Carlo), the `<R as FromPrimitive>::from_f64(...).expect("...")` site is the single point to revise.

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

- **[Risk] `FromPrimitive::from_f64` round-trips lose precision when `R` has higher precision than `f64`.** Calling `f128::from_f64(some_literal)` extends an `f64` to `f128`, but the bits beyond `f64` precision are zero — the value is the `f64` approximation lifted, not the true `f128` value of the source literal.
  → **Mitigation:** documented at every `<R as FromPrimitive>::from_f64(...).expect(...)` call site via the `expect` message. The `FromPrimitive` trait is a convenience for materializing constants from `f64` literals (the standard Rust literal type); callers who need true higher-precision constants use the precision's native constructor or `RealField::pi()` / `R::e()` / derived `RealField` expressions. The vast majority of `from_f64` call sites in `deep_causality_topology` are tolerance constants (`1e-12`) where `f64` precision is already overkill.

- **[Risk] Performance regression vs. hand-tuned `f64` paths.** `RealField` method calls go through trait dispatch. The Rust monomorphizer should inline to identical machine code at `R = f64`, but in rare cases (cross-crate generic instantiation behind `#[inline]` gates) it does not.
  → **Mitigation:** R0's verification task runs the existing `f64` benchmarks against the new generic surface at `R = f64` and asserts no measurable regression (>2%). If a regression appears, add `#[inline]` to the hot `RealField` impl methods on `f64` in `deep_causality_num`.

- **[Risk] The `GaugeGroup::structure_constant` method-level generic creates a turbofish burden at every call site.** Callers of `SU3::structure_constant(a, b, c)` must now write `SU3::structure_constant::<R>(a, b, c)` or rely on context-driven inference.
  → **Mitigation:** in practice every call site is inside a generic function or impl block that already has an `R: RealField` parameter in scope; inference resolves `R` automatically. The audit found ~5 internal call sites; all of them are in generic contexts. If a future call site needs the turbofish, it's a one-line addition.

- **[Risk] `Manifold` covariance / eigenvalue routines might depend on `f64`-only numerical libraries internally.** If `Manifold::eigen_covariance` calls a third-party eigenvalue solver that's `f64`-only, R0's generic return type is a lie — the result is always computed at `f64` and converted.
  → **Mitigation:** audit during implementation. If a true `f64`-only dependency exists, the conversion is `<R as FromPrimitive>::from_f64(eigenvalue_as_f64).expect("eigenvalue fits")` at the boundary, and a doc-comment flags the internal precision floor. This is acceptable for an internal-precision-limit case (the caller asked for `R`, the algorithm provided what it could). The crate's own algorithms (Cayley-Menger, deficit angles, Hodge ⋆) have no such dependency.

- **[Risk] The breaking changes ripple wider than expected.** A workspace-wide grep may find consumers we haven't audited.
  → **Mitigation:** R0's first task is `cargo build --workspace` after the refactor; every compile error is a missed call site. The compile-error list IS the migration checklist for downstream patches.

- **[Trade-off] R0 is pure overhead from a feature perspective.** No new user-visible capability ships; only the precondition refactor.
  → **Justification:** see proposal. The alternative (ship feature work against the hardcoded `f64` surface and refactor later) costs more in total work because every feature-change breaks again when the parameterization lands.

- **[Trade-off] R0 ships with no type aliases and no defaults.** Every call site that previously wrote `CubicalReggeGeometry<3>` now writes `CubicalReggeGeometry<3, f64>`. Mild ergonomic cost.
  → **Justification:** see Decision 9. Bridge code is bridge code; the cost of explicitness is small and one-time.

## Migration Plan

1. **`deep_causality_num`:** no changes needed. The existing `FromPrimitive` trait covers numeric-literal materialization. Verify by greppping the workspace for the four primitive constructors and confirming `FromPrimitive` is implemented for `f32`, `f64`, and `Float106`.
2. **`deep_causality_topology` — schema pass:** add `<R: RealField>` parameters and retype signatures. The bodies stay structurally identical at this stage — anywhere they reference an `f64` literal or `.into()`, those lines compile-error and are listed for step 3.
3. **`deep_causality_topology` — body rewrites:** replace every `f64` literal with a `RealField`-native expression (`R::one()`, `R::pi()`, `R::epsilon()`) or a `<R as FromPrimitive>::from_f64(literal).expect("...")` call. Remove every `as f64` and `.into()` round-trip.
4. **`deep_causality_topology` — internal call sites:** update every test file, example, benchmark, and internal helper to thread `R` or set explicit `<f64>`.
5. **`deep_causality_topology` — `f32` property tests:** duplicate the algorithmically-meaningful tests against `R = f32` with widened tolerances.
6. **`deep_causality_topology` — verification:** `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, benchmarks at `R = f64` (no regression > 2%), `bazel test //deep_causality_topology/...`.
7. **`deep_causality_physics` and `deep_causality_effects` — temporary pin:** add `::<f64>` turbofish at every call site that names a topology type. Each pin tagged `// TEMP: removed by generalize-{physics,effects}-over-realfield`. Workspace compiles, `make build` passes.
8. **Workspace-wide verification:** `make build`, `make test`. Address any cross-crate consumer the audit missed.
9. **Update downstream proposals:** revise `add-cubical-regge-calculus-core` and `add-cubical-regge-calculus-analytical` proposal / design / spec docs to reflect the now-generic topology surface. Kill `add-cubical-regge-calculus-analytical`'s Decision 8 (`Complex<f64>` shim is obsolete; use `ComplexField<R>` from `deep_causality_num`).
10. **Rollback:** revert the change set. Behavior at `R = f64` is bit-identical pre- and post-R0; rollback restores the old hardcoded API without observable change.

## Open Questions

1. **(Resolved by Decision 2 pivot)** Originally: does `RealField` need `from_f64` only, or also `from_i64`? Resolution: neither. R0 does not add methods to `RealField`. Numeric-literal materialization uses the existing `FromPrimitive` trait, which already covers `from_f64`, `from_f32`, `from_i64`, `from_i32`, and all other primitive conversions.
2. **Does `Manifold::eigen_covariance` use an internal `f64`-only eigenvalue solver?** Audit-time check. If yes, document the internal precision floor; the public return type still generalizes to `R`.
3. **Does `deep_causality_rand` need an `R: RealField`-aware random-number primitive?** Recommendation: no in R0. Convert `f64` samples via `FromPrimitive::from_f64` at the boundary (Decision 7). If demand appears, a follow-up change adds native `R`-precision sampling.
4. **Does the project use `proptest` or a hand-rolled multi-precision test harness?** Audit at implementation time; macro-generate the `f32` duplicates if there's no shared harness.
5. **Is there a `cargo-public-api` or similar tool that can verify the "zero hardcoded `f64`" invariant mechanically?** Recommendation: add a CI grep step (`! grep -r 'f64' deep_causality_topology/src/ --include='*.rs' | grep -v -E '(test|comment|doc)'`) to keep the invariant from regressing. Implementation detail; not strictly part of R0.
