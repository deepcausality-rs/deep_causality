## Why

Gap 1 of the plasma-blackout analysis (`openspec/notes/plasma-blackout/gap-analysis.md` §4) is the CFD ↔
tensor-network bridge. Steps 1–6 closed its **solver core**: a periodic 2-D incompressible Navier–Stokes
flowfield now lives in, and evolves as, a tensor train (`QttIncompressible2d`), driven through the
`CfdFlow` DSL and verified 2nd-order against Taylor–Green. **What remains is the last ~20–25%** — the
piece the flagship's chain step [4] (*MPS flowfield → drag + heat flux*) actually reads:

1. an **immersed body** in the tensor-train flow (gap-one note §3.4 — flagged "the fiddliest part of the
   literature; rank-sensitive"), and
2. the **surface observables** that ride it — drag/lift, and a neutral wall heat flux (gap-one note §3.5
   flagship targets).

This change lumps all remaining Gap-1 work into one specification. With it, Gap 1 is closed: the bridge
yields the momentum/thermal observables the flagship needs, and the only outstanding flagship physics
(electron density, *reacting* heat flux) is explicitly **Gap 2**, not Gap 1.

The method is **Brinkman volume penalization**, the standard immersed-body treatment for periodic/spectral
solvers (and Peddinti et al.'s MPS immersed-object approach): a body mask drives the velocity to the body
velocity inside the solid via a forcing term, with **no cut cells** — which matters because the periodic
power-of-two QTT grid is uniform and cut-cell Hodge stars have no low-rank QTT form (gap-one note §4),
whereas a (smoothed) mask MPS does.

## What Changes

- **Body mask in QTT** (`qtt-immersed-body`): encode a **smoothed** body indicator `χ_body(x, y)` (a
  volume-fraction field, smoothed over a few cells so its tensor-train rank stays bounded — sharp
  step functions are high-rank) as a `CausalTensorTrain`, plus a `body_mask_2d` helper for the analytic
  cylinder/array cases.
- **Penalized no-slip marcher**: `QttImmersed2d` — `QttIncompressible2d` plus the Brinkman term
  `−(1/η)·χ_body ⊙ (u − u_body)` added to the rate each step (fused Hadamard + round), driving the
  velocity to `u_body` (zero for a static wall) inside the body; recompressed and projected as before.
- **Surface observables** (`qtt-surface-observables`): drag/lift from the **penalization-force integral**
  `F = (1/η) ∫ χ_body ⊙ (u_body − u) dV` (a tensor-train contraction — `inner` with the mask, no surface
  reconstruction), nondimensionalized to `C_d`/`C_l`; and a neutral **wall heat flux** from a penalized
  passive scalar `T` advected–diffused on the same rollout (the temperature wall-gradient contraction).
  Exposed through the QTT observe set + `Report`.
- **Validation**: a self-verifying `qtt_cylinder_verification` example gating **no-slip** (interior
  velocity at the penalization floor), the **accuracy-vs-bond** convergence (the headline QTT-CFD metric,
  gap-one note §5), and **physical drag** (positive, `O(1)`); the committed DEC cylinder `C_d` is reported
  as a **cross-reference**, disclaimed for the periodic-blockage difference (an absolute match is not
  claimed — the periodic box is not the DEC inflow/outflow configuration).
- Bound: real `R: CfdScalar + ConjugateScalar<Real = R>`. Purely additive; the periodic
  `QttIncompressible2d` path is unchanged (the body is opt-in).

### Non-Goals (explicit hand-off)
Electron density / Park-2T ionization and *reacting* heat flux — these are **Gap 2** (separate change);
the neutral passive-scalar heat flux here is the bridge's thermal-observable seam they will plug into. Also
out: 3-D, graded/cut-cell QTT geometry, and the trajectory / EPP axes (Gaps 3–4). This change **closes
Gap 1** and nothing more.

## Capabilities

### New Capabilities
- `qtt-immersed-body`: a rank-controlled body-mask tensor train and a Brinkman-penalized no-slip 2-D
  incompressible tensor-train marcher.
- `qtt-surface-observables`: drag/lift via the penalization-force contraction and a neutral wall
  heat-flux via a penalized passive scalar, validated by cross-validation against the DEC solver and an
  accuracy-vs-bond curve.
