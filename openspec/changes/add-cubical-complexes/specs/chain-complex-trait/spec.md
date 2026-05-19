## ADDED Requirements

### Requirement: ChainComplex trait replaces CWComplex with static dispatch

The crate `deep_causality_topology` SHALL expose a public trait `ChainComplex` that describes any CW-style complex (simplicial, cubical, or user-defined) using only static dispatch. The trait MUST NOT use `Box<dyn Iterator>` or any other trait object in its method signatures. The pre-existing trait `CWComplex` SHALL be removed; no `pub use CWComplex = ChainComplex` alias is provided.

#### Scenario: ChainComplex uses GAT iterator

- **WHEN** a downstream crate implements `ChainComplex` for a concrete type
- **THEN** the trait requires an associated `type CellIter<'a>: Iterator<Item = Self::CellType> where Self: 'a`
- **AND** the method `fn cells(&self, k: usize) -> Self::CellIter<'_>` returns the iterator by value, not boxed

#### Scenario: No dyn Iterator in trait or impls

- **WHEN** the crate is built with `cargo build -p deep_causality_topology`
- **AND** the test suite is built with `cargo test -p deep_causality_topology`
- **THEN** `grep -RIn 'dyn Iterator' src/traits/ src/types/lattice/ src/types/cell_complex/ src/types/simplicial_complex/topology/` returns zero matches

### Requirement: Cell trait lives in its own module

The `Cell` marker trait SHALL be declared in `src/traits/cell.rs`, separately from `ChainComplex`. Both SHALL be re-exported from `src/traits/mod.rs` and from `src/lib.rs`. The previous bundling of `Cell` inside `cw_complex.rs` is replaced by the split.

#### Scenario: Cell is importable on its own

- **WHEN** a user writes `use deep_causality_topology::Cell;`
- **THEN** the import resolves
- **AND** `Cell` and `ChainComplex` are declared in distinct source files (`src/traits/cell.rs` and `src/traits/chain_complex.rs` respectively)

### Requirement: ChainComplex exposes boundary and coboundary via Cow

`ChainComplex` SHALL require methods to retrieve both the boundary matrix ∂_k and the coboundary matrix δ_k for any grade k. Both methods SHALL return `std::borrow::Cow<'_, CsrMatrix<i8>>` so that cache-rich implementors can vend `Cow::Borrowed` (zero copy) and compute-on-demand implementors can vend `Cow::Owned` without forcing a `clone()` at the trait boundary. `coboundary_matrix` is a brand-new method on the trait surface — today's `CWComplex` exposes only `boundary_matrix`, and every existing implementor SHALL gain the new method as part of this change.

#### Scenario: Boundary and coboundary are first-class trait methods returning Cow

- **WHEN** a generic function `fn f<K: ChainComplex>(k: &K)` is written
- **THEN** it can call `k.boundary_matrix(grade)` and `k.coboundary_matrix(grade)`, each returning `Cow<'_, CsrMatrix<i8>>`
- **AND** the call compiles without referencing any concrete complex type

#### Scenario: SimplicialComplex satisfies the trait via zero-copy Cow::Borrowed

- **WHEN** `SimplicialComplex<C>` implements `ChainComplex`
- **THEN** `boundary_matrix(k)` returns `Cow::Borrowed(&self.boundary_operators[k - 1])`
- **AND** `coboundary_matrix(k)` returns `Cow::Borrowed(&self.coboundary_operators[k])`
- **AND** no `CsrMatrix` clone is performed on the read path
- **AND** the returned matrix contents are bit-for-bit identical to those used today by `manifold/differential/exterior_cpu.rs`

#### Scenario: Lattice satisfies the trait via lazy-memoized Cow::Owned

- **WHEN** `Lattice<D>` implements `ChainComplex`
- **THEN** `coboundary_matrix(k)` lazily computes `boundary_matrix(k + 1).into_owned().transpose()` on first call, stores the result in an internal `Mutex<HashMap<usize, CsrMatrix<i8>>>`, and returns `Cow::Owned(matrix.clone())` from the cache on subsequent calls
- **AND** the implementation does not use `unsafe`
- **AND** the cache uses `std::sync::Mutex` (not `RefCell`) so that `Lattice<D>` remains `Send + Sync`, preserving the existing `Arc<Lattice<D>>` consumers in `gauge_field_lattice`

#### Scenario: CellComplex satisfies the trait without internal memoization

- **WHEN** `CellComplex<C>` implements `ChainComplex`
- **THEN** `coboundary_matrix(k)` computes the transpose of `boundary_matrix(k + 1)` on each call and returns `Cow::Owned`
- **AND** lazy memoization is intentionally not added in this stage (its usage pattern does not justify the `RefCell` complexity)

### Requirement: Boundary matrix shape and transpose-coboundary identity hold for every impl

Every implementor of `ChainComplex` SHALL satisfy two algebraic invariants: (a) `boundary_matrix(k)` has shape `(num_cells(k - 1), num_cells(k))` for every `k > 0`; (b) `coboundary_matrix(k)` is the transpose of `boundary_matrix(k + 1)` as a sparse matrix. These invariants SHALL be verified by a parametric conformance test that runs against `SimplicialComplex`, `Lattice<D>`, and `CellComplex<C>`.

#### Scenario: Shape invariant holds across all impls

- **WHEN** the conformance test invokes `boundary_matrix(k)` on each of `SimplicialComplex<f64>::new(...)`, `Lattice::<3>::cubic_open(4)`, and a small `CellComplex<...>`
- **THEN** for every `k` in `1..=complex.max_dim()`, the returned matrix has `nrows == num_cells(k - 1)` and `ncols == num_cells(k)`

#### Scenario: Transpose invariant holds across all impls

- **WHEN** the conformance test compares `coboundary_matrix(k)` against `boundary_matrix(k + 1)` for each impl
- **THEN** for every `k` in `0..complex.max_dim()`, the two matrices are transposes of each other (entry-by-entry equality after index swap)

### Requirement: ChainComplex exposes an associated Metric type

`ChainComplex` SHALL expose an associated `type Metric;` so that `Manifold<K, F>` can carry an optional metric typed per-complex without forcing `dyn`, an enum, or coupling `Manifold` to any concrete metric type. The trait MUST NOT impose bounds on `Metric` — bounds belong on use sites where metric-specific operations are invoked.

Implementations:
- `SimplicialComplex<T>::Metric = ReggeGeometry<T>` — the existing Regge geometry stays the simplicial path's metric.
- `Lattice<D>::Metric = CubicalMetric<D>` — a new unit-edge metric type introduced in this change set.
- `CellComplex<C>::Metric = ()` — no metric for the abstract cell-complex type; the unit type satisfies the trait without imposing a real metric.

#### Scenario: Each impl declares its Metric

- **WHEN** a downstream user reads the public API
- **THEN** `<SimplicialComplex<f64> as ChainComplex>::Metric` resolves to `ReggeGeometry<f64>`
- **AND** `<Lattice<3> as ChainComplex>::Metric` resolves to `CubicalMetric<3>`
- **AND** `<CellComplex<MyCell> as ChainComplex>::Metric` resolves to `()`

#### Scenario: Manifold uses K::Metric

- **WHEN** the user constructs `Manifold::<SimplicialComplex<f64>, f64>::with_metric(complex, data, Some(metric), 0)`
- **THEN** the `metric` argument's type is `Option<ReggeGeometry<f64>>` (i.e. `Option<<SimplicialComplex<f64> as ChainComplex>::Metric>`)
- **AND** the construction is type-checked without runtime dispatch

#### Scenario: No bound on Metric at the trait level

- **WHEN** the `ChainComplex` trait is inspected
- **THEN** the associated `type Metric;` declaration has no `where` clause and no supertrait bound
- **AND** bounds (e.g. `K::Metric: Clone`) appear only on `impl` blocks that need them

### Requirement: ChainComplex preserves topological queries

`ChainComplex` SHALL retain the queries already provided by `CWComplex`: cell count per grade, maximum dimension, and k-th Betti number.

#### Scenario: Topological queries preserved on Lattice rename

- **WHEN** `CubicalComplex<3>::torus([4, 4, 4])` is constructed
- **THEN** `num_cells(k)`, `max_dim()`, and `betti_number(k)` return the same values that today's `Lattice<3>::cubic_torus(4)` returns for every k in `0..=3`
