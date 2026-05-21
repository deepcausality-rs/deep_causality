## ADDED Requirements

### Requirement: Zero hardcoded `f64` or `f32` in the public API of `deep_causality_physics`, with one carve-out for physical constants

The crate's public API SHALL contain no hardcoded `f64` or `f32` in any struct field, function/method signature, trait method, error variant, or trait bound. Every floating-point *quantity that participates in a calculation* SHALL be `R: RealField` for some `R` chosen by the caller.

The single carve-out is **physical constant declarations under `deep_causality_physics/src/constants/` and the PDG quark-mass constants in `nuclear/pdg.rs`**. These SHALL remain `pub const X: f64 = literal` and SHALL NOT be converted to `pub fn name<R: RealField>() -> R` form. The rationale: the values themselves do not benefit from precision-parametricity (exact-defined CODATA constants fit in `f64` exactly; measured constants have measurement uncertainty far below `f64` precision), and the breaking-change cost of converting 76 constants and updating every downstream consumer is not justified by zero precision gain.

Consumers at `R` precision SHALL convert the constant at the call site via `R::from_f64(SPEED_OF_LIGHT)` (using `RealField::from_f64` added by R0). This is a one-token cost at the call site equivalent to a `speed_of_light::<R>()` function call.

A second permitted exception is internal RNG sample conversions (`R::from_f64(rng.random::<f64>())` in the Lund fragmentation routines), each tagged `// PERMITTED-F64: RNG boundary`.

#### Scenario: Grep finds no public `f64`/`f32` outside permitted carve-outs

- **WHEN** the command `grep -rn -E '(\bf64\b|\bf32\b)' deep_causality_physics/src/ --include='*.rs'` is run and the output is filtered to lines representing public signatures, struct fields, trait methods, error variants, or trait bounds (and is NOT filtered to remove `constants/` paths)
- **THEN** the filtered output SHALL contain only:
  - lines under `deep_causality_physics/src/constants/` declaring `pub const X: f64 = literal`,
  - the analogous PDG quark-mass `pub const` declarations in `nuclear/pdg.rs`,
  - lines tagged `// PERMITTED-F64: RNG boundary`

#### Scenario: Physical constants stay as `pub const X: f64`

- **WHEN** the source is searched for `pub const \w+: f64` within `deep_causality_physics/src/constants/` and `nuclear/pdg.rs`
- **THEN** the existing declarations SHALL be preserved unchanged (no `pub fn` conversion)

- **AND WHEN** the same search is run anywhere else in `deep_causality_physics/src/`
- **THEN** zero matches SHALL appear (no `pub const X: f64` outside the carved-out locations)

#### Scenario: No `From<f64>`, `Into<f64>`, or `Mul<f64, Output = T>` trait bounds remain

- **WHEN** the source is searched for `From<f64>`, `Into<f64>`, and `Mul<f64, Output`
- **THEN** zero matches SHALL appear in any `impl` block or trait bound declaration

#### Scenario: Consumer uses a constant at `f32`

- **WHEN** an `f32`-precision calculation needs the speed of light
- **THEN** the call site SHALL write `f32::from_f64(SPEED_OF_LIGHT)` (or `R::from_f64(SPEED_OF_LIGHT)` inside generic code) — the constant value SHALL round-trip through `f64` losslessly because `299_792_458.0` is an exact integer that fits in both `f32` and `f64` mantissas without precision loss

### Requirement: Every wrapper struct is generic over `R: RealField`

All 47 wrapper structs identified by the audit SHALL be parameterized over `R: RealField`. Simple newtypes (`Mass`, `Speed`, `Acceleration`, `Force`, `Torque`, `Length`, `Area`, `Volume`, `MomentOfInertia`, `Frequency`, `Stress`, `Stiffness`, `SpacetimeInterval`, `AmountOfSubstance`, `HalfLife`, `Activity`, `EnergyDensity`, `ElectricPotential`, `MagneticFlux`, `Entropy`, `Efficiency`, `FocalLength`, `OpticalPower`, `Wavelength`, `NumericalAperture`, `BeamWaist`, `RayHeight`, `RayAngle`) SHALL have the shape `pub struct Name<R: RealField>(R)`.

Multi-field structs (`FourMomentum`, `WeakIsospin`, `LightconeEndpoint`, `LundParameters`, `ParticleData`) SHALL have the shape `pub struct Name<R: RealField> { field_1: R, field_2: R, ... }`.

Cross-crate-storage wrappers (`PhysicalVector`, `SpacetimeVector`, `PhysicalField`, `AbcdMatrix`, `JonesVector`, `StokesVector`, `ComplexBeamParameter`) SHALL propagate `R` through their storage type (e.g. `pub struct PhysicalVector<R: RealField>(pub CausalMultiVector<R>)`).

#### Scenario: Mass at `f64` and `f32`

- **WHEN** `Mass::<f64>::new(5.0_f64)` and `Mass::<f32>::new(5.0_f32)` are constructed
- **THEN** both calls SHALL compile, and both wrappers SHALL store the value at the requested precision

#### Scenario: Multi-field struct at custom precision

- **WHEN** `FourMomentum::<f64>::new(1.0, 0.0, 0.0, 0.0)` is constructed
- **THEN** the result SHALL be a `FourMomentum<f64>` with all four components at `f64` precision

#### Scenario: Photonics complex storage at `f32`

- **WHEN** `JonesVector::<f32>::new(...)` is constructed
- **THEN** the internal storage SHALL be `CausalTensor<Complex<f32>>` with no `f64` round-trip

### Requirement: Wrapper struct constructors, accessors, and conversions retype

Every wrapper's `new(val: R) -> Result<Self, Error>`, `new_unchecked(val: R) -> Self`, `value(&self) -> R`, and `From<Wrapper<R>> for R` SHALL retype from the pre-change `f64` shape.

#### Scenario: Constructor validates against the wrapper's invariant at any `R`

- **WHEN** `Mass::<f32>::new(-1.0_f32)` is called
- **THEN** the result SHALL be `Err(...)` (mass must be non-negative)

- **AND WHEN** `Mass::<f32>::new(0.0_f32)` is called
- **THEN** the result SHALL be `Ok(Mass(0.0_f32))`

#### Scenario: `From<Wrapper<R>> for R` round-trip

- **WHEN** `let m: Mass<f64> = Mass::new(2.5).unwrap();` and then `let v: f64 = m.into();`
- **THEN** `v` SHALL equal `2.5_f64`

### Requirement: ODE / RK4 / Kalman-filter solvers operate on `R: RealField` without `From<f64>` bounds

The RK4 step in `relativity/gravity.rs`, the ODE step in `chronometric/solve_gm.rs`, the ADM state in `theories/general_relativity/adm_state.rs`, and the Kalman filter in `dynamics/estimation.rs` SHALL bound `R: RealField` only. Every internal `<T as From<f64>>::from(literal)` site SHALL be rewritten to `R::from_f64(literal)` or a `RealField`-native expression.

#### Scenario: RK4 step at `f64` produces baseline-identical results

- **WHEN** the existing `f64`-precision RK4 test cases are run after this change set
- **THEN** every result SHALL match the pre-change baseline bit-identically

#### Scenario: RK4 step at `f32`

- **WHEN** the same test cases are run at `R = f32`
- **THEN** every result SHALL match the analytical expected value within `f32::EPSILON * <reasonable factor>` tolerance

### Requirement: Lund fragmentation random-sampling parameters generalize to `R: RealField`

`select_quark_flavor`, `select_meson_spin`, `generate_transverse_momentum`, and adjacent routines SHALL take `R`-typed control parameters (`strange_suppression: R`, `vector_fraction: R`, `sigma: R`). The internal RNG sample is preserved as `let rnd: f64 = rng.random();` followed by `let rnd: R = R::from_f64(rnd);`, with the `f64` line tagged `// PERMITTED-F64: RNG boundary`.

#### Scenario: Lund fragmentation at `f64`

- **WHEN** `select_quark_flavor(&mut rng, 0.3_f64)` is called
- **THEN** the behavior SHALL match the pre-change `f64` baseline

### Requirement: PDG database constructs `ParticleData<R>` on demand

The PDG particle database SHALL be exposed as `pub fn pdg_database<R: RealField>() -> Vec<ParticleData<R>>` (or equivalent on-demand constructor). No `static`-lifetime `f64`-typed PDG table SHALL survive.

#### Scenario: PDG lookup at any `R`

- **WHEN** `pdg_database::<f64>()` is called and the result is searched for a known PDG ID
- **THEN** the returned `ParticleData<f64>` SHALL contain the correct mass / charge / spin values at `f64` precision

### Requirement: Cross-crate consumption propagates `R: RealField`

Every site in `deep_causality_physics` that consumes `CausalMultiVector`, `CausalTensor`, `Complex`, `SimplicialManifold`, `Manifold`, or any other now-generic type from a sibling crate SHALL propagate `R` instead of pinning to `::<f64>`. Every `// TEMP: removed by generalize-physics-over-realfield` tag introduced by R0 SHALL be removed by this change set.

#### Scenario: No R0-introduced temp pins remain in physics

- **WHEN** the source is searched for `// TEMP: removed by generalize-physics-over-realfield`
- **THEN** zero matches SHALL appear

### Requirement: Behavior at `R = f64` is bit-identical to the pre-change baseline

Every test that passed before this change set at `f64` precision SHALL pass after this change set at `f64` precision with bit-identical numerical results.

#### Scenario: Full test suite at `f64`

- **WHEN** `cargo test -p deep_causality_physics` is run after this change set is applied
- **THEN** every existing test SHALL pass; every floating-point comparison SHALL match the pre-change result bit-for-bit

### Requirement: Property-test pass at `R = f32`

Every algorithmically-meaningful test (wrapper validation roundtrips, RK4 convergence, Kalman filter, Lund fragmentation distribution moments, Maxwell evolution invariants, PDG lookup correctness, physical-constant retrieval) SHALL be duplicated against `R = f32` with widened tolerances.

#### Scenario: `f32` duplicate exists for every algorithmic `f64` test

- **WHEN** the test suite is enumerated for tests in the categories above
- **THEN** every `f64` test SHALL have a corresponding `_f32` test in the same file

### Requirement: `cargo build --workspace` succeeds after this change set lands

The workspace SHALL compile cleanly after this change set lands. `deep_causality_effects` MAY temporarily pin its physics consumption to `::<f64>` (tagged `// TEMP: removed by generalize-effects-over-realfield`).

#### Scenario: Workspace builds with one command

- **WHEN** `cargo build --workspace` is run after this change set is applied
- **THEN** the build SHALL succeed with no errors

#### Scenario: Effects temp pins are greppable

- **WHEN** the source is searched for `// TEMP: removed by generalize-effects-over-realfield`
- **THEN** every effects-side physics-consumption pin SHALL be tagged with that marker
