## Context

Stage-4 immersed bodies are carried by a `CutCellRegistry`: each intersected lattice cell is a
`CutCell` tagged `Fluid | Cut | Solid`, with a clipped fluid volume, per-axis apertures (wetted face
fractions), and a list of `CutFaceFragment`s (each an area plus an outward unit normal pointing from
solid into fluid). Two mechanisms consume this geometry today:

- **The cut Hodge star** (`dual_fluid_fraction`) clips dual volumes by the wetted fraction, so the
  operators see the blockage. This already handles *no-penetration in the mass sense* (blocked flux
  is down-weighted).
- **The no-slip constraint** (`CutCellRegistry::solid_incident_edges` → `NoSlipConstraint`) pins to
  zero every edge incident to a `Solid` cell, and feeds the **constrained Leray projector**
  (`leray_project_constrained_opts`), a KKT projection onto `{divergence-free} ∩ {u|_E = 0}` for a
  binary edge set `E`.

The gap: the no-slip wall is imposed on the **staircase boundary of the `Solid` region**, not on the
true surface, which lies inside the `Cut` cells. `Cut` cells carry unconstrained flow. The effective
wall therefore sits up to a cell off the real surface and is axis-aligned, which mispredicts
separation and adds dissipation. Measured: the cylinder does not shed at `Re = 100` for ≤ 16 cells/D;
the wake stays steady (a sub-critical discretization).

Constraints from the repo: no `unsafe`, static dispatch, one-type-one-module, full coverage of
library code, examples verified by running. The marched solver `step(&self, …)` is stateless and must
stay so; the projector is the single chokepoint every stage and the re-entry pass go through.

## Goals / Non-Goals

**Goals:**
- Impose no-slip / no-penetration at the wetted cut face of `Cut` cells, using the fragment normal
  and apertures, so the discrete body is smooth rather than staircased.
- Lower the resolution at which the cylinder sheds at `Re = 100` to ≤ 24 cells/D and move `C_d`
  toward Lehmkuhl.
- Keep existing consumers working: the constrained/open projector and the cut star compose with the
  new constraint, and existing call sites are unchanged (additive surface only).
- Reduce exactly to the staircase set on axis-aligned `Solid` layers (consistency gate), and stay
  bit-identical on empty-registry / periodic / wall-only paths.

**Non-Goals:**
- Moving / rotating immersed surfaces (prescribed time-dependent wall velocity). Deferred.
- High-Re wall functions / turbulence modelling (Stage 5).
- Slip (free-slip) on the cut face; this change is no-slip / no-penetration only.
- Changing the time integrator or the cut Hodge star derivation.

## Decisions

### Decision 1 — Impose the wall condition at the cut-face fragment, not at solid-incident edges

The constraint source becomes the `Cut` cell's fragment(s) and apertures. A fully-`Solid` cell keeps
its zero-interior pin (no flow inside the body); a `Cut` cell contributes a wall condition located at
its fragment. This is the load-bearing change; everything below is how to express it in the
edge-cochain framework.

*Alternative considered:* keep zeroing whole edges but pick the set by an aperture threshold
(pin a `Cut` cell's edges when its fluid fraction is below ½). This shifts the staircase by ~half a
cell and needs no projector change, but it is still a staircase and still axis-aligned, so it is kept
only as a fallback/■stepping-stone, not the target.

### Decision 2 — Express the wall condition as aperture-weighted linear constraints in a generalized KKT projector (additive)

The existing constrained projector already solves a KKT system for `{δu = 0} ∩ {u|_E = 0}`. Generalize
it to also accept a set of **weighted linear constraints** `Cᵀ u = b` (the fragment no-slip rows),
solved in the same projection. The binary edge set is the special case where each constraint fixes one
edge to zero with infinite weight. Each `Cut` cell contributes:

- a **no-penetration row**: the reconstructed velocity normal component at the fragment is zero
  (`n · u_face = 0`), weighted by the fragment area;
- a **no-slip (tangential) row**: the reconstructed tangential velocity at the fragment is zero,
  weighted by the wetted measure.

The reconstruction `u_face` is the area/aperture-weighted interpolation of the cell's incident edges
(the same `sharp`-style metric averaging the diagnostics use), so the constraint is a sparse linear
combination of edge coefficients. The projector exposes this through a **new additive method**
(e.g. `leray_project_constrained_weighted_opts`); the existing `*_constrained_opts` / `*_open_opts`
signatures and behavior are unchanged, satisfying "no API break" and keeping a single projector
family.

*Literature-grounded form (Kirkpatrick et al. 2003, §2.3.2–2.3.3).* Their staggered cut-cell method
gives the concrete recipe this design mirrors: the no-slip enters the viscous operator as a
**one-sided wall-normal gradient to the surface**, `∂u/∂n ≈ N·(u_e − u_b)/Δh` with `u_b = 0` and `Δh`
the **true** perpendicular distance from the node to the immersed surface (not the staircase
distance); and the wall shear / friction drag uses the **actual surface area inside the cell** with
`u_i/Δh ≈ S_ij · N_j` (the strain-rate tensor contracted with the surface normal). Two consequences:
(1) the cut-face rows should be built from the true `Δh` and the fragment normal, so the constraint
"reaches" the real surface, and (2) the same `S_ij · N_j` form is what `viscous_surface_force`
already computes for `C_d` — so the friction diagnostic should switch from its current central
difference to this one-sided wall-normal gradient with true `Δh`. Symmetry preservation across the
cut (skew-symmetric convection, SPD diffusion) follows Dröge & Verstappen (2005), the method most
structurally identical to this DEC solver, validated on the Re=100 cylinder.

*Alternatives considered:*
- **Direct-forcing IBM** (add a body force each step driving `u_face → 0`). Decouples from the
  projector but introduces a tunable forcing strength and a non-divergence-free correction that the
  next projection must clean up; rejected as less robust and harder to make consistent with the
  energy-conserving formulation.
- **Pure aperture-thresholded binary set** (Decision 1 alternative). Cheapest, but does not deliver
  the smooth boundary the goal needs; retained only as a fallback if the weighted KKT proves
  ill-conditioned.

### Decision 3 — Reduction and consistency

When a cell's apertures are all 0 or 1 with axis-aligned fragments (a `Solid` layer on a wall), the
weighted rows collapse to the binary wall-tangential pins, so the generalized projector returns the
staircase result bit-for-bit. This is the consistency gate that protects the Poiseuille/Ghia and
"axis-aligned solid layer reproduces the wall Poiseuille" results.

### Decision 4 — Geometry lives in topology, assembly in physics

The fragment→constraint derivation (which edges interpolate a fragment, the weights from area and
apertures) is cut-geometry and belongs in `deep_causality_topology`, beside `solid_incident_edges`
(e.g. a `CutCellRegistry::cut_face_constraints`). The physics crate's `NoSlipConstraint` assembles the
returned rows and hands them to the generalized projector. This mirrors how B4/B5 already split
geometry (topology) from wiring (physics).

## Risks / Trade-offs

- **[Generalized KKT conditioning]** Weighted constraint rows can degrade the projection CG's
  conditioning, especially for slivers. → Reuse the existing cell-merging volume-fraction floor and
  the Jacobi-preconditioned CG; gate on the cylinder run converging within budget; fall back to the
  aperture-thresholded binary set (Decision 2 alternative) if convergence is unacceptable.
- **[Reconstruction order]** The fragment-velocity interpolation is at best second-order; the cut
  face is a flat facet of a curved surface. → Accuracy is still a large improvement over the
  staircase; the validation gate is shedding at ≤ 24/D and `C_d` trend, not a pointwise wall law.
- **[It may not be enough alone]** The hypothesis is that the staircase raises the discrete critical
  Reynolds number above 100; the literature supports the weaker, certain statement that the staircase
  is markedly less accurate near the wall and needs a finer grid for the same accuracy (Kirkpatrick
  et al. 2003 show cut-cell errors ~4× lower in velocity and ~10× lower in vorticity/strain at Re=40,
  worst for the staircase at 20°–90° where the boundary layer is thinnest — but both *converge*). So
  smoothing the body is expected to lower the resolution needed for correct separation, not
  necessarily to make 12/D shed. → Compose with resolution; the gate allows ≤ 24/D. If the resolved
  body still will not shed at 24/D, that narrows the cause toward pure resolution rather than the
  boundary representation.
- **[Divergence-free guarantee]** Adding constraint rows must not break the `δu = 0` interior
  property. → The KKT formulation enforces both simultaneously (as the current constrained projector
  does for the binary case); a divergence-residual test gates this.
- **[Scope creep into moving bodies]** The row form admits a non-zero right-hand side (`Cᵀu = b`),
  which is the natural hook for prescribed wall motion. → Keep `b = 0` here; document the hook,
  defer the moving-body case.

## Open Questions

- Exact weighting of the no-slip vs no-penetration rows relative to the area and aperture measures —
  to be pinned by the consistency reduction and a small analytic cut-face test.
- Whether the no-penetration row is redundant given the cut Hodge star already down-weights blocked
  flux, or whether both are needed for the tangential condition to bite. Resolve with an ablation on
  the cylinder (penetration row on/off).
- Whether the generalized projector should subsume the binary `zeroed_edges` path internally
  (single code path) or keep both (binary fast path + weighted path). Prefer subsuming if it stays
  bit-identical on the binary case.

## References

- M. Dröge, R. Verstappen (2005). *A new symmetry-preserving Cartesian-grid method for computing flow
  past arbitrarily shaped objects.* Int. J. Numer. Methods Fluids 47(8–9):979–985. DOI 10.1002/fld.924.
  — The structurally identical method (skew-symmetric convection + SPD diffusion, energy-conserving)
  with cut cells; Re=100 cylinder reference: St=0.165, C_d=1.24 (pressure 0.93 + friction 0.31), θ_sep=117°.
- M.P. Kirkpatrick, S.W. Armfield, J.H. Kent (2003). *A representation of curved boundaries for the
  solution of the Navier–Stokes equations on a staggered three-dimensional Cartesian grid.* J. Comput.
  Phys. 184(1):1–36. DOI 10.1016/S0021-9991(02)00013-X. — The concrete cut-cell no-slip recipe
  (one-sided wall-normal gradient with true Δh; wall shear via `S_ij·N_j` and actual surface area) and
  the cell-linking small-cell fix.
- R. Mittal, G. Iaccarino (2005). *Immersed Boundary Methods.* Annu. Rev. Fluid Mech. 37:239–261. DOI
  10.1146/annurev.fluid.37.061903.175743. — Taxonomy; cut-cell finite-volume is the route to strict
  local+global conservation; rationale for rejecting forcing/penalization for rigid bodies.
- M.S. Mohamed, A.N. Hirani, R. Samtaney (2016). *Discrete exterior calculus discretization of
  incompressible Navier–Stokes equations over surface simplicial meshes.* J. Comput. Phys. 312:175–191.
  DOI 10.1016/j.jcp.2016.02.028. — DEC-NS foundation; simplicial + stream-function, no immersed cut
  cells, confirming this change is novel rather than a port.
