# park2t-ionization-kernels Specification

## Purpose
TBD - created by archiving change add-park2t-blackout-tier-a. Update Purpose after archive.
## Requirements
### Requirement: Park-2T quantity newtypes

`deep_causality_physics` SHALL provide quantity newtypes `ElectronDensity` (m⁻³), `IonizationFraction`
(dimensionless), `ElectronTemperature` (K), `VibrationalTemperature` (K), `MassFraction` (dimensionless), and
`ReactionRate`, each generic over `R: RealField`, each constructed through a validating constructor that rejects
physically invalid values (negative density/temperature/rate; fraction outside `[0, 1]`) with a `PhysicsError`,
following the existing `Temperature::new` newtype pattern. The existing `PlasmaFrequency` and `DebyeLength`
(`quantities/mhd/`) SHALL be reused, not duplicated.

#### Scenario: Constructors enforce invariants
- **WHEN** a newtype is constructed with an out-of-range value (negative density, a mass fraction above 1, a
  negative temperature)
- **THEN** the constructor returns a `PhysicsError` rather than an invalid quantity

#### Scenario: Valid construction round-trips
- **WHEN** a newtype is constructed with an in-range value and its accessor read back
- **THEN** the stored value equals the input

### Requirement: Pointwise Park-2T kernels

`deep_causality_physics` SHALL provide, in a dedicated `kernels/hypersonic/` domain, pure pointwise kernels —
each a free `fn name_kernel<R: RealField>(…) -> Result<Quantity<R>, PhysicsError>` that holds no state and does
not discretize space (gradients/divergences are supplied by the caller): a vibrational-relaxation kernel
(Landau–Teller `dT_ve/dt = (T_tr − T_ve)/τ_vt`, Millikan–White `τ_vt`), an Arrhenius-rate kernel
(`k(T) = A·T^n·exp(−E_a/(k_B T))`, forward and backward), an ionization-fraction kernel (Saha-equilibrium and
rate-based forms) producing `ElectronDensity` — whose equilibrium target SHALL include electron-impact-produced
electrons as well as NO⁺, because RAM-C (~7.6 km/s) sits in the mixed associative + electron-impact ionization
band (Aiken–Carter–Boyd 2025), so an NO⁺-only target under-predicts the equilibrium `n_e`; a plasma-frequency
kernel `ω_p = √(n_e e²/(ε₀ m_e))`
constructing the existing `PlasmaFrequency` newtype (`mhd/plasma.rs` currently has Debye-length and
Larmor-radius kernels but no plasma-frequency kernel — it is added here, reusing the `mhd` constants and
newtype), a Rankine–Hugoniot normal-shock temperature-jump kernel, a
recovery-temperature reconstruction kernel, and a Tier-A fitted ionization surrogate. Each kernel SHALL have a
`PropagatingEffect` wrapper in `kernels/hypersonic/wrappers.rs`, be registered in `kernels/hypersonic/mod.rs`,
and be flattened at `lib.rs`.

#### Scenario: Kernels are pure and dynamic
- **WHEN** a kernel is evaluated on two different input states
- **THEN** it returns two different outputs computed solely from those inputs — no captured state, no hardcoded
  result (the dynamic-by-construction invariant)

#### Scenario: Only constants of nature are literal
- **WHEN** the kernel sources are inspected
- **THEN** the only float literals are constants of nature and cited model coefficients (Park `A/n/E_a`,
  Millikan–White `τ` fits) defined in `deep_causality_physics/src/constants/` and lifted via `R::from_f64`; no
  temperature, density, fraction, or frequency value is a literal

### Requirement: Pointwise validation against published reference values

The Park-2T kernels SHALL be validated **in `deep_causality_physics`, in isolation** (pointwise, before any
solver integration) against published reference values: the Arrhenius and relaxation kernels against Park
two-temperature tables; the rate-based ionization kernel SHALL recover the Saha equilibrium value in the
`τ → 0` limit; the ionization/electron-density kernel SHALL reproduce a **specified RAM-C II station** (a
recorded altitude, e.g. the 71 km peak) within a stated tolerance. The tolerance SHALL be set wide enough to
absorb the known Tier-A model biases and SHALL be justified, not hand-tuned: the two-temperature (`T_ve = T_e`)
lumping that over-predicts peak `n_e` by ~2× (Farbar–Boyd–Martin 2013) and the documented rate-set sensitivity
(Gupta vs Park). The anchor references are RAM-C II (NASA Langley 1970), the *Fluid Dynamics* 2022 Park-2T
reproduction, and the Aiken–Carter–Boyd 2025 review.

#### Scenario: Saha limit recovered
- **WHEN** the rate-based ionization kernel is driven with a relaxation time approaching zero at fixed
  thermodynamic state
- **THEN** its ionization fraction converges to the Saha-equilibrium kernel's value within rounding tolerance

#### Scenario: Reference point reproduced
- **WHEN** the ionization/electron-density kernel is evaluated at a documented RAM-C II / Park-2T reference
  condition
- **THEN** the electron density matches the published value within the recorded tolerance, where that
  tolerance explicitly absorbs the named Tier-A biases (the ~2× two-temperature-lumping over-prediction and the
  rate-set sensitivity), not an unexplained margin

