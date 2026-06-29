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
- A separate electron-translational temperature (the **3T** model, `T_e ≠ T_ve`). Tier-A lumps `T_e = T_ve`
  (Park 2T). The 3T upgrade that Farbar–Boyd–Martin (2013) and Clarey–Greendyke (2019) show matters most in
  the recombination-dominated wake is **LER-native** — it is one additional relaxing `T_e` scalar with its own
  `τ_e` — so it is deferred as a clean add-on, not a redesign. This is the named remedy for the ~2× peak-`n_e`
  over-prediction (Risks).
- Modifying the QTT marcher's **solver math** (the spectral-projection / Brinkman `advance`) — that is reused
  unchanged. **In scope (D5/D8):** generalizing the `PhysicsStage` seam over a `FlowSnapshot` read-view and
  giving `QttMarchRun` a between-step coupling host (the original "QTT core untouched" non-goal was the root
  cause of the missing coupling and is revised).

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

*Literature framing.* The LER stage is an instance of the **macro–micro moment-reduction** pattern — keep the
conserved low-order flow state exact and carry a small compressed correction that relaxes toward equilibrium —
formalized for moment systems by Koellermeier–Krah–Kusch (DLRA/POD on the hyperbolic shallow-water *moment*
equations, 2023), Issan et al. (POD on a Hermite moment hierarchy with a learned closure, 2025), and
Peng–McClarren–Frank (low-rank on the angular `P_N` moments, 2020). Tier-A's carried `α`/`T_ve` scalars are
the "micro" correction; this is the recognized closure structure LER specializes, not an ad-hoc integration
trick.

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

*Regime grounding (Aiken–Carter–Boyd 2025 review).* Ionization is associative-dominated below ~7 km/s and
electron-impact-dominated above ~9 km/s; RAM-C / orbital reentry (~7.6 km/s) sits in the **mixed band**. So
`τ_ion` is grounded in the associative channel (the slower, *rate-limiting* onset timescale — correct), while
the equilibrium **target `α_eq` carries electron-impact-produced electrons as well as NO⁺**, since at ~7.6 km/s
they are a non-negligible fraction of the equilibrium `n_e`. The exact `α_eq` split is calibrated against the
RAM-C reference at step 2; it remains one `α_eq` evaluation, not a marched second channel.

### D5 — The QTT marcher gains the between-step coupling seam (revised)

*Superseded the original "QTT marcher core untouched" decision after a first-principles review.* The QTT
marcher (`QttMarchRun`) marches `(u, v)` tensor trains and samples diagnostics; it had **no** `PhysicsStage`
seam (that lived only in the DEC `MarchRun`). The original D5 assumed the reacting scalars could ride
`advance_scalar` and that `QttObserve` could emit `n_e` "unchanged" — but neither was wired, so the LER stages
had no host on the QTT rollout. Driving the coupling from the verification example would be a workaround
(physics in a test harness), so the seam is built into the engine instead.

**Root cause:** the coupling seam was nailed to DEC types — `StepContext` over-exposed a DEC `Manifold` /
`SolenoidalField`. A between-step coupling is a functor on the marcher's *auxiliary* state and must communicate
with the primary solver through only (i) a **read-view** of the primary state and (ii) the auxiliary
**field-bag** (`CoupledField` scalars + `Ambient`). The DEC `manifold()/velocity()` accessors are an
over-coupling that no shipped stage uses.

**Resolution (D8 details the mechanism):** narrow the seam to its essence and host it in *both* marchers:

- `StepContext` becomes the **universal read-view** via a **backing sum type** (`Dec { manifold, velocity }` |
  `Qtt {}`): `dt()` / `step()` are universal; `manifold()` / `velocity()` / `sample_velocity()` are
  DEC-only and fallible (`None` / `Err` under the `Qtt` backing — semantically correct, a manifold-sampling
  stage cannot run without a manifold). `PhysicsStage<const D, R>` and **every stage impl are unchanged**; the
  QTT marcher simply constructs a `Qtt`-backed `StepContext`. (A `FlowSnapshot` *trait* with `PhysicsStage`
  generic over it was the first sketch; it forces higher-ranked `for<'a>` bounds across every config/marcher
  site because `StepContext` is a per-step borrow — the sum type realizes the same read-view abstraction with
  no churn to the stage trait, and is honest about there being exactly two marcher backings.)
- Primary-state projections (e.g. `"speed"`) flow through **published `CoupledField` scalars** — the marcher
  publishes what its coupling needs (DEC by sampling; QTT by dequantizing its TT state). The blackboard
  principle: communicate only through the field-bag.
- `QttMarchRun` hosts the coupling: each step it publishes projections, **transports the auxiliary scalar
  fields with `advance_scalar`** (so `T_tr`/species advect and stay tensor trains — true to the QTT
  compression thesis), runs `coupling.apply`, reads back `Ambient`, and samples the blackout observables
  (`n_e`, plasma frequency, dwell) into the `Report` via `add_series`.
- The LER scalars are tensor trains; at the LER seam they are dequantized to `Vec<R>`, updated pointwise, and
  re-`round`ed (exact, re-compresses). TT-cross (`apply_nonlinear`) is the Tier-B upgrade for large `L`.

The **solver math (advance / spectral projection) is untouched**; only the march loop's state threading and the
config change. The proof the cut is right: the three LER stages built against the narrowed seam port to the QTT
host **unchanged** (they read only `dt` + named scalars). `wall_heat_flux` remains the reused neutral thermal
diagnostic.

### D8 — One coupling seam, two marcher hosts (the `StepContext` backing sum type)

`StepContext` carries a backing `enum { Dec { manifold, velocity }, Qtt {} }` and keeps `PhysicsStage<const D,
R>` exactly as-is. `MarchRun` builds a `Dec`-backed context; `QttMarchRun` builds a `Qtt`-backed one — the same
stage value runs under both because it reads only the universal `dt()`/`step()` plus published `CoupledField`
scalars. `QttMarchConfig` carries an optional coupling (`Coupling<S>`) and the initial reacting scalar fields;
the QTT solver reads `Ambient` per step (ν / freestream) so the `ν(T)` channel is real rather than
construction-fixed. *Alternative — `PhysicsStage` generic over a `FlowSnapshot` trait:* rejected — it forces
higher-ranked `for<'a> PhysicsStage<…, StepContext<'a, …>>` bounds at every config/marcher site (the context is
a per-step borrow) and adds a type parameter to every stage impl, for openness that no third marcher needs; the
sum type is the honest, churn-free realization. *Alternative — drive the coupling from the example via the run
hook:* rejected (a workaround that puts physics in the test harness, not the engine).

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
- **[Known limitation, quantified] Two-temperature (`T_ve = T_e`) lumping over-predicts peak `n_e`.**
  Farbar–Boyd–Martin (2013) show that adding a separate electron-translational energy equation cuts peak
  plasma density by **~2×** and improves RAM-C agreement; Tier-A's single `T_ve` therefore over-predicts the
  very quantity blackout depends on. **Mitigation:** the RAM-C electron-density gate tolerance is set wide
  enough to absorb the ~2× lumping bias (alongside the reconstruction and rate-set sensitivities), and the
  bias is named explicitly in the verification README — not silently folded into a hand-tuned tolerance. The
  3T remedy is LER-native (a second relaxing `T_e` scalar) and is recorded as a Non-Goal upgrade, not a
  redesign.

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
- **Reaction channel set (literature-resolved direction).** Tier-A grounds `τ_ion` in the dominant
  associative-ionization channel (N + O → NO⁺ + e⁻). The Aiken–Carter–Boyd 2025 review places RAM-C
  (~7.6 km/s) in the **mixed associative + electron-impact band**, so the equilibrium target `α_eq` should
  carry electron-impact electrons too (not only NO⁺) to reach the RAM-C onset; the associative channel remains
  the rate-limiting `τ_ion`. The open part is only the precise `α_eq` split — decided against the RAM-C
  reference during step 2 — and it stays one `α_eq` evaluation, not a marched network. A fully resolved
  multi-channel electron-impact treatment is Tier-B+.
- **`c_p` provenance** — the recovery reconstruction needs a mixture `c_p`. Tier-A uses a config/cited
  frozen-mixture `c_p`; a composition-dependent `c_p(Y_s)` is a Tier-B refinement.
