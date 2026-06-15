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

**Guiding star (the success metric this design optimises for):** a **reference-quality Re=100 cylinder
validation that runs in minutes, not hours** — St within a few percent of 0.164 and `C_d` near 1.24
(0.93 pressure + 0.31 friction). The lever is *lowering the shedding-threshold resolution*: accuracy
and speed are coupled here, because a fragment-accurate wall both sharpens St/`C_d` and lets the wake
shed on a much coarser (hence far cheaper) grid. Concretely: dropping the threshold from ~24/D to
~16/D is ≈ (24/16)² fewer cells × a larger `dt` ≈ ~3× less work to the developed state, on top of the
already-shipped warm-start and loose-tolerance speedups. Every choice below is judged first by whether
it moves that needle.

**Goals:**
- Impose no-slip / no-penetration at the wetted cut face of `Cut` cells, using the fragment normal
  and apertures, so the discrete body is smooth rather than staircased.
- **Lower the shedding threshold** at `Re = 100` toward ~16 cells/D (currently ~24/D with the
  staircase) so a developed validation run completes in minutes; move St toward 0.164 and `C_d` toward
  1.24 (pressure 0.93 + friction 0.31).
- Keep existing consumers working: the constrained/open projector and the cut star compose with the
  new constraint, and existing call sites are unchanged (additive surface only).
- Reduce exactly to the staircase set on axis-aligned `Solid` layers (consistency gate), and stay
  bit-identical on empty-registry / periodic / wall-only paths.

**Locked plan (decided with the user):** primary mechanism is the **aperture-weighted fragment
kinematic constraint** (Decision 2, Lever A2), built on the **existing cut Hodge star** (Lever B1, no
new metric code), reached via an **A1 → A2 phasing** that shares the generalized projector so the only
go/no-go cost is the cheap Group A slice. The **one-sided wall-normal friction diagnostic** (Lever C)
is an independent, free accuracy win on the friction-`C_d` component. A metric refinement (Lever B2)
is deferred unless the single-cut-cell gate shows the near-wall gradient is wrong.

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

### Decision 2 — No-slip is a kinematic constraint at the fragment (KKT projector); the cut Hodge star carries the wall metric

*(Resolves the Decision-2-vs-Kirkpatrick tension raised at apply time. The two earlier threads —
"projector constraint" and "viscous-operator one-sided gradient" — are not alternatives in DEC; one
is the constraint, the other is the metric, and they compose.)*

In this DEC solver no-slip is a **kinematic** condition (`u = 0` at the wall), already enforced by the
**constrained Leray projector**, *not* by a finite-volume diffusive-flux stencil. The viscous term is
`−ν Δ₁ u` with `Δ₁ = δd + dδ`, `δ = ⋆⁻¹ d ⋆`; the wall geometry enters the operator **only through the
Hodge star**, and the cut star (`dual_fluid_fraction`) already clips dual volumes by the wetted
fraction. So the aperture-resolved change has two complementary, DEC-native levers and **adds no
viscous stencil**:

- **Lever A — the constraint subspace (the new work).** Replace the binary staircase pins with
  **aperture-weighted linear constraints** added to the same KKT projection (`Cᵀu = 0`): per cut cell,
  a **no-penetration** row `n·u_face = 0` (fragment outward normal) and a **tangential no-slip** row,
  where `u_face` is the cut cell's velocity reconstructed from its incident edges with the per-axis
  apertures as weights — so the condition is enforced at the wetted fragment, not at the staircase
  edges. The binary set is the special case (single-edge, unit-weight, zero-target rows), so the
  existing path stays bit-identical. Exposed as a **new additive method**
  (`leray_project_constrained_weighted_opts`); existing projector signatures/behaviour are unchanged.
- **Lever B — the metric (already shipped, no new code).** The cut Hodge star is the DEC analogue of
  Kirkpatrick's "true wall distance Δh in the wall-normal gradient": the clipped dual volumes make
  `Δ₁` see the wetted geometry. Lever A composes on top of it.

**Where Kirkpatrick (2003) actually applies.** Their one-sided wall-normal gradient and `S_ij·N_j`
wall shear are a *finite-volume* construction; in DEC the gradient distance lives in the Hodge star
(Lever B), so there is **no separate viscous-flux stencil to modify**. The `S_ij·N_j` form with the
true `Δh` *does* apply directly to the read-only **`viscous_surface_force` diagnostic** (the
friction-`C_d` post-process), which should switch from its central difference to the one-sided
wall-normal gradient — but that is a separate diagnostic refinement, not the solver's no-slip
mechanism. Symmetry preservation across the cut (skew-symmetric convection, SPD diffusion) follows
Dröge & Verstappen (2005), the method structurally identical to this solver.

**Phasing (de-risks the formulation before the compute-heavy cylinder run).**
- *Phase 1:* the aperture-weighted rows above, gated by a **single-cut-cell analytic test** — build one
  cut cell of known geometry, project, and check the reconstructed fragment velocity is zero to
  tolerance. This validates the formulation cheaply, closing the gap that the axis-aligned reduction
  test (no cut cells) cannot.
- *Phase 2 (only if Phase 1 underperforms on the cylinder):* tune the row weights / reconstruction.

*Alternatives considered:*
- **Cell-centre (aperture-blind) constraint** — pin each cut cell's `sharp` centre velocity to zero
  (`Σ axis-edges = 0`). Simplest and the degenerate case of Lever A, but places the wall at the cell
  centre (a half-cell-shifted smoothed staircase); kept only as a fallback if the weighted rows are
  ill-conditioned.
- **Direct-forcing IBM** — a body force driving `u_face → 0`. Decouples from the projector but needs a
  tuned forcing strength and a non-divergence-free correction the next projection must clean up;
  rejected as less robust and inconsistent with the energy-conserving formulation.
- **Modifying a viscous-flux stencil (literal Kirkpatrick)** — rejected: DEC has no such stencil; the
  metric (Lever B) is the correct home, and it already exists.

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

- Exact weighting of the no-slip vs no-penetration rows relative to the area/aperture measures.
  *Resolution path:* pinned by the **single-cut-cell analytic test** (Decision 2, Phase 1) — the
  weights are whatever drives the reconstructed fragment velocity to zero on a known cut geometry —
  plus the axis-aligned reduction. No longer gated on the compute-heavy cylinder run.
- Whether the no-penetration row is redundant given the cut star (Lever B) already down-weights
  blocked flux. *Resolution path:* a cheap on/off ablation on the single-cut-cell test, not the
  cylinder.
- Whether the weighted projector subsumes the binary `zeroed_edges` path (single code path) or keeps
  both. *Leaning:* subsume — the binary set is the unit-weight special case (Decision 2) — provided
  the binary-equivalence test stays bit-identical (Group B task 2.2).

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
