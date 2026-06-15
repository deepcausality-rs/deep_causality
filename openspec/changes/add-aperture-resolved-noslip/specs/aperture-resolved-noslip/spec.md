## ADDED Requirements

### Requirement: Cut-face no-slip derivation

The system SHALL derive the immersed no-slip / no-penetration constraint of a `Cut` cell from that
cell's cut-geometry — its `CutFaceFragment`s (each carrying a `(D−1)`-area and an outward unit
normal) and its per-axis apertures — rather than from the binary "edge incident to a `Solid` cell"
test. The derived constraint SHALL represent the wall condition at the wetted sub-cell surface, so
that the discrete body follows the immersed surface continuously rather than as an axis-aligned
stair-step.

#### Scenario: A cut cell constrains from its fragment, not its whole edge ring

- **WHEN** a registry contains a `Cut` cell with a non-empty fragment set and non-trivial apertures
- **THEN** the immersed constraint derived for that cell is a function of the fragment area, the
  fragment outward normal, and the cell's apertures
- **AND** it is not the unconditional zeroing of every edge incident to the cell

#### Scenario: Fully-fluid and fully-solid cells are unaffected by the cut-face path

- **WHEN** a cell is full `Fluid` (every aperture 1, no fragments) or full `Solid` (every aperture 0)
- **THEN** the cut-face derivation contributes no partial constraint for that cell
- **AND** a full `Solid` cell still pins its interior to zero (no flow inside the body)

### Requirement: No-penetration and no-slip at the wetted surface

The constrained velocity field SHALL satisfy no-penetration (zero flux normal to the immersed
surface) and no-slip (zero tangential velocity relative to a static body, or the prescribed wall
velocity for a moving body) on the wetted cut face, to an accuracy that improves with cut-cell
resolution. The normal direction used SHALL be the fragment's stored outward unit normal.

#### Scenario: No flow penetrates a static immersed body

- **WHEN** the solver marches a flow past a static aperture-resolved immersed body
- **THEN** the velocity reconstructed at the body has no component through the wetted cut face
  (no-penetration) within the projection's solve tolerance
- **AND** the tangential velocity at the wetted face is zero (no-slip) within the same tolerance

### Requirement: Reduction to the staircase set on axis-aligned solid layers

The aperture-resolved constraint SHALL reduce to the existing staircase (wall-tangential) edge set on
an immersed body modelled as `Solid` cells coincident with an axis-aligned wall (apertures 0 or 1,
axis-aligned fragments), so that the axis-aligned no-slip results do not move.

#### Scenario: Axis-aligned solid layer matches the wall solver

- **WHEN** an immersed `Solid` cell layer is placed coincident with a vertex-collocated no-slip wall
- **THEN** the aperture-resolved constraint set equals the staircase `solid_incident_edges` set for
  that configuration
- **AND** the marched steady state reproduces the analytic wall profile (e.g. Poiseuille for the
  reduced channel height) to rounding, as the staircase path already does

### Requirement: Composition with the constrained projector and cut Hodge star

The aperture-resolved constraint SHALL be expressed so that the existing constrained and open Leray
projectors (`leray_project_constrained_opts` / `leray_project_open_opts`) and the cut Hodge star
(`dual_fluid_fraction`) consume it without an API break. The marched field SHALL remain
divergence-free (interior) to the projection tolerance with the aperture-resolved body in place, the
same guarantee the staircase path provides.

#### Scenario: Divergence-free march with an aperture-resolved body

- **WHEN** a flow is marched past an aperture-resolved immersed body through the existing solver
- **THEN** the interior divergence residual stays at the projection's solve tolerance every step
- **AND** no new solver entry point or projector signature is introduced to support the constraint

### Requirement: Non-regression of non-immersed paths

The change SHALL NOT alter axis-aligned wall behavior or the fully-periodic path. An empty cut-cell
registry SHALL leave the march bit-identical to the Stage-3 result.

#### Scenario: Empty registry is bit-identical

- **WHEN** the solver runs with an empty `CutCellRegistry`
- **THEN** the marched state is bit-identical to the same run with no registry attached

#### Scenario: Periodic and wall-only paths are unchanged

- **WHEN** the lattice is fully periodic, or wall-bounded with no immersed body
- **THEN** the constrained-edge set and the marched result are unchanged from before this change

### Requirement: Aperture-resolved cylinder sheds at lower resolution, fast

With the aperture-resolved body, the isolated-cylinder validation SHALL develop a von-Kármán street at
a **lower resolution than the staircase** (which sheds only at ~24 cells/D, marginally), the target
being a sustained street at **~16 cells/D**, so a developed `Re=100` run reaches the measurement window
in **minutes** rather than hours. The Strouhal number SHALL be within a few percent of the reference
(`St(Re=100) ≈ 0.164–0.165`, Williamson; Dröge–Verstappen) and the recovered drag `C_d` SHALL move
toward the reference relative to the staircase — the matched symmetry-preserving cut-cell reference
(Dröge–Verstappen 2005) gives `C_d ≈ 1.24` (pressure ≈ 0.93 + friction ≈ 0.31; friction ≈ 25 %),
against an experimental `C_d ≈ 1.24–1.33`.

#### Scenario: Sheds at a coarser grid than the staircase

- **WHEN** the cylinder validation harness runs at `Re = 100` with the aperture-resolved body at a
  resolution where the staircase body does **not** shed (e.g. 16 cells/D)
- **THEN** the wake probe shows a sustained (non-decaying) shedding oscillation
- **AND** the estimated Strouhal number is within a few percent of `0.164`
- **AND** the same run with the staircase body at that resolution stays steady

### Requirement: One-sided wall-normal friction diagnostic

The `viscous_surface_force` friction-drag diagnostic SHALL evaluate the wall shear with a **one-sided
wall-normal gradient to the true surface distance** (`S_ij · N_j` with the fragment normal and the
perpendicular distance `Δh`), per Kirkpatrick et al. (2003), rather than the central difference it
currently uses. This is an independent, read-only refinement of the friction-`C_d` component and SHALL
NOT change the solver's no-slip mechanism.

#### Scenario: Friction drag uses the true wall distance

- **WHEN** the friction surface force is evaluated on a cut body
- **THEN** the wall shear is computed from the one-sided wall-normal gradient and the true distance
  `Δh`, not a central difference across the cell
- **AND** the change is confined to the diagnostic; the marched solver result is unchanged
