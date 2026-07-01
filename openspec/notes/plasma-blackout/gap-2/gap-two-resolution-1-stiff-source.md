<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 1 — stiff source integration without an explicit-splitting wall

**What this is.** A TRIZ/ARIZ resolution of the first load-bearing assumption hidden in the
[Gap-2 plan](gap-two-reacting-plasma.md): that the Park-2T relaxation/ionization source terms are
non-stiff enough to advance with an **explicit** between-step `PhysicsStage` update. They are not — chemistry
runs at nanoseconds, the flow timestep at micro-to-milliseconds, a ~10³ separation that makes explicit
operator-splitting unstable or forces the marcher timestep to collapse. This note resolves the contradiction
**without abandoning the clean between-step kernel/stage architecture**.

It is one of three coupled resolutions that share a single mechanism — the
**Lagging-Equilibrium Relaxation (LER)** pattern (defined in
[Resolution 3](gap-two-resolution-3-ionization-lag.md) §C). See also
[Resolution 2](gap-two-resolution-2-temperature-provenance.md).

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** advance a process whose intrinsic timescale is ~10³× shorter than
  the carrier timestep, without the carrier timestep collapsing to the fast scale.
- **System / main function:** the between-step `PhysicsStage` (`coupling.rs`); *to advance the source term
  (relaxation, ionization) from local state, each step, stably.*
- **The constraint treated as fixed — the lever:** that the kernel computes a **rate** `ω̇` and the marcher
  integrates it **explicitly** across `Δt`. Drop this and the stiffness wall disappears.

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** integrate the source *explicitly* across the full `Δt` → the stage stays pure, pointwise, and
  composable, but the scheme is **unstable** under stiffness.
- **TC-2:** integrate the source *implicitly* with a global Newton solve over the coupled field → stable,
  but it **couples physics correctness to the compression/marcher machinery** and destroys the
  pure-pointwise kernel split (the very split the Gap-2 architecture is built on).

**A3 — Intensify.** Push the chemistry timescale `τ → 0` (instantaneous reaction). The *rate* becomes
infinite — explicit Euler is hopeless. But the **integrated answer is trivial**: the state jumps exactly to
its equilibrium target. *The integrated increment is best-behaved precisely where the rate is worst.* The
solution is visible from the extreme: integrate the increment, never the rate.

**A5 — Resources already present (no new substance):**
- `Δt` is **already in `StepContext`** — the stage can integrate over it internally.
- The dominant source has **known analytic structure**: linear relaxation (Landau–Teller
  `dT_ve/dt = (T_tr − T_ve)/τ`) has a **closed-form exponential** solution; the nonlinear ionization source
  linearizes to one.
- The `PhysicsStage`/`Coupling` seam **already isolates the source from transport** — that isolation *is*
  the operator split; only the integrator inside it is wrong.

**A7 — Smart Little People.** Put tiny agents in one cell. Each knows `τ` and `Δt`. They do **not** compute a
derivative and step forward (that overshoots when `τ ≪ Δt`). They look up *where they want to be* (the
target) and move exactly the analytic fraction `1 − e^{−Δt/τ}` of the way there. They **cannot overshoot** —
the exponential is bounded by construction. No global coupling, no iteration, no stability limit. The little
people *are* the exponential integrator.

**Physical contradiction:** the update must be *explicit* (at the stage/marcher interface — pure, between-step)
**and** *implicit* (in its integration — stable). **Resolve by separation across scale:** explicit at the
macro interface, exact/implicit *within* the stage.

→ Reformulation cracks it. The matrix lookup is confirmatory, not needed.

---

## B. Solve — exponential / linearly-implicit integration inside the stage

**Change the kernel contract.** Today's implied signature returns a rate the marcher must integrate:

```
fn source_kernel(state) -> Rate          // marcher integrates explicitly → stiff, unstable
```

Make the kernel return the **integrated increment over `Δt`** (`Δt` is already in context):

```
fn relax_kernel<R: RealField>(x: X<R>, target: X<R>, tau: Time<R>, dt: Time<R>) -> Delta<R>
```

For the linear relaxation that dominates 2-T physics this is **closed-form and unconditionally stable**:

```
x(t+Δt) = target − (target − x(t)) · exp(−Δt / τ)
```

No iteration, no Jacobian, **still a pure pointwise `fn<R: RealField>`** — it simply takes `τ` and `dt` as
arguments. For the nonlinear ionization source, the same shape with a **linearly-implicit one-step update**
(exponential Rosenbrock / Patankar): linearize the source about the cell's current state, integrate the
linear part exactly. Still per-cell, still pure, **no global solve**.

**The `PhysicsStage`/`Coupling` architecture is untouched.** Only the integrator inside the stage changes
from explicit Euler to exponential. The `ThermalRelax` template the Gap-2 stages copy *should already* be
this; the rule is now explicit: **every stiff stage integrates its source in closed form over `Δt`, not by an
explicit rate step.**

> **TRIZ principles used:** separation by scale (resolves the physical contradiction); **#15 Dynamics**;
> **#1 Segmentation** (a stage may take invisible internal substeps if a single exponential is too coarse);
> **#25 Self-service** (the kernel integrates its own ODE). **Effects database:** the exponential integrator
> for linear relaxation is the textbook stable treatment of a relaxation operator — standard physics, not an
> invention.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes. "Explicit *and* implicit" is resolved by scale
  separation: explicit at the interface, exact-integral in the interior. No trade — the marcher is stable
  **and** the stage stays pure and pointwise.
- **Only A5 resources?** Yes. No global implicit solver, no Newton over the field, no new substance. Uses
  `Δt` (already in context) and the analytic exponential.
- **Satisfies the main function / IFR?** Yes — the source advances from local state every step, stably,
  inside the existing seam.

**New harm (the next problem):** operator-split accuracy is now first-order in `Δt` (Lie splitting). If
blackout-onset *timing* must be sharp, upgrade to **Strang splitting** (half-step source / full transport /
half-step source) — free, same kernels, second-order. **[holds under precondition: Strang if timing-critical]**

**Generalized method.** *A stiff between-step source is integrated in closed form (exponential for linear
relaxation; linearly-implicit for the nonlinear source) so the kernel returns the increment over `Δt`, not the
rate. Stiffness is confined inside the stage; the marcher and the split are untouched.* This is the
integrator half of the shared **LER** pattern — the equilibrium *target* it relaxes toward is supplied by
[Resolution 3](gap-two-resolution-3-ionization-lag.md), computed from the state-derived temperature of
[Resolution 2](gap-two-resolution-2-temperature-provenance.md).

**Inverse / scaling.** As `τ → 0` the update degrades gracefully to "jump to target" (the equilibrium limit,
exactly the Saha validation limit). As fidelity → ∞, the same interface accepts a true multi-step stiff ODE
integrator behind the stage without changing the marcher — the Tier-A → Tier-B path.

---

## Verification gates (what a spec/PR must prove)

1. **Stability at stiffness:** a relaxation stage with `τ = Δt / 1000` stays bounded and monotone toward the
   target over a long run (explicit Euler would diverge). **[holds: exponential is unconditionally stable]**
2. **Exactness on the linear case:** the relaxation kernel reproduces `target − (target − x₀)·e^{−Δt/τ}` to
   round-off — it is the analytic solution, so the test is an equality, not a tolerance.
3. **Purity:** the kernel is a free `fn<R: RealField>(…) -> Result<_, PhysicsError>` taking `τ`, `dt` —
   no state, no global solve, no `dyn` (honors the kernel convention, gap-two §1).
4. **Dynamic invariant (gap-two §1.2):** two states in → two increments out; `τ` is a function of state /
   config, never a hardcoded schedule.

---

## Related

- [`gap-two-reacting-plasma.md`](gap-two-reacting-plasma.md) §1.1 (`PhysicsStage` idiom), §3.2 (`IonizationStage`),
  §4 (the stiffness this resolves) — the plan this note hardens.
- [`gap-two-resolution-2-temperature-provenance.md`](gap-two-resolution-2-temperature-provenance.md) — the
  state-derived temperature that feeds the equilibrium target.
- [`gap-two-resolution-3-ionization-lag.md`](gap-two-resolution-3-ionization-lag.md) — the **LER** method this
  integrator completes; the relaxation *target* and *timescale*.
- `deep_causality_cfd/src/types/flow/coupling.rs` — `PhysicsStage` / `Coupling`, the seam left untouched.
- `deep_causality_physics/src/kernels/` — where the closed-form `relax_kernel` family lands.
