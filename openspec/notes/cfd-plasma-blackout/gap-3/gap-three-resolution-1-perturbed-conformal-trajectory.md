<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 3, Resolution 1 — the trajectory/timing axis: perturbed-conformal splitting (preliminary)

**What this is.** A TRIZ/ARIZ **preliminary** resolution of [Gap 3](../gap-analysis.md) — the trajectory axis
(the Two-Time conformal lift, the 6D filter, the IERS relativistic timing) — the one axis the tensor work does
not touch and that the gap analysis keeps `[open]`. It is written **on the assumption that the Tier-B CFD
resolutions ([4](../gap-2/gap-two-resolution-4-body-fit-parameter.md)–[9](../gap-2/gap-two-resolution-9-moment-closure-turbulence.md))
work as intended**: the compressible marcher delivers a trustworthy aero force / heat each step, and the Tier-A
LER delivers `n_e` and the GNSS-denial flag (`BlackoutTrigger`). It is explicitly **preliminary** — the
aero-coupling interface is not yet built, so the factoring below is provisional and **should be revisited once
the Tier-B marcher's force/observable interface lands (Stage 4+)**.

The finding: the trajectory axis is the **fifth instance of the confinement family** (LER/time, FAC/space,
DLRA/Schmidt basis, moment-closure/statistics) — *confine the non-inverse-square forcing into a between-step
perturbation and keep the conformal gravity/relativity core exact.* The trajectory becomes structurally
**identical to the CFD axis**: an exact-structured flow + a between-step perturbation stage.

**The one place the perturbation stops being small — peak dynamic pressure — is not an open problem; it is a
regime change**, and DeepCausality already solves regime change with the `grmhd/` coupling-layer detector
(`select_metric`). The aerodynamic deceleration *is* the perturbation-to-gravity ratio `ε = a_aero/a_grav` (the
g-load): `ε ≪ 1` in coast / blackout onset (the **perturbative** regime, where the matrix exponential is exact
and branch-cheap) and `ε ≳ 1` in the dense lower atmosphere (the **aero-dominated** regime). This is the
classical **Encke → Cowell** crossover, handled by **adopting the grmhd regime detector**: compute `ε` from
state, compare to a config threshold, and **select the integrator** the way grmhd selects the metric. So the
trajectory axis gets its *own* regime change — perturbative-conformal ↔ aero-dominated-direct — exactly parallel
to the continuum→plasma switch on the CFD side, under the *same* coupling pattern. Peak-q is not a wart; it is a
second showcase of the flagship's core thesis.

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**, **[preliminary]**.

> **Scheduling (bundled work item).** Gap 3 is solved **right after Gap 2 closes**, and is **bundled with the
> [chemistry-fidelity gap](chemistry-fidelity-gap.md)** — the RAM-C electron-density precision upgrade
> (T_ve-controlled ionization → 3-T electron energy → finite-rate ionization network) that takes Stage-4 peak
> `n_e` from the surrogate's ~12× down toward the production ~3–4× band. Both are deferred past Gap-2 closure
> and ride the transported `T_tr`/`T_ve`/species fields the compressible marcher provides. The two together are
> the post-Gap-2 work package: **trajectory/timing axis + reacting-chemistry fidelity.**

---

## 0. Frame

- **Key problem (no solution words):** advance the reentry trajectory and its onboard clock through GNSS
  blackout, **branch-cheaply**, when part of the dynamics has exact linear structure and part (aero) does not.
- **System / main function:** the trajectory + timing causaloid (`examples/avionics_examples/hypersonic_2t/model.rs`,
  today a skeleton — `predict` is a first-order Euler `X += G·dt`, `correct()` is a no-op, the conformal
  embedding/generator are hand-set, gravity/clock are absent); *to propagate the 6D conformal state + clock per
  counterfactual branch and correct it from measurements.*
- **The constraint treated as fixed — the lever:** that the propagator must handle the **whole** force law at
  once. Drop it: the **inverse-square family is exactly 2T-linear**; everything else (J2+ geopotential, aero) is
  a **perturbation**.

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** propagate the full nonlinear EOM (gravity + aero) by ODE per branch → faithful, but it is **nonlinear
  ODE integration per branch** — exactly the cost the Two-Time accelerator exists to kill — and yields no
  closed form for cheap branching.
- **TC-2:** propagate only the conformal part by matrix exponential → branch-cheap and exact, but **ignores aero
  / J2** → wrong trajectory in the atmosphere (where blackout lives).

**A3 — Intensify.** Push aero → 0 (exo-atmospheric coast): TC-2 is exact and *free* — the matrix exponential
**is** the trajectory. Push aero → large (peak dynamic pressure): aero ~ gravity, the perturbation is **not
small**. The extremes expose the split axis **and** where it strains — peak-q is the make-or-break, the coast is
free.

**A5 — Resources already present (no new substance):**
- The **2T matrix exponential** `X(τ) = e^{Gτ} X(0)` — exact for the inverse-square family (Bars Sp(2,R)),
  branch-cheap. (The skeleton's `X += G·dt` is the **first-order truncation** of exactly this — the upgrade is
  to the true exponential.)
- The **operator-split (Lie/Strang) pattern** already used throughout the codebase (LER, the Tier-B IMEX) —
  exact flow + a between-step perturbation stage.
- The **Tier-B compressible flowfield** (assumed working): supplies the **aero force / heat** as the
  perturbation each step.
- The **Tier-A LER `n_e` / `BlackoutTrigger`**: supplies the **GNSS-denial flag** that gates the filter.
- The **`PhysicsStage` / `Coupling` between-step seam**: the very seam the aero kick rides — trajectory ≅ CFD.
- The **two-time `τ`**: the proper-time evolution parameter; the **`τ ↔ t` relation *is* the relativistic clock
  correction** (not a bolt-on).
- The **Sp(2,R) gauge constraints** `X·X = X·P = P·P = 0`: a **manifold to project onto** (the Leray /
  conservation-projection analogue).
- The **`grmhd/` regime detector** (`calculate_curvature` → `select_metric`): a **built** coupling-layer pattern
  — compute an indicator from state, compare to a config threshold, **switch the governing model** — directly
  reusable to switch the *integrator*.
- The **g-load `ε = a_aero/a_grav`**: already computed for the **Effect Ethos** corridor gate — the regime
  indicator is a first-class observable, free.

**A7 — Smart Little People.** The little navigator does not integrate the tangled force. It **glides along the
exact conformal arc** (a matrix multiply), and **once per step takes a small kick** sideways for aero + J2 (read
off the flowfield), then **snaps back onto the constraint surface**. Its watch runs on `τ`; the gap between `τ`
and ground-time is the relativistic correction it **carries through blackout**. **When the kicks grow as large
as the glide** (dense air), it stops pretending they are small and **switches to direct integration** — the same
move the GRMHD navigator makes when curvature crosses its threshold.

**Physical contradiction:** the propagator must be **linear** (for the matrix-exponential speed — the whole
point of the accelerator) **and nonlinear/direct** (because in dense air aero forcing dominates). **Resolve by
two separations:** (i) **by scale** — within the perturbative regime, split linear-exact conformal core +
nonlinear-perturbative aero between steps (the Tier-B IMEX / Tier-A LER analogue); (ii) **on condition** — a
**regime detector** (`ε = a_aero/a_grav` vs a config threshold) switches the *whole* integrator from
matrix-exponential (Encke) to direct (Cowell) when aero stops being a perturbation. The same parameter is
"linear" under one regime and "direct" under the other, separated by the coupling layer — the `grmhd` pattern.

→ Reformulation cracks it; the trajectory axis becomes the same shape as everything already solved — **including
its own regime change.**

---

## B. Solve — four coupled pieces, each an already-proven pattern

**B1 — Perturbed-conformal propagation (the trajectory split).**
Split the force into a **conformal core** (monopole + post-Newtonian Schwarzschild = the inverse-square family)
and a **non-conformal remainder** (J2 & higher geopotential harmonics, **plus aero**).
- **Core:** exact via the 2T matrix exponential `e^{Gτ}` — branch-cheap, zero-lag. (Replaces the skeleton's
  first-order `X += G·dt`.)
- **Remainder:** a **between-step perturbation stage** (Lie/Strang split) — aero read from the Tier-B
  flowfield, geopotential harmonics from EGM, held piecewise-constant (or low-order) over the substep.
This is the corridor §6 "2T-exact gravity + aero-as-perturbation" factoring, now recognized as the **standard
operator split the codebase already runs**. Clean bonus: J2+ harmonics are non-inverse-square, so they ride the
**same** perturbation channel as aero — the split separates "exact Kepler/Schwarzschild" from "everything else"
in one stroke.

**B2 — The 6D filter as predict + constraint-projected correct (replaces the `correct()` no-op).**
- **Predict:** the B1 propagation.
- **Correct:** a measurement update (GNSS pseudorange when available, INS always) **followed by projection back
  onto the Sp(2,R) constraint surface** (`X·X = X·P = P·P = 0`) — the gauge-fixing analogue of the CFD **Leray
  projection** (divergence-free) and the Tier-B **conservation projection** (conserved totals). *Evolve freely,
  then project onto the constraint manifold* — the recurring pattern.
- **During GNSS denial** (the `BlackoutTrigger` fires): run **predict-only**, carry the clock correction
  internally, and reconcile with a constraint-projected update when a fix returns. Each counterfactual branch
  propagates predict-only through blackout; provenance logs the carried correction.

**B3 — Timing as the two-time `τ ↔ t` gauge relation (dynamic-invariant native).**
The relativistic clock correction is **not** a bolted-on bias model: it is the relation between proper-time `τ`
(the Sp(2,R) evolution parameter) and coordinate-time `t`, computed from the **dynamic metric** —
`g₀₀ = −(1 − 2GM/rc²)` from the real `GM`/`r` (GR potential) and `γ(v)` (SR dilation); IERS terms are this
relation at higher order. This honors the dynamic-by-construction mandate (gap-analysis Gap 3): metric/curvature
from state, only `G`, `c`, EGM/IERS coefficients literal in `constants/`. The two-time formalism is the *natural*
home for relativistic timing — `τ` vs `t` is what 2T physics is *about*.

**B4 — Regime-gated integrator selection (the `grmhd/select_metric` pattern, adopted).**
B1's split is favorable *only* while aero is a perturbation. Detect the regime from state and switch the
integrator, exactly as `grmhd/` switches the metric:

```text
indicator  ε = a_aero / a_grav          // = the g-load; a_aero from the Tier-B flowfield, a_grav = GM/r² (state)
select     ε < ε_switch  → perturbed-conformal (B1, Encke-like, matrix-exponential, branch-cheap)
           ε ≥ ε_switch  → direct integration  (Cowell, accurate where aero dominates)
```

`ε_switch` is **config** (the Encke→Cowell threshold, ~0.1–1); `ε` is **computed from state** — the
dynamic-by-construction invariant, the same split as grmhd's `curvature_threshold` (config) vs
`curvature_intensity` (state). The selection is a `select_*`-style coupling stage on the `PropagatingEffect`
chain; downstream propagation consumes the chosen integrator. Because `ε` is the same g-load the **Effect Ethos**
gate already evaluates, the regime indicator costs nothing extra. The branch-cheap matrix-exponential therefore
holds across **coast and blackout onset** — where the counterfactual branches fan out — and gracefully hands off
to direct integration in the dense lower atmosphere, where the branches have largely converged and the
decision is committed. The trajectory's regime change runs **under the same coupling layer** as the
continuum→plasma switch (corridor §3/§4 [3]); the demonstrator now shows regime change on **both** axes.

> **TRIZ principles used:** **separation by scale** (conformal core vs perturbation, B1) **and separation on
> condition** (regime-switched integrator, B4 — the key one the peak-q objection forced); **#1 Segmentation**;
> **#2 Taking out** (extract the non-conformal forcing into its own carrier); **#24 Intermediary** (`τ` as the
> proper-time carrier); **#23 Feedback** (the filter, and the regime detector); projection-onto-constraint is the
> **#25 Self-service / restore** move. **Effects database:** operator splitting, **constrained Kalman filtering**,
> and **Encke/Cowell regime-switched orbit integration** are all standard — the contribution is recognizing them
> as the confinement-family pattern plus the `grmhd` regime-coupling, with `τ↔t` as the timing.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes — linear-exact and nonlinear-real coexist by the
  split; the accelerator survives because the **core** stays a matrix exponential and only the bounded remainder
  is perturbative.
- **Only A5 resources?** Yes — the matrix exponential, the split seam, the Tier-B flowfield (assumed), the LER
  flag, the Sp(2,R) constraints, `τ`. Nothing new.
- **Satisfies the function?** Yes — branch-cheap in the perturbative regime (coast / blackout onset), with a
  state-driven switch to direct integration where aero dominates.

**New harm / open (this is preliminary):**
- **Peak-q is a regime change, not an open node `[handled by B4]`.** At peak dynamic pressure the aerodynamic
  deceleration reaches and *exceeds* the Newtonian weight — peak reentry g-loads run several-to-tens of g, so
  `ε = a_aero/a_grav ≳ 1` (this is the g-load, *not* the weak-field relativistic terms, which stay `~10⁻⁹` and
  never bite here). That is the classical **Encke → Cowell crossover**: the matrix-exponential (Encke) split is
  efficient only while aero is a small perturbation; when aero dominates, the standard remedy is direct (Cowell)
  integration. B4 detects this from state (`ε` vs a config threshold) and switches the integrator — the
  `grmhd/select_metric` pattern. So the only thing "lost" past the crossover is **branch-cheapness, not
  correctness**, and it is lost in the dense lower atmosphere *after* the counterfactual decision window, where
  it costs least. **Residual `[holds under precondition]`:** the **handover** itself — the two integrators must
  agree in an overlap band, and the switch needs hysteresis to avoid chatter near `ε ≈ ε_switch`.
- **Constraint-projection uniqueness `[holds under precondition]`** — is there always a unique nearest
  constraint-satisfying state (gauge-fixing ambiguity)? Standard in constrained filtering, but needs a fixed
  gauge choice.
- **Conformal-lift fidelity `[holds]`** — only monopole + PN is *exactly* inverse-square; J2+ harmonics share
  the perturbation channel with aero, raising that channel's load at low altitude (where aero is largest anyway).
- **Preliminary by construction `[preliminary]`** — the aero-coupling interface (what the Tier-B marcher exposes
  as a force/flux, and at what cadence) is unbuilt, so the factoring (piecewise-constant vs higher-order aero
  hold, substep cadence) is provisional; **revisit after Tier-B Stage 4+.**

**Generalized method.** *Split off the inverse-square (conformal) core and propagate it exactly by the 2T matrix
exponential; carry all non-conformal forcing (geopotential harmonics + aero) as a bounded between-step
perturbation; run the 6D filter as predict + projection onto the Sp(2,R) constraint manifold; read the
relativistic clock correction off the dynamic `τ↔t` metric relation; and **gate the whole integrator behind a
state-driven regime detector** (`grmhd/select_metric`) that hands off to direct integration when the
perturbation stops being small.* This is the **fifth confinement instance** — *evolve the structured part
exactly, confine the rest to a between-step carrier, project onto the constraint surface, and switch regime when
the structure breaks.* Trajectory ≅ CFD-IMEX ≅ LER, and its Encke→Cowell switch ≅ the continuum→plasma switch.

**Inverse / scaling.** As aero → 0 (exo-atmospheric coast) the method is a pure matrix exponential — exact and
free. As `ε = a_aero/a_grav` crosses the threshold (dense atmosphere) the regime detector switches to direct
integration — branch-cheapness is given up *observably*, by an explicit coupling decision, not silently. The
branch-cheap advantage is **maximal exactly during coast and the blackout-onset window**, which is where the
counterfactual rollouts live; the regime switch falls *after* it, where the decision is already committed.

---

## Verification gates (preliminary — to harden after Tier-B Stage 4+)

1. **Coast exactness:** with aero = 0 and monopole + PN gravity, the matrix-exponential propagation matches the
   analytic Kepler / Schwarzschild orbit to round-off (and beats the skeleton's first-order Euler measurably).
2. **Split accuracy:** with aero + J2 as perturbations, a Strang-split trajectory matches a high-fidelity ODE
   reference within tolerance over a representative reentry arc; error scales with substep as expected.
3. **Constraint preservation:** the 6D state stays on the Sp(2,R) surface (`X·X, X·P, P·P` within tol) across
   predict + correct, including a GNSS-denial → fix-return cycle.
4. **Dynamic timing:** the carried clock correction equals `−(1 − 2GM/rc²) + γ(v)` computed **from state**
   (ns-level), not a constant, and matches an IERS reference at the appropriate order — the dynamic-invariant
   test for the timing axis.
5. **Blackout carry + path-dependence:** through a `BlackoutTrigger` predict-only window the clock correction is
   carried internally and reconciled on fix-return; two counterfactual histories yield two clock/position
   outcomes (the strengthened §1.2 dynamic-invariant test, on the trajectory axis).
6. **Regime switch (the `grmhd` pattern):** the `ε = a_aero/a_grav` indicator is computed from state and drives a
   `select_integrator` coupling against a config threshold; gate that (a) the perturbed-conformal and direct
   integrators **agree within tolerance in an overlap band** around `ε_switch`, (b) the switch is
   **hysteresis-stable** (no chatter), and (c) the matrix-exponential advantage is **measured** (branch cost in
   the perturbative regime vs direct), confirming it holds across the coast/blackout-onset window. The
   aero-dominated leg falls back to direct integration — correct, and outside the decision-critical window.

---

## Related

- [`../plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) §2 (where relativity actually bites), §3.1
  (the Two-Time accelerator), §6 (the aero-coupling seam this resolves preliminarily).
- [`../gap-analysis.md`](../gap-analysis.md) Gap 3 — the skeleton state + the dynamic-curvature mandate this honors.
- [`../gap-2/gap-two-resolution-1-stiff-source.md`](../gap-2/gap-two-resolution-1-stiff-source.md) — LER, the
  **temporal twin** (closed-form exact core + between-step source).
- [`../gap-2/gap-two-resolution-7-feature-adaptive-coordinate.md`](../gap-2/gap-two-resolution-7-feature-adaptive-coordinate.md)
  — the confinement family this extends to a fifth axis.
- `examples/avionics_examples/hypersonic_2t/model.rs` — the skeleton: the first-order `predict` to upgrade to
  `e^{Gτ}`, the `correct()` no-op to replace with predict + constraint projection, the hand-set generator to
  derive from the dynamic metric.
- `examples/physics_examples/grmhd/model.rs` — **the adopted regime-detection mechanism**
  (`calculate_curvature` → `select_metric`: state-derived indicator vs config threshold → switch the governing
  model), here reused as the Encke→Cowell `select_integrator` coupling (B4); also the proxy curvature
  (`g₀₀ = −0.9` …) to replace with the state-computed metric (the same dynamic-invariant mandate).
