<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2 — reacting / ionized physics (Park-2T → electron density → blackout)

**What this is.** A focused plan for closing **Gap 2** of the
[plasma-blackout gap analysis](gap-analysis.md): the reacting / ionized physics that turns the (now
closed, Gap 1) tensor-train flowfield into the flagship's actual regime driver — *vibrational–electron
nonequilibrium → ionization → electron density → plasma frequency → GNSS/comms blackout* (chain steps
[2]/[3], feeding [4]). Its organizing decision, per the project owner, is an **architecture split**:
the **physics kernels go in `deep_causality_physics`; the solver stays in `deep_causality_cfd`.**

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**, **[speculative]**.

---

## 1. The split is already the house pattern — confirm it, don't invent it

The instinct ("dedicated physics kernels in the physics crate, solver in the CFD crate") is **exactly the
existing three-layer architecture**. Gap 2 follows it; it does not introduce a new one.

| Layer | Where | What it is | Gap-1 example | Gap-2 addition |
|---|---|---|---|---|
| **Kernel** (pure, pointwise) | `deep_causality_physics/src/kernels/<domain>/` | free `fn name_kernel<R>(…) -> Result<Quantity<R>, PhysicsError> where R: RealField` — stateless, no space discretization | `compressible_ns_*` kernels, `convective_acceleration_kernel` | Park-2T relaxation, Arrhenius rates, ionization, 2-T EOS |
| **Effect wrapper** | `…/kernels/<domain>/wrappers.rs` | lifts each kernel to `PropagatingEffect<T>` (via `deep_causality_haft`) for causal composition | `hydrostatic_pressure`, `vorticity_transport` | `ionization`, `arrhenius_rate`, … |
| **Regime evaluator / theory** | `deep_causality_cfd/src/theories/` | reaches the kernels to assemble `*_rhs` + `*_rhs_effect` + `FluidTheory` realizations | `compressible_ns_{continuity,momentum,energy}_rhs` | a reacting-NS `*_rhs` family |
| **Solver / coupling** | `deep_causality_cfd/src/solvers/` + `src/types/flow/` | the marcher (DEC, QTT) and the **`PhysicsStage` between-step coupling** DSL | `QttIncompressible2d`, `QttImmersed2d`, `ThermalRelax`/`ViscosityArrhenius` | a reacting marcher (or surrogate) + `IonizationStage`/`EosStage`/`BlackoutTrigger` |

**Verified facts grounding this:**

- **Kernel convention** (`deep_causality_physics/src/kernels/fluids/`): a kernel is a free function generic
  over `R: RealField` (sometimes `+ FromPrimitive`), taking/returning **typed quantity newtypes** and
  `Result<_, PhysicsError>`, computing *one pointwise relation* and **not discretizing space** — the caller
  supplies divergences/gradients. E.g. `compressible_ns_energy_rhs<R>(rho, u, div_rho_u_e, div_p_u,
  div_tau_dot_u, div_q, body_force)` (`kernels/fluids/compressible.rs`).
- **Quantity newtypes** (`deep_causality_physics/src/quantities/`) enforce invariants (e.g. `Temperature::new`
  rejects `< 0`). Plasma-relevant types **already present** in `quantities/mhd/`: `PlasmaFrequency`,
  `DebyeLength`, `Conductivity`, `Diffusivity`, `LarmorRadius`. **Missing** (to add): `ElectronDensity`,
  `IonizationFraction`, `ElectronTemperature`, `VibrationalTemperature`, `MassFraction`, `ReactionRate`.
- **The coupling seam exists** (`cfd/src/types/flow/coupling.rs`): `PhysicsStage` is a between-step
  transform reading a `StepContext` (manifold, velocity, `dt`) and mutating a `CoupledField` (named scalar
  fields + the per-step `Ambient` the marcher reads). It composes statically by cons-tuple (`()` identity,
  `(Head, Tail)` sequence) — **no `dyn`**. `ThermalRelax` (relax a `"temperature"` field) →
  `ViscosityArrhenius` (write `ν(T)` back to the ambient) is the **working template** an `IonizationStage`
  copies. *Adding a coupled physics is a small `PhysicsStage` impl, not a DSL change.*
- **The thermal seam is already stubbed**: the Gap-1 immersed-body change added a *neutral* wall heat-flux
  via a penalized passive scalar (`QttImmersed2d::advance_scalar`, `wall_heat_flux`). That passive scalar is
  the seam the reacting energy equation **replaces** — `T` becomes the `T_tr`/`T_ve` fields, the
  penalized-scalar transport becomes the two-temperature energy transport.

**Why the split is correct (not just convenient):**

- **Kernels are solver-agnostic and reusable.** The pointwise Park-2T / ionization relations are the same
  whether driven by the DEC solver, the QTT marcher, or a future one — and the existing `mhd/plasma.rs`
  kernels already serve `grmhd/`, not just CFD. Keeping them in `physics` keeps them reusable and lets them
  be **unit-tested against published data** (RAM-C, Park tables) in isolation, free of any discretization.
- **The solver owns discretization + compression + time-marching** — the QTT representation, the rounding,
  the projection. Folding pointwise chemistry into the marcher would couple *physics correctness* to the
  *compression machinery*; that is the wrong seam and would make both untestable.
- **Tier-A/Tier-B swap for free.** The Tier-A parametric ionization surrogate *is* just a fitted kernel; the
  Tier-B validated finite-rate closure is a different kernel with the **same signature**. Swapping fidelity
  never touches the solver or the stage — only which kernel the stage calls.

### 1.1 Two composition idioms — both already demonstrated in the examples

The kernels are pure; what *composes* them is the question, and the codebase already shows **two** seams
Gap 2 uses for two different jobs. Consult both before writing:

- **In-marcher, between-step coupling → the CFD `PhysicsStage` / `Coupling`** (static cons-tuple, mutates a
  `CoupledField`; `coupling.rs`). This is the per-timestep update *inside* the rollout — the
  `ThermalRelax → ViscosityArrhenius` loop is the literal template an `IonizationStage` (`{T_tr, T_ve,
  species} → n_e`) and `EosStage` follow. Used where the chemistry must advance *with* the flow each step.

- **Causal-chain / branch composition → the Causal Monad (`PropagatingEffect`, `CausalFlow`,
  `bind_or_error`)** — haft's effect system, the substrate the regime classifier (step [2]), the
  model-selection coupling (step [3]), and the counterfactual branches (step [5]) ride. Two examples pin it:
  - **`examples/physics_examples/grmhd/`** — the **regime-adaptive model-selection** precedent the corridor
    calls "grmhd-style" (step [3]). `main.rs` is `CausalFlow::value(state).and_then(|s|
    model::calculate_curvature(s).into()).and_then(|s| model::select_metric(s).into())…` — **Step 2,
    `select_metric`, dynamically picks the governing metric from the computed curvature against a
    threshold.** Gap 2's classifier is the *same shape*: compute the Park-2T **ionization fraction**, then a
    `select_regime`/coupling stage picks `continuum-NS → reacting → ionized` (and `GNSS → DENIED`) against a
    plasma-frequency threshold. Each stage returns a `PropagatingEffect`, adapted with `.into()`, and the
    chain short-circuits on error instead of swallowing it.
  - **`examples/physics_examples/multi_physics_pipeline/`** — the **kernel-stage chain** template:
    `klein_gordon(…).bind_or_error(stage_a,…).bind_or_error(stage_b,…)…`, where every stage is a standalone
    physics-kernel wrapper (`klein_gordon`, `heat_diffusion`, `born_probability` — i.e. `deep_causality_physics`
    kernels lifted to `PropagatingEffect`). Its README states the exact Gap-2 lesson outright: *"the Causal
    Monad pattern remains the same — only the stage implementations change"* (its "Path to Production"
    section swaps toy stages for production ones). That **is** the Tier-A-surrogate → Tier-B-validated-closure
    swap, demonstrated: the `ionization → EOS → blackout` chain is authored once; only the kernel behind each
    stage changes with fidelity.

**So the Gap-2 stages have a home in *both* seams, by role:** the `IonizationStage`/`EosStage` run
between-step inside the QTT rollout (`PhysicsStage`); the regime classifier + model selection + the blackout
trigger + the counterfactual bank are `CausalFlow`/`bind_or_error` stages over standalone kernel wrappers
(grmhd `select_metric` + multi_physics chain). Neither idiom is new; Gap 2 is filling in stage
*implementations*, not inventing composition.

### 1.2 Dynamic by construction — **no hardcoded physics** (a hard invariant)

Every 2-T / ionization quantity, and everything related to curvature, is **computed from local state by a
kernel each step — never a fixed value, a fabricated proxy, or a hardcoded schedule.** This is the binding
rule for Gap 2 (and the curvature it feeds), because it is exactly what the current skeletons get wrong and
what makes the flagship's *counterfactuals* meaningful (if `n_e` were a constant, every what-if branch would
return the same flowfield — the branches differ *only because* the physics is recomputed from each branch's
state).

**The anti-pattern to eliminate (verified, in the skeletons):**
- `grmhd/model.rs:59-71` hardcodes the metric (`g_00 = -0.9`, `g_11 = 1.1`) and a **synthetic proxy** Ricci
  tensor / scalar (`ricci = g_uv * -0.1`, `scalar_r = -0.4`). The Einstein tensor is computed by a real
  kernel — but it is fed *fabricated curvature*. That is precisely what must become dynamic.
- `hypersonic_2t/model.rs` has a "simplified for demo" conformal embedding and a `correct()` no-op
  (gap-analysis §4 Gap 3).

**What "dynamic" means, concretely:**
- **Park 2-T:** `T_tr`, `T_ve`, the ionization fraction `α`, `n_e`, `ω_p`, and the reaction rates are each
  the *output of a kernel* applied to the current `(ρ, T_tr, T_ve, {Y_s})` — recomputed every step, per
  branch. The blackout flag is a **computed** `ω_p` compared to the **configured** comms band, not a flip at
  a hardcoded step index. The Park rate-table coefficients (`A`, `n`, `E_a`) are *fitted model parameters*,
  not state — see the constants rule below.
- **Curvature (the grmhd-style coupling it feeds):** the metric `g_uv` is computed from the physical state —
  `g_00 = −(1 − 2GM/rc²)` from the actual `GM`/`r` (and the SR `γ(v)` for the timing causaloid) via
  `generate_schwarzschild_metric` — and the **Ricci/curvature from the metric** (the field-equation /
  energy-momentum route, the real `einstein_tensor` inputs), **not** `−0.1·g` / `−0.4`. The regime threshold
  the `select_metric`/classifier compares against is *config*; the curvature it compares *is computed*.

**The one allowed literal — and the bright line.** Genuine **constants of nature** (`k_B`, `e`, `ε₀`,
`m_e`, `N_A`) and **published model coefficients** (Park reaction `A/n/E_a`, Millikan–White τ fits, EGM/IERS
gravity terms) live as `pub const … : f64` under `deep_causality_physics/src/constants/` and lift into the
working precision via `R::from_f64(…)` at the call site — they are *measured invariants and fitted
parameters*, not runtime state. The bright line: **a constant of nature or a cited model coefficient may be
a literal in `constants/`; a temperature, density, fraction, frequency, metric component, or curvature value
may not** — it is computed from state by a kernel, or it is a `config` input the caller supplies. No magic
numbers inside kernels or stages; no `g_00 = -0.9`; no `α = 0.3`; no "blackout at step 200".

This invariant is **testable**: a Gap-2 kernel fed two different states must return two different outputs
(no constant-folding to a fixed answer), and a counterfactual branch with a perturbed seed must produce a
perturbed `n_e`/blackout result — the regression guard against a proxy creeping back in.

---

## 2. The physics to follow (SOTA, from gap-analysis §2 Axis 2)

- **Park two-temperature model** — the standard hypersonic thermochemical-nonequilibrium reference: separate
  translational–rotational `T_tr` and vibrational–electron `T_ve`; dissociation/ionization **lag** the flow,
  and the lagging electron density is what raises the plasma frequency above the comms band.
- **RAM-C-II air-ionization simulation**, *Fluid Dynamics* (Springer, 2022) — Park-2T electron density vs.
  the canonical flight data; the modern, citable companion to the RAM-C anchor.
- **Vibrational–electron heating, thermodynamically consistent** — arXiv:2506.11457 (2025); **ion mobility's
  impact on electron density/temperature** — arXiv:2410.12760 (2024). Current `T_ve` refinements.
- **Data-driven lookup-table reduction for hypersonic chemical nonequilibrium** — arXiv:2210.04269. The
  surrogate-table route Tier A explicitly permits.
- **MPS simulation of reacting shear flows** — Pinkston et al., arXiv:2512.13661 (2025): species +
  Arrhenius sources carried in MPS form via TT-cross — the precedent for riding the reacting source terms on
  the *same* QTT rollout (the tie-in to Gap 1).

---

## 3. What must be built, and where

### 3.1 Physics crate — the kernels (`deep_causality_physics`)

**New quantity newtypes** (`src/quantities/` — likely a new `quantities/plasma/` or extend `mhd`/`thermodynamics`):
`ElectronDensity` (m⁻³, ≥0), `IonizationFraction` (0–1), `ElectronTemperature` / `VibrationalTemperature`
(K, ≥0), `MassFraction` (0–1 per species), `ReactionRate` (1/s or mol·m⁻³·s⁻¹). Reuse the existing
`PlasmaFrequency`, `DebyeLength`, `Conductivity`. **[holds: mechanical — follows the `Temperature` newtype pattern]**

**New kernels** — a dedicated domain `src/kernels/hypersonic/` (Park-2T reentry physics), reusing
`mhd/plasma.rs` for the plasma-frequency/blackout closure:

| Kernel | Relation | Honesty |
|---|---|---|
| `vibrational_relaxation_kernel` | Landau–Teller `dT_ve/dt = (T_tr − T_ve)/τ_vt` (Millikan–White τ) | [holds: textbook] |
| `arrhenius_rate_kernel` | `k(T) = A·T^n·exp(−E_a/(k_B T))`, forward/backward | [holds: textbook] |
| `species_source_kernel` | net production `ω̇_s` from the rate set (dissociation, exchange, ionization) | [holds under precondition: reaction set chosen] |
| `ionization_fraction_kernel` | Saha (equilibrium) **or** rate-based (nonequilibrium) `α` → `ElectronDensity` | [holds; nonequilibrium is the physical one] |
| `plasma_frequency_kernel` | `ω_p = √(n_e e²/(ε₀ m_e))` → compare to comms band | [holds — `PlasmaFrequency` newtype + `mhd/plasma.rs` exist] |
| `two_temperature_eos_kernel` | `p(ρ, T_tr, T_ve, {Y_s})` mixture closure | [holds under precondition: mixture model] |
| **Tier-A:** `park2t_ionization_surrogate_kernel` | a *fitted* `α(ρ, T, …)` (the lookup-table-reduction route) | [holds: Tier-A escape hatch] |

Each gets a `PropagatingEffect` wrapper in `kernels/hypersonic/wrappers.rs`; register in
`kernels/hypersonic/mod.rs` and flatten at `lib.rs` (the established `pub use crate::kernels::<domain>::*`).
**These kernels do not discretize space and hold no state** — pure pointwise relations, individually
testable against Park tables / RAM-C.

### 3.2 CFD crate — the solver + coupling (`deep_causality_cfd`)

- **Regime evaluator** (`theories/`): a reacting-NS `*_rhs` family (species transport + the two-temperature
  energy split), reaching the §3.1 kernels — the `compressible_ns_*_rhs` pattern extended with chemistry.
  **[holds under precondition: compressible marcher — see §4]**
- **Coupling stages — placed by role across the two idioms of §1.1:**
  - **Between-step `PhysicsStage`** (`types/flow/coupling.rs`, copying `ThermalRelax`/`ViscosityArrhenius`):
    `IonizationStage` (reads the `T_tr`/`T_ve`/species scalar fields from `CoupledField`, calls the
    ionization kernel, writes back `n_e`) and `EosStage` (the two-temperature pressure closure into the
    ambient). These advance *with* the flow each step; `Coupling::between_steps().then(IonizationStage)
    .then(EosStage)` composes them statically, exactly like the existing thermal loop.
  - **`CausalFlow` / `bind_or_error` stages** (the grmhd `select_metric` shape): the **regime classifier**
    (Knudsen + ionization fraction + GNSS state → governing-model selection, corridor step [3]) and the
    **`BlackoutTrigger`** (`n_e → plasma_frequency_kernel →` comms-band compare → GNSS-denied flag) — and
    the step-[5] counterfactual bank chains. These are per-branch / per-decision, not per-cell.
  **[holds: both seams exist and are demonstrated — §1.1]**
- **The reacting marcher**: drive the §3.1 source terms on the **QTT rollout** (Pinkston et al.: species +
  Arrhenius via TT-cross at controlled rank), reusing the `QttImmersed2d::advance_scalar` thermal seam for
  the `T_tr`/`T_ve` transport. **Precondition: this needs a *compressible* QTT marcher** (the built
  `QttIncompressible2d` is the wrong physics for a hypersonic shock — see §4). **[open: compressible QTT]**
- **The blackout observable** rides the existing QTT observe set (`n_e`, plasma frequency, blackout dwell)
  alongside drag/heat flux.

---

## 4. The hard precondition: a compressible QTT marcher (the shared Gap-2 / Tier-B wall)

Gap 2's physics is *compressible* and *shock-bearing*; the built QTT solver is incompressible. The reacting
kernels are pointwise and solver-agnostic (they will unit-test against RAM-C the day they are written), but
**marching them on a real reentry flowfield needs a compressible QTT marcher** — density/energy transport,
an EOS pressure closure, and **shock-capturing**. As established in the
[corridor note §6](plasma-blackout-corridor.md) and [gap-analysis §4 Gap 2](gap-analysis.md), the Mach-25
shock is near-discontinuous → **high tensor-train rank** (verified with the immersed-body mask: sharp → high
bond, smoothed → bounded), so shock-capturing in QTT needs artificial viscosity / shock smoothing
(physically honest — the true shock is a few mean free paths thick), TT-cross for the nonlinear/source terms,
and aggressive rounding. The micrometer shock/sheath resolution rides QTT's multi-resolution property
(uniform-fine grid, rank-localized cost), *not* an adaptive mesh. **[open: compressible + shock-rank
control — Tier-B]**

**Tier-A escape hatch (corridor §7):** skip the compressible shock entirely — a **parametric Park-2T
ionization surrogate** (`park2t_ionization_surrogate_kernel`, §3.1) over a quasi-1D/reduced flow gives
`n_e → plasma frequency → blackout trigger` *without* a validated reacting CFD solve. This is a kernel + an
`IonizationStage` + the trigger — **buildable now on the incompressible/reduced rollout**, and it is the
honest deliverable for the flagship's first vertical slice. **[holds under precondition: surrogate
acceptable for Tier A]**

---

## 5. Validation anchors

- **RAM-C II flight (NASA Langley, 1970)** — the canonical ionized-reentry **electron-density / blackout**
  dataset; the reference for the ionization kernel and blackout-onset timing (cross-check against the
  *Fluid Dynamics* 2022 Park-2T reproduction).
- **Apollo reentry blackout durations** — public; a sanity check on blackout dwell.
- **Park two-temperature model** — the standard `T_tr`/`T_ve` reference; validate the relaxation + rate
  kernels against its tables.
- **Saha equation** — the equilibrium-ionization limit the rate-based kernel must recover as `τ → 0`.

The kernels validate **in `deep_causality_physics`, in isolation** (pointwise vs. published values), before
any solver integration — the payoff of the split.

---

## 6. Staged plan (Tier-A first)

1. **Quantity newtypes** (`physics/quantities/`) — `ElectronDensity`, `IonizationFraction`,
   `Electron`/`VibrationalTemperature`, `MassFraction`, `ReactionRate`. Cheap, mechanical.
2. **Pointwise kernels** (`physics/kernels/hypersonic/`) — vibrational relaxation, Arrhenius rate, the
   ionization/Saha kernel, plasma frequency (reuse `mhd`), 2-T EOS, and the **Tier-A surrogate** — each with
   a `PropagatingEffect` wrapper, **unit-tested against Park / RAM-C**.
3. **Tier-A coupling** (`cfd/types/flow/`) — `IonizationStage` + `BlackoutTrigger` (+ `EosStage`) driving the
   kernels over the temperature/species scalar fields of the existing QTT rollout (reuse
   `advance_scalar`/`wall_heat_flux`); emit `n_e` / plasma-frequency / blackout-dwell observables. **This is
   the smallest honest slice that makes the regime change physically real.**
4. **Reacting `*_rhs`** (`cfd/theories/`) — species transport + the two-temperature energy split, for the
   verification solvers.
5. **[Tier-B] Compressible QTT marcher** — density/energy + EOS + shock-capturing (§4); ride the reacting
   sources on it via TT-cross (Pinkston et al.).

Steps 1–3 are buildable now and unblock the flagship's steps [2]/[3]; step 5 is the genuine open research.

**Gate on every step:** the **dynamic-by-construction invariant (§1.2)** — each kernel's output is a
function of the state it is given (verified by the two-states-two-outputs test), no fabricated proxies, no
hardcoded schedules; the only literals are constants of nature / cited coefficients in `constants/`.

---

## 7. Sources

- Park, C. — *Nonequilibrium Hypersonic Aerothermodynamics* (the 2-T model).
- RAM-C-II air ionization — *Fluid Dynamics* **57** (Springer, 2022).
- Vibrational–electron heating — arXiv:2506.11457 (2025); ion mobility — arXiv:2410.12760 (2024).
- Lookup-table reduction (surrogate route) — arXiv:2210.04269.
- Reacting MPS (the QTT tie-in) — Pinkston et al., arXiv:2512.13661 (2025).

---

## 8. Related

- [`gap-analysis.md`](gap-analysis.md) §4 Gap 2 — the gap this note drills into.
- [`gap-one-cfd-tensor-bridge.md`](gap-one-cfd-tensor-bridge.md) — the **closed** Gap 1 this builds on; its
  §3.4 neutral wall heat-flux is the thermal seam Gap 2 replaces.
- [`../plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) §3.2 (Park-2T regime driver), §6
  (shock-rank / compressible-solver seam), §7 (Tier-A surrogate).
- `deep_causality_physics` `kernels/` + `quantities/` — the kernel/newtype conventions §1 documents.
- `deep_causality_cfd` `types/flow/coupling.rs` — the `PhysicsStage` seam the in-marcher Gap-2 stages plug into.
- `examples/physics_examples/grmhd/` — the **regime-adaptive model-selection** coupling (`select_metric` keyed
  on curvature) the corridor calls "grmhd-style"; the precedent for the Gap-2 classifier keyed on ionization.
- `examples/physics_examples/multi_physics_pipeline/` — the **Causal Monad kernel-stage chain**
  (`bind_or_error` over standalone physics-kernel wrappers); its "only the stage implementations change"
  lesson **is** the Tier-A-surrogate → Tier-B-closure swap.
