## Why

The Stage-4 immersed no-slip is a **staircase**: `CutCellRegistry::solid_incident_edges` pins every
edge incident to a `Solid` top cell to zero and leaves `Cut` cells carrying full flow, so a smooth
body is represented as a blocky stair-step. Measured consequence: the isolated-cylinder validation
harness does **not** shed a von-Kármán street at `Re = 100` at 12 or 16 cells/D. It converges to a
steady symmetric wake, because the staircase mispredicts separation and adds spurious dissipation,
raising the discrete critical Reynolds number above 100. That blocks the D2/D3 validation gate
(Strouhal vs Williamson, drag vs Lehmkuhl) at any affordable resolution, since the staircase forces
the grid toward ~32 cells/D before the wake goes unstable.

Aperture-resolved no-slip applies the wall condition at the *actual* immersed surface inside each cut
cell, using geometry the `CutCell` already carries (the cut-face fragment's area and outward normal,
and the per-face apertures). A smooth body separates correctly and dissipates less, so the wake sheds
at a markedly lower resolution. This is the deferred B4 refinement (recorded in
`add-cut-cells-and-immersed-boundaries` tasks.md B4 and `registry.rs::solid_incident_edges`).

## What Changes

- **Cut-face no-slip constraint (NEW).** Derive the immersed no-slip / no-penetration condition from
  each `Cut` cell's `CutFaceFragment` (area + outward unit normal) and per-axis apertures, rather than
  from the binary "edge touches a Solid cell" test. `Cut` cells stop being all-or-nothing at the edge
  level; the wall velocity is enforced on the wetted sub-cell face.
- **Constraint assembly update.** `NoSlipConstraint` consumes the aperture-resolved set in place of
  (or alongside, for fully-`Solid` cells) the staircase set. The result must still be a constraint
  the existing **constrained / open Leray projector** accepts unchanged, so the projector and the cut
  Hodge star (`dual_fluid_fraction`) compose with no API break.
- **`Solid`-interior pinning is retained** for fully-dry cells (no flow inside the body); only the
  fluid↔solid *interface* changes from staircase edges to the resolved cut face.
- **Validation rung.** The cylinder harness re-run shows shedding and a Strouhal within a few percent
  of `0.164` at ≤ 24 cells/D (the staircase does not shed there), and an improved `C_d` against
  Lehmkuhl. No new solver plumbing.

No breaking changes to Stage 1–3 behavior: axis-aligned walls (Poiseuille, Ghia) are untouched, an
empty registry stays bit-identical to Stage-3, and a `Solid`-cell body coincident with an axis-aligned
wall reproduces the existing wall-tangential set.

## Capabilities

### New Capabilities
- `aperture-resolved-noslip`: enforcing immersed no-slip / no-penetration on the wetted cut-face
  fragment of a `Cut` cell (area- and normal-resolved), its reduction to the staircase set on
  axis-aligned solid layers, and its composition with the constrained/open Leray projector and the
  cut Hodge star.

### Modified Capabilities
<!-- None. The staircase immersed no-slip lives in the not-yet-archived
     `add-cut-cells-and-immersed-boundaries` change, so there is no spec in `openspec/specs/` to
     delta; the existing `no-slip-viscous` capability (axis-aligned walls) is unchanged. -->

## Impact

- **Affected specs (new):** `aperture-resolved-noslip`.
- **Affected code:** `deep_causality_topology` — cut-face constraint derivation on
  `CutCellRegistry` / `CutCell` (geometry is topology's responsibility, beside
  `solid_incident_edges`); `deep_causality_physics` — `NoSlipConstraint` assembly and its hand-off to
  the constrained/open projector. The cylinder validation example (`dec_cylinder_validation`) is the
  runnable gate.
- **No new dependencies.** Repo rules hold: no `unsafe`, static dispatch, one-type-one-module, full
  test coverage of added library code; the example is verified by running.
- **Relationship to D2/D3:** unblocks the quantitative cylinder validation at affordable resolution;
  the staircase path remains correct (and is the reduction target), so prior results do not move.
