## Why

`deep_causality_physics` is "`f64` in disguise" at scale. An audit found ~147 distinct `f64` public-API locations across seven physics domains: 47 wrapper structs that bake `f64` directly into their newtype storage (`Mass(f64)`, `Speed(f64)`, `Acceleration(f64)`, `Force(f64)`, `Torque(f64)`, `FourMomentum { e: f64, px: f64, py: f64, pz: f64 }`, etc.), ~50 public methods on those wrappers (constructors `new(val: f64)`, accessors `value(&self) -> f64`, `From<Wrapper> for f64` impls), 76 physical-constant declarations as `pub const X: f64 = ...`, ~18 free-function parameters, and ~15 trait-bound sites using `From<f64>` round-trips.

The pattern across the crate is uniform: a domain-specific wrapper type encapsulates a `f64` numeric value, often with construction-time validation (e.g. mass must be non-negative, half-life must be positive). The validation is the contribution of the wrapper; the underlying `f64` is incidental.

Physical constants (`SPEED_OF_LIGHT`, `PLANCK_CONSTANT`, `BOLTZMANN_CONSTANT`, etc.) are a different case: they are *values*, not computations, and the values themselves do not benefit from precision-parametricity. Exact-defined CODATA constants (post-2019 SI) fit in `f64` exactly; measured constants have measurement uncertainty far below `f64` precision. This change set therefore **keeps physical constants as `pub const X: f64`** and leaves the precision conversion to consumers (`R::from_f64(SPEED_OF_LIGHT)` at the call site, using R0's `RealField::from_f64`).

The wrapper-and-method surface is where the precision-parametric abstraction matters. Two concrete problems:

1. **High-precision physics is impossible.** A relativity calculation that wants `f128` for the post-Newtonian expansion is blocked today because every wrapper type (`Mass<f64>`, `FourMomentum<f64>`, etc.) and every solver (RK4, Kalman filter, ADM evolution) is locked to `f64`. The constants themselves are not the limit; the *calculation* is.
2. **Memory-bound workloads can't pick `f32`.** A particle-physics simulation tracking millions of `FourMomentum` instances at `f32` precision (3 bytes saved per component × 4 components × 10^6 instances = 12 MB savings) is the standard pattern in production HEP code; today the crate forces `f64`.
3. **Post-R0 topology consumption is blocked.** After R0 ships `deep_causality_topology` generic over `R: RealField`, `deep_causality_physics` carries temporary `::<f64>` pins at every consumption site (`CausalMultiVector::<f64>`, `CausalTensor::<f64>`, `SimplicialManifold::<f64>`). Those pins exist precisely to compile the workspace during the R0→physics gap. This change set removes them.

The fix is the same audit-and-rewrite pattern R0 applies to topology: thread `R: RealField` through every wrapper struct, retype every method, drop the `From<f64>` round-trip crutches in the solvers. Same hard rip-and-replace policy. Same propagation discipline — library code propagates the parameter; only end-consumer call sites (binaries, examples, benchmarks) bind a concrete `R`. **The 76 physical-constant declarations stay as `pub const X: f64`** — explicitly carved out in the spec because converting them would produce zero precision benefit at the cost of a wide breaking-change ripple to downstream consumers.

## What Changes

**Invariant after this change set ships:** the public API of `deep_causality_physics` contains **zero** hardcoded `f64` or `f32` in any struct field, function/method signature, trait method, error variant, or trait bound — with one explicit carve-out for `pub const X: f64` declarations under `constants/` and the PDG quark masses in `nuclear/pdg.rs`. Every floating-point quantity that *participates in a calculation* is `R: RealField` for some `R` chosen by the caller. Physical constants stay as `pub const X: f64`; consumers convert via `R::from_f64(CONST)` at call sites.

Concretely:

- **All 47 wrapper structs are parameterized over `R: RealField`.** `Mass(f64)` becomes `Mass<R: RealField>(R)`. Same for `Speed<R>`, `Acceleration<R>`, `Force<R>`, `Torque<R>`, `Length<R>`, `Area<R>`, `Volume<R>`, `MomentOfInertia<R>`, `Frequency<R>`, `Stress<R>`, `Stiffness<R>`, `SpacetimeInterval<R>`, `AmountOfSubstance<R>`, `HalfLife<R>`, `Activity<R>`, `EnergyDensity<R>`, `ElectricPotential<R>`, `MagneticFlux<R>`, `Entropy<R>`, `Efficiency<R>`, `FocalLength<R>`, `OpticalPower<R>`, `Wavelength<R>`, `NumericalAperture<R>`, `BeamWaist<R>`, `RayHeight<R>`, `RayAngle<R>` (~28 simple newtypes). The multi-field structs — `FourMomentum<R> { e: R, px: R, py: R, pz: R }`, `WeakIsospin<R> { isospin: R, i3: R, hypercharge: R }`, `LightconeEndpoint<R>`, `LundParameters<R>`, `ParticleData<R>` — same treatment.
- **The 7 wrapper structs that store `CausalMultiVector<f64>` or `CausalTensor<f64>` or `Complex<f64>` parameterize through to the storage:** `PhysicalVector<R>(CausalMultiVector<R>)`, `SpacetimeVector<R>(CausalMultiVector<R>)`, `PhysicalField<R>(CausalMultiVector<R>)`, `AbcdMatrix<R>(CausalTensor<R>)`, `JonesVector<R>(CausalTensor<Complex<R>>)`, `StokesVector<R>(CausalTensor<R>)`, `ComplexBeamParameter<R>(Complex<R>)`. These require `R0` (topology) and the parallel multivector / tensor parameterization to be in place; this change set assumes they are.
- **Every wrapper's constructor (`new`, `new_unchecked`), accessor (`value`), and `From<Wrapper> for R` impl retypes.** ~135 mechanical method signature changes.
- **Physical constants stay as `pub const X: f64`.** Spec-level carve-out. Consumers at `R` precision read constants via `R::from_f64(SPEED_OF_LIGHT)` at the call site. Rationale: zero precision benefit (exact-defined values fit in `f64` exactly; measured values have measurement uncertainty below `f64` precision), and converting 76 constants would ripple to every downstream binary / example / test that names a constant.
- **Every `T: From<f64>` and `S: From<f64> + Into<f64>` bound is dropped.** `relativity/gravity.rs`'s RK4 solver, `chronometric/solve_gm.rs`'s ODE step, `theories/general_relativity/adm_state.rs`'s initialization all retype to `R: RealField` only.
- **Every internal `<T as From<f64>>::from(literal)` is rewritten to `R::from_f64(literal)`** (using the `RealField::from_f64` method added by R0).
- **Free-function parameters typed `f64`** (e.g. `select_quark_flavor<R: Rng>(rng, strange_suppression: f64)`, `boost_z(beta: f64)`, the 11-parameter `propagate_bayes_factor` Kalman filter) retype to `R: RealField`.
- **Topology / multivector / tensor consumption sites unpin.** Every `::<f64>` temporary pin tagged `// TEMP: removed by generalize-physics-over-realfield` is removed; the call propagates `R` through naturally.
- **Test utilities, examples, and benchmarks** retype with explicit `::<f64>` at end-consumer construction sites. The library surface itself stays generic.

**Hard rip-and-replace. No bridge code, no legacy compatibility shims, no type aliases, no parallel `f64`-flavored methods.** The same policy as R0.

**Propagation policy:** library code (everything in `deep_causality_physics/src/`) propagates `R: RealField` further upstream. Only end-consumer call sites bind a concrete `R`. This is consistent with R0's policy.

**One downstream library is affected: `deep_causality_effects`.** Effects consumes `deep_causality_physics` types in its `EffectData` enum (`physics::nuclear::FourMomentum`, etc., though the audit found these are mediated through `EffectData::Custom(Arc<dyn Any>)` rather than typed variants). The effects→physics gap is the same situation as the R0→physics gap: effects pins to `::<f64>` until its own change set lands. Each pin is tagged `// TEMP: removed by generalize-effects-over-realfield`.

## Capabilities

### New Capabilities

- `physics-realfield-generic`: The contract that every public-API surface in `deep_causality_physics` is parameterized over `R: RealField` from `deep_causality_num`, with zero hardcoded `f64` or `f32` in any struct field, function signature, trait method, error variant, or trait bound. Physical constants under `constants/` and `nuclear/pdg.rs` are explicitly carved out and stay as `pub const X: f64`. Covers all 47 wrapper structs, the relativity / chronometric / GR ODE solvers, the Lund fragmentation routines, and the photonics / electromagnetism / mechanics / thermodynamics / nuclear domain surfaces.

### Modified Capabilities

<!-- None at the spec-folder level. The cubical Regge calculus and Hodge decomposition specs are unshipped (no entry in openspec/specs/). The topology-realfield-generic capability (added by R0) is independent of this change set — physics is a downstream consumer, not a modifier of topology's spec. -->

## Impact

- **Crate affected:** `deep_causality_physics` only. R0 (`generalize-topology-over-realfield`) must have shipped first because physics consumes `CausalMultiVector`, `CausalTensor`, and `SimplicialManifold` — the topology / multivector / tensor parameterizations have to be in place.
- **Cross-crate dependencies:** consumes the now-generic `deep_causality_topology`, `deep_causality_tensor`, `deep_causality_multivector`. The temporary `::<f64>` pins R0 introduces are removed in this change set; consumption flows through with `R` as a propagated parameter.
- **Breaking changes (deliberate):** every consumer that names a physics type with hardcoded `f64` must update. Migration is mechanical — set the type parameter at the end-consumer site (`Mass::<f64>::new(5.0)` instead of `Mass::new(5.0)`).
- **Physical constants stay as `pub const X: f64`** — no downstream breakage on the constants surface. Consumers at `R` precision add a one-token `R::from_f64(SPEED_OF_LIGHT)` at the call site.
- **Effort estimate:** the audit estimates ~12 hours of focused work, broken into seven domain phases of ~1.5–2 hours each (reduced from ~14 hours by dropping the constants migration):
  1. Infrastructure (verify R0's `RealField::from_f64` and the three sibling constructors are available; verify the existing `pub const X: f64` constants stay untouched).
  2. Mechanics & Materials (~12 wrappers).
  3. Electromagnetism (3 wrappers + Maxwell solver).
  4. Thermodynamics (2 wrappers).
  5. Relativity & Chronometry (2 wrappers + RK4 / ODE solvers).
  6. Nuclear & Particle Physics (7 wrappers + Lund fragmentation + PDG database).
  7. Photonics (11 wrappers).
- **LOC estimate:** ~50 method-signature changes per domain × 7 domains = ~350 signature changes. ~1000 LOC of body rewrites (replacing `<T as From<f64>>::from(literal)` round-trips with `R::from_f64`); the 76-constant rewrite is dropped from scope.
- **Tests:** every existing `f64` test retypes to explicit `::<f64>`. New `f32` property tests duplicated per domain, with widened tolerances. Estimated ~50 new test functions.
- **Sequencing:** **R0 must ship first.** Sibling priority with R0 once R0 lands. `generalize-effects-over-realfield` is independent of this change set (effects can wait); during the gap, effects pins physics consumption to `::<f64>`.
- **What is NOT in scope:** new physics functionality; performance tuning beyond preserving the `R = f64` baseline; generalizing over `ComplexField<R>` for sites where complex values already appear (`Complex<f64>` in photonics) — those sites become `Complex<R>` mechanically, but no new complex-valued physics is added; changing the algorithmic content of any solver (RK4, Kalman filter, Maxwell evolution); replacing physical constants with measured-vs-defined-CODATA distinctions or unit-tracking type machinery (a separate concern, out of scope).
- **Reference:** workspace-wide audit of `deep_causality_physics` public API (~147 `f64` locations across 7 domains, conversation context 2026-05-21). R0 (`generalize-topology-over-realfield`) proposal and design.
