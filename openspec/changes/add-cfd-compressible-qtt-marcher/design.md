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

**The make-or-break and structural risks have been put through ARIZ** (resolutions
[4](../../notes/plasma-blackout/gap-2/gap-two-resolution-4-body-fit-parameter.md),
[5](../../notes/plasma-blackout/gap-2/gap-two-resolution-5-dynamic-rank-lever.md),
[6](../../notes/plasma-blackout/gap-2/gap-two-resolution-6-implicit-acoustics.md),
[7](../../notes/plasma-blackout/gap-2/gap-two-resolution-7-feature-adaptive-coordinate.md)). Three independent
passes — *body-fit generality*, *does the static rank lever survive marching*, *does AMEn converge* — collapsed
onto **one mechanism, the Feature-Adaptive Coordinate (FAC), the spatial dual of the Tier-A LER pattern**: *move
the difficulty out of the field and into a cheap, low-rank, data-supplied, feedback-updated coordinate map.* FAC
is realized here as (a) a `MetricProvider` seam with a body-fit **blend parameter** (D8), (b) a **feature-pinned,
feedback-updated** map that makes bounded rank true *by construction* under marching (D9), and (c) a **split
acoustic operator** whose stiff constant-coefficient core has a **closed-form low-rank inverse** so the implicit
step never depends on an unproven AMEn convergence (D10). LER confines stiffness in *time*; FAC confines
singularities in *space*; the field carries neither.

## Goals / Non-Goals

**Goals:** a compressible reacting QTT marcher for the **reentry forebody/sheath** (the region that drives
blackout) that (1) reproduces the Sod exact Riemann solution, (2) reproduces the RAM-C stagnation-line
electron density with the reused Tier-A LER stack, (3) holds a 2-D forebody bow shock at bounded χ in a
body-fitted coordinate, and (4) makes `T_tr`/`T_ve` real transported states (retiring the Tier-A
recovery-temperature reconstruction) — all dynamic-by-construction and within the constraints.

**Non-Goals (and why each is consistent with the goal):**

- **Turbulence / LES closure** — the forebody sheath that drives blackout is laminar-to-transitional; the
  laminar forebody is the right model for the RAM-C electron density. (Where turbulence *does* matter — the
  near-wake — resolution
  [9](../../notes/plasma-blackout/gap-2/gap-two-resolution-9-moment-closure-turbulence.md) shows it can be a
  **modeled** region: a Reynolds/moment (RANS k–ω / γ–Reθ) closure delivering the **mean** `n_e` at low rank,
  built as LER stages + a low-rank eddy-viscosity coefficient MPO. *Instantaneous* turbulent structure stays
  out of scope — never needed for `n_e` — and RANS fidelity is the standard caveat; the closure itself is a
  future increment, not part of this change's staged scope.)
- **The 3-D *wake*** — out of scope. Blackout is a *forebody* phenomenon (the antenna/sheath region is
  upstream of the wake); and a meaningful wake requires the turbulence that is itself a non-goal. The wake
  rank is recorded as an **open research question** (the `qtt_rank_3d` residual), *not* a gated deliverable —
  any 3-D run reports a forebody result and treats the wake as out-of-scope.
- **Ablation / TPS coupling** — RAM-C II was deliberately ablation-controlled to isolate *air* ionization, so
  excluding ablation matches the validation anchor.
- **Radiation transport** — the RAM-C / ~7.6 km/s orbital-reentry regime is convection-dominated (radiation
  matters above ~10–11 km/s).
- **A general unstructured-mesh capability** — the body-fitted *structured* coordinate **is** the measured
  rank lever, so structured-only is required by the architecture, not a sacrifice. (Within structured maps,
  generality is *not* sacrificed: resolution
  [4](../../notes/plasma-blackout/gap-2/gap-two-resolution-4-body-fit-parameter.md) makes body-fittedness a
  free blend parameter `λ` behind a `MetricProvider` seam — `λ=0` is Cartesian capture for any geometry,
  `λ=1` is full fitting; see D8.)
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
— fully explicit:* rejected (acoustic-CFL-limited timestep). The earlier "AMEn may not converge" risk is
**discharged by D10**: the implicit step is built on a closed-form constant-coefficient inverse, so AMEn (if
used at all) only ever solves a small perturbation of the identity. Still stage-gated (Stage 3).

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

### D8 — The coordinate is a `MetricProvider` seam with a body-fit blend parameter

(Resolution [4](../../notes/plasma-blackout/gap-2/gap-two-resolution-4-body-fit-parameter.md).) Promote the
hard-wired map to a static-dispatch `MetricProvider<D, R>` trait (yields the metric MPOs + Jacobian); the
marcher is generic over it and never branches on geometry. `CartesianIdentity`, `BodyFittedCoordinate`, and a
`BlendedMap<A, B>` cons-tuple are impls — the same pattern as `PhysicsStage`/`Coupling`. Body-fittedness becomes
a continuous **blend** `T_λ = (1−λ)·T_identity + λ·T_fitted` (a blend of low-rank maps is low-rank), optionally a
**spatial field** `λ(ξ)` (fitted near the body, Cartesian in the far field). This converts the D1 "structured
xor general" trade-off into a free parameter at zero asymptotic rank cost. *Alternative — keep the analytic map
hard-wired:* rejected (binds the solver to one geometry for no rank benefit). *Residual:* a convex blend of two
charts can fold a cell → guard with a bounded `λ`-gradient + positive-Jacobian homotopy check.

### D9 — The fitted map is feedback-updated: bounded rank *by construction*

(Resolution [5](../../notes/plasma-blackout/gap-2/gap-two-resolution-5-dynamic-rank-lever.md), the make-or-break.)
The static studies measured `O(10)` for a *snapshot*; marching could still grow rank via nonlinear steepening.
Resolution: re-pin the `η=const` interface to the **live** shock (max `|∇p|`) **each step**, so the discontinuity
is **coordinate-stationary** ⇒ `O(1)` rank on its axis for all time. Rank is held by the *coordinate*, not by
`round()` (a backstop). The Rusanov viscosity needed for stability doubles as the thickness floor / rank cap on
un-pinned features (the contact). This makes shock-fitting (D1) and dynamic rank control the **same mechanism** —
fitting is not bolted onto the bulk, it *is* the coordinate the bulk runs in. *Boundary:* one dominant feature
per structured map is pinnable; the multi-feature unsteady **wake** is not (the out-of-scope `qtt_rank_3d`
residual — now explained, not just stipulated).

### D10 — Split acoustic operator with a closed-form constant-coefficient inverse

(Resolution [6](../../notes/plasma-blackout/gap-2/gap-two-resolution-6-implicit-acoustics.md), the make-or-break.)
Split the acoustic operator `A = A₀ + A₁`: `A₀` is the constant-coefficient (reference `ρ̄, c̄`)
Laplacian/Helmholtz — stiff, with a **known closed-form low-rank inverse MPO**; `A₁` is the variable-coefficient
remainder — non-stiff, treated explicitly or with one defect-correction sweep. This is the spatial-acoustic
analogue of LER (closed-form *inverse* instead of a closed-form *exponential*). If AMEn is kept, `A₀⁻¹` is its
exact self-preconditioner, so it solves `I + A₀⁻¹A₁` (a perturbation of identity) — converting "does AMEn
converge?" into the measurable "is `‖A₀⁻¹A₁‖ < 1`?". D9's fitting keeps the worst coefficient jump at the
interface (exact RH), so the interior the inverse acts on is smooth — **the rank lever also conditions the
solve**. Fallback ladder: defect-correction → preconditioned AMEn → explicit small `Δt` (the prior plan's
floor). *Alternative — implicit on the full variable-coefficient operator:* rejected (the unproven-convergence
gamble).

## Risks / Trade-offs

- **[Risk, reframed by D9] Shock-fitting coupled to a QTT bulk is unprecedented.** Static re-coordinatization
  gives `O(10)`; a working interface-tracking fitted *marcher* that composes and stays low-rank dynamically is
  not demonstrated. → **Reframing
  ([Res 5](../../notes/plasma-blackout/gap-2/gap-two-resolution-5-dynamic-rank-lever.md)):** fitting *is* the
  dynamic `MetricProvider` (D9), so the "coupling" is not a bolt-on — the bulk simply runs in the fitted
  coordinate, and a coordinate-stationary front is `O(1)` rank by construction. → **Mitigation:** Stage 4 starts
  at 1-D (trivially low rank), gating each geometry step; the capture-and-thicken fallback (D1, = `λ→0` in D8) is
  the escape hatch. The honest residual is **multi-feature** pinning (the wake), explicitly out of scope.
- **[Scope boundary, not a risk] The wake is out of scope.** A fitted coordinate aligns shock + wall but not
  a separated/unsteady wake, and a real wake needs turbulence (a non-goal). The blackout sheath is on the
  *forebody*, upstream of the wake, so the goal does not need it. → **Decision:** Stage 6 marches and
  validates the **forebody** in 3-D; the wake rank stays an **open research question** (the `qtt_rank_3d`
  residual), reported if incidentally observed but never gated or asserted.
- **[Risk] Burgers → Euler system.** The studies used scalar Burgers; the system adds contact discontinuities,
  acoustic waves, expansion fans with unmeasured rank. → **Mitigation:** Sod (Stage 2) measures the system's
  rank directly before the curved-shock stages.
- **[Risk, discharged by D10] AMEn on the compressible operator may not converge.** → **Resolution
  ([Res 6](../../notes/plasma-blackout/gap-2/gap-two-resolution-6-implicit-acoustics.md)):** don't make the full
  operator implicit — split off the constant-coefficient core with a closed-form low-rank inverse and treat the
  remainder as a bounded perturbation; AMEn, if used, is preconditioned to `I + small`. Stage 3 now gates the
  *perturbation bound* `‖A₀⁻¹A₁‖ < 1` on the fitted interior; the explicit-small-`Δt` fallback remains the floor.
- **[Trade-off, resolved by D8] Structured/body-fitted only.** Buys the rank win; the apparent cost was
  generality. **Resolution
  ([Res 4](../../notes/plasma-blackout/gap-2/gap-two-resolution-4-body-fit-parameter.md)):** the
  `MetricProvider` seam + blend parameter `λ` keeps the solver general across structured geometries (`λ=0`
  Cartesian capture for any body, `λ=1` full fitting) at zero asymptotic rank cost — structured-only, but not
  geometry-locked.

## Migration Plan

Additive; no existing public API changes. Stage order (each gated, exit-nonzero verification):

0. **3-D codec + operators** (`qtt-codec-3d`) — extend the bridge; gate against analytic derivatives.
1. **Body-fitted coordinate** (`body-fitted-qtt-coordinate`) — Jacobian + chain-rule operators; gate the
   `O(10)`-rank lever and free-stream preservation.
   - **0.5. `MetricProvider` seam + body-fit blend** (D8, Res 4) — lift the map behind a static-dispatch
     `MetricProvider<D,R>` (`CartesianIdentity` / `BodyFittedCoordinate` / `BlendedMap`); gate a `λ`-sweep on the
     *same* solver reproducing `√side` at `λ=0` and `O(10)` at `λ=1`, with `det J > 0` throughout.
2. **Compressible flux** (`compressible-qtt-flux`) — conservative Rusanov/HLLC + EOS; gate **Sod**.
3. **IMEX + conservation/positivity** (`qtt-imex-time-integration`) — **split acoustic step (D10):** closed-form
   constant-coefficient inverse + perturbation remainder + rank-1 conservation fixup + entropy variables; gate
   stability-beyond-acoustic-CFL, the perturbation bound `‖A₀⁻¹A₁‖ < 1` on the fitted interior, and zero
   conserved-quantity drift.
4. **Shock fitting** (`qtt-shock-fitting`) — 1-D **feedback-updated** fitted normal shock (D9, exact RH) →
   **RAM-C stagnation line** with the reused Tier-A LER stack; gate the RAM-C electron density *and* bounded
   `max_bond` over the march (the dynamic lever).
5. **2-D body-fitted compressible reacting marcher** (`compressible-reacting-qtt-marcher`) — bow shock; gate the
   by-construction invariant (pinned-feature bond `O(1)`) vs the Cartesian control (`λ=0`) reproducing `√side`.
6. **3-D forebody** (`compressible-qtt-validation`) — march and validate the forebody sheath in 3-D in the
   body-fitted coordinate; gate bounded forebody χ. The wake is out of scope (reported, never gated).

Rollback: each stage is a new module + example; if a stage's gate cannot be met it is not declared done and
the marcher is used at the last passing stage (e.g. Stage 4 RAM-C stagnation line is a valid standalone
deliverable even if Stage 6 is not reached).

## Open Questions

- **Wake rank (out of scope here)** — does a separated/unsteady wake stay bounded in the fitted coordinate?
  Left as the standing `qtt_rank_3d` research question; this change scopes to the forebody and does not
  attempt to settle it (it needs turbulence, a non-goal).
- **HLLC vs Rusanov** *(leaning resolved by D9/Res 5)* — fitting supplies the sharp jump by exact RH, so the
  interior flux only transports the *smooth* relaxation zone, where Rusanov's diffusivity is acceptable; the
  theme demotes the flux-scheme choice. Still confirm against the RAM-C electron density at Stage 4 before
  closing it; promote HLLC only if the sheath `n_e` demands it.
- **Coordinate generation** *(de-risked by D8)* — analytic vs computed is now a swappable `MetricProvider` impl;
  start analytic (sphere-cone is closed-form) and add a computed grid later with no solver change.
- **Where the 3-D operators live** — `deep_causality_cfd/tensor_bridge` vs a shared home in
  `deep_causality_tensor`; default to the cfd bridge (mirrors the 1-D/2-D operators) unless reuse demands
  otherwise.
- **Blend-Jacobian validity (new, Res 4)** — is the bounded-`λ`-gradient + positive-Jacobian guard always
  sufficient to keep `T_λ` a diffeomorphism, or are there geometries where the Cartesian and fitted charts are
  too twisted to blend directly? Open; gate `det J > 0` across the `λ`-sweep at Stage 0.5.
- **Multi-feature pinning / the wake (new, Res 5; lever found, Res 8)** — FAC pins one dominant feature per
  structured map; a multi-feature unsteady wake needs more. Resolution
  [8](../../notes/plasma-blackout/gap-2/gap-two-resolution-8-spectral-pinning.md) finds the lever: **pin the
  discontinuity geometrically (FAC) and carry the rest spectrally via DLRA/`tdvp`** (rank-adaptive). This does
  not make turbulence low-rank, but it (a) extends closure to the **transitional near-wake** and (b) turns the
  out-of-scope boundary into a **measurable `K(t)` tripwire** (the adaptive Schmidt rank) rather than an
  a-priori exclusion. For the **fully-turbulent** part, resolution
  [9](../../notes/plasma-blackout/gap-2/gap-two-resolution-9-moment-closure-turbulence.md) reframes it as a
  **modeled** region — a RANS/moment closure (LER stages + a low-rank eddy-viscosity MPO) delivering the
  **mean** `n_e` — so the irreducible residual shrinks to *instantaneous fine structure* (never needed for
  `n_e`) + *RANS fidelity* (the standard hypersonic-CFD caveat). Both the FAC+DLRA hybrid and the moment closure
  are future increments behind the same `Marcher`/`Coupling` seam, not part of this change's staged scope.
