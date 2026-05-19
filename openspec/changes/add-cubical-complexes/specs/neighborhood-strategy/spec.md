## ADDED Requirements

### Requirement: Neighborhood strategy trait uses static dispatch

The crate `deep_causality_topology` SHALL expose a public trait `Neighborhood<K: ChainComplex>` that describes how to enumerate the neighbors of a cell in a complex of type `K`. The trait MUST NOT use `dyn` or `Box<dyn ...>`. Concrete strategy implementors SHALL be zero-sized types where the strategy carries no data, so that the cost of passing one to a generic function monomorphizes to nothing.

#### Scenario: Strategy is a zero-sized parameter

- **WHEN** the user calls `manifold.neighbors(VonNeumann, cell_id)` on a `Manifold<CubicalComplex<3>, F>`
- **THEN** `std::mem::size_of::<VonNeumann>()` is `0`
- **AND** the call compiles and yields the 6 face-adjacent 3-cells around `cell_id` on a `CubicalComplex<3>`

#### Scenario: User-defined strategies plug in without forking the crate

- **WHEN** a downstream user declares `struct HalfSpace { plane: u8 }` and implements `Neighborhood<CubicalComplex<3>>` for it
- **THEN** `manifold.neighbors(HalfSpace { plane: 2 }, cell_id)` compiles and runs against the user-defined strategy

### Requirement: Chain-complex-generic strategies are derived from boundary and coboundary

`FaceAdjacent` and `CofaceAdjacent` SHALL be implemented as `Neighborhood<K>` for every `K: ChainComplex`. Their definitions SHALL depend only on `K::boundary_matrix` and `K::coboundary_matrix` respectively, so that they extend to any complex satisfying the trait.

#### Scenario: FaceAdjacent on a simplicial complex

- **WHEN** `FaceAdjacent` is used on a `SimplicialManifold<C, F>` to query neighbors of a 2-simplex σ
- **THEN** the returned iterator yields every other 2-simplex τ such that σ and τ share a 1-simplex (edge)

#### Scenario: FaceAdjacent on a cubical complex coincides with Von Neumann on top cells

- **WHEN** `FaceAdjacent` and `VonNeumann` are each applied to the same top-dimensional cube on `CubicalComplex<3>`
- **THEN** the two iterators yield the same set of cells (order not required to match)

### Requirement: Grid-specific strategies are only implemented for CubicalComplex

`VonNeumann`, `Moore`, and `KRing<const K: usize>` SHALL be implemented as `Neighborhood<CubicalComplex<D>>` only. They SHALL NOT be implemented for `SimplicialComplex`. This asymmetry is intentional: Moore and KRing rely on the regular-grid coordinate structure and Chebyshev metric, which have no principled simplicial analogue.

#### Scenario: Moore on a 3D cube yields 26 neighbors

- **WHEN** `Moore` is applied to an interior top-cell of `CubicalComplex<3>::cubic_open(8)`
- **THEN** the iterator yields exactly 26 cells (3³ − 1)

#### Scenario: KRing on a 2D grid uses Chebyshev distance

- **WHEN** `KRing::<2>` is applied to an interior top-cell of `CubicalComplex<2>::square_open(20)`
- **THEN** the iterator yields exactly 24 cells (5² − 1)
- **AND** every yielded cell has Chebyshev distance at most 2 from the source cell

#### Scenario: Moore is not implementable on SimplicialComplex

- **WHEN** a user attempts to call `simplicial_manifold.neighbors(Moore, cell_id)`
- **THEN** the code fails to compile because `Moore` does not implement `Neighborhood<SimplicialComplex<C>>`

### Requirement: Periodic and open boundary conditions are respected

For `CubicalComplex<D>` with periodic boundaries, neighborhood strategies SHALL wrap across the boundary. For open boundaries, neighbors that would lie outside the shape SHALL be omitted.

#### Scenario: Von Neumann on a torus wraps

- **WHEN** `VonNeumann` is applied to a corner top-cell of `CubicalComplex<2>::square_torus(4)`
- **THEN** the iterator yields 4 neighbors, including cells from the opposite edges

#### Scenario: Von Neumann on an open grid omits out-of-bounds

- **WHEN** `VonNeumann` is applied to a corner top-cell of `CubicalComplex<2>::square_open(4)`
- **THEN** the iterator yields exactly 2 neighbors (the two in-bounds face-adjacent cells)
