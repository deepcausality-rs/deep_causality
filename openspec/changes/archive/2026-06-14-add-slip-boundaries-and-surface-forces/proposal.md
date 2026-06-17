## Why

The `add-boundary-zone-abstraction` change shipped the streamwise inflow/outflow surface, which
unblocked **part** of the isolated-cylinder Reynolds ladder (`add-cut-cells-and-immersed-boundaries`
D2/D3 — Strouhal and drag vs Williamson and Lehmkuhl et al. 2013). But it is necessary, not
sufficient: a faithful *isolated* cylinder and its **drag** still need two capabilities the solver
does not have.

1. **Lateral isolation.** The only lateral boundaries today are no-slip walls (which *confine* the
   cylinder — wrong blockage and wrong Strouhal) or periodicity (a periodic *array*, not an
   isolated body). A true isolated cylinder needs a **free-slip / far-field** lateral boundary
   (no penetration, zero tangential shear), so the top/bottom do not impose a spurious boundary
   layer.
2. **Drag.** There is no surface-force diagnostic for the immersed body. The cut-face fragments
   already carry the surface geometry (area + outward normal), but nothing integrates the pressure
   and viscous traction over them to a force — so `C_d`/`C_l` cannot be reported.

Without these, building the D2/D3 ladder would mean validating a *confined or periodic-array*
cylinder against *isolated-cylinder* references — an overclaim. This change supplies the two
missing primitives so the ladder can be built honestly.

**3D is in scope** and near-free: the solver, the DEC operators, the boundary zones, and the
surface-force diagnostic are all `const D`-generic (the dec suite already runs at `D=3`), so the
isolated-cylinder validation extends to a 3D box with an extruded cut cylinder with no new
machinery — only added test coverage. This reaches the **3D-transition regime** (Re ≈ 200–300, a
feasible DNS rung) and, in principle, **Re ≈ 3900 by DNS** (the canonical benchmark — supported by
the structure-preserving scheme but **compute-bound**, so not a routine/CI rung). A *cheap* high-Re
path (a turbulence closure) and wall functions are **not** in scope (Stage 5).

## What Changes

- **Free-slip / far-field boundary zone (NEW, `slip-boundary`).** A `SlipWall` (free-slip)
  `BoundaryZone` on a lattice face: **no penetration** (zero wall-normal flux — already the
  projection's Neumann condition at a closed face) with a **free tangential** velocity (zero
  shear), in contrast to the auto-derived no-slip that pins the tangential edges. Realized by
  excluding the face's wall-tangential edges from the no-slip constraint set (a zone that *un-pins*
  a face), with the viscous operator giving the zero-shear (Neumann) tangential condition through
  the existing boundary-clipped Hodge star. Reduces to the closed/no-slip behavior on any face not
  declared slip.
- **Surface-force diagnostic (NEW, `surface-force-diagnostic`).** Integrate the force on an
  immersed cut body — `F = ∮ (−p n + μ(∇u+∇uᵀ)·n) dA` — over its `CutFaceFragment`s (each carrying
  area and outward normal), using the projection's recovered pressure potential and the
  edge-cochain velocity gradient. Reports the force vector and the nondimensional `C_d`/`C_l` given
  a reference velocity and length. Precision-generic over `R: RealField`.

## Dependencies and sequencing

- **Depends on:** `add-boundary-zone-abstraction` (the `BoundaryZone` framework, the open-boundary
  projection) and `add-cut-cells-and-immersed-boundaries` (the `CutCellRegistry` and
  `CutFaceFragment` surface). Both are landed/archived or in progress.
- **Unblocks:** `add-cut-cells-and-immersed-boundaries` D2/D3 — the isolated-cylinder external-flow
  domain (west `Inflow`, east `Outflow`, far-field `SlipWall` top/bottom, immersed cut cylinder)
  with `C_d` and Strouhal reported. That ladder is implemented and closed **in that change** after
  this one lands.

## Non-Goals

- **No turbulence closure** (LES/RANS) and **no wall functions** — a cheap high-Re path is Stage 5.
  (3D **DNS** is in scope; Re ≈ 3900 is reachable by DNS but compute-bound, hence not a routine
  rung — it is supported, not gated.)
- No moving/curved-surface force refinements beyond the shipped fragment quadrature.
