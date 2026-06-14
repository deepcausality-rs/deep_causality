## Why

Boundary conditions in the DEC Navier–Stokes solver have **no common abstraction**. Each
kind lives in a different place by accident of history: no-slip is a constraint set baked
into the rate; a moving wall is a static `lift: Vec<(usize, R)>` on the solver; a body force
is a term inside `DecNsRate`; periodicity is a flag on `LatticeComplex`; an immersed body is a
`CutCellRegistry` on the metric; and the Stage-4 uncertain inflow had **nowhere to live** — it
was realized by bolting a sensor onto the moving-wall lift (`add-cut-cells-and-immersed-boundaries`
Group C). There is no first-class noun for "a boundary zone."

Two consequences follow, and both block the program:

1. **Inflow/outflow recurs as a special case with no home.** The isolated-cylinder Reynolds
   ladder (`add-cut-cells-and-immersed-boundaries` tasks D2/D3 — Strouhal/drag vs Williamson and
   Lehmkuhl et al. 2013, Re 100–3900) needs a true open-boundary surface. The Leray projection
   today enforces **zero net boundary flux** at every non-periodic face — the moving-wall path
   even rejects a wall-normal component ("wall-normal flux is the projection's Neumann
   condition"). So the validation that Stage 5 presupposes cannot be written.
2. **The proposed `FluidDynamics` DSL (`causal_cfd.md`) cannot be built on scattered BCs.** The
   DSL's promise — "composable inflow and outflow zones", multi-physics, counterfactual
   interventions on material/mesh/temperature — requires every boundary to be a first-class,
   composable term. Without that, the DSL has nothing uniform to compose.

This change supplies the missing abstraction and the one irreducible numerical operator behind
it, so inflow/outflow stops being a recurring special case and becomes a solved primitive.

## What Changes

- **`BoundaryZone` abstraction with layered dispatch (NEW, `boundary-zone-abstraction`).** A
  first-class, composable boundary term that declares **which solver stage(s) it affects** — not
  a single `apply()`. The DEC step has distinct stages where a BC can act (topology, metric/Hodge
  star, rate source, projection constraint + inhomogeneous lift, projection net-flux
  compatibility, boundary time-update); a zone overrides only the layers it touches and is a
  no-op on the rest. The solver collects a set of zones and queries, at each stage, the zones that
  contribute to it. Existing BCs (no-slip, moving wall, body force, immersed cut body) are
  re-expressed as zones with no behavioural change; the wall/periodic/Poiseuille paths stay
  bit-identical.
- **Net-flux mixed-BC Leray projection (NEW, `open-boundary-flux-projection`).** The one
  irreducible numerical task, written **once** behind the open-boundary zones: a
  pressure-Poisson / Hodge solve that admits **nonzero net boundary flux** with mixed boundary
  conditions (prescribed normal flux at inflow, a pressure-reference outflow that fixes the
  nullspace and absorbs the balancing flux, zero normal flux at walls). Precision-generic over
  `R: RealField`; reduces **exactly** to the current closed-domain projection when no open
  boundary is present.
- **Inflow zone (NEW, `open-boundary-inflow`).** A prescribed wall-normal Dirichlet inflow,
  contributing constrained edges + lift (L3) and net inflow flux to the compatibility balance
  (L4).
- **Outflow zone (NEW, `open-boundary-outflow`).** A convective / zero-gradient outflow
  contributing a boundary time-update (L5) and the pressure-reference face that balances the net
  flux (L4).
- **Uncertain boundary source, generalized cross-domain (NEW, `uncertain-boundary-source`).** The
  Group-C `UncertainInflowZone` mechanism — presence-gate a `MaybeUncertain<R>` stream
  (`lift_to_uncertain`), collapse to a scalar, and on a dropout substitute the last-good value via
  `.intervene` recorded in the `EffectLog` — extracted from "inflow" specifically into a reusable
  wrapper that supplies the time-varying value of **any** Dirichlet zone (inflow, moving wall) or,
  in principle, any sensor-fed parameter in any domain. The existing `UncertainInflowZone` becomes
  a thin application of this general source.

## Dependencies and sequencing

- **Depends on:** the `uncertain-realfield-generic` line (landed) and the generic
  `Uncertain<R>` reduction (landed in `add-cut-cells-and-immersed-boundaries`), so the uncertain
  boundary source is `R`-typed with no cast island.
- **Unblocks:** `add-cut-cells-and-immersed-boundaries` D2/D3 (the isolated-cylinder Re-ladder is
  written with the Inflow/Outflow zones, then that change is closed) and the third change
  `consolidate-causal-cfd-fluiddynamics` (the `FluidDynamics` DSL composes these zones).
- This change is the **prerequisite** for the DSL/consolidation and must land (and archive)
  before the cut-cells change is closed.

## Non-Goals

- No new IO / mesh formats / STL (carried from the no-new-IO posture).
- No turbulent inflow synthesis, sponge layers, or non-reflecting characteristic BCs beyond a
  first convective/zero-gradient outflow (deferred; the zone seam admits them later).
- No crate move — this lands in `deep_causality_physics` where the solver lives; the `causal_cfd`
  consolidation is the third change.
- The `FluidDynamics` DSL itself is the third change; this change only makes boundaries
  composable terms the DSL can later wrap.
