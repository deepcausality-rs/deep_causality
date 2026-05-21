# Work Notes — generalize-topology-over-realfield

Living log of decisions and rationale during the R0 implementation. Append-only; newer entries on top.

## Option 2C pivot landed — HKT restored, cross-algebra composition works

`cargo build --workspace --all-targets` clean; `cargo test --workspace` 9429 passed / 0
failed; `cargo clippy --workspace --all-targets` clean. The full haft trait surface
(`HKT`, `Functor`, `Foldable`, `Pure`, `Monad`, `CoMonad`, `Applicative`) is implemented
on `ManifoldWitness<C>` against the existing `deep_causality_haft` traits with no
inherent-method shim. The earlier "stricter than trait" blocker is gone because
`Manifold<K, F>` no longer carries a struct-level `F: RealField` bound. Metric is
preserved across `fmap`/`bind`/`extend`.

**Concrete shape changes (Option 2C):**

- `ChainComplex::Metric` is a plain associated type (was a GAT `Metric<R: RealField>`).
- `SimplicialComplex<R: RealField>` binds `type Metric = ReggeGeometry<R>;` (the existing
  `T` parameter is renamed to `R` and bounded `RealField`).
- `LatticeComplex<const D, R: RealField>` gains the precision parameter (was `<D>`) with
  a `PhantomData<R>` field; binds `type Metric = CubicalReggeGeometry<D, R>;`.
- `Manifold<K, F>` is unconstrained in `F` at the struct level. Per-impl-block `F:
  RealField` bounds are added only where numerical operations against `F` require them
  (covariance, simplex volume in metric precision, Laplacian, codifferential).
- `simplex_volume_squared` returns the **metric precision** `C` (not data precision `D`),
  since Cayley-Menger inputs are edge lengths from the metric.

**Lattice gauge HKT impls deferred (same pattern as `StrictCausalTensorWitness`).**
`LatticeGaugeField<G, D, M, R>` keeps `R: RealField` at the struct level (because
`lattice: Arc<LatticeComplex<D, R>>` requires it). The haft trait impls on
`LatticeGaugeFieldWitness` have the same "stricter than trait" issue Manifold used
to have, and no clean Option-2C-style decoupling exists (the gauge field's beta is
intrinsically a real coupling). The inherent functional surface — `map_field`,
`scale_field`, `zip_with`, `identity_field` — is preserved. Tests for the dropped
trait impls (`test_pure`, `test_functor`, `test_applicative`, `test_monad`) are
removed; tests for the inherent surface remain.

**Examples updated to the `FloatType = f64` alias pattern.** Topology examples
(`cubical_heat_diffusion`, `lattice_gauge_simulation`, `manifold_analysis`,
`differential_field`, `hkt_graph_convolution`) now declare a `pub type FloatType =
f64;` alias at the top and thread it through every numerical site, matching the
convention from `examples/mathematics_examples/effect_kalman_predict_correct`. The
alias makes the precision-as-parameter intent visible: swap `f64` for `f32` or
`Float106` and re-run without touching the algorithm.

## Earlier status (pre-pivot) — kept for historical reference

`cargo build --workspace --all-targets` clean; `cargo test --workspace` reports
9433 passed, 0 failed; `cargo clippy --workspace --all-targets` clean (no warnings).
The architectural skeleton plus the cascade fixes documented below land the precision-
as-parameter invariant: no hardcoded `f64`/`f32` in `deep_causality_topology`'s struct
fields, function/method signatures, trait methods, error variants, or trait bounds.

## Examples drop unused `CoMonad` imports

After dropping the haft trait impls on `ManifoldWitness`, examples that imported
`deep_causality_haft::CoMonad` to enable trait-method-call syntax no longer need the
import — they call the inherent methods (`ManifoldWitness::extend`/`extract`) directly.
Removed the dangling imports from `triple_hkt_stress_field`,
`effect_diffusion_on_manifold` (kept `Monad`, `Pure`), `tensor_x_topology_laplacian`,
and `capstone_spinor_minkowski`.

## SimplicialComplex<R>, R collapse in differential modules

`SimplicialComplex<T>` stores `hodge_star_operators: Vec<CsrMatrix<T>>` — i.e. `T` is the
precision used by the complex's mass-matrix entries. Pre-refactor, `Manifold<SimplicialComplex<C>, D>`
left `C` and `D` independent, which is broken under the precision-as-parameter rule: the
Hodge mass matrix must share precision with the field data. The differential modules
(`codifferential`, `hodge`, `laplacian`) collapse to `impl<R: RealField> Manifold<SimplicialComplex<R>, R>`.
Non-numeric modules (`topology_*`, `display`, `neighbors`) keep `<C, D>` because they
don't cross-multiply the two; `C` there is a phantom-ish coordinate.

## hkt_manifold: drop `HKT` impl, inherent methods only

The `HKT` trait declares `type Type<T> where T: Satisfies<Self::Constraint>`, and impls
cannot add stricter bounds. But `Type<T> = Manifold<_, T>` requires `T: RealField`. The
GAT is structurally unsatisfiable without rewriting `deep_causality_haft` — out of scope.

`ManifoldWitness<C>` / `GenericManifoldWitness<K>` are kept as marker types with the
same conceptual role, but they no longer impl `HKT`/`Functor`/`Foldable`/`Pure`/`Monad`/
`CoMonad`. The functional surface is provided as **inherent methods** with the appropriate
`RealField` bounds. Callers switch from `<W as Functor<_>>::fmap` to `W::fmap`.
`Applicative::apply` is dropped: function-typed `Manifold<_, Func>` data is fundamentally
incompatible with the `F: RealField` invariant — that's a feature, not a bug.

## Architectural skeleton (committed)

- `ChainComplex::Metric` is a **GAT**: `type Metric<R: RealField>`. This keeps the chain complex (a purely combinatorial object) free of any precision parameter while letting each implementor name a precision-parametric metric type. Rejected the alternative of stamping a phantom `R` onto `LatticeComplex<D, R>` because it inverts the mathematical structure (combinatorial type carrying phantom precision) just to satisfy the type checker.
- `Manifold<K, F>` is the canonical form. `F: RealField` is the precision parameter for both the data tensor and the metric (`K::Metric<F>`).
- `SimplicialManifold<C, F>` and `SimplicialManifoldWitness<C>` are textbook aliases preserved for ergonomics. Not bridge code — they are thin re-namings of the simplicial instantiation.
- `Functor::fmap` on `GenericManifoldWitness<K>` **drops the metric** when `A → B`. Reason: under the GAT, `K::Metric<A>` and `K::Metric<B>` are distinct types with no precision-preserving conversion. Callers that need to preserve a metric across `fmap` must reattach on the output.
- `Pure`/`Monad`/`Applicative`/`CoMonad` for `GenericManifoldWitness<K>` are deferred to a follow-up: they need `K: Default` or simplicial-specific bounds. Simplicial fast path remains via `SimplicialManifoldWitness<C>`.

## Numeric primitives

- **`FromPrimitive`, not new `RealField` methods.** Earlier attempt added `from_f64`/`from_f32`/`from_i64`/`from_i32` to `RealField` and produced ~50 ambiguous-method-resolution errors against the pre-existing `FromPrimitive` trait that already declared the same names with `Option<Self>` return.
- Pattern for literal constants in generic code: `<R as FromPrimitive>::from_f64(2.0).expect("2.0 is representable in every RealField")`. The expect message documents the invariant; any sane `RealField` impl must round-trip these literals.

## Migration policy

- Hard rip-and-replace. No bridge code, no type aliases over the old `Manifold<C, D>`/`ReggeGeometry<f64>` signatures, no default type parameters.
- Library code propagates `R: RealField` upstream. Only end-consumer call sites (binaries, examples, benchmarks) bind concrete `R`.
- The public API of `deep_causality_topology` must contain **zero** hardcoded `f64`/`f32` in struct fields, function/method signatures, trait methods, error variants, or trait bounds when complete.

## Branch / commit

- Working on branch `refactor/generalize-topology-over-realfield`. Architectural skeleton committed there; remaining cascade (~204 errors at start of resume) lands as follow-up commits on the same branch. User merges to main themselves.
