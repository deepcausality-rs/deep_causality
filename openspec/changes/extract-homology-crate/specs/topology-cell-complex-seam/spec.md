## ADDED Requirements

### Requirement: The geometric half of the trait stays in topology
`deep_causality_topology` SHALL define `CellComplex: ChainComplex` carrying the associated types
`CellType`, `CellIter` and `Metric`, together with `cells(k)` and `uniform_lattice_layout()`.

These five items describe a complex that comes from a space. A Riemannian metric and a per-axis
lattice extent are not part of a chain complex, and a trait that carried them could not describe a
parity-check code.

#### Scenario: All three shipped complexes implement both traits
- **WHEN** `SimplicialComplex`, `CellComplex<C>` and `LatticeComplex` are built
- **THEN** each implements `ChainComplex` and `CellComplex`, and the boundary operators are produced
  by the same code as before

#### Scenario: Cell enumeration reaches the chain groups
- **WHEN** `cells(k)` is enumerated on any implementor
- **THEN** the count equals `num_cells(k)` from the supertrait

### Requirement: Existing consumers compile without an edit
`deep_causality_topology` SHALL re-export `ChainComplex`, `HomologyField` and `Gf2Chain`, so that an
existing `use deep_causality_topology::ChainComplex` continues to resolve.

`deep_causality_cfd` and `deep_causality_physics` name the trait in twelve files and call only
`num_cells` and `uniform_lattice_layout`. Requiring them to change an import converts an additive
change into a coordinated one for no benefit.

#### Scenario: Dependents build with no source or manifest edit
- **WHEN** `deep_causality_cfd` and `deep_causality_physics` are built after the extraction, with no
  edit to any `Cargo.toml` and no edit to any `use` statement
- **THEN** both compile and their test suites pass

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
