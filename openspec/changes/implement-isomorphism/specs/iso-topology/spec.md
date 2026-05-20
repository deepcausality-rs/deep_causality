## ADDED Requirements

### Requirement: `SimplicialComplex<T>` -> `CellComplex<Simplex>` via Tier 1 `From`, with `TryFrom` reverse

The crate `deep_causality_topology` SHALL expose `impl<T> From<SimplicialComplex<T>> for CellComplex<Simplex>` that lifts every simplex in the simplicial complex into a cell of the cell complex without altering structure. The reverse SHALL be `impl<T> TryFrom<CellComplex<Simplex>> for SimplicialComplex<T>` returning a typed error variant when any cell of the input is not a simplex. The forward direction SHALL be total; the reverse SHALL be partial.

#### Scenario: Forward From lifts simplicial complex into cell complex

- **WHEN** a downstream user invokes `CellComplex::<Simplex>::from(simplicial)` for a `SimplicialComplex<T>` containing N simplices
- **THEN** the resulting cell complex SHALL contain exactly N cells
- **AND** every cell SHALL be the corresponding simplex from the input

#### Scenario: TryFrom succeeds for cell complexes whose every cell is a simplex

- **WHEN** a downstream user invokes `SimplicialComplex::<T>::try_from(cell_complex)` for a cell complex whose every cell is a `Simplex`
- **THEN** the result SHALL be `Ok(...)` containing a simplicial complex with the same simplices

#### Scenario: TryFrom rejects cell complexes with non-simplex cells

- **WHEN** a downstream user invokes `SimplicialComplex::<T>::try_from(cell_complex)` for a cell complex containing any non-simplex cell
- **THEN** the result SHALL be `Err(...)` with an error variant identifying the failing cell
- **AND** the result SHALL NOT panic

#### Scenario: No algebraic marker subtraits apply

- **WHEN** a reviewer inspects the simplicial / cell-complex iso
- **THEN** no `GroupIso`, `RingIso`, `FieldIso`, `AlgebraIso`, or `DivisionAlgebraIso` impls SHALL be present
- **AND** the iso operates at the base `From` / `TryFrom` level only

### Requirement: `LatticeComplex<D>` <-> `DualLatticeComplex<D>` Poincaré dual via named Tier 2 witness

The crate `deep_causality_topology` SHALL expose a named witness `PoincareIso<const D: usize>` implementing `Iso<LatticeComplex<D>, DualLatticeComplex<D>>`. The `to_target` method SHALL produce the Poincaré dual: every k-cell of the primal lattice becomes a (D-k)-cell of the dual, preserving incidence relations (boundary in primal corresponds to coboundary in dual). The `to_source` method SHALL invert the same mapping. The iso SHALL be a full bijection on the cell-set level (no partiality).

#### Scenario: to_target swaps cell dimensions

- **WHEN** a downstream user invokes `<PoincareIso<3> as Iso<LatticeComplex<3>, DualLatticeComplex<3>>>::to_target(primal)` for a 3D lattice
- **THEN** every 0-cell (vertex) in the primal SHALL map to a 3-cell in the dual
- **AND** every 1-cell (edge) SHALL map to a 2-cell
- **AND** every 2-cell (face) SHALL map to a 1-cell
- **AND** every 3-cell (cube) SHALL map to a 0-cell

#### Scenario: Round-trip identity holds in both directions

- **WHEN** the test suite runs `assert_witness_iso_round_trip::<PoincareIso<D>, LatticeComplex<D>, DualLatticeComplex<D>>(primal, dual)` for D ∈ {1, 2, 3} with matching independent inputs
- **THEN** the assertion SHALL pass

#### Scenario: Boundary-coboundary law is pinned

- **WHEN** the test suite invokes a domain-specific helper `assert_poincare_dualizes_boundary::<D>` against a representative `LatticeComplex<D>`
- **THEN** the boundary operator on the primal SHALL agree with the coboundary operator on the dual under the iso
- **AND** the symmetric case (coboundary on primal vs boundary on dual) SHALL also hold

#### Scenario: No algebraic marker subtraits apply

- **WHEN** a reviewer inspects `PoincareIso<D>`
- **THEN** no marker subtrait impls SHALL be present (neither lattice type implements `Group`/`Ring`/`Field`)
- **AND** the iso operates at the base `Iso<S, T>` level only
