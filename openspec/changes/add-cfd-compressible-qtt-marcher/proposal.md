<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Why

The plasma-blackout flagship needs a **compressible, shock-bearing** flowfield: a Mach-25 reentry bow
shock, the post-shock thermochemistry, and the ionized sheath. The built QTT solver
(`QttIncompressible2d`/`QttImmersed2d`) is **incompressible** — the wrong governing equations for a shock —
and the Tier-A change [`add-park2t-blackout-tier-a`](../add-park2t-blackout-tier-a/proposal.md) deliberately
rides it with a recovery-temperature *reconstruction* stand-in. This change builds the **Tier-B compressible
shock-capturing QTT marcher** that retires the stand-in and carries the real flowfield, so the Tier-A
reacting/ionization physics runs on a genuine post-shock thermodynamic state.

This is buildable now, not open hand-waving, because the make-or-break risk is **measured**. Four
self-verifying rank studies (`deep_causality_cfd/studies/`,
[Tier-B note](../../notes/plasma-blackout/gap-2/tier-b-compressible-marcher.md)) settled the central
question — *does the flowfield stay low tensor-train rank?* — and named the fix:

- The rank driver is **coordinate alignment, not sharpness or curvature**.
- A realistically-formed **3-D** curved shock *captured on a Cartesian grid* has **`χ ~ √side` (unbounded)**;
  the **same shock in a body-fitted / shock-aligned coordinate** is **`χ ~ O(10)` (constant)**.
- **Artificial viscosity is not the lever** (it cannot remove curvature, and over-thickening is
  diffusion-CFL-unstable → needs an **implicit / IMEX** step).

So the architecture is fixed by evidence: **singularity confinement** — a body-fitted coordinate that aligns
the shock to an axis, an exact Rankine–Hugoniot interface, and an IMEX acoustic step — the spatial dual of the
Tier-A LER pattern. This change specifies that solver end to end, staged so the first milestone (the RAM-C
stagnation line) is buildable on day one and each later stage is gated.

## What Changes

- **3-D QTT codec + operators** in `deep_causality_cfd/tensor_bridge/` — `quantize_3d`/`dequantize_3d` and
  `gradient_{x,y,z}` / `laplacian_3d` / divergence MPOs, extending the existing 1-D/2-D bridge. (None exist
  today.)
- **A body-fitted / shock-aligned curvilinear coordinate** with a **low-rank Jacobian** — the measured
  mandatory rank lever; transforms the finite-difference operators by chain rule so the curved bow shock and
  the wall become axis-aligned coordinate surfaces (`χ ~ O(10)`, not `√side`).
- **A conservative compressible flux** on conservative variables `(ρ, ρu, ρE, {ρY_s})` in tensor-train form —
  an approximate Riemann flux (Rusanov/HLLC) + an **EOS pressure closure via TT-cross**, in entropy / log
  variables for positivity.
- **IMEX time integration** — explicit convective + **implicit acoustic** step via `solve::linear` (AMEn),
  inside the acoustic/diffusion CFL; **conservation-preserving rounding** (carry the conserved totals + a
  rank-1 projection fixup, because `round` minimizes Frobenius error, not invariants).
- **Shock fitting** — the bow shock as a **tracked moving interface** with the **exact Rankine–Hugoniot jump**
  applied across it; each side stays smooth (low-rank), and the wrong-shock-speed coupling caveat is removed.
- **The compressible reacting marcher** — ride the **Tier-A reacting/ionization LER sources unchanged** on the
  compressible rollout via TT-cross (Pinkston et al.); `T_tr`/`T_ve` become real transported states (retiring
  the Tier-A reconstruction).
- **Validation** — a Sod shock tube (exact Riemann reference), the **RAM-C stagnation-line** electron density
  (the buildable milestone), a 2-D blunt-body bow shock with **measured bounded χ**, and a 3-D **forebody**
  run — each a self-verifying example. (The 3-D **wake** is out of scope — it needs turbulence, a non-goal,
  and is downstream of the blackout sheath; its rank stays the standing `qtt_rank_3d` research question.)
- **Reuses unchanged:** the Tier-A Park-2T kernels, quantity newtypes, LER `PhysicsStage`s, `BlackoutTrigger`,
  and the `Marcher`/`Coupling`/`CausalFlow` substrate.

## Capabilities

### New Capabilities

- `qtt-codec-3d`: 3-D QTT field codec (`quantize_3d`/`dequantize_3d`) and 3-D finite-difference MPO operators
  (`gradient_{x,y,z}`, `laplacian_3d`, divergence), extending the 1-D/2-D `tensor_bridge`.
- `body-fitted-qtt-coordinate`: a smooth curvilinear coordinate that aligns the shock/body to coordinate
  surfaces, carried as a **low-rank Jacobian**, with chain-rule-transformed operators — the measured
  mandatory rank lever (`χ ~ O(10)` vs captured `√side`).
- `compressible-qtt-flux`: conservative compressible Euler/NS in tensor-train form — conservative variables,
  an approximate Riemann flux, and an EOS closure via TT-cross, in positivity-preserving (entropy/log)
  variables.
- `qtt-imex-time-integration`: IMEX time stepping (explicit convection + implicit-acoustic AMEn) within the
  CFL, plus conservation- and positivity-preserving rounding.
- `qtt-shock-fitting`: the shock as a tracked moving interface with the exact Rankine–Hugoniot jump, coupled
  to the QTT bulk, keeping each side smooth and removing the wrong-shock-speed caveat.
- `compressible-reacting-qtt-marcher`: the assembled marcher carrying the Tier-A reacting/ionization LER
  sources on the compressible fitted rollout (RAM-C stagnation line → 2-D blunt body → 3-D).
- `compressible-qtt-validation`: the verification suite — Sod analytic, RAM-C stagnation-line `n_e`, 2-D
  bow-shock bounded χ, 3-D forebody (wake out of scope), with flight/analytic cross-references.

### Modified Capabilities

<!-- None. This change adds new capabilities and reuses the existing qtt-* and Tier-A capabilities unchanged;
     it does not alter their requirements. -->

## Impact

- **`deep_causality_cfd`** — new `tensor_bridge` 3-D codec/operators; a new `coordinate/` (body-fitted
  Jacobian); a new `solvers/qtt/compressible/` family (flux, IMEX, fitting, the marcher); reuses the Tier-A
  `physics`-crate kernels and the `PhysicsStage`/`Marcher`/`CausalFlow` substrate unchanged; new
  `studies/`-style verification examples under `verification/`.
- **`deep_causality_tensor`** — likely additions to the operator algebra for n-D / chain-rule operators and
  conservation-aware rounding; the AMEn `solve::linear` is reused (its convergence on the variable-coefficient
  compressible operator is a stage-gated risk).
- **Dependencies** — no new external crates.
- **Tests / Bazel** — new test modules mirror the src tree, registered in `mod.rs` and `tests/BUILD.bazel`;
  100% coverage of new library code; examples are coverage-exempt.
- **Constraints** — static dispatch, no `dyn`/`unsafe`/lib-macros; crate-root imports; lib float literals
  confined to `constants/` mapping via `from_f64`; the dynamic-by-construction invariant
  (gap-2 §1.2) holds (metric/curvature/thermo computed from state).
- **Sequencing** — Tier-A (`add-park2t-blackout-tier-a`) is the prerequisite physics layer, reused unchanged
  (the recovery-temperature reconstruction excepted — superseded by transported `T_tr`/`T_ve`). This change is
  staged (Stages 0–6 in the design) so the **RAM-C stagnation line (Stage 4)** is the buildable milestone,
  while the 2-D/3-D fitted compressible marcher (Stages 5–6) carries the genuine open-research nodes named in
  the [Tier-B note](../../notes/plasma-blackout/gap-2/tier-b-compressible-marcher.md) §4. The 3-D wake is out
  of scope (a standing research question, not a node in this change).
