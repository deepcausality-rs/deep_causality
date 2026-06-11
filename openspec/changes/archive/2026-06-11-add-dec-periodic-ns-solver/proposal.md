# Add the Periodic DEC-Native Incompressible Navier–Stokes Solver

## Why

Stage 0 (`add-dec-solver-foundations`, archived 2026-06-11) closed every
pre-solver gap in `openspec/notes/cfd/cfd-gap.md`: the wedge and interior
product (G1), the de Rham/sharp transfer maps (G2), the typed fluid forms
including the `SolenoidalField` type-state (G3), the pinned sign and
orientation conventions (G4), and the well-posedness of projection on
periodic lattices (G6). Every operator the solver needs now exists as a
published, tested API. What does not exist is the solver: nothing in the
monorepo marches a velocity field through time.

Stage 1 of `openspec/notes/cfd/cfd-roadmap.md` is the smallest artifact that
proves the composition claim end-to-end: an incompressible Navier–Stokes
solver on periodic lattices in 2D and 3D, DEC-native (velocity is an edge
1-form for the entire solve), assembled from the existing algebras — the DEC
operators in topology, the `Rk4`/`EndoArrow` integration arrows in calculus,
the typed forms in physics — with no new mathematics. It is also the data
source for Stage 2 (the causal-analysis tap) and the substance of the CFD
community challenge entry: the solver core must be a visible composition of
library calls, not a bespoke numerical kernel.

## What Changes

All new code lands in `deep_causality_physics` under
`src/theories/fluid_dynamics/dec/`, composing published APIs from
`deep_causality_topology` and `deep_causality_calculus`. The governing
formulation is the rotational (Lamb) form under Leray projection
(`cfd-gap.md` §2):

```text
∂u♭/∂t = P( − i_u ω + ν Δ u♭ + g♭ ),   ω = d u♭,   Δ_dR = −∇²
```

1. **Rate assembly**: a rate-field closure evaluating the DEC right-hand
   side `− i_u(du♭) − ν Δ_dR u♭ + g♭` on a `VelocityOneForm<R>`, composed
   from `Manifold::{exterior_derivative, interior_product, laplacian}`. The
   viscous sign follows the Stage 0 pin (`Δ_dR = −∇²`, so `+ν∇²u ≡ −νΔ_dR u`).
2. **March step**: `Rk4` over `VelocityOneForm<R>` (which already carries
   exactly the `Clone + Add + Mul<R>` bounds), then the fallible bind chain —
   `leray_project` (re-entering `SolenoidalField<R>`, CG failure
   short-circuits) then `cfl_check` (advective CFL and the diffusive limit
   `dt ≤ C·dx²/(2Dν)`, violation short-circuits). Pure numerics in the
   arrow, fallible plumbing in `Result`; a `PropagatingEffect` wrapper in
   the crate's existing wrapper tradition connects the step to the causal
   monad.
3. **Run loop**: a solver type owning the configuration (ν, dt, body force,
   CG options, CFL safety factors) with `step()` and `run(n)` /
   `run_until(stop)` built on the bind chain; initial conditions are seeded
   through the de Rham map and projected once at `t = 0`.
4. **Diagnostics**: kinetic energy, enstrophy, helicity (3D), maximum
   pointwise speed (via `sharp`, feeding the CFL check), discrete
   divergence; pressure recovery as an opt-in diagnostic that emits **both**
   conventions — the Bernoulli potential (directly from the projection's
   grade-0 potential) and static pressure (Bernoulli minus the kinetic
   0-form), resolving open question 3 of the gap note by exposing the pair.
5. **Validation ladder** (`cfd-gap.md` §7 items 4–6 and 8 in CI; item 7 as
   a runnable example): 2D Taylor–Green energy decay vs. `exp(−2νt)` with
   convergence tables; 2D-in-3D Taylor–Green exercising every 3D code path;
   inviscid energy and helicity conservation at `ν = 0`; the 2D double
   shear layer with roll-up and 2D conservation-character gates, tapping
   the existing coherent-structure kernels; a 3D Taylor–Green Re-1600
   example program producing the dissipation-rate curve at prototype
   resolutions.

## Capabilities

### New Capabilities

- `dec-ns-rate`: the DEC right-hand-side assembly — convective term
  `−i_u(du♭)` via wedge/star composition, viscous term with the pinned sign,
  optional body force; the rate closure the integration arrows consume.
- `dec-ns-march`: the time march — `Rk4` step over the typed state, the
  `leray_project` and `cfl_check` binds, the solver configuration and run
  loop, initial-condition seeding and the `t = 0` projection, and the
  `PropagatingEffect` wrapper.
- `dec-ns-diagnostics`: energy, enstrophy, helicity, max-speed, divergence
  reporting, and the opt-in two-convention pressure recovery.
- `dec-ns-validation`: the Taylor–Green ladder (2D, 2D-in-3D), inviscid
  invariants, the double shear layer, convergence-order assertions in CI,
  and the Re-1600 example.

### Modified Capabilities

None. The four Stage 0 capabilities (`dec-exterior-algebra`,
`de-rham-transfer`, `leray-projection`, `typed-fluid-forms`) are consumed
as published; no requirement of theirs changes.

## Impact

- **`deep_causality_physics`**: new folder module
  `src/theories/fluid_dynamics/dec/`; new exports from `lib.rs`; new test
  tree `tests/theories/fluid_dynamics/dec/`; Bazel test-suite additions.
- **Dependency promotion**: `deep_causality_calculus` moves from
  dev-dependency to runtime dependency of `deep_causality_physics` (Tier 1 →
  Tier 4 edge; acyclic, and the AGENTS.md dependency table gains one entry).
  No other crate's dependencies change.
- **`examples/avionics_examples/`**: one new example binary (Taylor–Green,
  including the Re-1600 3D case behind a size parameter) beside the
  existing `cfd_taylor_green` example, which it extends to the DEC solver.
- **No new features in topology, calculus, haft, or num**: the stage
  exists to prove their published APIs compose. Defects discovered upstream
  during assembly are fixed immediately at their source with their own
  tests — monorepo rule: if you find a bug, you fix the bug — never worked
  around in the solver.
