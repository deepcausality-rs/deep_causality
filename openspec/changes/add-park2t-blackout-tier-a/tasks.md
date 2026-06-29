<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## 1. Quantity newtypes + constants (`deep_causality_physics`)

- [x] 1.1 Add newtypes `ElectronDensity`, `IonizationFraction`, `ElectronTemperature`,
  `VibrationalTemperature`, `MassFraction`, `ReactionRate` (generic over `R: RealField`) under
  `src/quantities/` (new `quantities/plasma/` or extend `quantities/mhd/` to match module granularity —
  design D7), each with a validating `new` returning `Result<Self, PhysicsError>`: density/rate/temperature
  reject `< 0` (`PhysicalInvariantBroken` / `ZeroKelvinViolation`), fractions reject outside `[0,1]`
  (`NormalizationError`). Follow the `Temperature::new` pattern. Reuse `PlasmaFrequency`, `DebyeLength`.
- [x] 1.2 Register the newtypes in their `quantities/<module>/mod.rs` and flat-export via `lib.rs`.
- [x] 1.3 Add `src/constants/hypersonic.rs` with the Park reaction `A/n/E_a` coefficients (incl. the dominant
  associative-ionization channel N + O → NO⁺ + e⁻) and Millikan–White `τ_vt` fit coefficients, as
  `pub const … : f64` with cited comments; register in `constants/mod.rs`. Reuse the existing
  `BOLTZMANN_CONSTANT`, `ELEMENTARY_CHARGE`, `VACUUM_ELECTRIC_PERMITTIVITY`, `ELECTRON_MASS`.
- [x] 1.4 Tests (`tests/` mirror, f64): each constructor accepts valid input and rejects each out-of-range
  case with the correct `PhysicsError` variant; round-trip accessor equals input. Register in `mod.rs` +
  `tests/BUILD.bazel`.

## 2. Pointwise Park-2T kernels + wrappers (`deep_causality_physics/src/kernels/hypersonic/`)

- [x] 2.1 Create the `kernels/hypersonic/` domain (`mod.rs`, `wrappers.rs`) and flatten at `lib.rs`
  (`pub use crate::kernels::hypersonic::*;`).
- [x] 2.2 `vibrational_relaxation_kernel<R>` — Landau–Teller increment toward `T_tr` with Millikan–White
  `τ_vt` (the LER closed-form: `T_ve' = T_tr − (T_tr − T_ve)·exp(−Δt/τ_vt)`); pure `fn<R: RealField +
  FromPrimitive>(…) -> Result<VibrationalTemperature<R>, PhysicsError>`. Constants via `R::from_f64`.
- [x] 2.3 `arrhenius_rate_kernel<R>` — `k(T) = A·T^n·exp(−E_a/(k_B T))`, forward/backward → `ReactionRate`.
- [x] 2.4 `ionization_fraction_kernel<R>` — Saha-equilibrium and rate-based forms → `IonizationFraction` /
  `ElectronDensity`; the `α_eq` target includes NO⁺ + e⁻ so it can be nonzero.
- [x] 2.5 `plasma_frequency_kernel<R>` — `ω_p = √(n_e e²/(ε₀ m_e))` constructing the existing
  `PlasmaFrequency` newtype (NEW kernel — `mhd/plasma.rs` lacks it; add it there, reusing the `mhd` constants).
- [x] 2.6 `rankine_hugoniot_temperature_kernel<R>` — normal-shock post-shock temperature ratio from `M_∞`
  (and `γ`) → `Temperature` (`T_post`).
- [x] 2.7 `recovery_temperature_kernel<R>` — `T_tr = T_post − ½|u|²/c_p` (pointwise; `|u|`, `c_p` supplied).
- [x] 2.8 `park2t_ionization_surrogate_kernel<R>` — the Tier-A fitted `α(ρ, T, …)` equilibrium target
  (lookup-table-reduction route).
- [x] 2.9 `PropagatingEffect` wrapper for each kernel in `wrappers.rs` (`match kernel(…){ Ok(v) =>
  PropagatingEffect::pure(v), Err(e) => PropagatingEffect::from_error(CausalityError::from(e)) }`, importing
  `PropagatingEffect`/`CausalityError` from `deep_causality_core`).
- [x] 2.10 **Isolation validation tests** (f64, `tests/` mirror): Arrhenius + relaxation kernels vs Park-2T
  table values; `ionization_fraction_kernel` recovers the Saha-equilibrium value as `τ → 0`; an
  electron-density point reproduces a **named RAM-C II station** (record the altitude, e.g. 71 km peak) within
  a tolerance set wide enough to absorb the named Tier-A biases — the ~2× two-temperature-lumping
  over-prediction (Farbar–Boyd–Martin 2013) and the Gupta-vs-Park rate-set sensitivity — with the tolerance
  justified in a comment (anchors: RAM-C II, *Fluid Dynamics* 2022, Aiken–Carter–Boyd 2025), not hand-tuned.
  Register in `mod.rs` + `tests/BUILD.bazel`.

## 3. LER stage mechanism + state-derived temperature (`deep_causality_cfd/src/types/flow/`)

- [x] 3.1 Add an `ler` helper (closed-form exponential relaxation `x_eq − (x_eq − x)·exp(−Δt/τ)` over a named
  `CoupledField` scalar) used by the stages below — bound `R: CfdScalar`, `Δt` from `StepContext::dt()`.
- [x] 3.2 `IonizationStage<R>` as a `PhysicsStage<D, R: CfdScalar>` — reads `T_tr`/species scalars from
  `CoupledField`, computes `α_eq(ρ, T_tr)` (the target carries electron-impact electrons as well as NO⁺ — RAM-C
  is in the mixed band, Aiken–Carter–Boyd 2025 — so it is not NO⁺-only) and `τ_ion` (grounded in the dominant
  associative-rate Arrhenius coefficient as the rate-limiting onset, computed from `T` — not a constant),
  relaxes the carried `α`/`n_e` scalar via the LER helper, writes back `n_e`. Static composition with
  `Coupling::between_steps().then(…)`.
- [x] 3.3 `EosStage<R>` as a `PhysicsStage` — two-temperature pressure closure into the `Ambient` (the
  `ViscosityArrhenius`→ambient template). Note in-scope effect on the incompressible ambient is limited;
  keep the interface so Tier-B reuses it.
- [x] 3.4 Recovery-temperature wiring — build the `T_tr` scalar field each step from the computed velocity
  via `recovery_temperature_kernel` + the mandatory `rankine_hugoniot_temperature_kernel` (config `M_∞`,
  `T_∞`, `c_p`); store as the `"T_tr"` scalar the `IonizationStage` reads. No prescribed schedule.
- [x] 3.5 Tests (f64): LER helper equals the analytic exponential to round-off; stays bounded/monotone at
  `τ = dt/1000` (explicit Euler diverges); static `(A,B)` composition type-checks with no `dyn`;
  `IonizationStage` produces `α ≠ α_eq` under a fast temperature ramp and `α → α_eq` as it slows. Register in
  `mod.rs` + `tests/BUILD.bazel`.

## 4. Coupling-seam generalization + blackout trigger + observables (`deep_causality_cfd`)

- [x] 4.0a **Generalize the seam (design D5/D8).** Give `StepContext` a backing sum type
  (`Dec { manifold, velocity } | Qtt {}`): `dt()`/`step()` universal; `manifold()`/`velocity()`/
  `sample_velocity()` DEC-only and fallible (`Option`/`Err` under `Qtt`). `PhysicsStage<const D, R>` and every
  stage impl stay **unchanged** (the realization chosen over a `FlowSnapshot` trait to avoid higher-ranked
  bounds; design D8). The QTT marcher constructs a `Qtt`-backed `StepContext`. Existing DEC `MarchRun` +
  `coupling_tests` keep passing.
- [x] 4.0b **QTT coupling host.** `QttMarchRun::run_coupled` takes the coupling + initial `CoupledField` as run
  arguments (not config-generic): each step publishes `"speed"` (dequantized), transports the carried reacting
  fraction `"alpha"` via the solver's `advance_scalar` (it stays a tensor train), applies the coupling, and
  samples the blackout observables. The spectral-projection / Brinkman `advance` math is unchanged.
- [x] 4.1 `BlackoutTrigger` as a `PropagatingEffect`-returning classifier (the canonical causal-monad seam,
  matching `theories/wrappers.rs`): `n_e → plasma_frequency_kernel →` compare to the **config** comms band →
  GNSS/comms-denied flag. (`CausalFlow`/`bind_or_error` is not used elsewhere in the cfd crate; the
  `PropagatingEffect` wrapper is the seam.)
- [x] 4.2 Extend `QttObserve` with opt-in `n_e` / plasma-frequency / blackout-dwell flags and emit them as
  `Report` series (`add_series`) from the QTT march run (via the 4.0b coupling host), alongside the existing
  diagnostics. Reuse `wall_heat_flux` for the neutral thermal series.
- [x] 4.3 Tests (f64): trigger raises the flag above the band and not below; the march run emits the `n_e`,
  plasma-frequency, and dwell series; counterfactual path-dependence — two histories reaching a comparable
  endpoint via different temperature ramps yield different `n_e`/dwell (the strengthened dynamic test).
  Register in `mod.rs` + `tests/BUILD.bazel`.

## 5. Self-verifying verification example (`deep_causality_cfd/verification/qtt_park2t_blackout/`)

- [x] 5.1 `config.rs` (configuration only), `main.rs` (the `CfdFlow::qtt_march` run + ionization coupling +
  blackout trigger), `print_utils.rs` (measure/verify) — mirroring `qtt_cylinder_verification`. Register as
  an `[[example]]` in `Cargo.toml`; add the row to `verification/README.md`.
- [x] 5.2 Gate (exit non-zero on break) the six LER criteria: (i) stability at stiffness (`τ = dt/1000`
  bounded/monotone); (ii) exponential exactness on the linear relaxation (equality to round-off); (iii) the
  RH-jump peak `T_post` lands in the ~10⁴ K band at `M ≈ 25` (not the cold isentropic value); (iv) lag is
  real and the Saha limit is recovered as `τ → 0`, with `τ_ion` grounded in the dominant rate, and the lag
  reproduces the qualitative electron-density **overshoot** (`n_e` above local equilibrium on a rise-then-relax
  history; Lin et al. 1962); (v) counterfactual path-dependence (two histories → two blackout outcomes); (vi)
  ionized-species target nonzero (`n_e > 0`). Each failing gate names itself.
- [x] 5.3 `baseline.txt` + `README.md` (human-readable labeled report) reporting the published reference
  cross-references (RAM-C II electron density / onset; *Fluid Dynamics* 2022; Aiken–Carter–Boyd 2025 review;
  Park-2T tables; Saha limit; Apollo dwell) **with Tier-A disclaimers** — incompressible rollout, `T_tr` is a
  recovery-temperature reconstruction (not a true post-shock path), first-order Lie split — and the **named
  quantified biases**: the ~2× two-temperature (`T_ve = T_e`) lumping over-prediction (Farbar–Boyd–Martin 2013;
  3T fix is an LER-native deferral), non-Maxwellian EEDF, and Gupta-vs-Park rate-set sensitivity, reported as
  the reason the `n_e` tolerance is what it is; no absolute coupled-CFD match claimed.

## 6. Finalize

- [x] 6.1 `make format && make fix` (both crates changed); clippy `--all-targets` clean — **fix, don't
  suppress** (no `#[allow]`); `cargo test -p deep_causality_physics` and `cargo test -p deep_causality_cfd`
  green; run the verification example (`cargo run -p deep_causality_cfd --example qtt_park2t_blackout`) →
  exit 0.
- [x] 6.2 Confirm constraints: static dispatch only, no `dyn`/`unsafe`/lib-macros; crate-root imports; lib
  float literals confined to `constants/` mapping via `from_f64`; no magic numbers / hardcoded schedules /
  fabricated proxies in kernels or stages (the dynamic-by-construction invariant); 100% coverage of new code;
  every new test module registered in `mod.rs` and `tests/BUILD.bazel` for both crates.
- [x] 6.3 `openspec validate add-park2t-blackout-tier-a --strict` passes.
- [x] 6.4 Mark **Gap 2 Tier-A closed** in `openspec/notes/plasma-blackout/gap-analysis.md` §4 Gap 2 and the
  gap-2 note §6; point Tier-B (compressible QTT marcher, reacting `*_rhs`, multi-mode relaxation, shock-rank
  control) at its sibling change `add-cfd-compressible-qtt-marcher` (specified, staged) rather than labelling
  it merely "open".
