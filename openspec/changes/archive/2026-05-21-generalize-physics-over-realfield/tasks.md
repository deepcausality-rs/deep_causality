## 1. Preconditions

- [ ] 1.1 Verify R0 (`generalize-topology-over-realfield`) has shipped. Run R0 task 11 invariant greps inside `deep_causality_topology/src/` — must produce zero hits.
- [ ] 1.2 Verify all four constructors (`RealField::from_f64`, `from_f32`, `from_i64`, `from_i32`) are available on `deep_causality_num`'s `RealField` trait with impls for `f32` and `f64`.
- [ ] 1.3 Grep `// TEMP: removed by generalize-physics-over-realfield` across the workspace; record every site as the cleanup checklist for Phase 9.
- [ ] 1.4 Snapshot current benchmark numbers for at least `deep_causality_physics` mechanics, relativity, photonics — used in Phase 10 to verify no regression.

## 2. Phase 2 — Mechanics & Materials

- [ ] 2.1 Retype `Mass(f64)` → `Mass<R: RealField>(R)` in `deep_causality_physics/src/dynamics/quantities.rs:11`. Update `new`, `new_unchecked`, `value`, `From<Mass<R>> for R`.
- [ ] 2.2 Repeat for the 10 sibling wrappers: `Speed` (line 49), `Acceleration` (line 87), `Force` (line 115), `Torque` (line 143), `Length` (line 169), `Area` (line 205), `Volume` (line 241), `MomentOfInertia` (line 277), `Frequency` (line 313).
- [ ] 2.3 Retype `Stress(f64)` → `Stress<R: RealField>(R)` in `materials/quantities.rs:13` and `Stiffness(f64)` → `Stiffness<R: RealField>(R)` at line 31.
- [ ] 2.4 Retype `PhysicalVector(pub CausalMultiVector<f64>)` → `PhysicalVector<R: RealField>(pub CausalMultiVector<R>)` in `dynamics/kinematics.rs:10`. Remove the R0 pin.
- [ ] 2.5 Retype the Kalman filter in `dynamics/estimation.rs` — `propagate_bayes_factor` at lines 107–113 takes `x_pred: &CausalTensor<R>` and ~10 other parameters at `R`.
- [ ] 2.6 Retype `Default` impls for wrappers that derived it (`Mass`, `Speed`, etc.) to manual `impl<R: RealField> Default` using `R::zero()`.
- [ ] 2.7 Add explicit `::<f64>` to existing tests; add `_f32` duplicates for the validation roundtrip and Kalman-filter convergence tests.

## 3. Phase 3 — Electromagnetism

- [ ] 3.1 Retype `ElectricPotential(f64)` → `ElectricPotential<R>(R)` in `em/quantities.rs:11`.
- [ ] 3.2 Retype `MagneticFlux(f64)` → `MagneticFlux<R>(R)` at line 32.
- [ ] 3.3 Retype `PhysicalField(pub CausalMultiVector<f64>)` → `PhysicalField<R>(pub CausalMultiVector<R>)` at line 54. Remove R0 pin.
- [ ] 3.4 Retype every Maxwell-solver function in `em/` to propagate `R`. Audit-time: re-grep `em/` for `f64` and patch each hit.
- [ ] 3.5 Constants in `constants/electromagnetic.rs` stay as `pub const X: f64`; verify by re-grep no `pub const` declarations were touched. Maxwell-solver call sites that read constants convert via `R::from_f64(CONST)` at the call site.
- [ ] 3.6 Update existing EM tests to explicit `::<f64>`; add `_f32` duplicates for the Maxwell evolution invariants.

## 4. Phase 4 — Thermodynamics

- [ ] 4.1 Retype `Entropy(f64)` → `Entropy<R>(R)` in `thermodynamics/thermodynamics_quantities.rs:10`.
- [ ] 4.2 Retype `Efficiency(f64)` → `Efficiency<R>(R)` at line 31.
- [ ] 4.3 Constants in `constants/thermodynamics.rs` stay as `pub const X: f64` (Boltzmann, Stefan-Boltzmann, etc.). Internal call sites convert via `R::from_f64(CONST)`.
- [ ] 4.4 Update existing thermodynamics tests to explicit `::<f64>`; add `_f32` duplicates for the validation roundtrips.

## 5. Phase 5 — Relativity & Chronometry

- [ ] 5.1 Retype `SpacetimeInterval(f64)` → `SpacetimeInterval<R>(R)` in `relativity/quantities.rs:10`.
- [ ] 5.2 Retype `SpacetimeVector(pub CausalMultiVector<f64>)` → `SpacetimeVector<R>(pub CausalMultiVector<R>)` at line 35. Remove R0 pin.
- [ ] 5.3 In `relativity/gravity.rs`, drop the `T: Field + Float + From<f64>` bound (lines 27, 82, 146); replace with `R: RealField`.
- [ ] 5.4 In the same file, rewrite every `<T as From<f64>>::from(literal)` to `R::from_f64(literal)`. The audit identified lines 52, 218, 228, 244–247; verify by re-grep after the edit.
- [ ] 5.5 In `chronometric/solve_gm.rs:161, 196`, drop the `R: RealField + From<f64>` bound's `+ From<f64>` half.
- [ ] 5.6 In `theories/general_relativity/adm_state.rs:18`, replace `S: Field + Clone + From<f64> + Into<f64>` with `S: RealField`.
- [ ] 5.7 Constants in `constants/universal.rs` stay as `pub const X: f64` (`SPEED_OF_LIGHT`, `NEWTONIAN_CONSTANT_OF_GRAVITATION`, `PLANCK_CONSTANT`, etc.). RK4 / ODE / ADM call sites convert via `R::from_f64(CONST)` where they read these.
- [ ] 5.8 Update existing relativity / chronometry tests to explicit `::<f64>`; add `_f32` duplicates for the RK4 convergence and ADM evolution tests.

## 6. Phase 6 — Nuclear & Particle Physics

- [ ] 6.1 Retype `AmountOfSubstance(f64)` → `AmountOfSubstance<R>(R)` in `nuclear/quantities.rs:10`.
- [ ] 6.2 Repeat for `HalfLife` (line 36), `Activity` (line 72), `EnergyDensity` (line 98).
- [ ] 6.3 Retype `FourMomentum { e: f64, px: f64, py: f64, pz: f64 }` → `FourMomentum<R: RealField> { e: R, px: R, py: R, pz: R }` at lines 133–137. Retype `new` (line 146), `from_mass_and_momentum` (line 153), `at_rest` (line 160), `boost_z` (line 257) and every other method on the type.
- [ ] 6.4 Retype `WeakIsospin { isospin: f64, i3: f64, hypercharge: f64 }` → `WeakIsospin<R: RealField>` in `theories/weak_force/weak_isospin.rs:14–17`.
- [ ] 6.5 Retype `LightconeEndpoint` in `nuclear/lund/kinematics.rs:19–23` and `LundParameters` in `nuclear/quantities.rs:349–351`.
- [ ] 6.6 Retype Lund fragmentation routines in `nuclear/lund/flavor.rs` — `select_quark_flavor`, `select_meson_spin`, `generate_transverse_momentum`. Generic over `R: RealField` for control parameters; preserve the `let rnd: f64 = rng.random();` line tagged `// PERMITTED-F64: RNG boundary`; convert via `R::from_f64`.
- [ ] 6.7 Retype `ParticleData` in `nuclear/pdg.rs:15–26` to `ParticleData<R: RealField>`. Replace `static`-lifetime PDG table with `pub fn pdg_database<R: RealField>() -> Vec<ParticleData<R>>`.
- [ ] 6.8 Retype `pub fn pdg_mass(pdg_id: i32) -> f64` at line 174 to `pub fn pdg_mass<R: RealField>(pdg_id: i32) -> Option<R>` (returning `Option` for unknown IDs is cleaner; verify the existing behavior on unknown IDs and pick the right error shape).
- [ ] 6.9 PDG quark-mass constants in `nuclear/pdg.rs` (M_U, M_D, M_S, M_C, M_B), particle constants in `constants/particle.rs`, and electroweak constants in `constants/electro_weak.rs` stay as `pub const X: f64`. Internal consumers convert via `R::from_f64(CONST)` at the call site.
- [ ] 6.10 Update existing tests to explicit `::<f64>`; add `_f32` duplicates for FourMomentum Lorentz-invariant preservation, Lund fragmentation distribution moments, and PDG lookup tests.

## 7. Phase 7 — Photonics

- [ ] 7.1 Retype the 7 simple photonics newtypes in `photonics/quantities.rs`: `FocalLength` (line 13), `OpticalPower` (27), `Wavelength` (41), `NumericalAperture` (63), `BeamWaist` (85), `RayHeight` (107), `RayAngle` (121).
- [ ] 7.2 Retype `AbcdMatrix(CausalTensor<f64>)` → `AbcdMatrix<R: RealField>(CausalTensor<R>)` at line 134. Remove R0 pin.
- [ ] 7.3 Retype `JonesVector(CausalTensor<Complex<f64>>)` → `JonesVector<R: RealField>(CausalTensor<Complex<R>>)` at line 147. Remove R0 pin.
- [ ] 7.4 Retype `StokesVector(CausalTensor<f64>)` → `StokesVector<R: RealField>(CausalTensor<R>)` at line 161. Remove R0 pin.
- [ ] 7.5 Retype `ComplexBeamParameter(Complex<f64>)` → `ComplexBeamParameter<R: RealField>(Complex<R>)` at line 191.
- [ ] 7.6 Retype every method on these photonics types to propagate `R`.
- [ ] 7.7 Constants in `constants/atomic.rs` (electron mass, Bohr radius, Rydberg constant, etc.) stay as `pub const X: f64`. Photonics call sites convert via `R::from_f64(CONST)`.
- [ ] 7.8 Update existing photonics tests to explicit `::<f64>`; add `_f32` duplicates for ABCD-matrix round-trip and Jones-vector polarization tests.

## 8. Phase 8 — Constants carve-out verification

- [ ] 8.1 Constants in `constants/earth.rs` (GM, radius, J2, etc.) stay as `pub const X: f64`.
- [ ] 8.2 Grep `pub const \w+: f64` across `deep_causality_physics/src/`. Every hit MUST fall under one of: (a) `constants/{universal,atomic,electromagnetic,thermodynamics,particle,electro_weak,earth}.rs`, (b) the PDG quark-mass constants in `nuclear/pdg.rs`. Any hit outside these locations is either (i) a misplaced constant that should be moved into `constants/`, or (ii) an algorithm-internal tolerance that should be replaced with a `RealField`-native expression — decide per occurrence.
- [ ] 8.3 Sweep for any remaining `f64` literals in struct field declarations (audit must have caught all 47 wrappers but verify).

## 9. Phase 9 — Remove R0 temporary pins, install effects pins

- [ ] 9.1 Run `grep -rn '// TEMP: removed by generalize-physics-over-realfield' deep_causality_physics/src/`. Every hit corresponds to a site that should now propagate `R` instead of pinning to `::<f64>`. Remove each pin and verify the call compiles via `R` propagation.
- [ ] 9.2 Run `cargo build --workspace`. Compile errors from `deep_causality_effects` enumerate every physics-consumption site needing a temporary pin. For each, add `::<f64>` at the construction site and tag the line `// TEMP: removed by generalize-effects-over-realfield`.
- [ ] 9.3 Confirm `cargo build --workspace` succeeds.
- [ ] 9.4 Confirm `cargo test --workspace` succeeds — every existing test passes; bit-identical at `R = f64`.

## 10. Phase 10 — Verification

- [ ] 10.1 `cargo build -p deep_causality_physics` succeeds.
- [ ] 10.2 `cargo test -p deep_causality_physics` passes — every existing test plus every new `_f32` duplicate.
- [ ] 10.3 `cargo clippy -p deep_causality_physics -- -D warnings` is clean.
- [ ] 10.4 `cargo fmt --check` is clean.
- [ ] 10.5 `bazel test //deep_causality_physics/...` passes.
- [ ] 10.6 Compare current benchmarks against Phase 1.4 snapshot. No regression >2% at `R = f64`.
- [ ] 10.7 Run the four invariant greps from R0 task 11 (`f64`, `From<f64>`, `Into<f64>`, `Mul<f64`) inside `deep_causality_physics/src/`. Permitted hit locations: (a) `// PERMITTED-F64`-tagged lines (RNG boundary in Lund), (b) `pub const X: f64` declarations under `constants/` and `nuclear/pdg.rs`. Any hit outside these is a defect.
- [ ] 10.8 Additional grep: `pub const \w+: f64` inside `deep_causality_physics/src/`. Hits MUST appear only under `constants/` and `nuclear/pdg.rs`; zero hits elsewhere.
- [ ] 10.9 `make build` and `make test` workspace-level pass.

## 11. Commit prep

- [ ] 11.1 Stage the changes per AGENTS.md (Golden Rule 1 — agent does not commit; user commits).
- [ ] 11.2 Draft a commit message summarizing the physics generalization, the 47 wrappers retyped over `R: RealField`, the `pub const X: f64` constants kept as-is (spec carve-out), the `_f32` test additions, and the temporary effects pins installed in Phase 9.
- [ ] 11.3 Leave the commit for the user to inspect and run.
