## ADDED Requirements

### Requirement: Zero hardcoded `f64` or `f32` in the public API of `deep_causality_physics`

The crate's public API SHALL contain no hardcoded `f64` or `f32` in any struct field, function/method signature, trait method, error variant, trait bound, or `pub const` declaration. Every floating-point quantity SHALL be `R: RealField` for some `R` chosen by the caller. The only permitted exceptions are internal RNG sample conversions (`R::from_f64(rng.random::<f64>())` in the Lund fragmentation routines), each tagged `// PERMITTED-F64: RNG boundary`.

#### Scenario: Grep finds no public `f64`/`f32` outside permitted exceptions

- **WHEN** the command `grep -rn -E '(\bf64\b|\bf32\b)' deep_causality_physics/src/ --include='*.rs'` is run and the output is filtered to lines representing public signatures, struct fields, trait methods, error variants, trait bounds, or `pub const` declarations
- **THEN** the filtered output SHALL contain only lines tagged `// PERMITTED-F64`

#### Scenario: No `pub const X: f64` survives

- **WHEN** the source is searched for `pub const \w+: f64` and `pub const \w+: f32`
- **THEN** zero matches SHALL appear

#### Scenario: No `From<f64>`, `Into<f64>`, or `Mul<f64, Output = T>` trait bounds remain

- **WHEN** the source is searched for `From<f64>`, `Into<f64>`, and `Mul<f64, Output`
- **THEN** zero matches SHALL appear in any `impl` block or trait bound declaration

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

### Requirement: Physical constants are functions, not `pub const` declarations

Every physical constant in `constants/{universal,atomic,electromagnetic,thermodynamics,particle,electro_weak,earth}.rs` and the PDG quark-mass constants in `nuclear/pdg.rs` SHALL be exposed as `pub fn name<R: RealField>() -> R { R::from_f64(literal) }`. No `pub const X: f64` or `pub const X: f32` SHALL survive.

The function form SHALL be marked `#[inline]` so the LLVM optimizer inlines the call to a constant load at `R = f64`.

#### Scenario: Speed of light at `f64`

- **WHEN** `speed_of_light::<f64>()` is called
- **THEN** the result SHALL equal `299_792_458.0_f64`

#### Scenario: Speed of light at `f32`

- **WHEN** `speed_of_light::<f32>()` is called
- **THEN** the result SHALL equal `299_792_458.0_f32` (with `f32` precision rounding applied by `from_f64`)

#### Scenario: Inlined performance at `R = f64`

- **WHEN** a hot loop reads `speed_of_light::<f64>()` per iteration
- **THEN** the LLVM-emitted code SHALL be equivalent (within negligible variation) to the same hot loop reading a `pub const SPEED_OF_LIGHT: f64`; no function-call overhead SHALL appear in the inner loop

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
