## ADDED Requirements

### Requirement: Manifold is generic over any ChainComplex

`Manifold` SHALL be generic over the underlying chain complex. Its type signature SHALL be `Manifold<K: ChainComplex, F>` where `K` is the complex and `F` is the field data type. The previous signature `Manifold<C, D>` (where the complex was hard-wired to `SimplicialComplex<C>`) SHALL be removed.

#### Scenario: Manifold can wrap a simplicial complex

- **WHEN** the user constructs `SimplicialManifold::<C, f64>::new(simplex, data, None)` where `SimplicialManifold<C, F> = Manifold<SimplicialComplex<C>, F>`
- **THEN** the construction succeeds
- **AND** all simplicial-specific tests that pass today continue to pass without modification

#### Scenario: Manifold can wrap a cubical complex

- **WHEN** the user constructs `Manifold::<CubicalComplex<3>, f64>::new(complex, data, Some(metric))`
- **THEN** the construction succeeds
- **AND** the resulting value carries the cubical complex, field tensor, optional metric, and a cursor

### Requirement: Differential operators read the complex through the trait

The methods that compute exterior derivative and codifferential on a `Manifold<K, F>` SHALL access the underlying complex's boundary and coboundary operators only through the `ChainComplex` trait. They SHALL NOT pattern-match on the concrete complex type or read complex-specific fields directly.

The Hodge ⋆ and Laplacian operators are *currently simplicial-only* and read from `SimplicialComplex::hodge_star_operators`, which is not part of the `ChainComplex` trait. They are implemented on `Manifold<SimplicialComplex<C>, D>` (i.e. `SimplicialManifold<C, D>`), not on the generic `Manifold<K, F>`. Adding a Hodge ⋆ method to `ChainComplex` so the cubical path can carry it is deferred to a follow-up issue (per design D7: "Hodge ⋆ on non-unit / non-regular cubes — defer to a follow-up"). This scope decision is intentional: the unit-edge cubical case ships in this change set; the irregular metric and the generic Hodge ⋆ are tracked separately.

The helper functions `is_oriented` and `has_boundary` in `manifold/utils/utils_manifold.rs` SHALL also route through `ChainComplex::boundary_matrix` so the audit test below applies to them uniformly. They take `&SimplicialComplex<T>` directly today; the routing keeps the no-direct-field-access invariant consistent without requiring an exception clause.

#### Scenario: Exterior derivative is generic

- **WHEN** the exterior derivative of a 1-form is computed on `SimplicialManifold<C, f64>` (after Stage B) or on `Manifold<CubicalComplex<2>, f64>` (after Stage C wires the impl)
- **THEN** both computations succeed
- **AND** for the simplicial case the result equals today's `manifold/differential/exterior.rs` output bit-for-bit (renamed from `exterior_cpu.rs` as part of the Stage A `_cpu` cleanup)
- **AND** the codifferential follows the same pattern (`codifferential.rs`, renamed from `codifferential_cpu.rs`)

#### Scenario: No direct field access on concrete complex

- **WHEN** the codebase is scanned with `grep -RIn "complex.coboundary_operators\|complex.boundary_operators" src/types/manifold/`
- **THEN** there are no matches outside the trait-impl boundary

#### Scenario: Differential reads go through zero-copy Cow on SimplicialComplex

- **WHEN** an exterior-derivative / codifferential / Hodge / Laplacian operation runs on `SimplicialManifold<C, F>`
- **THEN** each `coboundary_matrix(k)` call returns `Cow::Borrowed`
- **AND** no `CsrMatrix<i8>::clone` is performed on the read path (verified by an `#[inline(never)]` instrumented test or a `cargo flamegraph` smoke run if available; at minimum, the source path uses `&*cow` rather than `cow.into_owned()`)

### Requirement: Comonad iteration is unchanged, neighborhood is queried inside the closure

The existing `ManifoldWitness<C>` (and its alias `SimplicialManifoldWitness<C>`) SHALL remain the simplicial-specific HKT entry point. Its `Functor` / `Monad` / `Applicative` / `CoMonad` impls continue to operate on `Manifold<SimplicialComplex<C>, _>`. `CoMonad::extend` on this witness SHALL continue to iterate the cursor over every cell (preserving today's order and cursor semantics) and pass the manifold view to the user's closure.

A separate arbitrary-K manifold witness (e.g. `GenericManifoldWitness<K>`) over any `K: ChainComplex` SHALL be introduced in Stage C (per tasks.md task 3.11a) so the comonad pattern can apply to `Manifold<CubicalComplex<D>, F>` as well. It is NOT introduced in Stage B because the existing witness's `Functor`/`Monad`/etc. impls assume simplicial-specific bounds in their bodies (e.g. cloning a default-constructible complex); lifting `K` requires fresh bounds and an HKT-machinery redesign that is orthogonal to the Stage B genericization of the `Manifold` struct itself.

A new helper `Manifold::neighbors<N: Neighborhood<K>>(&self, n: N, cell: CellId) -> N::Iter<'_>` SHALL be added in Stage C so that the closure can pick a neighborhood strategy at the point of use.

#### Scenario: extend preserves iteration order

- **WHEN** `<ManifoldWitness<C> as CoMonad<...>>::extend(&m, f)` is called on `SimplicialManifold<C, F>` (the simplicial entry point shipped in Stage B)
- **THEN** the closure `f` is invoked once for each cell index `i` in `0..m.data.len()` in ascending order, each time with `m.cursor = i`
- **AND** the cursor / iteration semantics carry over to the Stage C `GenericManifoldWitness<K>` once it ships

#### Scenario: Neighborhood query inside extend closure

- **WHEN** the user writes a closure `|view| view.neighbors(Moore, view.cursor()).map(|c| view.data_at(c)).sum::<f64>()`
- **AND** passes it to `CoMonad::extend` on `Manifold<CubicalComplex<3>, f64>`
- **THEN** every output cell holds the sum of its 26 Moore neighbors' input values

### Requirement: Manifold carries an optional metric typed per complex

`Manifold<K, F>` SHALL store its optional metric as `metric: Option<K::Metric>`, using the `ChainComplex::Metric` associated type. This routes the metric type through the trait at compile time without `dyn`, without an enum at the Manifold level, and without coupling `Manifold` to any concrete metric implementation.

#### Scenario: Simplicial metric stays ReggeGeometry

- **WHEN** the user calls `SimplicialManifold::<f64, f64>::with_metric(complex, data, Some(regge), 0)`
- **THEN** the metric argument's static type is `Option<ReggeGeometry<f64>>`
- **AND** all existing Regge geometry tests pass without modification

#### Scenario: Cubical metric is the unit-edge case

- **WHEN** the user calls `Manifold::<CubicalComplex<3>, f64>::with_metric(complex, data, Some(CubicalMetric::unit()), 0)`
- **THEN** the metric argument's static type is `Option<CubicalMetric<3>>`
- **AND** every edge length used by volume / Hodge computations is `1.0`

#### Scenario: Non-uniform cubical metrics deferred

- **WHEN** non-unit / scaled / curved cubical metrics are needed
- **THEN** they are out of scope for this change set
- **AND** they SHALL be tracked in a separate follow-up issue

#### Scenario: Unit-edge cubical metric

- **WHEN** the user constructs `Manifold::with_metric(CubicalComplex::<3>::cubic_torus(4), data, unit_metric)`
- **THEN** every edge length used by volume / Hodge computations is `1.0`
- **AND** the construction succeeds without invoking the (unimplemented) irregular cubical metric path

#### Scenario: Simplicial Regge geometry untouched

- **WHEN** existing simplicial Regge-geometry tests are run
- **THEN** they pass without modification — the simplicial path is not refactored as part of this change
