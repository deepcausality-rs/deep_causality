<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 2 — a state-derived driving temperature without a compressible marcher

**What this is.** A TRIZ/ARIZ resolution of the second load-bearing assumption hidden in the
[Gap-2 plan](gap-two-reacting-plasma.md): that the temperature driving ionization must come *either* from a
compressible energy equation (shock-compression heating — the unbuilt Tier-B marcher) *or* from a prescribed
schedule. The built QTT solver is **incompressible** and carries no energy equation, so on the face of it the
driving temperature can only be imposed — which collides head-on with the
**dynamic-by-construction invariant** ([gap-two §1.2](gap-two-reacting-plasma.md): no hardcoded schedules;
quantities computed from state). This note dissolves the false dichotomy.

One of three coupled resolutions sharing the **Lagging-Equilibrium Relaxation (LER)** mechanism; see
[Resolution 1](gap-two-resolution-1-stiff-source.md) and [Resolution 3](gap-two-resolution-3-ionization-lag.md).

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** obtain a temperature field that varies with the flow state, on a
  solver that carries no energy equation, without building one.
- **System / main function:** the incompressible QTT rollout + its passive-scalar carrier
  (`advance_scalar`); *to present ionization with a temperature that is a function of the computed flow.*
- **The constraint treated as fixed — the lever:** the unstated belief that temperature is *either* a
  transported PDE state *or* a prescribed input. **It is neither — it is a diagnostic of the velocity field
  the solver already produces.**

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** solve a compressible energy equation → temperature is physically grounded, but it requires the
  Tier-B compressible shock-capturing marcher (not built; open research).
- **TC-2:** prescribe a temperature schedule → buildable now, but it is a hardcoded input that violates the
  dynamic invariant and makes every counterfactual branch return the same blackout.

**A3 — Intensify.** Push TC-2 to the extreme: a *fully* prescribed `T(x,t)` makes the whole flagship a
lookup table — the flow does nothing. That extreme exposes the real requirement: the temperature's **spatial
structure must be produced by the computed flow**, even if its scale is set by the flight condition.

**A5 — Resources already present (no new substance):**
- The incompressible solver **already produces a velocity field `u(x)`** — and kinetic energy is exactly what
  converts to heat at stagnation. The "lost" compressible heating is **latent in `|u|`**.
- **Total/stagnation enthalpy is conserved along streamlines** (Bernoulli/Crocco) — a known *effect* that
  maps velocity to static temperature with no PDE.
- The flight condition (`U_∞`, `M_∞`, `T_∞`) is **legitimately config** — gap-two §1.2 explicitly allows a
  caller-supplied config input; what it forbids is a hardcoded *field*.
- The **Rankine–Hugoniot normal-shock jump** is a closed-form algebraic map from freestream to post-shock
  state (config `M_∞`).

**A7 — Smart Little People.** The little people sit on the velocity field. Where the flow stagnates (the
nose), they convert all the kinetic energy they carry into heat; where it accelerates, they cool. They read
the temperature *off the flow they are already in* — no one hands them a schedule.

**Physical contradiction:** temperature must be *computed from state* (dynamic invariant) yet *not require an
energy PDE* (no compressible marcher). **Resolve by substituting the field for a relation:** replace the
missing energy *PDE* with an algebraic *enthalpy relation* on the existing velocity field.

→ Reformulation cracks it.

---

## B. Solve — temperature as an enthalpy reconstruction of the computed velocity field

The incompressible solver carries `u(x)`. **Temperature is a diagnostic of it** via stagnation/total
enthalpy — the kinetic energy the incompressible flow *does* resolve is exactly what converts to heat at
stagnation:

```
T_tr(x) = T_post − |u(x)|² / (2 c_p)
```

where `T_post` is the post-shock stagnation temperature reached from the **config** flight condition through
an **algebraic Rankine–Hugoniot normal-shock jump** (config `M_∞`, `T_∞`):

```
T_post = T_∞ · rankine_hugoniot_temperature_ratio(M_∞)
```

The **spatial structure** (hot where the computed flow stagnates at the nose, cooler where it accelerates)
comes entirely from the **computed `|u|`** field. Only `M_∞`, `T_∞` are config. Change `U_∞` and the entire
temperature field moves — **emergent, not hardcoded**. This honors §1.2 exactly: flight condition is
caller config (allowed); the *field* is computed from state.

> **TRIZ principles used:** **#25 Self-service** (the velocity field is its own thermometer); **#2 Taking out
> / extraction** (extract the thermal information already latent in the kinetic field); **#28 Mechanics
> substitution** (replace the missing energy *PDE* with an algebraic enthalpy *relation*); **#35 Parameter
> change** (temperature as a derived parameter, not a transported state). **Effects database:**
> Bernoulli/Crocco stagnation-enthalpy conservation + the Rankine–Hugoniot jump — both closed-form.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes. Temperature is computed from state (the velocity
  field) **and** needs no energy PDE — the false dichotomy is gone.
- **Only A5 resources?** Yes. Uses the existing velocity field, the enthalpy relation, the RH jump, and
  config flight conditions. No compressible marcher introduced.
- **Satisfies the dynamic invariant?** Yes — the *field* is a function of the computed flow; only the scalar
  flight condition is config.

**New harm — the weakest link, flag it loudly.** The isentropic recovery relation **alone is too cold to
ionize.** A real Mach-25 bow shock raises `T_tr` to ~10⁴ K via the Rankine–Hugoniot jump, far above the
isentropic recovery value. **The RH normal-shock jump is mandatory** as the `T_∞ → T_post` map — without it
the reconstructed field never reaches ionization temperatures and the slice silently produces no plasma.
With it, the *magnitude* is honest to first order and the *structure* is emergent. **[holds under
precondition: RH jump applied]**

**Second harm.** This reconstructs the **stagnation / boundary-layer recovery heating**, not the true
post-shock thermodynamic *path* (entropy layer, real-gas effects). It is the right driver for a Tier-A
demonstrator; it does **not** replace the Tier-B compressible energy solve. Label it as a recovery-temperature
reconstruction wherever it appears. **[open: true post-shock path is Tier-B]**

**Generalized method.** *A missing transported quantity is recovered as an algebraic diagnostic of a quantity
the solver already carries, using a conservation relation (here: static temperature from velocity via
stagnation enthalpy), with scale set by config and structure set by computed state.* This is the
**target-source** half of the shared **LER** pattern: the reconstructed `T_tr` is what the equilibrium
ionization target `α_eq(ρ, T_tr)` of [Resolution 3](gap-two-resolution-3-ionization-lag.md) reads, and the
relaxation it drives is integrated by [Resolution 1](gap-two-resolution-1-stiff-source.md).

**Inverse / scaling.** As the solver gains a real energy equation (Tier-B), the same `T_tr` symbol is fed by
the transported temperature instead of the reconstruction — the ionization stage above it never changes. The
reconstruction is the Tier-A stand-in behind a stable interface.

---

## Verification gates (what a spec/PR must prove)

1. **Emergence:** two different freestream velocities produce two different temperature *fields* (not a
   rescaled constant) — the structure tracks the computed stagnation pattern. **[holds]**
2. **Magnitude sanity:** with the RH jump at `M_∞ ≈ 25`, peak reconstructed `T_post` lands in the
   ~10⁴ K band that RAM-C / Park report — not the cold isentropic value. **This is the make-or-break gate.**
3. **Dynamic invariant (§1.2):** `T_tr` field is a pure function of the computed `|u|` and the config
   `(M_∞, T_∞)`; grep shows no literal temperature schedule, no `T = const`.
4. **Honesty label present:** the observable/report names this a *recovery-temperature reconstruction*, Tier-A,
   not a post-shock thermodynamic solve.

---

## Related

- [`gap-two-reacting-plasma.md`](gap-two-reacting-plasma.md) §1.2 (dynamic invariant), §3.2 (the scalar seam),
  §4 (the compressible-marcher precondition this sidesteps for Tier-A).
- [`gap-two-resolution-3-ionization-lag.md`](gap-two-resolution-3-ionization-lag.md) — consumes this `T_tr`
  as the equilibrium-ionization target.
- [`gap-two-resolution-1-stiff-source.md`](gap-two-resolution-1-stiff-source.md) — the stable integrator the
  ionization relaxation rides.
- [`plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) §6 (shock-rank seam), §7 (Tier-A surrogate).
- `deep_causality_cfd` `solvers/qtt/` (`advance_scalar`) and `deep_causality_physics/src/kernels/` — where the
  RH-jump + enthalpy reconstruction kernels land.
