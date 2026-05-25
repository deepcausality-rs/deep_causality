## 1. Scaffolding & new units

- [x] 1.1 Create `deep_causality_physics/src/kernels/fluids/governing.rs`, `constitutive.rs`, `kinematics.rs`, `dimensionless.rs`, `turbulence.rs`, `coherent_structures.rs`, `compressible.rs`, `boundary_layer.rs`, `ideal_flow.rs` as empty stubs registered in `kernels/fluids/mod.rs` behind `pub(crate) mod`. Leave each `pub use` line commented out until its group's gates close.
- [x] 1.2 Create `deep_causality_physics/src/theories/fluid_dynamics/mod.rs` with empty `incompressible_ns.rs`, `compressible_ns.rs`, `euler.rs`, `stokes.rs` submodules; register the parent in `theories/mod.rs`.
- [x] 1.3 Append three new newtypes to `deep_causality_physics/src/kernels/fluids/quantities.rs` (following the existing concatenated-quantities convention in that file): `KinematicViscosity<R>` (m²/s), `SpecificEnthalpy<R>` (J/kg), `WallShearStress<R>` (Pa). Each gets `Default`, `new(val) -> Result<Self, PhysicsError>` enforcing `is_finite()` and non-negative invariant where appropriate, `new_unchecked`, `value(&self) -> R`, `From<…> for f64`. Existing `Viscosity<R>` (dynamic, Pa·s) is reused unchanged.
- [x] 1.4 Add unit tests for the three new newtypes in `deep_causality_physics/tests/kernels/fluids/quantities_tests.rs`. Coverage: `new` happy path, `new` rejection of negatives (where invariant applies), `new` rejection of non-finite, `new_unchecked`, `value()` round-trip, `Default`, `Debug`, `Clone`, `Copy`, `PartialEq`, `From<…> for f64`. File already registered in `tests/kernels/fluids/mod.rs`; Bazel glob `kernels/fluids/*_tests.rs` picks it up automatically.
- [x] 1.5 Re-export the three new newtypes from `deep_causality_physics/src/lib.rs` — already covered by the existing `pub use crate::kernels::fluids::*;` glob; no edit needed.
- [x] 1.6 `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo fmt --check` all clean. 1016 tests pass; 22 new `quantities_tests` (KinematicViscosity, SpecificEnthalpy, WallShearStress) all green.

## 1B. Typed vector and tensor newtype family (D1 reversal)

Reverses the original D1 "raw arrays for vectors and tensors" decision; see `design.md` §D1 for the rationale. Lands before Group 2 because every later kernel group consumes these types.

- [x] 1B.1 Appended four vector newtypes to `kernels/fluids/quantities.rs`: `Velocity3<R>`, `VorticityVector<R>`, `AccelerationVector<R>`, `BodyForceDensity<R>`. Each wraps `[R; 3]` with finiteness check, `new_unchecked`, `value`, `into_inner`, bidirectional `From` conversions, `Default` (zero vector), and the standard derived traits.
- [x] 1B.2 Appended four rank-2 tensor newtypes: `VelocityGradient<R>` (finiteness only, Jacobian convention pinned by docstring), `StrainRateTensor<R>` (finiteness + symmetry via exact equality `S_ij == S_ji`), `RotationRateTensor<R>` (finiteness + zero diagonal + antisymmetry `Ω_ij == −Ω_ji`), `CauchyStress<R>` (finiteness + symmetry). Per the spec correction during implementation: `From<[[R; 3]; 3]> for Self` is provided only on `VelocityGradient` (finiteness-only); invariant-bearing tensors expose only `From<Self> for [[R; 3]; 3]` so the invariant bypass via `new_unchecked` is explicit at the call site rather than silent.
- [x] 1B.3 Added 56 tests in `quantities_tests.rs` covering: `new` happy path for each newtype, finiteness rejection, symmetry / antisymmetry / zero-diagonal rejection, `new_unchecked` bypass, `value` borrow, `into_inner` consume, `From` round-trip for finiteness-only types, `Default` is zero, and trait coverage. Property test landed: a finite `VelocityGradient` decomposes as `S + Ω` and both halves pass the invariant-bearing constructors.
- [x] 1B.4 `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1072 tests pass, 56 new), `cargo fmt --check` all clean.

## 2. Kinematic kernels (foundational — many downstream kernels depend on these)

All kernels in this group consume `&VelocityGradient<R>` (Jacobian convention) and return typed outputs per `specs/fluid-dynamics-kernels/spec.md` "Kinematic kernels".

- [x] 2.1 Implemented `strain_rate_tensor_kernel` and `rotation_rate_tensor_kernel` in `kernels/fluids/kinematics.rs`. Both return via `new_unchecked` because `0.5·(G ± Gᵀ)` guarantees symmetry / antisymmetry exactly in IEEE 754.
- [x] 2.2 Implemented `vorticity_from_gradient_kernel(grad_u) -> VorticityVector<R>` (infallible) and `velocity_gradient_invariants_kernel(grad_u) -> Result<(R, R, R), PhysicsError>` using the Chong–Perry–Cantwell convention with `tr(A²)` unrolled to avoid clippy `needless_range_loop`.
- [x] 2.3 Implemented `helicity_density_kernel(u, ω) -> R` (infallible dot product) and `enstrophy_density_kernel(ω) -> Result<R, PhysicsError>`.
- [x] 2.4 Added 20 tests in `tests/kernels/fluids/kinematics_tests.rs`: strain rate symmetry, strain rate vanishes for rigid-body rotation, strain rate equals input for pure strain (Galilean invariance via signature), rotation rate antisymmetry, rotation rate vanishes for pure strain, decomposition `∇u = S + Ω` (property test), vorticity on a known field, vorticity zero for irrotational flow, invariants P = 0 for incompressible flow, invariants on a known diagonal matrix, invariants on zero gradient, helicity as dot product, helicity sign-flip under full-parity reflection, helicity zero for orthogonal vectors, enstrophy positivity and known value, enstrophy on zero vorticity, enstrophy non-negative on multiple cases, f32 precision sweep on strain rate and enstrophy.
- [x] 2.5 Added causal wrappers in `kernels/fluids/wrappers.rs` for all six kinematic kernels and 6 corresponding wrapper tests in `tests/kernels/fluids/wrappers_tests.rs`.
- [x] 2.6 Uncommented `pub use kinematics::*` in `kernels/fluids/mod.rs`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1098 tests pass: 1072 pre-existing + 20 kinematics + 6 wrapper), `cargo fmt --check` all clean. No `#[allow]` suppressions added.

## 3. Governing-equation kernels

Spec corrections applied before implementation: (a) Galilean-invariance scenario replaced with the correct linearity-in-velocity-offset scenario (the convective term `(u·∇)u` is not Galilean invariant alone — only the full material derivative is); (b) `vorticity_transport_kernel` signature gains `grad_u: &VelocityGradient<R>` for the vortex-stretching term; (c) the coarse `energy_rhs_kernel` is replaced with three pointwise building blocks (`kinetic_energy_density_kernel`, `viscous_dissipation_rate_kernel`, `pressure_work_kernel`) — the full energy RHS evaluator lands in Group 14 (compressible NS theory).

- [x] 3.1 Implemented `convective_acceleration_kernel`, `viscous_diffusion_kernel`, `pressure_gradient_force_kernel` (errors on `ρ = 0`) in `kernels/fluids/governing.rs`.
- [x] 3.2 Implemented `continuity_rhs_kernel`, `vorticity_transport_kernel` (with the `grad_u` parameter added per the spec correction), `scalar_advection_diffusion_kernel`.
- [x] 3.3 Implemented the three energy building blocks: `kinetic_energy_density_kernel`, `viscous_dissipation_rate_kernel`, `pressure_work_kernel`.
- [x] 3.4 Added 27 tests in `tests/kernels/fluids/governing_tests.rs` covering: convective acceleration on a known field, linearity-in-velocity-offset property test (replaces the wrong Galilean-invariance scenario), pressure-gradient force errors on `ρ = 0`, viscous diffusion linearity in ν, continuity reduces to 0 for incompressible divergence-free flow, continuity picks up `∇ρ` and `div u` terms, inviscid-Helmholtz limit of vorticity transport, vorticity diffusion is proportional to ν, vortex-stretching vanishes when `ω·grad_u_rows = 0`, scalar transport reduces to pure advection at `D = 0` and pure diffusion at `u = 0` and source-only at zero everything, kinetic energy density positivity and known values, viscous-dissipation double-contraction on a known tensor pair, pressure-work sign agreement, f32 precision sweep on convective and viscous kernels.
- [x] 3.5 Added 10 causal wrappers in `kernels/fluids/wrappers.rs` for every governing kernel, plus 10 wrapper tests including the error-path scenario for `pressure_gradient_force`.
- [x] 3.6 Uncommented `pub use governing::*` in `kernels/fluids/mod.rs`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1134 tests pass, +36 since Group 2), `cargo fmt --check` all clean. No `#[allow]` suppressions added.

## 4. Constitutive kernels

- [ ] 4.1 Implement `newtonian_viscous_stress_kernel`, `newtonian_viscous_stress_with_bulk_kernel`, `power_law_apparent_viscosity_kernel` in `kernels/fluids/constitutive.rs`.
- [ ] 4.2 Tests: Newtonian stress vanishes in rigid-body motion; Stokes-hypothesis form is the `ζ = 0` special case (bit-equality); power-law equals Newtonian at `n = 1`; sign convention (positive in tension) pinned by a prescribed-input test; precision-backend sweep.
- [ ] 4.3 Causal wrappers + tests.
- [ ] 4.4 Uncomment + re-export. Build/clippy/tests clean.

## 5. Dimensionless number kernels

- [ ] 5.1 Implement the 18 dimensionless-number kernels in `kernels/fluids/dimensionless.rs`: Reynolds, Mach, Froude, Weber, Prandtl, Peclet, Strouhal, Knudsen, Richardson, Rayleigh, Grashof, Eckert, Schmidt, Lewis, Stokes, Capillary, Bond, Nusselt.
- [ ] 5.2 Tests: Reynolds composition from `(u, L, ν)`; `Pe = Re · Pr`; `Le = Sc / Pr`; one direct prescribed-input test per number; precision-backend sweep on representative numbers.
- [ ] 5.3 Causal wrappers + tests.
- [ ] 5.4 Uncomment + re-export. Build/clippy/tests clean.

## 6. Turbulence quantity kernels

- [ ] 6.1 Implement `turbulent_kinetic_energy_kernel`, `dissipation_rate_kernel`, `kolmogorov_length_kernel`, `kolmogorov_time_kernel`, `kolmogorov_velocity_kernel`, `taylor_microscale_kernel`, `integral_length_scale_kernel`, `reynolds_stress_kernel`, `eddy_viscosity_boussinesq_kernel` in `kernels/fluids/turbulence.rs`.
- [ ] 6.2 Tests: Kolmogorov-scale algebraic identities `η · u_η / ν = 1` and `η / (u_η τ_η) = 1`; B5 Taylor-microscale identity `λ² ε = 15 ν k`; dissipation rate ≥ 0; precision-backend sweep.
- [ ] 6.3 Causal wrappers + tests.
- [ ] 6.4 Uncomment + re-export. Build/clippy/tests clean.

## 7. Coherent-structure detector kernels

- [ ] 7.1 Implement `q_criterion_kernel`, `lambda2_kernel`, `delta_criterion_kernel`, `swirling_strength_kernel` in `kernels/fluids/coherent_structures.rs`. `lambda2` and `swirling_strength` use a `[[R; 3]; 3]` eigenvalue computation; reuse existing eigenvalue infrastructure if available in `deep_causality_num`, otherwise implement an internal 3×3 symmetric eigenvalue helper (private to this module) with full unit tests.
- [ ] 7.2 Tests: Q-criterion identity `Q + 0.5·‖S‖² − 0.5·‖Ω‖² = 0` across precision backends; λ₂ < 0 inside a Burgers-vortex core; Δ-criterion sign matches the published flow-classification table on representative inputs; swirling strength = 0 in irrotational flow; precision-backend sweep.
- [ ] 7.3 Causal wrappers + tests.
- [ ] 7.4 Uncomment + re-export. Build/clippy/tests clean.

## 8. Compressible-flow thermodynamic kernels

- [ ] 8.1 Implement `speed_of_sound_ideal_gas_kernel`, `specific_enthalpy_kernel`, `total_enthalpy_kernel`, `total_pressure_isentropic_kernel`, `total_temperature_isentropic_kernel`, `entropy_production_rate_kernel` in `kernels/fluids/compressible.rs`.
- [ ] 8.2 Tests: `T_0 = T` at `M = 0`; entropy production ≥ 0 for any physically valid input; isentropic relations recover textbook reference values at `M = 1` and `M = 2`; speed of sound for air at 288.15 K matches the published 340.29 m/s to within precision tolerance; precision-backend sweep.
- [ ] 8.3 Causal wrappers + tests.
- [ ] 8.4 Uncomment + re-export. Build/clippy/tests clean.

## 9. Boundary-layer kernels

- [ ] 9.1 Implement `wall_shear_stress_newtonian_kernel`, `friction_velocity_kernel`, `viscous_length_scale_kernel`, `y_plus_kernel`, `viscous_sublayer_velocity_kernel`, `log_law_velocity_kernel`, `skin_friction_coefficient_kernel` in `kernels/fluids/boundary_layer.rs`.
- [ ] 9.2 Tests: `y⁺` scales linearly with wall distance; viscous sublayer law equals `y⁺` exactly; log-law parameters `κ = 0.41`, `B = 5.0` recover a published `u⁺` value at `y⁺ = 100` to within tolerance; precision-backend sweep.
- [ ] 9.3 Causal wrappers + tests.
- [ ] 9.4 Uncomment + re-export. Build/clippy/tests clean.

## 10. Ideal-flow primitive kernels

- [ ] 10.1 Implement `dynamic_pressure_kernel`, `bernoulli_total_head_kernel`, `stream_function_2d_kernel`, `velocity_potential_2d_kernel`, `circulation_kernel`, `kutta_joukowski_lift_kernel` in `kernels/fluids/ideal_flow.rs`.
- [ ] 10.2 Tests: dynamic pressure quadratic-scaling; Kutta–Joukowski lift = 0 at zero circulation; circulation around a closed loop of a known uniform-flow + point-vortex agrees with the analytical `Γ`; precision-backend sweep.
- [ ] 10.3 Causal wrappers + tests.
- [ ] 10.4 Uncomment + re-export. Build/clippy/tests clean.

## 11. Theory layer — incompressible Newtonian NS

- [ ] 11.1 Implement `incompressible_ns_rhs_kernel` in `theories/fluid_dynamics/incompressible_ns.rs` composing `convective_acceleration_kernel`, `pressure_gradient_force_kernel`, `viscous_diffusion_kernel`, and body-force addition. Docstring restates sign convention and equation form.
- [ ] 11.2 Tests under `tests/theories/fluid_dynamics/incompressible_ns_tests.rs`: inviscid limit recovers `euler_momentum_rhs_kernel`; creeping-flow limit recovers `stokes_momentum_rhs_kernel`; body-force linearity property test; Galilean invariance on convective term; precision-backend sweep.
- [ ] 11.3 Causal wrapper + tests.
- [ ] 11.4 Re-export from `theories/fluid_dynamics/mod.rs` and `lib.rs`. Build/clippy/tests clean.

## 12. Theory layer — Euler

- [ ] 12.1 Implement `euler_momentum_rhs_kernel` in `theories/fluid_dynamics/euler.rs`. Signature has no viscosity input.
- [ ] 12.2 Tests: equal to `incompressible_ns_rhs_kernel` at `ν = 0` and `laplacian_u = 0`; signature lacks viscosity (compile-time check via type tests); precision-backend sweep.
- [ ] 12.3 Causal wrapper + tests.
- [ ] 12.4 Re-export. Build/clippy/tests clean.

## 13. Theory layer — Stokes

- [ ] 13.1 Implement `stokes_momentum_rhs_kernel` in `theories/fluid_dynamics/stokes.rs`. Signature has no `u` / `grad_u` inputs.
- [ ] 13.2 Tests: equal to `incompressible_ns_rhs_kernel` with `u = 0` and `grad_u = 0`; signature lacks convective inputs; precision-backend sweep.
- [ ] 13.3 Causal wrapper + tests.
- [ ] 13.4 Re-export. Build/clippy/tests clean.

## 14. Theory layer — compressible NS

- [ ] 14.1 Implement `compressible_ns_continuity_rhs_kernel`, `compressible_ns_momentum_rhs_kernel`, `compressible_ns_energy_rhs_kernel` in `theories/fluid_dynamics/compressible_ns.rs`. Each docstring states conserved variable, sign convention, and equation form. Energy equation uses total-energy form `E = e + 0.5·‖u‖²`.
- [ ] 14.2 Tests: continuity reduces to 0 for incompressible divergence-free flow; momentum reduces to incompressible NS at constant `ρ` and `∇·u = 0`; energy-dissipation term ≥ 0 for any Newtonian fluid; precision-backend sweep on representative inputs.
- [ ] 14.3 Causal wrappers + tests.
- [ ] 14.4 Re-export. Build/clippy/tests clean.

## 15. Final integration & gates

- [ ] 15.1 Verify `deep_causality_physics/Cargo.toml` has no new dependencies and the dep set is unchanged from before this change.
- [ ] 15.2 Update `deep_causality_physics/tests/BUILD.bazel` to include every new test file under `tests/kernels/fluids/` and `tests/theories/fluid_dynamics/` and `tests/units/`. Run `bazel build //deep_causality_physics/...` and `bazel test //deep_causality_physics/...` to verify Bazel registration is complete.
- [ ] 15.3 Run `make format && make fix` on the whole repo.
- [ ] 15.4 Run `cargo build -p deep_causality_physics` in release and debug profiles; both clean.
- [ ] 15.5 Run `cargo clippy -p deep_causality_physics --all-targets -- -D warnings`; clean with no `#[allow]` suppressions added.
- [ ] 15.6 Run `cargo test -p deep_causality_physics`; all tests pass across precision backends.
- [ ] 15.7 Run the project's coverage tooling on `deep_causality_physics` and verify 100% line coverage on every new src file; document any justified unreachable-code skips per AGENTS.md §"Code testing".
- [ ] 15.8 Final review: confirm proposal scope matches what shipped, every kernel in the spec exists at the published signature, every spec scenario has a corresponding test, every regime function in `theories/fluid_dynamics/` composes published kernels with no inline re-derivation. Prepare commit message and hand off to user per AGENTS.md golden rule (agents never `git commit`).
