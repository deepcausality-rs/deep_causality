## ADDED Requirements

### Requirement: SimplicialComplex implements ChainComplex

`SimplicialComplex<C>` SHALL implement the `ChainComplex` trait. The implementation SHALL preserve today's caching strategy: the pre-computed `boundary_operators` and `coboundary_operators` collections continue to live on the complex and are returned through the trait methods. No structural change to `SimplicialComplex`'s storage is required.

#### Scenario: Boundary matrix returned from cache

- **WHEN** `<SimplicialComplex<C> as ChainComplex>::boundary_matrix(k)` is called
- **THEN** the returned `CsrMatrix<i8>` is the value already cached in `boundary_operators[k]`
- **AND** the cached value is bit-for-bit identical to what today's code reads

#### Scenario: Coboundary matrix returned from cache

- **WHEN** `<SimplicialComplex<C> as ChainComplex>::coboundary_matrix(k)` is called
- **THEN** the returned `CsrMatrix<i8>` is the value already cached in `coboundary_operators[k]`

#### Scenario: Cell iteration is static dispatch

- **WHEN** `<SimplicialComplex<C> as ChainComplex>::cells(k)` is called
- **THEN** the returned iterator's type is the concrete `SimplicialComplex` cell iterator, not a `Box<dyn Iterator>`
- **AND** the iterator yields every k-simplex exactly once

### Requirement: SimplicialComplex public surface is unchanged

The rename of `CWComplex` to `ChainComplex` and the genericization of `Manifold` SHALL NOT change the public surface of `SimplicialComplex<C>` itself. Existing constructors, getters, and methods on `SimplicialComplex` SHALL remain bit-for-bit identical.

#### Scenario: Existing simplicial tests pass unchanged

- **WHEN** `cargo test -p deep_causality_topology` is run after the change
- **THEN** every pre-existing test under `tests/types/simplicial_complex/` passes without source modification
