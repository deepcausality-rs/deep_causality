## ADDED Requirements

### Requirement: CubicalComplex is the public textbook name

The crate `deep_causality_topology` SHALL expose `CubicalComplex<const D: usize>` and `CubicalCell<const D: usize>` as the public types representing a D-dimensional cubical complex and its elementary cubes respectively. The previous names `Lattice` and `LatticeCell` SHALL be removed from the public surface. No `pub use Lattice = CubicalComplex` aliases are provided.

#### Scenario: Public names match textbook usage

- **WHEN** a user writes `use deep_causality_topology::CubicalComplex;`
- **THEN** the import resolves
- **AND** `use deep_causality_topology::Lattice;` fails to compile with an unresolved-import error

#### Scenario: Cell type renamed in lock-step

- **WHEN** a user writes `use deep_causality_topology::CubicalCell;`
- **THEN** the import resolves to the type previously known as `LatticeCell`
- **AND** the type retains its `Cell` trait impl, `boundary()` method, `vertices()` method, and bitmask-based orientation encoding unchanged

### Requirement: CubicalComplex implements ChainComplex

`CubicalComplex<D>` SHALL implement the `ChainComplex` trait, exposing static-dispatch cell iteration, boundary and coboundary matrices, max dimension, and Betti numbers.

#### Scenario: Iteration is static dispatch

- **WHEN** a generic function `fn f<K: ChainComplex>(k: &K)` is called with `&CubicalComplex<3>::torus([8, 8, 8])`
- **THEN** `k.cells(2)` yields each 2-cell exactly once without boxing
- **AND** the count equals `k.num_cells(2)`

#### Scenario: Boundary on a cube matches LatticeCell semantics

- **WHEN** the boundary matrix `boundary_matrix(2)` is constructed for `CubicalComplex<2>::square_open(3)`
- **THEN** for every 2-cell, its column has nonzero entries exactly at the four 1-cells produced by `CubicalCell::boundary()`, with matching signs

#### Scenario: Periodic boundaries preserved

- **WHEN** `CubicalComplex<2>::square_torus(4)` is constructed
- **THEN** `betti_number(0) == 1`, `betti_number(1) == 2`, `betti_number(2) == 1` (the homology of a 2-torus)

### Requirement: Module layout follows textbook naming

The source layout SHALL move from `src/types/lattice/` to `src/types/cubical_complex/`, and the HKT extension SHALL move from `src/extensions/hkt_lattice/` to `src/extensions/hkt_cubical_complex/`. Internal submodules referencing the old name (`dual_lattice`, `specialized`) SHALL be renamed for consistency. The physics-specific submodule `gauge_field_lattice` retains its name because "lattice gauge theory" is the established physics term and renaming would be semantically wrong.

#### Scenario: Bazel BUILD files updated

- **WHEN** `bazel build //deep_causality_topology/...` is run
- **THEN** the build succeeds with the new module paths
- **AND** `tests/BUILD.bazel` references `cubical_complex/` rather than `lattice/`

#### Scenario: Test tree mirrors source tree

- **WHEN** an engineer looks for tests covering `CubicalComplex`
- **THEN** they find them under `tests/types/cubical_complex/` with file names matching source files plus the `_tests` suffix
- **AND** every test file is registered in its parent `mod.rs` with `#[cfg(test)]`
