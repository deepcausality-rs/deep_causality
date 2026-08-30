## ADDED Requirements

### Requirement: The geometric half of the trait stays in topology
`deep_causality_topology` SHALL define `CellularComplex: ChainComplex` carrying the associated types
`CellType`, `CellIter` and `Metric`, together with `cells(k)` and `uniform_lattice_layout()`.

These five items describe a complex that comes from a space. A Riemannian metric and a per-axis
lattice extent are not part of a chain complex, and a trait that carried them could not describe a
parity-check code.

#### Scenario: All three shipped complexes implement both traits
- **WHEN** `SimplicialComplex`, `CellComplex<C>` and `LatticeComplex` are built
- **THEN** each implements `ChainComplex` and `CellularComplex`, and the boundary operators are produced
  by the same code as before

#### Scenario: Cell enumeration reaches the chain groups
- **WHEN** `cells(k)` is enumerated on any implementor
- **THEN** the count equals `num_cells(k)` from the supertrait

### Requirement: Consumers of the homology half compile without an edit
`deep_causality_topology` SHALL re-export `ChainComplex`, `HomologyField` and `Gf2Chain`, so that an
existing `use deep_causality_topology::ChainComplex` continues to resolve and every homology method
reached through it continues to resolve.

A re-export carries a name, and it cannot carry a method to a trait that no longer owns it. Code
that reached a *geometry* method — `cells` or `uniform_lattice_layout` — through a `ChainComplex`
import must name `CellularComplex` instead, because that is the trait the method now belongs to.
`CellularComplex` has `ChainComplex` as a supertrait, so the change is the import line and nothing
else.

Measured over the workspace at the time of the extraction: `deep_causality_cfd` and
`deep_causality_physics` name the trait in twelve files. Eighteen `num_cells` call sites and every
`deep_causality_physics` file compile untouched. Two sites in `deep_causality_cfd` needed the
import — `src/solvers/dec/spectral_diffusion.rs` calling `uniform_lattice_layout()`, and
`tests/solvers/dec/cut_cell_wiring_tests.rs` calling `cells()` at four places.

#### Scenario: A homology-only consumer builds with no edit at all
- **WHEN** a dependent that calls only `num_cells`, `max_dim`, `boundary_matrix`,
  `coboundary_matrix`, `betti_number` or `betti_number_over` through a `ChainComplex` import is
  built after the extraction
- **THEN** it compiles with no edit to its `Cargo.toml` and no edit to any `use` statement

#### Scenario: A geometry consumer needs the owning trait named, and nothing more
- **WHEN** a dependent calls `cells` or `uniform_lattice_layout` through a `ChainComplex` import
- **THEN** the build fails until the import names `CellularComplex`, and changing that one line is
  sufficient — no call site, signature or bound elsewhere in the file changes

#### Scenario: The re-exported names resolve to the moved definitions
- **WHEN** a caller imports `ChainComplex` from topology and another imports it from homology
- **THEN** both name the same trait and a value implementing one satisfies the other

#### Scenario: Version requirements are unchanged
- **WHEN** the four crates depending on topology are inspected after the change
- **THEN** each still requires `version = "0.7"` and picks up the new patch with no file touched

### Requirement: Matrix conversions live with the matrices
The `CsrMatrix<i8>` to dense `i64` widening SHALL move out of `deep_causality_topology` into
`deep_causality_linear`'s conversion module, beside the conversions already there.

It is a matrix conversion with no homological content, and it currently sits as a private helper in
the crate furthest from the matrices.

#### Scenario: The widening is available to any caller of linear
- **WHEN** a crate depending only on `deep_causality_linear` needs an `i8` sparse matrix widened to
  dense `i64`
- **THEN** the conversion is public in that crate

#### Scenario: Rank over the rationals is unchanged
- **WHEN** `HomologyField::Rational` computes a rank before and after the move
- **THEN** the two agree for every shipped complex at every grade

### Requirement: Geometry-coupled objects stay where their geometry is
`Chain<T>` and `cup_product` SHALL remain in `deep_causality_topology`.

`Chain<T>` holds an `Arc<SimplicialComplex<T>>` and compares complexes with `Arc::ptr_eq`. The cup
product needs `SplittableCell::split` for the Alexander–Whitney diagonal; a chain complex has no
multiplication, and a diagonal approximation comes from cells.

#### Scenario: The homology crate does not name them
- **WHEN** the public surface of `deep_causality_homology` is enumerated
- **THEN** neither `Chain<T>` nor any cup product appears

#### Scenario: The cup product keeps its current results
- **WHEN** the cup product is computed on a tetrahedron at every degree pair before and after the
  change
- **THEN** the results agree exactly

### Requirement: The extraction changes no computed answer
Every Betti number, rank, chain operation and cup product SHALL return the value it returned before
the change, for every complex under test over both coefficient fields.

This is a relocation, not a reimplementation. A changed answer means something moved that should not
have.

#### Scenario: The whole workspace suite passes
- **WHEN** `bazel test //...` runs after the change
- **THEN** every test passes, and the count is at least the count before the change

#### Scenario: Betti numbers agree across the move
- **WHEN** `betti_number_over(k, field)` is evaluated at every grade of every shipped complex over
  both fields, before and after
- **THEN** every pair agrees
