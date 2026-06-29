<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Context

The plasma-blackout flagship needs a compressible, shock-bearing QTT flowfield. The built solver is
incompressible; Tier-A ([`add-park2t-blackout-tier-a`](../add-park2t-blackout-tier-a/proposal.md)) rides it
with a recovery-temperature reconstruction. This change builds the Tier-B compressible marcher and retires the
reconstruction, reusing the Tier-A physics layer unchanged.

The architecture is **fixed by measurement**, not chosen by taste. Four self-verifying rank studies
(`deep_causality_cfd/studies/`, summarized in
[`gap-2/tier-b-compressible-marcher.md`](../../notes/plasma-blackout/gap-2/tier-b-compressible-marcher.md)):

- Rank driver is **coordinate alignment**, not sharpness/curvature.
- 3-D captured curved shock: **`χ ~ √side` (unbounded)**; body-fitted: **`χ ~ O(10)` (constant)**.
- Thickening is **not** the lever; over-thickening is **diffusion-CFL-unstable** → IMEX mandatory.
- QTT storage always beats dense in 3-D asymptotically; the *solve* cost (`χ²`–`χ³`) is what the coordinate
  must control.

**Verified substrate this builds on** (this session's API checks): `CausalTensorTrain` + MPO algebra
(`from_cores`, `apply`, `compose`, `add/sub/neg/scale`, `round`), `cross`/`apply_nonlinear` (TT-cross),
`solve::linear` (AMEn), the 1-D/2-D `tensor_bridge` codec + operators (the pattern to extend to 3-D), the
`Marcher` trait (`State = CausalTensorTrain`, `advance(&state, &ambient)`), the `CfdScalar +
ConjugateScalar<Real = R>` bound, and the `PhysicsStage`/`Coupling`/`CausalFlow` substrate the Tier-A stages
ride.

## Goals / Non-Goals

**Goals:** a compressible reacting QTT marcher for the **reentry forebody/sheath** (the region that drives
blackout) that (1) reproduces the Sod exact Riemann solution, (2) reproduces the RAM-C stagnation-line
electron density with the reused Tier-A LER stack, (3) holds a 2-D forebody bow shock at bounded χ in a
body-fitted coordinate, and (4) makes `T_tr`/`T_ve` real transported states (retiring the Tier-A
recovery-temperature reconstruction) — all dynamic-by-construction and within the constraints.

**Non-Goals (and why each is consistent with the goal):**

- **Turbulence / LES closure** — the forebody sheath that drives blackout is laminar-to-transitional; the
  laminar forebody is the right model for the RAM-C electron density.
- **The 3-D *wake*** — out of scope. Blackout is a *forebody* phenomenon (the antenna/sheath region is
  upstream of the wake); and a meaningful wake requires the turbulence that is itself a non-goal. The wake
  rank is recorded as an **open research question** (the `qtt_rank_3d` residual), *not* a gated deliverable —
  any 3-D run reports a forebody result and treats the wake as out-of-scope.
- **Ablation / TPS coupling** — RAM-C II was deliberately ablation-controlled to isolate *air* ionization, so
  excluding ablation matches the validation anchor.
- **Radiation transport** — the RAM-C / ~7.6 km/s orbital-reentry regime is convection-dominated (radiation
  matters above ~10–11 km/s).
- **A general unstructured-mesh capability** — the body-fitted *structured* coordinate **is** the measured
  rank lever, so structured-only is required by the architecture, not a sacrifice.
- **Rebuilding the Tier-A physics** — the Tier-A **kernels, quantity newtypes, and LER `PhysicsStage`s are
  reused unchanged**; the one Tier-A piece this change *does* retire is the **recovery-temperature
  reconstruction** (replaced by transported `T_tr`/`T_ve` — goal 4). "Reused unchanged" therefore means the
  physics layer, not the incompressible-era temperature stand-in.
- **A production-grade Riemann-solver zoo** — Rusanov is the robust baseline, HLLC the sharper option;
  fitting (D1) supplies the shock jump by exact Rankine–Hugoniot, so flux diffusivity only touches the
  *smooth* relaxation zone (HLLC is promoted from optional to required only if RAM-C needs it — Open
  Questions).

## Decisions

### D1 — Shock *fitting* in a body-fitted coordinate, not shock *capturing* on Cartesian

The measured `√side`-vs-`O(10)` gap makes this non-optional. Confine the discontinuity to a tracked interface
(exact RH) and align the bow shock + wall to coordinate surfaces, so the bulk field stays smooth (low-rank).
*Alternative — capture-and-thicken on Cartesian:* rejected as the primary (χ ~ √side solve cost; over-thicken
is CFL-unstable), kept as the documented fallback for shock topologies fitting cannot handle (complex
shock–shock interaction). This is the spatial dual of the Tier-A LER "singularity confinement".

### D2 — Conservative variables + approximate Riemann flux + EOS via TT-cross

Conservative `(ρ, ρu, ρE, {ρY_s})` so RH jumps are correct; Rusanov baseline (robust, simple wave-speed),
HLLC option (sharper contact). EOS pointwise via TT-cross on smooth fields (fitting keeps them smooth, which
is what makes TT-cross converge at bounded rank). *Alternative — primitive variables:* rejected (non-
conservative → wrong shock speeds).

### D3 — IMEX: implicit acoustics (AMEn), explicit convection

The acoustic CFL at micrometre cells is brutal and is *acoustic*, not source, stiffness — orthogonal to LER.
Treat the fast pressure mode implicitly via `solve::linear` (AMEn exists), convection explicitly. *Alternative
— fully explicit:* rejected (acoustic-CFL-limited timestep). Risk: AMEn convergence on the variable-coefficient
compressible operator is unproven — stage-gated (Stage 3).

### D4 — Conservation- and positivity-preserving rounding

`round` minimizes Frobenius error, not invariants → carry the conserved totals and apply a rank-1 projection
fixup after each round (cheap, low-rank). Evolve entropy/log variables so positivity is structural.
*Alternative — constrain the SVD rounding directly:* rejected as harder and less modular than the carry +
rank-1 correction.

### D5 — Reuse the Tier-A physics layer verbatim

The kernel/solver split means the Park-2T kernels, newtypes, LER `PhysicsStage`s, and `BlackoutTrigger` run on
this marcher unmodified; only `T_tr`/`T_ve` change from reconstructed to transported. This is why Tier-A is a
prerequisite, not a throwaway. *Alternative — a bespoke reacting compressible kernel set:* rejected (rebuilds
validated Tier-A work).

### D6 — Stage 3-D operators by extending the 2-D bridge

`quantize_3d` + `gradient_{x,y,z}`/`laplacian_3d` follow the existing `from_cores` shift-operator + stencil
pattern. This is the prerequisite infrastructure (Stage 0), mostly engineering.

### D7 — Staged delivery with a buildable day-one milestone

Stage order is chosen so the **RAM-C stagnation line** (1-D fitted shock + reused Tier-A LER) is reachable
early and is the honest first deliverable, before the 2-D/3-D fitted compressible marcher (the research-bearing
stages).

## Risks / Trade-offs

- **[Risk] Shock-fitting coupled to a QTT bulk is unprecedented.** Static re-coordinatization gives `O(10)`;
  a working interface-tracking fitted *marcher* that composes and stays low-rank dynamically is not
  demonstrated. → **Mitigation:** Stage 4 starts at 1-D (trivially low rank), gating each geometry step; the
  capture-and-thicken fallback (D1) is the escape hatch.
- **[Scope boundary, not a risk] The wake is out of scope.** A fitted coordinate aligns shock + wall but not
  a separated/unsteady wake, and a real wake needs turbulence (a non-goal). The blackout sheath is on the
  *forebody*, upstream of the wake, so the goal does not need it. → **Decision:** Stage 6 marches and
  validates the **forebody** in 3-D; the wake rank stays an **open research question** (the `qtt_rank_3d`
  residual), reported if incidentally observed but never gated or asserted.
- **[Risk] Burgers → Euler system.** The studies used scalar Burgers; the system adds contact discontinuities,
  acoustic waves, expansion fans with unmeasured rank. → **Mitigation:** Sod (Stage 2) measures the system's
  rank directly before the curved-shock stages.
- **[Risk] AMEn on the compressible operator may not converge.** → **Mitigation:** Stage 3 gates AMEn
  convergence in isolation; fall back to a smaller explicit timestep if needed (correct, just slower).
- **[Trade-off] Structured/body-fitted only.** Buys the rank win at the cost of generality — acceptable for a
  single-bow-shock reentry geometry; not a general CFD solver.

## Migration Plan

Additive; no existing public API changes. Stage order (each gated, exit-nonzero verification):

0. **3-D codec + operators** (`qtt-codec-3d`) — extend the bridge; gate against analytic derivatives.
1. **Body-fitted coordinate** (`body-fitted-qtt-coordinate`) — Jacobian + chain-rule operators; gate the
   `O(10)`-rank lever and free-stream preservation.
2. **Compressible flux** (`compressible-qtt-flux`) — conservative Rusanov/HLLC + EOS; gate **Sod**.
3. **IMEX + conservation/positivity** (`qtt-imex-time-integration`) — AMEn acoustic step + rank-1 conservation
   fixup + entropy variables; gate stability-beyond-acoustic-CFL and zero conserved-quantity drift.
4. **Shock fitting** (`qtt-shock-fitting`) — 1-D fitted normal shock (exact RH) → **RAM-C stagnation line**
   with the reused Tier-A LER stack; gate the RAM-C electron density.
5. **2-D body-fitted compressible reacting marcher** (`compressible-reacting-qtt-marcher`) — bow shock; gate
   bounded χ vs the Cartesian control.
6. **3-D forebody** (`compressible-qtt-validation`) — march and validate the forebody sheath in 3-D in the
   body-fitted coordinate; gate bounded forebody χ. The wake is out of scope (reported, never gated).

Rollback: each stage is a new module + example; if a stage's gate cannot be met it is not declared done and
the marcher is used at the last passing stage (e.g. Stage 4 RAM-C stagnation line is a valid standalone
deliverable even if Stage 6 is not reached).

## Open Questions

- **Wake rank (out of scope here)** — does a separated/unsteady wake stay bounded in the fitted coordinate?
  Left as the standing `qtt_rank_3d` research question; this change scopes to the forebody and does not
  attempt to settle it (it needs turbulence, a non-goal).
- **HLLC vs Rusanov** — is Rusanov's diffusivity acceptable for the sheath electron density, or is HLLC
  needed? Fitting supplies the jump by exact RH, so this is about the *relaxation-zone* diffusion; decide
  against RAM-C at Stage 4.
- **Coordinate generation** — analytic sphere-cone map vs a computed body-fitted grid; start analytic
  (sphere-cone is closed-form) and revisit if geometry generalizes.
- **Where the 3-D operators live** — `deep_causality_cfd/tensor_bridge` vs a shared home in
  `deep_causality_tensor`; default to the cfd bridge (mirrors the 1-D/2-D operators) unless reuse demands
  otherwise.
