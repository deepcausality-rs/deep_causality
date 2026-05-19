## ADDED Requirements

### Requirement: SimplicialComplex implements ChainComplex with zero-copy Cow

`SimplicialComplex<C>` SHALL implement the `ChainComplex` trait. The implementation SHALL preserve today's caching strategy: the pre-computed `boundary_operators` and `coboundary_operators` collections continue to live on the complex and are returned through the trait methods as `Cow::Borrowed`. No structural change to `SimplicialComplex`'s storage is required. The `Cow::Borrowed` return SHALL guarantee that no `CsrMatrix` clone is performed on the read path used by `Manifold`'s differential operators in Part B.

#### Scenario: Boundary matrix returned as zero-copy Cow::Borrowed

- **WHEN** `<SimplicialComplex<C> as ChainComplex>::boundary_matrix(k)` is called with `k > 0`
- **THEN** the returned `Cow<'_, CsrMatrix<i8>>` is `Cow::Borrowed(&self.boundary_operators[k - 1])`
- **AND** no clone of the underlying `CsrMatrix` is performed
- **AND** the dereferenced value is bit-for-bit identical to what today's `boundary_operator_cpu(k)` returns

#### Scenario: Coboundary matrix returned as zero-copy Cow::Borrowed

- **WHEN** `<SimplicialComplex<C> as ChainComplex>::coboundary_matrix(k)` is called
- **THEN** the returned `Cow<'_, CsrMatrix<i8>>` is `Cow::Borrowed(&self.coboundary_operators[k])`
- **AND** no clone of the underlying `CsrMatrix` is performed
- **AND** the dereferenced value is bit-for-bit identical to what today's `coboundary_operator_cpu(k)` returns

#### Scenario: Index convention matches today's storage layout

- **WHEN** `boundary_matrix(k)` is called on a `SimplicialComplex<C>` for `k > 0`
- **THEN** the returned matrix has `nrows == num_cells(k - 1)` and `ncols == num_cells(k)` (i.e. it represents ∂_k, with the existing `boundary_operators[k - 1]` indexing convention preserved)
- **AND** calling `boundary_matrix(0)` panics or returns an empty matrix consistent with the trait's documented behavior at grade 0

#### Scenario: Cell iteration is static dispatch

- **WHEN** `<SimplicialComplex<C> as ChainComplex>::cells(k)` is called
- **THEN** the returned iterator's type is the concrete `SimplicialComplex` cell iterator, not a `Box<dyn Iterator>`
- **AND** the iterator yields every k-simplex exactly once

### Requirement: SimplicialComplex public surface is unchanged

The rename of `CWComplex` to `ChainComplex` and the genericization of `Manifold` SHALL NOT change the public surface of `SimplicialComplex<C>` itself. Existing constructors, getters, and methods on `SimplicialComplex` SHALL remain bit-for-bit identical.

#### Scenario: Existing simplicial tests pass unchanged

- **WHEN** `cargo test -p deep_causality_topology` is run after the change
- **THEN** every pre-existing test under `tests/types/simplicial_complex/` passes without source modification
