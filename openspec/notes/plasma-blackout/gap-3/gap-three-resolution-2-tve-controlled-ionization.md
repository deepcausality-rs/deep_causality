<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 3, Resolution 2 — the chemistry-fidelity axis: T_ve-controlled ionization (implemented)

**What this is.** A TRIZ/ARIZ resolution (via the `invent` skill) of the **chemistry-fidelity** half of the
[Gap-3 bundle](chemistry-fidelity-gap.md) — the RAM-C peak electron-density precision. Unlike
[Resolution 1 (the trajectory axis)](gap-three-resolution-1-perturbed-conformal-trajectory.md), which is
preliminary, **this one is built, measured, and gated**: it takes the Stage-4 RAM-C peak `n_e` from the
single-temperature surrogate's **~12× over-prediction down to ~1.1× of the RAM-C II anchor**, with no
fudging — only the Park two-temperature mechanism the disclaimer already admitted was missing.

The finding: the 12× error is the **fourth instance of the confinement family** (LER/time, FAC/space,
shock-fitting/interface, now controller/relaxation-state). *Evolve the structured fast part exactly
(translational energy, RH jump), confine the lag to a closed-form between-state (the vibrational-electron
temperature via the LER kernel), and route the rate/equilibrium through the physically-correct controlling
temperature.* The chemistry axis becomes structurally **identical to the LER temporal twin**: an exact-fast
core + a closed-form lagging controller.

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**, **[preliminary]**.

> **Result (measured, `verification/qtt_ramc_stagline`):**
>
> | Quantity | Single-T surrogate (was) | **Park-2T controller (now)** | RAM-C II |
> |---|---|---|---|
> | ionization fraction `α` | 4.63×10⁻³ | **4.10×10⁻⁴** | ~3.8×10⁻⁴ |
> | peak `n_e` (m⁻³) | 1.22×10²⁰ (+1.1 dec) | **1.09×10¹⁹ (+0.0 dec)** | 1×10¹⁹ |
> | error vs anchor | **~12×** | **~1.1×** | — |
>
> The landing sits *inside* the production reacting-CFD band (DPLR/LAURA reach ~2–3×) and well within the
> ~2–5× chemistry-model spread — better than the Gap-3 note's ~3–4× target.
>
> **Lever 2 (3-T electron-energy separation) was prototyped and *not adopted*** — it brackets low
> (`n_e ≈ 2.7×10¹⁸`, ~3.7×) without improving the ~1.1× point estimate, so the added code bought nothing and
> was reverted. The negative result is recorded in §"Lever 2 — investigated, not adopted" below.

---

## 0. Frame

- **Key problem (no solution words):** the surrogate counts ionization the real post-shock flow has **not
  yet produced** — it equilibrates the electron population to a temperature the electrons do not yet hold.
- **System / main function:** the stagnation-line ionization closure — *to set the post-shock free-electron
  population `n_e`* that drives plasma frequency → blackout.
- **The constraint treated as fixed (the lever):** *that ionization is governed by the transported
  translational temperature `T_tr = T₂`.* `T₂ ≈ 8044 K` is correct **as heavy-particle translational
  energy** — it is the **wrong controller** for ionization. The shock dumps energy into translation first;
  the vibrational / electronic / free-electron bath fills *behind* it on a relaxation clock. Ionization
  rides the lagging bath, not the hot translation.

---

## A. Reformulate (the ARIZ spine)

**A1 — Components & functions.**
- Heavy-particle translation `T_tr` — useful (pressure, density, RH jump); **harmful when borrowed as the
  ionization controller** (over-energizes the electron balance).
- Vibrational-electron bath `T_ve` — the *physically correct* controller; **present in the crate but unwired
  into the ionization path** (insufficient function).
- Saha equilibrium target `α_eq(T)` — **excessive** when fed `T_tr` (chases a 0.15–0.37 equilibrium that
  never exists at the electron temperature).
- Associative-ionization lag `τ_ion` — real, but **too weak alone**: it relaxes toward a target that is
  itself too high and at the wrong temperature.
- Supersystem: the two clocks — residence time `t_res = standoff/u₂` and Millikan–White `τ_vt` — whose
  **ratio** decides how far `T_ve` has caught up. At RAM-C, `t_res/τ_vt ≈ 1` — exactly where the lag bites.

**A2 — Two technical contradictions.**
- **TC-1:** evaluate ionization at `T_tr` → faithful to the transported energy, no extra state — **but**
  over-predicts `n_e` 12× because translation is not where the electrons live yet.
- **TC-2:** evaluate ionization at a separately-tracked `T_ve` → physically correct — **but** *appears* to
  need a second transported energy equation the surrogate was built to avoid.

**A3 — Intensify (this is where it cracks).** Push the clock ratio to its extremes:
- `t_res/τ_vt → ∞` (fully relaxed): `T_ve → T_tr`, the two models **coincide** — TC-1 is exact, correctly so
  (the equilibrium limit).
- `t_res/τ_vt → 0` (frozen): `T_ve → T_∞` (cold), ionization is **absent** — the real nonequilibrium limit;
  TC-1 is maximally wrong, by orders of magnitude.

**The gap between the two models *is* the vibrational lag**, and it is bounded and computable from the two
clocks already present.

**A5 — Resources already present (the phantom cost dissolves).**
- The **`vibrational_relaxation_kernel`** (Millikan–White `τ_vt` + the *closed-form LER exponential*)
  **already computes `T_ve(t_res)`** from `(T_∞ → T_tr)` over the residence time. **The "second energy
  equation" of TC-2 is a phantom** — the relaxation is solved in closed form, the same LER trick trusted
  elsewhere. No new transported field.
- The **Saha and Arrhenius kernels are temperature-agnostic** — they take *a* temperature argument; nothing
  forces `T_tr`. The fix is **which temperature flows into the existing seam**, not a new kernel.
- Park's **rate-controlling temperature** `Tₐ = √(T_tr·T_ve)` (geometric mean, `q = ½`) — a *relation*, one
  `sqrt` over two resources already in hand.

**A6 — Operating Zone / Operating Time.** The conflict lives entirely **in the post-shock relaxation zone,
during the residence time**, where `T_ve < T_tr`. Outside it (`T_ve = T_tr`) the conflict vanishes. It
separates cleanly **on a condition (the relaxation state)**, not in global space/time → a
**separation-by-condition** signature → the solution is a controller *selected by the relaxation state*,
which `T_ve` already encodes.

**A7 — IFR + physical contradiction.**
- **IFR:** *The controller-element, using only the already-computed `T_ve` and `T_tr`, makes ionization
  equilibrate to the electron bath the flow actually holds during the residence time — eliminating the 12×
  over-count — without a transported equation or new harm.* Substituting resources for X: `T_tr` fails (the
  bug); `T_ve` works; **`Tₐ = √(T_tr·T_ve)` works and is Park-canonical**.
- **Physical contradiction:** the controlling temperature must be **`T₂` (hot — it *is* the transported
  translational energy) and not-`T₂` (cold — the electrons haven't caught up)** at the same post-shock
  point. **Smart Little People:** translational people arrive hot and immediately; vibrational/electron
  people trickle in on the `τ_vt` clock and are still cold at `t_res`. Ionization is a *handshake between a
  heavy-particle person and an electron person* — it proceeds at the **geometric mean of who's actually
  present**, `√(T_tr·T_ve)`, not at the temperature of the crowd that arrived first.

→ **Reformulation cracks it. No contradiction matrix needed.**

---

## B. Solve — separation by condition (the relaxation state)

Replace the single `T_tr` controller with the Park rate-controlling temperature **`Tₐ = √(T_tr·T_ve)`**,
`T_ve` from the existing `vibrational_relaxation_kernel` over `t_res`. Drive **both** the Saha equilibrium
target and the associative-ionization rate off `Tₐ` (one consistent controller — avoids the inconsistency of
mixing `T_e` for the target and `Tₐ` for the rate). The lag `α = α_eq·(1 − e^{−t_res/τ_ion})` is retained,
but now relaxes toward a target evaluated at the cold electron bath, so it lands far below the spurious
`T_tr` equilibrium.

**TRIZ principles used:** **#2 Taking out** (extract the ionization controller from the translational field
and give it its own correct carrier) + **#24 Intermediary** (`Tₐ` as the handshake temperature) +
**#3 Local Quality / separation by scale** (translational vs vibrational-electron energy at the same point).
**Effects database:** Park's two-temperature model and the geometric-mean controlling temperature are the
known physical effect — the contribution is recognizing them as the confinement-family pattern and that the
`T_ve` resource was already in the crate.

### Implementation (built)

- **`Park2tClosure<R>`** (`solvers/qtt/compressible/fitting.rs`) — the four gas properties the relaxation
  needs: free-stream `T_ve(0)`, post-shock pressure (atm), reduced mass (N₂–N₂ ≈ 7 amu), `θ_v(N₂)`.
- **`FittedNormalShock::stagnation_line_blackout_2t`** — relax `T_ve` (LER) → form `Tₐ = √(T_tr·T_ve)` →
  Saha target at `Tₐ` → associative-ionization lag at `Tₐ` → `n_e`, `ω_p`, blackout.
- **`verification/qtt_ramc_stagline`** — now reports the single-T surrogate *and* the 2-T controller side by
  side; the gate tightened from "~2 decades" to **"within ~3× of RAM-C II"**.

---

## C. Verify & harvest

- **Contradiction removed, not compromised?** **[holds]** — `T₂` keeps its correct job (transported
  translational energy, RH jump); ionization gets its correct controller (`Tₐ`). The two scales coexist at
  one point; nothing is averaged away.
- **Only A5 resources?** **[holds]** — `vibrational_relaxation_kernel`, the temperature-agnostic
  Saha/Arrhenius kernels, the typed temperatures, the two clocks. One added `sqrt`. **No new transported
  equation, no new substance/field.**
- **Satisfies the IFR / implementable?** **[holds]** — a controller-temperature swap at an existing seam;
  measured `n_e = 1.09×10¹⁹` (~1.1× of RAM-C), down from ~12×.

**New harm / open (the next problems):**
- **`τ_vt`-sensitivity `[holds under precondition]`** — the exact landing depends on the Millikan–White
  `τ_vt`, the reduced-mass / `θ_v` choice, and the `q = ½` exponent. This *is* the documented ~2–5×
  chemistry-model spread; the ~1.1× landing is honest but not a calibrated match, and the MW correlation
  (without the Park high-temperature limiting correction) is the controlling approximation.
- **2-T lumping `T_e = T_ve` `[investigated, not adopted — see "Lever 2" below]`** — the third
  electron-energy equation (Farbar–Boyd–Martin) was prototyped and reverted; it brackets the spread on the
  low side (~3.7×) rather than improving the ~1.1× point estimate, and a *faithful* 3-T needs more (e–ion
  Coulomb heating, the ionization-energy sink), not less. Closed unless the anchor is tightened.
- **Single associative channel `[open]`** — a finite-rate network (associative + thresholded electron-impact
  + recombination) is the remaining lever.

**Generalized method (the reusable invariant).** *When a relaxing system is being driven by the wrong
(fastest-arriving) energy mode, don't add a transported equation — compute the lagging mode in closed form
(LER) and route the rate/equilibrium through the physically-correct controlling temperature.* This is the
**confinement-family move**: evolve the structured fast part, confine the lag to a closed-form between-state,
and select the controller by the relaxation state. Chemistry ≅ LER-temporal ≅ shock-fitting ≅ the trajectory
split.

**Inverse / scaling.** As `t_res/τ_vt → ∞` the controller degrades **gracefully** to the single-temperature
model (`Tₐ → T₂`, correct in equilibrium — gated by `park2t_recovers_single_temperature_when_fully_relaxed`);
as `t_res/τ_vt → 0` it correctly **freezes** ionization. The advantage is maximal exactly in the
nonequilibrium relaxation zone where the real RAM-C flow lives.

---

## Lever 2 — investigated, not adopted (a negative result worth recording)

Lever 1 routes ionization through the geometric-mean **proxy** `Tₐ = √(T_tr·T_ve)` for the electron
controller. The natural next lever (Farbar–Boyd–Martin) is to **un-lump `T_e` from `T_ve`** — track the
electron temperature as its own relaxing scalar and evaluate the Saha target at the *resolved* `T_e`. This
was **prototyped end-to-end** (a two-target electron-energy LER toward `T_tr` via Appleton–Bray elastic
exchange and toward `T_ve` via e–V on the vibrational timescale) and then **reverted**, because it does not
improve the result. The finding:

- **It brackets low, not better.** The explicit 3-T is higher fidelity but predicts **less** ionization than
  the 2-T proxy — `n_e ≈ 2.7×10¹⁸` (~3.7× **low**) vs the 2-T's ~1.1×. The geometric-mean proxy turns out to
  be the better-calibrated *point* estimate; the two only **bracket** the ~2–5× spread. The 2-T result is
  already inside the production band (~2–3×) and within 10% of the anchor — there is nothing to improve.
- **The crux is an initial condition, not a constant.** A naive `T_e(0) = T_∞` (treating electrons like the
  frozen-cold vibrational bath) collapses `n_e` ~100× (`8.5×10¹⁶`, −2.1 dec) because the Saha target is
  *exponentially* sensitive to `T_e`. The physical fix is recognising that **electrons do not pre-exist —
  they are *created* in the post-shock bath**, so `T_e(0) = T_ve`. With that, the model lands at the ~3.7×
  above. *This is the durable insight from the exercise; it does not require the 3-T code to be kept.*
- **A faithful 3-T needs more, not less.** The prototype omits e–ion Coulomb heating and the
  ionization-energy sink — the terms a production 3-T carries to stay calibrated. Adding a half-built 3-T
  would trade a clean, calibrated ~1.1× surrogate for a more complex one that brackets low. Not worth it.

**Decision:** lever 2 is **closed as "investigated, not adopted."** The 2-T geometric-mean controller is the
shipped chemistry-fidelity model. A fully-calibrated 3-T (with the electron source terms) remains a possible
future lever, but only if the anchor itself is tightened beyond the ~2–3× the surrogate already achieves.

---

## Verification gates (built, in `tests/solvers/qtt/compressible_fitting_tests.rs` + the example)

1. **RAM-C landing:** `park2t_controller_marches_ramc_within_3x` — peak `n_e ∈ (3×10¹⁸, 3×10¹⁹)`, within ~3×
   of the RAM-C II anchor; blackout still triggers. (Measured 1.09×10¹⁹.)
2. **Suppression direction:** `park2t_controller_suppresses_below_single_temperature_surrogate` — the 2-T
   `n_e` sits strictly below the single-T surrogate (the cold bath suppresses ionization).
3. **Equilibrium limit:** `park2t_recovers_single_temperature_when_fully_relaxed` — at long residence the
   controller approaches the single-T Saha equilibrium within 10% (graceful degradation, no spurious
   suppression).
4. **Example gate:** `verification/qtt_ramc_stagline` self-verifies (exit nonzero on break) with the
   "within ~3× of RAM-C II (Park-2T controller)" `n_e` gate as the headline (it also prints the single-T
   surrogate for contrast).

---

## Related

- [`chemistry-fidelity-gap.md`](chemistry-fidelity-gap.md) — the error anatomy and the three levers this
  resolution implements the first (and dominant) of.
- [`gap-three-resolution-1-perturbed-conformal-trajectory.md`](gap-three-resolution-1-perturbed-conformal-trajectory.md)
  — the *other* half of the Gap-3 bundle (the trajectory/timing axis, still preliminary).
- [`../gap-2/gap-two-resolution-3-ionization-lag.md`](../gap-2/gap-two-resolution-3-ionization-lag.md) — the
  LER ionization-lag pattern this extends (the controlling temperature is now `Tₐ`, not `T_tr`).
- [`../gap-2/gap-two-resolution-1-stiff-source.md`](../gap-2/gap-two-resolution-1-stiff-source.md) — LER, the
  temporal twin of this confinement instance.
- `deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs` — `Park2tClosure`,
  `stagnation_line_blackout_2t`.
- `deep_causality_cfd/verification/qtt_ramc_stagline/` — the measured 12× → ~1.1× result.
</content>
</invoke>
