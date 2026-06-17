## Context

`add-boundary-zone-abstraction` made boundaries composable `BoundaryZone` terms and added the
open-boundary projection; structural no-slip on non-periodic axes is auto-derived
(`NoSlipConstraint`), and immersed bodies are a `CutCellRegistry` on the metric whose
`CutFaceFragment`s carry per-fragment area and outward normal. This change adds the two primitives
a faithful isolated-cylinder validation needs — a free-slip lateral boundary and a surface-force
diagnostic — without 3D or turbulence.

## Goals / Non-Goals

- **Goal:** a `SlipWall` (free-slip / far-field) zone; a drag/lift surface-force diagnostic over
  the immersed cut fragments. Both precision-generic **and dimension-generic** (`const D`), so they
  work in 2D and 3D; closed/no-slip paths bit-identical when no slip face is declared.
- **Non-Goal:** turbulence closure (LES/RANS), wall functions, moving/curved-surface force
  refinements. (3D DNS is in scope; cheap high-Re via a turbulence model is not.)

## Decisions

### D1: Free-slip is "un-pin the tangential edges", not a new operator

No-slip pins a wall's tangential edges to zero (Dirichlet); free-slip leaves them **free** (zero
tangential shear, a Neumann condition) while keeping zero wall-normal penetration. At a closed
lattice face the normal flux is already zero (the projection's Neumann condition), so free-slip is
exactly no-slip **minus** the tangential-edge constraint on that face. The auto no-slip derivation
currently constrains every non-periodic face; a `SlipWall` zone declares a face whose tangential
edges are **removed** from the no-slip set, leaving the boundary-clipped Hodge star to give the
zero-shear viscous condition.

Because the existing zone hooks only *add* contributions, the solver gains a small **un-pin**
seam: the no-slip set becomes `(auto walls ∪ zone constrained_edges) \ (zone slip_edges)`. A
`SlipWall` contributes its face's tangential edges through a new `collect_slip_edges` hook; the
solver subtracts them. No face declared slip ⇒ the set is unchanged ⇒ bit-identical.

(Free-slip composes with the open boundary: an isolated cylinder is west `Inflow`, east `Outflow`,
top/bottom `SlipWall`, plus the immersed cut body.)

### D2: The surface-force diagnostic integrates over the cut fragments

The hydrodynamic force on the immersed body is `F = ∮_S (−p n + μ (∇u + ∇uᵀ)·n) dA`. The diagnostic
walks the `CutCellRegistry`'s `Cut` cells, and for each `CutFaceFragment` (area `a`, outward unit
normal `n`):

- **Pressure term** `−p n a`: `p` is the cell's pressure, recovered from the projection's grade-0
  potential (`eval_projected_with_potential` already returns it — the Bernoulli `φ = p + ½|u|²` at
  `ρ = 1`; the diagnostic subtracts the `½|u|²` head to isolate `p`).
- **Viscous term** `μ (∇u + ∇uᵀ)·n a`: the velocity gradient at the fragment from the incident
  edge cochain (the DEC `sharp`/de-Rham-adjacent reconstruction already used for `max_speed`).

Summing over fragments gives the force vector; `C_d = F·x̂ / (½ ρ U² L)` and `C_l = F·ŷ / (…)` with
caller-supplied reference velocity `U` and length `L`. Precision-generic over `R: RealField`. The
diagnostic is read-only on a `SolenoidalField` snapshot — heavy, so it lives behind an explicit
call (validation/example use), not the per-step hot path.

### D3: Validation placement

Per the tests-fast / examples-verify split, the **quantitative** isolated-cylinder rungs (a long
march to developed shedding, Strouhal from the wake probe, `C_d` from the force diagnostic) land in
the `add-cut-cells-and-immersed-boundaries` D2/D3 **example**, after this change ships the
primitives. Here, the fast crate tests gate the primitives analytically:

- **slip-boundary**: a free-slip channel preserves a uniform tangential flow with no boundary layer
  (a plug profile stays plug), unlike the no-slip channel (which develops Poiseuille); the
  closed/no-slip path is bit-identical when no slip face is declared.
- **surface-force-diagnostic**: the force on a body in a known field matches the analytic surface
  integral (e.g. a uniform-pressure field gives zero net force; a linear pressure gradient gives
  the exact buoyancy-like force), to rounding.

Both primitives are gated in **2D and 3D** (they are `const D`-generic). The cut-cells D2/D3
example then carries the quantitative isolated-cylinder rungs: 2D laminar (Re ≈ 100–200, Strouhal +
`C_d`), a **3D DNS** rung in the transition regime (Re ≈ 200–300), and Re ≈ 3900 by DNS as a
compute-bound (non-CI) capability check — not a cheap modeled run.

## Risks / Trade-offs

- **Free-slip viscous correctness**: that the boundary-clipped star gives the right zero-shear
  Neumann condition for the un-pinned tangential edges must be gated (the plug-flow-preserved
  test). If the clip does not yield zero-shear cleanly, a viscous boundary stencil adjustment is
  needed (flagged, not pre-built).
- **Pressure recovery for drag**: the projection returns the Bernoulli potential, not bare `p`;
  isolating `p` (subtracting the dynamic head) is approximate near the body where `|u|²` varies —
  gated against the analytic force tests, with the pressure/viscous split reported separately so
  the drag can be cross-checked.
- **Force-diagnostic accuracy** depends on the fragment quadrature and the gradient reconstruction;
  the analytic gates bound it before the example trusts the `C_d` numbers.

## Migration

Additive: a new `SlipWall` zone and a new force diagnostic. The auto no-slip stays the default; the
un-pin seam is a no-op unless a slip face is declared.
