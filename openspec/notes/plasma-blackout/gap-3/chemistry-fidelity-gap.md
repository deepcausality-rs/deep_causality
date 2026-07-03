<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Chemistry-fidelity gap — RAM-C electron-density precision (deferred, bundled with Gap 3)

> **STATUS — lever 1 implemented; lever 2 investigated and not adopted** (see
> [Resolution 2](gap-three-resolution-2-tve-controlled-ionization.md)):
> - **Lever 1 — T_ve-controlled ionization (the dominant one): ✅ shipped.** Peak `n_e` from the
>   single-temperature surrogate's **~12× over-prediction to ~1.1×** of the RAM-C II anchor
>   (`α: 4.6×10⁻³ → 4.1×10⁻⁴` vs RAM-C `~3.8×10⁻⁴`). Saha/rate driven off `Tₐ = √(T_tr·T_ve)`.
> - **Lever 2 — 3-T electron-energy separation: prototyped, then reverted.** The explicit `T_e` is higher
>   fidelity but predicts *less* ionization (~3.7× low) — it brackets the spread rather than improving the
>   already-calibrated ~1.1×, so the added code bought nothing. The durable insight (electrons are *created*
>   in the post-shock bath, so `T_e(0) = T_ve`, not the frozen-cold free-stream) is recorded; the code is not
>   kept. A *faithful* 3-T would need e–ion Coulomb heating + the ionization-energy sink, not less.
>
> Lever 3 (finite-rate ionization network) is **proposed and design-reviewed, not yet built**: the change
> `openspec/changes/add-finite-rate-ionization-network/` carries the full specification, amended by the ARIZ
> re-evaluation in [Resolution 3](gap-three-resolution-3-chemistry-fidelity.md) (lagged atom pool from day
> one; detailed balance pinned in the thermal-equilibrium limit; channel-by-channel measurement). The
> trajectory/timing half of the bundle remains open
> ([Resolution 1](gap-three-resolution-1-perturbed-conformal-trajectory.md), preliminary).

**Scheduling.** This is a precision upgrade for the Gap-2 ionization physics, **deferred and lumped with
[Gap 3 (the trajectory axis)](gap-three-resolution-1-perturbed-conformal-trajectory.md)**, to be
solved **right after Gap 2 is closed**. It is *not* part of the Tier-A/B surrogate scope; the Stage-4 RAM-C
milestone deliberately gates an order-of-magnitude match. This note records the error anatomy and the levers
so the work is well-scoped when it starts.

---

## The measured gap

The Stage-4 stagnation-line verification (`verification/qtt_ramc_stagline`) lands peak
`n_e = 1.2×10²⁰ m⁻³` against the RAM-C II anchor `1×10¹⁹ m⁻³`: **≈12× (+1.1 decades) high**. A calibrated
production reacting-CFD code (DPLR / LAURA / US3D with finite-rate multi-temperature chemistry) reaches
**~2–3×** on RAM-C peak `n_e`; chemistry-model spread alone is ~2–5×. So 12× is ~4–6× looser than a
production code — the expected order-of-magnitude band for a surrogate, but not production accuracy.

## Where the error actually lives (it is **not** temperature or numerics)

| Quantity | This solver | RAM-C (real) | Ratio |
|---|---|---|---|
| Post-shock `T₂` | 8044 K | ~7000–8000 K | ~right |
| Ionization fraction `α` | 4.6×10⁻³ | ~3.8×10⁻⁴ | **~12×** |
| → `n_e` | 1.2×10²⁰ | 1×10¹⁹ | ~12× |

`T₂` (and the effective-γ that sets it) is roughly correct, and the IMEX / fitting stages are numerics that
do not touch `n_e`. The entire deviation is an **over-predicted ionization fraction**. The tell: Saha
*equilibrium* at 8000 K is `α≈0.15`; the real flow reaches `α≈3.8×10⁻⁴`, ~400× below equilibrium, because
ionization is rate-limited and never equilibrates. The Tier-A surrogate's single-rate lag only knocks
equilibrium down ~30×. **The lag is too weak, and it ionizes at the wrong (translational) temperature.**

## The levers to reach ~3–4× (all chemistry-model fidelity)

1. **T_ve-controlled ionization (the real Park-2T mechanism, the biggest single lever). — ✅ IMPLEMENTED,
   see [Resolution 2](gap-three-resolution-2-tve-controlled-ionization.md).** The surrogate ionized at the
   translational `T₂`; the physically correct control is the **lagging electron-vibrational temperature
   `T_ve`** (or the geometric-mean `Tₐ = √(T_tr·T_ve)`). In the post-shock zone `T_ve ≪ T_tr`
   (vibrational/electronic modes lag), which suppresses ionization — *that lag is the regime driver*. The
   Tier-A `vibrational_relaxation_kernel` (Millikan–White `τ_vt`, closed-form LER) already computes `T_ve`;
   the change drove the Saha target and the rate off `Tₐ`, not `T_tr`. Exponentially sensitive — **measured
   to move the full ~12× → ~1.1×** on its own (`stagnation_line_blackout_2t`), better than the ~3–4× target.
2. **3-T: separate electron energy (drop the `T_e = T_ve` lumping). — 🔬 INVESTIGATED, NOT ADOPTED, see
   [Resolution 2 §"Lever 2"](gap-three-resolution-2-tve-controlled-ionization.md).** The Farbar–Boyd–Martin
   direction, prototyped LER-native (`T_e` relaxing toward `T_tr` via Appleton–Bray `τ_eT` and `T_ve` via
   `τ_eV = τ_vt`, Saha target at the resolved `T_e`) and then reverted. It predicts *less* ionization
   (`n_e ≈ 2.7×10¹⁸`, ~3.7× low) — it brackets the spread rather than improving the already-calibrated ~1.1×.
   The durable insight: electrons are *created* in the post-shock bath (init `T_e = T_ve`), not frozen-cold
   from free-stream (a cold start collapses `n_e` ~100×). A faithful 3-T would need e–ion Coulomb heating +
   the ionization-energy sink — more, not less — so it is closed unless the anchor is tightened.
3. **Finite-rate ionization network instead of Saha-target × single associative rate.** RAM-C (~7.6 km/s) is
   *mixed* associative + electron-impact, the latter thresholded, plus recombination. A two-way finite-rate
   network with the right rates tracks the real buildup instead of chasing a too-high Saha target.

`(lever 1, several×) × (lever 2, ~2×)` already brackets the **12× → ~3–4×** target; lever 3 firms it into the
production band. None of these are the QTT marcher stages — they are the reacting `*_rhs` / multi-temperature
kernels in `deep_causality_physics` plus the LER stages.

## Why the marcher stages (5–6) are a prerequisite, not the fix

The 2-D/3-D compressible marcher does not set `n_e` magnitude (chemistry does), but it provides the **real
transported `T_tr` / `T_ve` / species fields** these closures consume — which Tier-A faked with the
recovery-temperature reconstruction. So this chemistry work rides on the marcher, and is correctly scheduled
**after** Gap 2 (the flowfield + Tier-A/B physics) is closed.

## Bundled work item (post-Gap-2)

Solve together with [Gap 3](gap-three-resolution-1-perturbed-conformal-trajectory.md):

- **(Gap 3)** the trajectory/timing axis (2T conformal lift, 6D filter, IERS timing, the regime-switched
  integrator).
- **(this gap)** the n_e chemistry-fidelity upgrade: T_ve-controlled ionization → 3-T electron energy →
  finite-rate ionization network, target **~3–4×** on RAM-C peak `n_e`.

## Related

- [`tier-b-compressible-marcher.md`](../gap-2/tier-b-compressible-marcher.md) — the Stage-4 milestone and its
  order-of-magnitude gate.
- [`gap-two-resolution-3-ionization-lag.md`](../gap-2/gap-two-resolution-3-ionization-lag.md) — the LER ionization-lag
  pattern these upgrades extend (more `(target, τ)` modes; T_ve as the control).
- [`../gap-analysis.md`](../gap-analysis.md) — the gap tracker registering this deferral.
- `deep_causality_cfd/verification/qtt_ramc_stagline/` — the measured 12× this targets.
