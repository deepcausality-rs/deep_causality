## ADDED Requirements

### Requirement: ChainComplex trait replaces CWComplex with static dispatch

The crate `deep_causality_topology` SHALL expose a public trait `ChainComplex` that describes any CW-style complex (simplicial, cubical, or user-defined) using only static dispatch. The trait MUST NOT use `Box<dyn Iterator>` or any other trait object in its method signatures. The pre-existing trait `CWComplex` SHALL be removed; no `pub use CWComplex = ChainComplex` alias is provided.

#### Scenario: ChainComplex uses GAT iterator

- **WHEN** a downstream crate implements `ChainComplex` for a concrete type
- **THEN** the trait requires an associated `type CellIter<'a>: Iterator<Item = Self::CellType> where Self: 'a`
- **AND** the method `fn cells(&self, k: usize) -> Self::CellIter<'_>` returns the iterator by value, not boxed

#### Scenario: No dyn Iterator in public API

- **WHEN** the crate is built with `cargo build -p deep_causality_topology`
- **AND** the test suite is built with `cargo test -p deep_causality_topology`
- **THEN** `grep -RIn "dyn Iterator" src/traits/` returns no matches in the `ChainComplex` definition or its impls

### Requirement: ChainComplex exposes boundary and coboundary uniformly

`ChainComplex` SHALL require methods to retrieve both the boundary matrix ∂_k and the coboundary matrix δ_k for any grade k, so that downstream code (notably `Manifold`'s differential operators) can read them without reaching into any concrete complex's storage layout.

#### Scenario: Boundary and coboundary are first-class trait methods

- **WHEN** a generic function `fn f<K: ChainComplex>(k: &K)` is written
- **THEN** it can call `k.boundary_matrix(grade)` and `k.coboundary_matrix(grade)` returning `CsrMatrix<i8>`
- **AND** the call compiles without referencing any concrete complex type

#### Scenario: SimplicialComplex satisfies the trait via its cached operators

- **WHEN** `SimplicialComplex<C>` implements `ChainComplex`
- **THEN** `coboundary_matrix(k)` returns the matrix from its pre-existing `coboundary_operators` cache
- **AND** the cached values are bit-for-bit identical to those used today by `manifold/differential/exterior_cpu.rs`

#### Scenario: CubicalComplex satisfies the trait without a pre-computed cache

- **WHEN** `CubicalComplex<D>` implements `ChainComplex`
- **THEN** `coboundary_matrix(k)` returns a matrix algebraically equal to `boundary_matrix(k+1).transpose()`
- **AND** the implementation MAY memoize lazily but MUST NOT require the caller to pre-populate it

### Requirement: ChainComplex preserves topological queries

`ChainComplex` SHALL retain the queries already provided by `CWComplex`: cell count per grade, maximum dimension, and k-th Betti number.

#### Scenario: Topological queries preserved on Lattice rename

- **WHEN** `CubicalComplex<3>::torus([4, 4, 4])` is constructed
- **THEN** `num_cells(k)`, `max_dim()`, and `betti_number(k)` return the same values that today's `Lattice<3>::cubic_torus(4)` returns for every k in `0..=3`
