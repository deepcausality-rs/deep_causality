<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 3 — nonequilibrium ionization lag without the stiff reaction network

**What this is.** A TRIZ/ARIZ resolution of the third load-bearing assumption hidden in the
[Gap-2 plan](gap-two-reacting-plasma.md): that a Tier-A ionization model must choose between a **memoryless
algebraic surrogate** `α(ρ, T)` — cheap but equilibrium, throwing away the nonequilibrium **lag** that *is*
the regime driver — and the **full finite-rate network**, which reintroduces the stiffness of
[Resolution 1](gap-two-resolution-1-stiff-source.md). This note shows the lag costs **one extra scalar state**,
not a reaction network, and in doing so it **defines the shared mechanism** the other two resolutions plug
into: the **Lagging-Equilibrium Relaxation (LER)** pattern.

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** reproduce ionization *history* (the flow has moved on before the
  electrons catch up) cheaply, without integrating the stiff network that produces it.
- **System / main function:** the ionization stage over the QTT rollout; *to present a blackout trigger with
  electron density that lags the flow, as the real corridor does.*
- **The constraint treated as fixed — the lever:** the belief that **"lag" requires "the full stiff reaction
  network."** False. Lag is a *relaxation* phenomenon — it needs **one** timescale, not N stiff reactions.

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** use the fitted algebraic surrogate `α_eq(ρ, T)` → cheap and non-stiff, but **memoryless** — it
  has no lag, so it is equilibrium ionization wearing a Park-2T label; the regime driver is lost.
- **TC-2:** use the full finite-rate closure → captures the lag, but **reintroduces the stiff network**
  (Resolution 1's problem) and a large reaction set.

**A3 — Intensify.** Push the surrogate to the extreme: instantaneous equilibrium everywhere
(`α ≡ α_eq`). That extreme says exactly what is missing — a *delay* between `α` and `α_eq`. A delay is the
cheapest dynamical object there is: a first-order relaxation. The whole network is needed only if you insist
on resolving *which* reactions cause the delay; for Tier-A you need only *that* there is one, with the right
timescale.

**A5 — Resources already present (no new substance):**
- The **equilibrium surrogate `α_eq(ρ, T)`** (Saha / fitted) already exists in the plan — reuse it, but
  **as a target, not as the answer.**
- The **`advance_scalar` carrier** (Gap 1) can carry a *relaxing* scalar `α`, not just an advected one.
- The **exponential integrator** of [Resolution 1](gap-two-resolution-1-stiff-source.md) integrates a
  first-order relaxation exactly and stably.
- A **physically-grounded timescale** is available from the dominant associative-ionization reaction rate —
  one Arrhenius coefficient, not the whole table.

**A7 — Smart Little People.** Each little person holds the current `α` and can see `α_eq` (where ionization
*wants* to be at this `ρ, T`). They do not jump there — they walk toward it at rate `1/τ_ion`. While the flow
changes faster than they can walk, `α` lags behind `α_eq`: **that gap is the nonequilibrium.** One number of
memory per cell.

**Physical contradiction:** ionization must be *at equilibrium* (cheap, algebraic) and *not at equilibrium*
(lagging — the physics). **Resolve by separation in time:** the *target* is the instantaneous equilibrium; the
*state* relaxes toward it with a finite time. Equilibrium and nonequilibrium coexist as target vs. state.

→ Reformulation cracks it — and yields the mechanism the whole Gap-2 Tier-A rides on.

---

## B. Solve — the Lagging-Equilibrium Relaxation (LER) pattern

The surrogate is **not the model; it is the target.** Carry `α` (or `n_e`, `T_ve`) as one extra scalar state
relaxing toward its cheap algebraic equilibrium, integrated by the closed-form scheme of Resolution 1:

```
α(t+Δt) = α_eq(ρ, T_tr) − (α_eq(ρ, T_tr) − α(t)) · exp(−Δt / τ_ion)
```

- **Memory / lag** = one scalar; `α ≠ α_eq` during transients is exactly the nonequilibrium regime driver.
- **Stiffness** = gone — it is the Resolution-1 exponential update, unconditionally stable.
- **Cheapness** = `α_eq` is the existing fitted/Saha surrogate; `τ_ion` is one timescale, not a network.
- **Equilibrium / Saha limit** = recovered as `τ_ion → 0` — precisely the validation anchor
  ([gap-two §5](gap-two-reacting-plasma.md): "Saha is the `τ → 0` limit the rate model must recover").
- **`T_tr`** = the state-derived recovery temperature of
  [Resolution 2](gap-two-resolution-2-temperature-provenance.md) — so the *target itself* is computed from
  state, not prescribed.

**The shared method, named once here.** *Carry K extra scalar states (e.g. `T_ve`, `α`). Each relaxes toward a
cheap algebraic equilibrium target computed from the current flow state, with a physically-grounded timescale,
advanced by a closed-form exponential update inside a between-step `PhysicsStage`.* This single LER mechanism
resolves all three Gap-2 contradictions: stable stiff integration (Res 1), a state-derived driver (Res 2), and
nonequilibrium lag (Res 3, here).

> **TRIZ principles used:** **separation in time** (target vs. state); **#35 Parameter change** (carry the lag
> as a derived state); **#23 Feedback** (the state relaxes toward its own equilibrium); **#1 Segmentation**
> (the full network collapses to its dominant relaxation mode). **Effects database:** relaxation-toward-
> equilibrium (BGK-like / Landau–Teller reduced kinetics) is a legitimate reduced model, not a hack.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes. Equilibrium (the target) and nonequilibrium (the
  lagging state) coexist by time-separation — no averaging, no compromise.
- **Only A5 resources?** Yes. Reuses the surrogate (as target), `advance_scalar`, the Resolution-1
  integrator, and one reaction rate. No full network, no new substance.
- **Captures the regime driver?** Yes — the lag `α − α_eq` is explicit and physical, and it makes the
  counterfactual branches genuinely path-dependent (two histories → two blackout outcomes, the deeper form of
  the §1.2 dynamic test).

**New harm — the weakest link, flag it.** `τ_ion` is the least-pedigreed new parameter. **Do not leave it a
free fit.** Ground it in the dominant associative-ionization reaction (N + O → NO⁺ + e⁻):
`τ_ion ≈ 1 / (k_f(T) · [M])` from one Park Arrhenius coefficient. That keeps it **dynamic** (a function of
`T`, not a constant) and traceable to a cited table. `τ_vt` for the vibrational relaxation is standard
Millikan–White and uncontroversial. **[holds under precondition: τ_ion grounded in the dominant rate]**

**Second harm.** Single-`τ` relaxation captures one mode; a real network has a spectrum of timescales.
Tier-A deliberately keeps the dominant one — say so in the report. The LER interface accepts more modes (more
`(target, τ)` pairs) without architectural change. **[open: multi-mode is Tier-B]**

**Inverse / scaling.** As fidelity → ∞, each `(α_eq, τ_ion)` pair is replaced by the full finite-rate network
**behind the same stage interface** — the literal Tier-A-surrogate → Tier-B-closure swap the Gap-2 note
promises ("only the stage implementations change", §1.1). As `τ → 0`, LER degrades to the algebraic
equilibrium surrogate — the validation limit. The method spans the whole fidelity axis from one interface.

---

## Verification gates (what a spec/PR must prove)

1. **Lag exists:** under a temperature ramp, `α` visibly trails `α_eq` by ~`τ_ion`; the gap vanishes as the
   ramp slows — nonequilibrium is real, not numerical. **[holds]**
2. **Saha limit:** as `τ_ion → 0`, `α → α_eq` (the equilibrium surrogate) to round-off — the validation
   anchor. **[holds]**
3. **`τ_ion` is dynamic:** grep shows `τ_ion` computed from the dominant rate `k_f(T)`, not a literal
   constant; honors §1.2.
4. **Path-dependence:** two counterfactual branches with different histories produce different `n_e` /
   blackout dwell — the lag carries memory across the branch (the strengthened §1.2 test).
5. **Ionized species present:** `α_eq` targets a nonzero ionization (the surrogate includes the NO⁺ + e⁻
   equilibrium) — otherwise `n_e ≡ 0` and there is nothing to lag. **[holds under precondition: ionized
   surrogate]**

---

## Related

- [`gap-two-reacting-plasma.md`](gap-two-reacting-plasma.md) §3.1 (the surrogate kernel, now a *target*), §5
  (the Saha `τ→0` limit), §6 (the staged Tier-A plan this fills in).
- [`gap-two-resolution-1-stiff-source.md`](gap-two-resolution-1-stiff-source.md) — the exponential integrator
  this relaxation rides (the other half of LER).
- [`gap-two-resolution-2-temperature-provenance.md`](gap-two-resolution-2-temperature-provenance.md) — the
  state-derived `T_tr` the equilibrium target `α_eq(ρ, T_tr)` reads.
- `deep_causality_physics/src/kernels/hypersonic/` — `ionization_fraction_kernel` (as the `α_eq` target) and
  the `τ_ion` rate kernel.
- `deep_causality_cfd/src/types/flow/coupling.rs` — the `IonizationStage` that carries `α` and applies LER.
