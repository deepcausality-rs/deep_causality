## ADDED Requirements

### Requirement: Cut-cell geometry carrier
The topology crate SHALL provide a `CutCell<D>` carrier and a `CutCellRegistry<D>` that
record, per intersected lattice cell, the clipped fluid volume, a per-face aperture (wetted
fraction in `[0, 1]`), the cut-face fragments (each with area, outward unit normal, and a
source-geometry tag), and a `Fluid | Cut | Solid` classification. Cells absent from the
registry SHALL be treated as full fluid cells on the existing uniform fast path. The
registry SHALL be sized to the boundary, not the volume.

#### Scenario: A registered cut cell overrides volume and apertures
- **WHEN** a cell intersected by an immersed surface is looked up in the registry
- **THEN** its clipped volume, per-face apertures, and cut-face fragments are returned, and an unregistered interior cell reports a full volume and unit apertures

### Requirement: Cut geometry feeds the existing Hodge-star dispatch
The cut volume and apertures SHALL feed the cell-volume / dual-volume path consumed by the
Hodge star, generalizing the existing axis-aligned boundary clip (the integer `2^{-b}`
factor) to a continuous wetted fraction. An axis-aligned planar cut SHALL reproduce the
Stage-3 wall clip to rounding, so existing wall-bounded results are unchanged.

#### Scenario: Axis-aligned cut equals the Stage-3 wall clip
- **WHEN** a `CutCell` describes an axis-aligned planar cut coincident with a lattice wall
- **THEN** the resulting clipped dual volumes equal the Stage-3 `boundary_clip` values to rounding

### Requirement: Surface intersection produces apertures and fragments
The crate SHALL compute cube ⋂ analytic-primitive (infinite cylinder, sphere, plane)
intersection in closed form, and cube ⋂ triangle intersection for STL inputs, each yielding
the clipped volume, per-face apertures, and cut-face fragments with outward normals.
Degenerate triangles and near-tangent cuts SHALL be handled explicitly rather than
producing invalid apertures.

#### Scenario: Cube cut by an analytic cylinder matches closed form
- **WHEN** a unit cube is intersected with an analytic cylinder of known radius and axis
- **THEN** the clipped volume and face apertures equal the closed-form values within tolerance at f64 and Float106
