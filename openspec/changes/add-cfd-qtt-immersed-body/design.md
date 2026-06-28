## Context

The QTT solver core (`add-cfd-qtt-incompressible-2d`, `add-cfd-qtt-flow-observe`) gives a periodic 2-D
incompressible NS flowfield as a tensor train, with the codec, lifted operators, spectral projection,
nonlinear convection, a `CfdFlow::qtt_march` pipeline, and TT-native scalar observables. This change adds
the last Gap-1 piece: an **immersed body** and the **surface observables** the flagship's step [4] reads
(drag, heat flux). It is glue + one physics term, not new tensor mathematics.

## Goals / Non-Goals

Goals: a rank-bounded body-mask tensor train; a Brinkman-penalized no-slip QTT marcher; drag/lift and a
neutral wall heat-flux as tensor-train contractions; cross-validation against the DEC solver and an
accuracy-vs-bond curve. All on the existing periodic power-of-two grid.

Non-Goals (Gap 2 / later): electron density / ionization, *reacting* heat flux, 3-D, cut-cell/graded QTT
geometry, the trajectory/EPP axes. The periodic no-body path stays bit-for-bit unchanged.

## Decisions

- **Brinkman volume penalization, not cut cells.** The body enters as a forcing term `f = −(1/η)·χ_body ⊙
  (u − u_body)` added to the velocity rate, with `η` a small permeability (penalization time). Inside the
  body `χ_body = 1` and the term stiffly drives `u → u_body` (zero for a static wall); outside `χ_body = 0`
  and it vanishes. This is the standard immersed-body method for periodic/spectral solvers and Peddinti et
  al.'s MPS approach. **Why not cut cells:** the QTT grid is uniform periodic power-of-two; cut-cell /
  graded Hodge stars have no low-rank QTT form (gap-one note §4), whereas a mask MPS does. Penalization
  keeps everything on the uniform lattice the codec and operators already assume.

- **Smoothed mask for bounded rank (the central risk).** A sharp 0/1 indicator is a 2-D step function —
  high tensor-train rank (a discontinuity needs many bonds). So the mask is a **smoothed volume fraction**:
  `χ_body = ½(1 − tanh(d(x,y)/δ))` over the signed distance `d` to the body surface, smeared over a few
  cells `δ`. The smoothing both (a) lowers the mask's bond dimension and (b) regularizes the penalization
  (a sharp mask aliases on the grid). The mask is built by sampling the analytic field and `quantize_2d`,
  then `round` — with a **rank report** so the smoothing width can be tuned against bond. `body_mask_2d`
  covers the analytic cylinder; a general mask is any sampled smoothed indicator. (TT-cross is the escape
  hatch if direct quantize-then-round is too high-rank.)

- **Explicit penalization, sub-stepped if stiff.** `η` must be small for a hard wall, which makes the term
  stiff; explicit Euler then needs `Δt ≲ η`. Decision: keep explicit stepping (consistent with the rest of
  the marcher) with `η` chosen at the resolution floor (`η ~ Δx²/ν` or a few `Δt`), and document the
  stability bound; an implicit penalization (a diagonal solve, since the term is diagonal in real space) is
  the escape hatch if the explicit bound is too restrictive. The penalization is applied **before** the
  projection so the projection cleans the divergence the forcing introduces.

- **Drag/lift from the penalization force — a contraction, no surface reconstruction.** The force the body
  exerts on the fluid is the integral of the penalization forcing: `F = (1/η) ∫ χ_body ⊙ (u_body − u) dV`,
  per component. As a tensor-train operation this is `inner(χ_body, u_body − u)` scaled by `(1/η)` and the
  cell volume — a single TT contraction, no boundary fiber or cut-cell surface needed. `C_d = F_x /
  (½ ρ U² D)`, `C_l = F_y / (…)`. This is the cleanest QTT observable: drag falls out of the same mask the
  body uses.

- **Neutral wall heat flux from a penalized passive scalar.** A temperature field `T` is advected–diffused
  on the same rollout (reusing the convection + Laplacian operators), with the body penalized to a wall
  temperature `T_w` (`−(1/η)·χ_body ⊙ (T − T_w)`). The wall heat flux is the penalization heat integral
  `Q = (1/η) ∫ χ_body ⊙ (T_w − T) dV` — the same contraction shape as drag. This stays **neutral** (no
  chemistry); it is the seam the Gap-2 reacting energy equation will replace, not duplicate.

- **Validation by self-contained invariants + a DEC cross-reference, not an inline match.** The QTT
  solver is periodic; an isolated cylinder needs inflow/outflow/far-field, which it does not have — so the
  periodic penalized box is *not* the DEC solver's configuration, and forcing an absolute `C_d` match would
  be dishonest (and running the ~510 s DEC cylinder inline is descoped). The gates are therefore
  self-contained: (a) the no-slip interior (`max|u|` inside the body at the penalization floor), (b) the
  **accuracy-vs-bond** convergence (the drag coefficient settling as the round bond cap rises — the
  Peddinti/Gourianov headline metric), and (c) physical drag (positive, `O(1)`). The committed DEC cylinder
  `C_d ≈ 1.345` (Re 100) is **reported as a cross-reference**, disclaimed for periodic blockage — exactly
  as `dec_cylinder_wake_verification` disclaims its Strouhal.

- **Module placement.** Mask + penalization in `solvers/qtt/` (a `QttImmersed2d` beside
  `QttIncompressible2d`, or an opt-in body field on it) and the mask helper in `tensor_bridge/`; the
  surface observables in `solvers/qtt/observe.rs`; the validation as
  `verification/qtt_cylinder_verification/`. The `CfdFlow::qtt_march` config gains an optional body + the
  drag/heat observe toggles.

## Risks / Trade-offs

- **Mask rank is the headline risk** (gap-one note §3.4: "BCs in QTT are the fiddliest part;
  rank-sensitive"). A sharp body is high-rank; the smoothed volume-fraction mask is the mitigation, with
  the smoothing width tuned against a measured bond. If a sharp body is unavoidably high-rank, TT-cross
  builds the mask at a capped rank. **[mitigated; must be demonstrated, not assumed]**
- **Penalization stiffness.** Small `η` → stiff explicit term → small `Δt`. Mitigation: `η` at the
  resolution floor + documented bound; implicit (diagonal) penalization as the escape hatch. **[bounded]**
- **Validation realism.** Periodic blockage means the absolute `C_d` differs from the isolated-cylinder
  band; the honest gate is cross-validation against the DEC solver on the matched setup + the
  accuracy-vs-bond trend, not an absolute number. **[honest seam, disclaimed]**
- **Scope discipline.** Neutral only. Electron density and reacting heat flux are Gap 2; this change
  provides the thermal-observable *seam* they plug into, and stops there.
