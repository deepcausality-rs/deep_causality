## 1. Scaffolding & new units

- [ ] 1.1 Create `deep_causality_physics/src/kernels/fluids/governing.rs`, `constitutive.rs`, `kinematics.rs`, `dimensionless.rs`, `turbulence.rs`, `coherent_structures.rs`, `compressible.rs`, `boundary_layer.rs`, `ideal_flow.rs` as empty stubs registered in `kernels/fluids/mod.rs` behind `pub(crate) mod`. Leave each `pub use` line commented out until its group's gates close.
- [ ] 1.2 Create `deep_causality_physics/src/theories/fluid_dynamics/mod.rs` with empty `incompressible_ns.rs`, `compressible_ns.rs`, `euler.rs`, `stokes.rs` submodules; register the parent in `theories/mod.rs`.
- [ ] 1.3 Add the seven new units (`DynamicViscosity<R>`, `KinematicViscosity<R>`, `Vorticity<R>`, `StrainRate<R>`, `MassFlux<R>`, `SpecificEnthalpy<R>`, `WallShearStress<R>`) under `deep_causality_physics/src/units/`, one file per type, each with `new(val) -> Result<Self, PhysicsError>`, `new_unchecked`, `value(&self) -> R`, `From<…> for f64` impls, mirroring the existing `Pressure<R>` / `Density<R>` style. Register in `units/mod.rs`.
- [ ] 1.4 Add unit tests for each new newtype under `deep_causality_physics/tests/units/<name>_tests.rs`: finite check, negative-value rejection where applicable, `value()` round-trip, default, debug, `From<…>` for f64. Register in `tests/units/mod.rs` and `tests/BUILD.bazel`.
- [ ] 1.5 Re-export new units from `deep_causality_physics/src/lib.rs`.
- [ ] 1.6 Run `cargo build -p deep_causality_physics` + `cargo clippy -p deep_causality_physics --all-targets -- -D warnings`; fix any lints at root cause (no `#[allow]`).

## 2. Kinematic kernels (foundational — many downstream kernels depend on these)

- [ ] 2.1 Implement `strain_rate_tensor_kernel`, `rotation_rate_tensor_kernel`, `vorticity_from_gradient_kernel` in `kernels/fluids/kinematics.rs`.
- [ ] 2.2 Implement `velocity_gradient_invariants_kernel` returning `(P, Q, R)` for `∇u`.
- [ ] 2.3 Implement `helicity_density_kernel` and `enstrophy_density_kernel`.
- [ ] 2.4 Add tests under `tests/kernels/fluids/kinematics_tests.rs` covering: strain + rotation reconstruct gradient (property test); helicity sign-flips under reflection (property test); enstrophy ≥ 0 (algebraic); Galilean invariance of strain rate; precision-backend tests across `f32`, `f64`, `Float106`.
- [ ] 2.5 Add causal wrappers in `kernels/fluids/wrappers.rs` for every kernel landed in 2.1–2.3; wrapper tests in `tests/kernels/fluids/wrappers_tests.rs`.
- [ ] 2.6 Uncomment kinematics `pub use` line in `kernels/fluids/mod.rs`. Re-export from `lib.rs`. `cargo build` + `clippy` + tests clean.

## 3. Governing-equation kernels

- [ ] 3.1 Implement `convective_acceleration_kernel`, `viscous_diffusion_kernel`, `pressure_gradient_force_kernel` in `kernels/fluids/governing.rs`.
- [ ] 3.2 Implement `continuity_rhs_kernel`, `vorticity_transport_kernel`, `scalar_advection_diffusion_kernel`, `energy_rhs_kernel` in the same file.
- [ ] 3.3 Add tests under `tests/kernels/fluids/governing_tests.rs`: Galilean invariance of convective acceleration; pressure-gradient error on `ρ ≤ 0`; continuity reduces to 0 for incompressible divergence-free flow; vorticity transport reduces to inviscid case at `ν = 0`; precision-backend sweep.
- [ ] 3.4 Add causal wrappers for every kernel in this group; wrapper tests.
- [ ] 3.5 Uncomment governing `pub use` + re-export. Build/clippy/tests clean.

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
