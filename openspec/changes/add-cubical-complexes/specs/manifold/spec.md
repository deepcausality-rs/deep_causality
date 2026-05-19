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

The methods that compute exterior derivative, codifferential, Hodge ⋆, and Laplacian on a `Manifold<K, F>` SHALL access the underlying complex's boundary and coboundary operators only through the `ChainComplex` trait. They SHALL NOT pattern-match on the concrete complex type or read complex-specific fields directly.

#### Scenario: Exterior derivative is generic

- **WHEN** the exterior derivative of a 1-form is computed on `Manifold<SimplicialComplex<C>, f64>` and on `Manifold<CubicalComplex<2>, f64>`
- **THEN** both computations succeed
- **AND** for the simplicial case the result equals today's `manifold/differential/exterior_cpu.rs` output bit-for-bit

#### Scenario: No direct field access on concrete complex

- **WHEN** the codebase is scanned with `grep -RIn "complex.coboundary_operators" src/types/manifold/`
- **THEN** there are no matches outside the trait-impl boundary

### Requirement: Comonad iteration is unchanged, neighborhood is queried inside the closure

`CoMonad::extend` on `Manifold<K, F>` SHALL continue to iterate the cursor over every cell (preserving today's order and cursor semantics) and pass the manifold view to the user's closure. A new helper `Manifold::neighbors<N: Neighborhood<K>>(&self, n: N, cell: CellId) -> N::Iter<'_>` SHALL be added so that the closure can pick a neighborhood strategy at the point of use.

#### Scenario: extend preserves iteration order

- **WHEN** `<ManifoldWitness<K> as CoMonad<...>>::extend(&m, f)` is called
- **THEN** the closure `f` is invoked once for each cell index `i` in `0..m.data.len()` in ascending order, each time with `m.cursor = i`

#### Scenario: Neighborhood query inside extend closure

- **WHEN** the user writes a closure `|view| view.neighbors(Moore, view.cursor()).map(|c| view.data_at(c)).sum::<f64>()`
- **AND** passes it to `CoMonad::extend` on `Manifold<CubicalComplex<3>, f64>`
- **THEN** every output cell holds the sum of its 26 Moore neighbors' input values

### Requirement: ReggeGeometry on cubical complexes supports the unit-edge case

`Manifold<CubicalComplex<D>, F>` SHALL accept a metric for the unit-edge case (every edge has length 1.0). Non-uniform / scaled / curved cubical metrics are out of scope for this change and SHALL be tracked in a separate follow-up issue.

#### Scenario: Unit-edge cubical metric

- **WHEN** the user constructs `Manifold::with_metric(CubicalComplex::<3>::cubic_torus(4), data, unit_metric)`
- **THEN** every edge length used by volume / Hodge computations is `1.0`
- **AND** the construction succeeds without invoking the (unimplemented) irregular cubical metric path

#### Scenario: Simplicial Regge geometry untouched

- **WHEN** existing simplicial Regge-geometry tests are run
- **THEN** they pass without modification — the simplicial path is not refactored as part of this change
