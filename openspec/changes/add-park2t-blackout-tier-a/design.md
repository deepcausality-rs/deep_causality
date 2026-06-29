<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Context

Gap 1 of the plasma-blackout corridor is closed: `deep_causality_cfd` has a verified QTT incompressible
flowfield (`QttIncompressible2d`/`QttImmersed2d`) with a Brinkman-penalized immersed body, a passive scalar
(`advance_scalar`), and surface observables. The regime driver — Park-2T ionization → electron density →
plasma frequency → blackout — does not exist. This change adds the **Tier-A** slice: the pointwise Park-2T
kernels in `deep_causality_physics` and the blackout coupling in `deep_causality_cfd`, riding the existing
**incompressible** rollout. Tier-B (a compressible shock-capturing QTT marcher) is open research and out of
scope.

The design rests on the **Lagging-Equilibrium Relaxation (LER)** mechanism
([gap-2 §1.4](../../notes/plasma-blackout/gap-2/gap-two-reacting-plasma.md) + the three resolution notes),
which resolves the three contradictions that otherwise make Tier-A unbuildable (stiff chemistry on an
explicit stage; a driving temperature that cannot emerge from an incompressible flow; nonequilibrium lag
without a stiff network).

**Verified existing API this change builds on** (confirmed against source):

- **Physics kernel convention** — free `fn name_kernel<R>(…) -> Result<Quantity<R>, PhysicsError> where R:
  RealField` (`+ FromPrimitive` when constants are lifted); `RealField`/`FromPrimitive` from
  `deep_causality_num`. Quantity newtypes validate in `new` (`Temperature::new` → `PhysicsError::
  ZeroKelvinViolation`). `PhysicsError` variants include `PhysicalInvariantBroken(String)`,
  `ZeroKelvinViolation`, `NormalizationError(String)`, `Singularity(String)`.
- **Constants** — `pub const … : f64` under `src/constants/` (`BOLTZMANN_CONSTANT`, `ELEMENTARY_CHARGE`,
  `VACUUM_ELECTRIC_PERMITTIVITY`, `ELECTRON_MASS` all exist), lifted via `R::from_f64(…)` at the call site.
- **Effect wrappers** — `pub fn name<R>(…) -> PropagatingEffect<T>` wrapping `match kernel(…) { Ok(v) =>
  PropagatingEffect::pure(v), Err(e) => PropagatingEffect::from_error(CausalityError::from(e)) }`.
  `PropagatingEffect<T>` and `CausalityError` are in **`deep_causality_core`**.
- **`lib.rs`** glob-flattens each kernel domain: `pub use crate::kernels::<domain>::*;`.
- **MHD quantities present** — `PlasmaFrequency`, `DebyeLength`, `Conductivity`, `Diffusivity`,
  `LarmorRadius`. **`mhd/plasma.rs` has `debye_length_kernel`, `larmor_radius_kernel` but NO
  plasma-frequency kernel.** The six new newtypes (`ElectronDensity`, …) do not exist.
- **CFD coupling** — `PhysicsStage<const D: usize, R: CfdScalar>::apply(&self, ctx: &StepContext<D,R>, field:
  &mut CoupledField<R>) -> Result<(), PhysicsError>`; static cons-tuple composition (`()`, `(A,B)`), no
  `dyn`; `Coupling::between_steps().then(…).build()`. `StepContext` exposes `manifold/velocity/dt/step`.
  `CoupledField` carries `Ambient` + named scalars (`scalar`/`scalar_mut`/`set_scalar`). **`ThermalRelax`
  currently uses explicit Euler** `T += rate·dt·(wall − T)` — the LER exponential is genuinely new.
- **Scalar bound** — CFD side is `CfdScalar` (`RealField + FromPrimitive + Default + … + MaybeParallel`); TT
  ops additionally require `ConjugateScalar<Real = R>`. Physics kernels bind bare `RealField`.
- **QTT solver** — `QttImmersed2d::advance_scalar(temp, u, v, t_wall, kappa) -> Result<CausalTensorTrain
  <R>>`; standalone `observe::wall_heat_flux(...)`. `QttObserve` builder flags: `{kinetic_energy,
  divergence, max_speed, bond, drag}` — **no heat-flux/n_e observable in the set yet**. `Report` carries
  `series: Vec<(String, Vec<R>)>` via `add_series`.
- **CausalFlow** — `CausalFlow::value(v).and_then(|s| wrapper(s).into())` (grmhd) or
  `.bind_or_error(stage_fn, "msg")` (multi-physics, `fn(Value, State, Option<Context>) ->
  PropagatingEffect<_>`, short-circuits on error). All in `deep_causality_core`.

## Goals / Non-Goals

**Goals:**

- Park-2T pointwise kernels + quantity newtypes in `deep_causality_physics`, validated **in isolation**
  against Park tables / RAM-C / the Saha limit before any solver use.
- The LER stage: a stiff source integrated by a **closed-form exponential / linearly-implicit** update inside
  a between-step `PhysicsStage`; the kernel returns the **increment over `Δt`**, not a rate.
- A **state-derived** `T_tr` (recovery-temperature reconstruction off the incompressible velocity field +
  mandatory Rankine–Hugoniot jump) — no prescribed schedule.
- Nonequilibrium ionization **lag** carried by one relaxing scalar with `τ_ion` grounded in the dominant
  associative-ionization rate; Saha recovered as `τ → 0`.
- `BlackoutTrigger` on the `CausalFlow` seam; blackout observables (`n_e`, plasma frequency, dwell).
- A self-verifying `verification/qtt_park2t_blackout/` example gating on the six LER criteria + published
  reference cross-references with Tier-A disclaimers.

**Non-Goals (deferred to Tier-B, explicitly out of scope):**

- A compressible QTT shock-capturing marcher; density/energy transport; the reacting `*_rhs` family on a
  compressible solver.
- Multi-mode relaxation spectra (single dominant `τ` only); shock-rank control / artificial viscosity.
- A full multi-species finite-rate reaction network (the Tier-A surrogate + one dominant ionization channel
  is the scope).
- Modifying the QTT marcher core or the `PhysicsStage`/`Coupling`/`CausalFlow` substrate (reused unchanged).

> **Tier-A does not depend on the Tier-B marcher, and Tier-B reuses Tier-A.** This change rides the
> *incompressible* rollout, where the field stays low-rank (measured: `qtt_rank_dynamic` shows linear marching
> is rank-safe), so nothing in the sibling Tier-B change gates it. Conversely, the Tier-B change
> [`add-cfd-compressible-qtt-marcher`](../add-cfd-compressible-qtt-marcher/proposal.md) reuses **every kernel,
> newtype, and LER stage built here, unchanged** — only the marcher underneath them changes. The Tier-B
> shock-rank obstacle is now *measured* (captured 3-D curved shock `χ ~ √side` vs `χ ~ O(10)` body-fitted), so
> a **shock-aligned coordinate + an implicit/IMEX step are mandatory** there — see
> [`gap-2/tier-b-compressible-marcher.md`](../../notes/plasma-blackout/gap-2/tier-b-compressible-marcher.md)
> and `deep_causality_cfd/studies/`. The one Tier-A-specific piece Tier-B retires is the recovery-temperature
> reconstruction (D3): a compressible marcher makes `T_tr` a real transported state. Recorded here so the
> Tier-A/Tier-B boundary stays explicit.

## Decisions

### D1 — Kernels in `deep_causality_physics`, coupling in `deep_causality_cfd` (the house 3-layer split)

Follow the existing pattern verbatim: pure pointwise kernels (no space discretization, no state) in a new
`kernels/hypersonic/` domain with a `wrappers.rs`; the between-step `PhysicsStage`s and the `CausalFlow`
stages in `deep_causality_cfd/src/types/flow/`. *Alternative considered:* fold chemistry into the marcher —
rejected, it couples physics correctness to the compression machinery and makes both untestable (gap-2 §1).

### D2 — The LER kernel returns the integrated increment, not the rate

A relaxation kernel computes `x_eq − (x_eq − x)·exp(−Δt/τ)` (linear) or a linearly-implicit one-step form
(nonlinear), taking `Δt` (from `StepContext::dt`). This is **unconditionally stable**, exact on the linear
case, pure, and pointwise — no global solve. *Alternatives:* explicit Euler (the current `ThermalRelax`
form) — rejected, unstable under ns-scale stiffness; a global implicit Newton solve — rejected, destroys the
pointwise kernel split and couples to the marcher. The closed-form exponential is the textbook stable
treatment of a relaxation operator. The split is first-order Lie; **Strang** (half-source/transport/half-
source) is the documented upgrade if onset timing needs second order.

### D3 — `T_tr` is a recovery-temperature reconstruction, not a prescribed field

`T_tr(x) = T_post − ½|u(x)|²/c_p`, with `T_post` from a **Rankine–Hugoniot normal-shock jump** off the
config flight Mach. Structure comes from the computed velocity field; only `M_∞`, `T_∞` are config. *Why the
RH jump is mandatory:* isentropic recovery alone tops out far below ionization temperatures; without the
shock jump the slice silently produces no plasma. *Alternative:* solve a compressible energy equation —
that is Tier-B and unbuilt; *alternative:* prescribe `T(x,t)` — rejected, violates the dynamic invariant.
Labeled honestly as a Tier-A reconstruction, not a true post-shock thermodynamic path.

### D4 — The surrogate is the relaxation *target*, not the answer

`IonizationStage` relaxes `α` (carried scalar) toward `α_eq(ρ, T_tr)` (the cheap Saha/fitted surrogate) with
`τ_ion`. The transient gap `α ≠ α_eq` **is** the nonequilibrium lag — one scalar of memory, not a reaction
network. `τ_ion` is computed from the dominant associative-ionization reaction rate (N + O → NO⁺ + e⁻),
`τ_ion ≈ 1/(k_f(T)·[M])`, **not** a free fit. *Alternatives:* memoryless algebraic `α(ρ,T)` — rejected,
loses the lag (the whole regime driver); full finite-rate network — rejected, reintroduces stiffness and is
Tier-B. As `τ_ion → 0` the stage recovers Saha (a verification gate).

### D5 — Blackout observables are new; the QTT marcher core is untouched

`QttObserve` gains opt-in flags for `n_e` / plasma frequency / blackout dwell, emitted as `Report` series via
the existing `add_series` path. The marcher, projector, and `advance_scalar` are reused unchanged; the
ionization/temperature scalars ride `advance_scalar` exactly like the Gap-1 passive scalar. `wall_heat_flux`
already exists as a standalone diagnostic and is reused.

### D6 — Validation is pointwise-first, then a self-verifying example

Kernels validate in `deep_causality_physics` against published values (Park tables, RAM-C point, Saha limit)
**before** any solver integration — the payoff of the split. The `verification/qtt_park2t_blackout/` example
(`[[example]]` target, config/main/print_utils/baseline/README) then gates the six LER criteria end-to-end
and reports reference cross-references with Tier-A disclaimers — mirroring `qtt_cylinder_verification`.

### D7 — Newtype placement and constants

The six new newtypes extend `quantities/` (a new `quantities/plasma/` module, or extend `quantities/mhd/` —
chosen at implementation time to match the existing module granularity), validating via
`PhysicalInvariantBroken` (density/rate/temperature ≥ 0) and `NormalizationError` (fraction in `[0,1]`). New
Park `A/n/E_a` and Millikan–White `τ` coefficients go in a new `constants/hypersonic.rs` as `pub const … :
f64`, lifted via `R::from_f64`. `k_B`, `e`, `ε₀`, `m_e` are reused from the existing constants modules.

## Risks / Trade-offs

- **[Risk] RH-jump magnitude is the make-or-break link.** Omitting it yields a too-cold field and `n_e ≡ 0`
  → the slice looks like it "runs" but proves nothing. **Mitigation:** a dedicated gate asserts peak `T_post`
  lands in the ~10⁴ K band at `M ≈ 25`; the verification fails loudly otherwise.
- **[Risk] `τ_ion` becomes a fudge factor.** A free-fit `τ_ion` makes the lag unphysical and silently breaks
  the dynamic invariant. **Mitigation:** `τ_ion` is computed from the dominant-rate Arrhenius coefficient; a
  gate checks it varies with `T` (not constant) and that the `τ → 0` limit recovers Saha.
- **[Risk] Incompressible flow is the wrong carrier physics.** The reconstruction captures stagnation/
  recovery heating, not the true post-shock path. **Mitigation:** scope is explicitly Tier-A; every
  reconstructed quantity is labeled; no absolute coupled-CFD match is claimed — RAM-C/Apollo are reported as
  cross-references with tolerances.
- **[Trade-off] First-order Lie split.** Cheaper but lower-order in `Δt`; onset *timing* may be soft.
  **Mitigation:** Strang noted as the upgrade; the verification reports onset with its split-order caveat.
- **[Risk] `ConjugateScalar`/`CfdScalar` bound creep.** TT contractions need `ConjugateScalar<Real = R>`.
  **Mitigation:** CFD-side stages bind `CfdScalar + ConjugateScalar<Real = R>` exactly as the existing QTT
  observe functions do; physics kernels stay on bare `RealField`.

## Migration Plan

Additive only — no existing public API changes. Stage the work Tier-A-first
([gap-2 §6](../../notes/plasma-blackout/gap-2/gap-two-reacting-plasma.md)): (1) newtypes + constants;
(2) pointwise kernels + wrappers, validated in isolation; (3) the LER stages + `BlackoutTrigger` +
observables over the existing rollout; (4) the self-verifying example. No rollback concern (new modules,
new `[[example]]`); if a gate cannot be met, the example fails and the slice is not declared closed. On
success, mark Gap 2 **Tier-A closed** in the notes, Tier-B still open.

## Open Questions

- **Newtype home** — `quantities/plasma/` (new) vs. extending `quantities/mhd/`? Resolve by matching the
  existing module granularity at implementation time; does not affect the public flat re-export.
- **Reaction channel set** — Tier-A commits to the dominant associative-ionization channel (N + O → NO⁺ +
  e⁻) for `τ_ion` and a Saha/fitted `α_eq` that includes NO⁺ + e⁻ so `α_eq > 0`. Is one channel enough to
  reproduce the RAM-C onset within the chosen tolerance, or is a 2–3 channel `α_eq` needed? Decide against
  the RAM-C reference during step 2; either way it is one `α_eq` evaluation, not a marched network.
- **`c_p` provenance** — the recovery reconstruction needs a mixture `c_p`. Tier-A uses a config/cited
  frozen-mixture `c_p`; a composition-dependent `c_p(Y_s)` is a Tier-B refinement.
