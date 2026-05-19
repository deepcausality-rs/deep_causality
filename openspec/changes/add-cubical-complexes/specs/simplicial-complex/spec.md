## ADDED Requirements

### Requirement: SimplicialComplex implements ChainComplex with zero-copy Cow

`SimplicialComplex<C>` SHALL implement the `ChainComplex` trait. The implementation SHALL preserve today's caching strategy: the pre-computed `boundary_operators` and `coboundary_operators` collections continue to live on the complex and are returned through the trait methods as `Cow::Borrowed`. No structural change to `SimplicialComplex`'s storage is required. The `Cow::Borrowed` return SHALL guarantee that no `CsrMatrix` clone is performed on the read path used by `Manifold`'s differential operators in Part B.

#### Scenario: Boundary matrix returned as zero-copy Cow::Borrowed

- **WHEN** `<SimplicialComplex<C> as ChainComplex>::boundary_matrix(k)` is called with `k > 0`
- **THEN** the returned `Cow<'_, CsrMatrix<i8>>` is `Cow::Borrowed(&self.boundary_operators[k - 1])`
- **AND** no clone of the underlying `CsrMatrix` is performed
- **AND** the dereferenced value is bit-for-bit identical to what today's `boundary_operator_impl(k)` returns (renamed from `boundary_operator_cpu` as part of the Stage A `_cpu` cleanup)

#### Scenario: Coboundary matrix returned as zero-copy Cow::Borrowed

- **WHEN** `<SimplicialComplex<C> as ChainComplex>::coboundary_matrix(k)` is called
- **THEN** the returned `Cow<'_, CsrMatrix<i8>>` is `Cow::Borrowed(&self.coboundary_operators[k])`
- **AND** no clone of the underlying `CsrMatrix` is performed
- **AND** the dereferenced value is bit-for-bit identical to what today's `coboundary_operator_impl(k)` returns (renamed from `coboundary_operator_cpu` as part of the Stage A `_cpu` cleanup)

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

### Requirement: Simplex implements the Cell marker trait

`Simplex` SHALL implement the `Cell` trait. `Cell::dim()` returns the simplex's topological dimension (`vertices.len() - 1`, or `0` for the empty simplex). `Cell::boundary()` returns the standard signed boundary chain: for each vertex `v_i` of a k-simplex `[v_0, ..., v_k]`, a (k-1)-face is produced by removing `v_i` with sign `(-1)^i`; the boundary of a 0-simplex is empty.

This requirement is recorded retroactively: the `Cell` impl is added as part of Stage A so that `<SimplicialComplex<T> as ChainComplex>::CellType = Simplex` satisfies the trait's `type CellType: Cell` bound. The pre-Stage-A codebase did not implement `Cell` for `Simplex` because the original `CWComplex` impl on `SimplicialComplex` did not exist.

#### Scenario: Simplex::dim returns one less than vertex count

- **WHEN** the user constructs `Simplex::new(vec![0, 1, 2])`
- **THEN** `<Simplex as Cell>::dim(&simplex)` returns `2`

#### Scenario: Simplex::boundary follows the standard signed-face convention

- **WHEN** the user calls `<Simplex as Cell>::boundary` on the triangle `Simplex::new(vec![0, 1, 2])`
- **THEN** the returned chain is `[(Simplex::new(vec![1, 2]), 1), (Simplex::new(vec![0, 2]), -1), (Simplex::new(vec![0, 1]), 1)]` (sign pattern `+, -, +` from `(-1)^i`)

#### Scenario: SimplicialComplex::betti_number is computed via SVD-based rank

- **WHEN** `<SimplicialComplex<T> as ChainComplex>::betti_number(k)` is called
- **THEN** the implementation lifts the boundary `CsrMatrix<i8>` matrices to `f64`, computes their ranks via SVD with tolerance `1e-5`, and returns `n_k - rank(∂_k) - rank(∂_{k+1})` (with `saturating_sub` to avoid underflow)
- **AND** this mirrors the technique already used by `CellComplex::betti_number` to keep the two impls consistent
