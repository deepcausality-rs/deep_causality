## ADDED Requirements

### Requirement: Cell-volume computation on `CubicalReggeGeometry<D>`

The system SHALL compute the k-volume of every k-cell in a `LatticeComplex<D>` from a `CubicalReggeGeometry<D>`, for every grade `k ∈ 0..=D` and every edge-length uniformity level (`UnitEdge`, `Uniform`, `PerAxis`, `PerEdge`). The computation MUST be exposed as inherent methods `cell_volume(&self, complex, cell_id, grade) -> f64` and `top_cell_volume(&self, complex, cell_id) -> f64` on `CubicalReggeGeometry<D>`.

#### Scenario: Unit-edge grid yields unit volume at every grade

- **WHEN** `CubicalReggeGeometry<D>::unit_edge()` is paired with any `LatticeComplex<D>` and `cell_volume(complex, cell_id, k)` is called for any cell at any grade `k`
- **THEN** the returned value SHALL equal `1.0` exactly (not within a tolerance) for every `k ∈ 0..=D`

#### Scenario: Per-axis grid yields product of active-axis lengths

- **WHEN** `CubicalReggeGeometry<D>::per_axis(lengths)` is used and `cell_volume(complex, cell_id, k)` is called for a k-cell whose orientation bitmask has active dimensions `{i₁, …, iₖ}`
- **THEN** the returned value SHALL equal `lengths[i₁] * … * lengths[iₖ]` to within `f64` round-off

#### Scenario: Top D-cube volume equals product of all axis lengths

- **WHEN** `top_cell_volume(complex, cell_id)` is called for any top D-cube on a per-axis metric
- **THEN** the returned value SHALL equal `axis_lengths.iter().product()` to within `f64` round-off

#### Scenario: Per-edge with uniform lengths matches per-axis

- **WHEN** a `PerEdge { lengths }` metric is constructed where every edge along axis `i` has length `axis_lengths[i]`
- **AND** `cell_volume(complex, cell_id, k)` is called on any k-cell
- **THEN** the returned value SHALL equal the value the equivalent `PerAxis { lengths: axis_lengths }` metric would return, within `1e-12` relative tolerance

### Requirement: Hinge enumeration on `LatticeComplex<D>`

The system SHALL expose a helper `hinge_top_cube_neighbors(&self, hinge_id: CellId) -> impl Iterator<Item = CellId>` on `LatticeComplex<D>` that yields, without duplicates, the `CellId` of every top D-cube incident to the given (D−2)-cell. The implementation MUST reuse the existing cached coboundary operator and MUST NOT introduce a new cache.

#### Scenario: Each interior hinge in 2D has the expected number of incident 2-cubes

- **WHEN** `hinge_top_cube_neighbors(hinge_id)` is called for an interior 0-cell (vertex) on an open 2D lattice of shape `[≥3, ≥3]`
- **THEN** the iterator SHALL yield exactly 4 distinct top-cube CellIds (the four squares meeting at the vertex)

#### Scenario: Each interior hinge in 4D has 4 incident 4-cubes

- **WHEN** `hinge_top_cube_neighbors(hinge_id)` is called for an interior 2-cell on an open 4D lattice of shape `[≥3, ≥3, ≥3, ≥3]`
- **THEN** the iterator SHALL yield exactly 4 distinct top-cube CellIds

#### Scenario: Periodic-boundary hinges include wrap-around neighbors

- **WHEN** the lattice is constructed with `periodic = [true; D]` and `hinge_top_cube_neighbors(hinge_id)` is called for any hinge on the boundary
- **THEN** the iterator SHALL yield the same number of distinct top-cube CellIds as an interior hinge of the same complex, with the wrap-around top cubes appearing in the result

#### Scenario: Result is deterministic and free of duplicates

- **WHEN** `hinge_top_cube_neighbors(hinge_id)` is called twice for the same `hinge_id` on the same complex
- **THEN** both iterators SHALL yield the same multiset of CellIds, and no CellId SHALL appear more than once

### Requirement: Dihedral-angle computation on `CubicalReggeGeometry<D>`

The system SHALL compute the dihedral angle a top D-cube contributes at one of its (D−2)-hinges via an inherent method `dihedral_angle(&self, complex, top_cube_id, hinge_id) -> f64` on `CubicalReggeGeometry<D>`. The returned value MUST be the Euclidean angle (in radians) between the two faces of the top cube that share the hinge, computed in closed form for `UnitEdge`, `Uniform`, `PerAxis`, and `PerEdge` metrics.

#### Scenario: Unit-edge grid yields π/2 dihedral angles everywhere

- **WHEN** the metric is `UnitEdge` (or `Uniform { length: L }` for any positive `L`) and `dihedral_angle(top_cube_id, hinge_id)` is called for any (top cube, incident hinge) pair
- **THEN** the returned value SHALL equal `std::f64::consts::FRAC_PI_2` to within `1e-15`

#### Scenario: Per-axis 2D grid satisfies arctan symmetry around a vertex

- **WHEN** the metric is `PerAxis { lengths: [a, b] }` for `a, b > 0` and `dihedral_angle(c, vertex)` is summed over all four top 2-cubes incident to an interior vertex
- **THEN** the sum SHALL equal `2π` to within `1e-12` (flat-space identity)

#### Scenario: Dihedral angle is well-defined for non-incident pairs (precondition / error)

- **WHEN** `dihedral_angle(top_cube_id, hinge_id)` is called for a `(top_cube_id, hinge_id)` pair where the hinge is not a face of the top cube
- **THEN** the call SHALL either return an `f64::NAN`, return `Err(...)`, or panic with a descriptive message — implementation-defined, but documented in the doc comment

#### Scenario: Per-edge case matches per-axis when edges of each axis share the same length

- **WHEN** a `PerEdge` metric is set so every edge along axis `i` has length `axis_lengths[i]`
- **AND** `dihedral_angle(top_cube_id, hinge_id)` is called for the same `(top_cube_id, hinge_id)` pair under both `PerEdge` and the equivalent `PerAxis`
- **THEN** both results SHALL agree to within `1e-12` relative tolerance

### Requirement: Deficit-angle computation on `CubicalReggeGeometry<D>`

The system SHALL compute the deficit angle at every (D−2)-hinge via an inherent method `deficit_angle(&self, complex, hinge_id) -> f64` on `CubicalReggeGeometry<D>`, defined as `2π − Σ_{c incident to hinge} dihedral_angle(c, hinge)`.

#### Scenario: Unit-edge open lattice has zero deficit at every interior hinge

- **WHEN** the metric is `UnitEdge` on an open `LatticeComplex<D>` of any shape and `deficit_angle(hinge_id)` is called for any interior hinge
- **THEN** the returned value SHALL be exactly `0.0` (the implementation MUST short-circuit this case so the result is bit-exact, not merely within machine epsilon)

#### Scenario: Unit-edge periodic lattice has zero deficit everywhere

- **WHEN** the metric is `UnitEdge` on a `LatticeComplex<D>` with `periodic = [true; D]` and `deficit_angle(hinge_id)` is called for any hinge
- **THEN** the returned value SHALL be exactly `0.0`

#### Scenario: Single-edge perturbation has bounded support

- **WHEN** a `PerEdge` metric starts at all-unit lengths, exactly one edge length is changed, and `deficit_angle(hinge_id)` is evaluated for every hinge
- **THEN** every hinge that is not a face of any top cube containing the perturbed edge SHALL still report `deficit_angle == 0.0` to within `1e-12`

### Requirement: Discrete Einstein–Hilbert (Regge) action

The system SHALL compute the discrete Einstein–Hilbert action of a cubical complex via an inherent method `regge_action(&self, complex) -> f64` on `CubicalReggeGeometry<D>`, defined as `S_R = Σ_h cell_volume(h, D-2) · deficit_angle(h)` summed over every (D−2)-hinge in the complex.

#### Scenario: Flat lattice has zero action

- **WHEN** the metric is `UnitEdge` (or `Uniform { length: L }` for any positive `L`) on a `LatticeComplex<D>` with any boundary conditions and `regge_action(complex)` is called
- **THEN** the returned value SHALL be exactly `0.0`

#### Scenario: Periodic vs. open boundary differs only by boundary contributions

- **WHEN** the same edge-length configuration is used to compute `regge_action(complex)` on an open lattice and on the same-shape lattice with `periodic = [true; D]`
- **THEN** the difference SHALL equal exactly the sum of `cell_volume(h, D-2) · deficit_angle(h)` over the hinges that lie on the open lattice's boundary, to within `1e-10` relative tolerance

#### Scenario: Lorentzian-marked metric is computed as Euclidean

- **WHEN** `regge_action(complex)` is called on a `CubicalReggeGeometry<D>` whose `timelike_axes` field is `Some(...)`
- **THEN** the returned value SHALL equal the value that would be returned by the same metric with `timelike_axes = None` (the field is ignored at this stage; Lorentzian computation is deferred to a follow-up change set), and the method's doc comment SHALL state this explicitly

### Requirement: No breaking changes to existing trait surfaces

The system SHALL deliver all new functionality via inherent methods on `CubicalReggeGeometry<D>` and one helper on `LatticeComplex<D>`. The `ChainComplex`, `Neighborhood`, `Manifold<K, F>`, and existing `CubicalReggeGeometry<D>` public API surfaces shipped by `add-cubical-complexes` MUST remain unchanged. No new public traits SHALL be introduced in this change set.

#### Scenario: Existing `Manifold<LatticeComplex<D>, F>` constructors compile unchanged

- **WHEN** downstream code that already constructs `Manifold::from_cubical(...)` or `Manifold::from_cubical_with_metric(...)` is compiled against this change set
- **THEN** the code SHALL compile and run without modification

#### Scenario: No new public traits exported from the crate

- **WHEN** the crate's public API is inspected after this change set lands
- **THEN** the set of publicly exported traits SHALL be identical to the set exported before this change set
