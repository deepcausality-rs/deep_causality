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

- [x] 4.1 Implemented `newtonian_viscous_stress_kernel(mu, S, div_u) -> Result<CauchyStress, _>`, `newtonian_viscous_stress_with_bulk_kernel(mu, zeta, S, div_u) -> Result<CauchyStress, _>`, `power_law_apparent_viscosity_kernel(K, n, shear_rate) -> Result<Viscosity, _>` in `kernels/fluids/constitutive.rs`. Newtonian stress returns via `CauchyStress::new_unchecked` because the algebra (`2μS` symmetric + diagonal bulk term) guarantees symmetry.
- [x] 4.2 Added 14 tests in `tests/kernels/fluids/constitutive_tests.rs`: stress vanishes for zero strain and zero divergence; stress is symmetric; incompressible reduces to `2μS` exactly; bulk correction lands only on the diagonal at isotropic dilatation (sanity: `2·1·1 − (2/3)·1·3 = 0`); bulk-with-`ζ=0` matches the Stokes-hypothesis kernel bit-for-bit; bulk-term-only contribution is diagonal; bulk kernel is symmetric; power-law reduces to Newtonian at `n=1` across multiple shear rates including 0; shear-thinning (`n<1`); shear-thickening (`n>1`); known-value sanity (`K=2, n=0.5, γ̇=4 ⇒ μ=1`); negative shear rate errors; zero shear rate with `n<1` produces infinity rejected by `Viscosity::new`; f32 precision sweep on Newtonian stress and power-law.
- [x] 4.3 Added 4 causal wrappers in `kernels/fluids/wrappers.rs` (Newtonian, Newtonian-with-bulk, power-law success and error paths) plus 4 wrapper tests.
- [x] 4.4 Uncommented `pub use constitutive::*` in `kernels/fluids/mod.rs`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1153 tests pass, +19 since Group 3), `cargo fmt --check` all clean. No `#[allow]` suppressions added.

## 5. Dimensionless number kernels

- [x] 5.1 Implemented all 18 dimensionless-number kernels in `kernels/fluids/dimensionless.rs`: Reynolds, Mach, Froude, Weber, Prandtl, Peclet, Strouhal, Knudsen, Richardson, Rayleigh, Grashof, Eckert, Schmidt, Lewis, particle Stokes, Capillary, Bond, Nusselt. Each returns `Result<R, PhysicsError>` and errors cleanly on zero denominators or non-physical input (negative `g·L` for Froude). Typed inputs (`Speed`, `Length`, `Density`, `Viscosity`, `KinematicViscosity`) where the existing newtypes apply; raw `R` for non-newtyped physical quantities (thermal diffusivity, thermal conductivity, surface tension, specific heat, expansion coefficient, mass diffusivity, particle relaxation time, mean free path, heat transfer coefficient, gravity, frequency, ΔT).
- [x] 5.2 Added 41 tests in `tests/kernels/fluids/dimensionless_tests.rs`: one known-value test per number; the three load-bearing algebraic identities `Pe = Re · Pr`, `Ra = Gr · Pr`, `Le = Sc / Pr` verified to numerical tolerance; error-path coverage for every kernel (zero denominator, non-physical input); f32 precision sweep on Reynolds and the `Pe = Re · Pr` identity.
- [x] 5.3 Added 18 causal wrappers + 18 wrapper smoke tests + 1 error-path wrapper test in `wrappers_tests.rs`.
- [x] 5.4 Uncommented `pub use dimensionless::*` in `kernels/fluids/mod.rs`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1212 tests pass, +59 since Group 4), `cargo fmt --check` all clean. No `#[allow]` suppressions added.

## 6. Turbulence quantity kernels

- [x] 6.1 Implemented 9 turbulence kernels in `kernels/fluids/turbulence.rs`: `turbulent_kinetic_energy_kernel`, `dissipation_rate_kernel`, `kolmogorov_length_kernel`, `kolmogorov_time_kernel`, `kolmogorov_velocity_kernel`, `taylor_microscale_kernel`, `integral_length_scale_kernel`, `reynolds_stress_kernel`, `eddy_viscosity_boussinesq_kernel`. Length / Speed return types where physically meaningful (Kolmogorov length/velocity, Taylor microscale, integral length scale); `Viscosity` for eddy viscosity; `CauchyStress` for Reynolds stress. Boussinesq closure uses the standard least-squares contraction `ν_t = −(R^dev : S) / (2 · S : S)`.
- [x] 6.2 Added 23 turbulence_tests covering: TKE known value + non-negativity; dissipation rate known value, vanishing for rigid-body rotation, non-negativity; Kolmogorov identities `η · u_η / ν == 1` and `η / (u_η · τ_η) == 1`; Taylor microscale identity `λ² · ε == 15 · ν · k` (Block B5); integral length-scale known value; Reynolds-stress trace equals `2k`; Boussinesq eddy viscosity on a canonical simple-shear setup recovers a prescribed `ν_t = 0.05`; error paths for every kernel (zero ε, negative k, zero ν, zero strain, negative-ν_t result); f32 precision sweep on Kolmogorov and Taylor identities.
- [x] 6.3 Added 9 causal wrappers + 11 wrapper smoke tests (including error paths for `kolmogorov_length` and `eddy_viscosity_boussinesq`).
- [x] 6.4 Uncommented `pub use turbulence::*` in `kernels/fluids/mod.rs`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1248 tests pass, +36 since Group 5), `cargo fmt --check` all clean. No `#[allow]` suppressions added. `needless_range_loop` in the dissipation and eddy-viscosity tensor contractions fixed at root cause (unrolled the 3×3 sum-of-squares; replaced index loops with zipped row iterators).

## 7. Coherent-structure detector kernels

- [x] 7.1 Implemented 4 detector kernels in `kernels/fluids/coherent_structures.rs`: `q_criterion_kernel` (direct `−0.5·tr(G²)` form), `delta_criterion_kernel` (generalized to the depressed-cubic discriminant per Chakraborty et al. 2005 — correct for both incompressible and compressible flow), `lambda2_kernel` (Jeong–Hussain middle eigenvalue of `S² + Ω²`), `swirling_strength_kernel` (Cardano on the depressed velocity-gradient characteristic polynomial). Private helpers: a closed-form Smith (1961) 3×3 symmetric eigenvalue routine and a sign-preserving real cube root — both allocation-free, no_std-friendly, generic over `R: RealField + FromPrimitive`.
- [x] 7.2 Added 18 coherent_structures_tests covering: spec scenario `Q + 0.5·‖S‖² − 0.5·‖Ω‖² = 0` (f64 and f32 precision); Q sign behavior on rigid-body rotation (positive) vs pure strain (negative); Δ > 0 for rotational flow, Δ < 0 for three distinct real eigenvalues, Δ = 0 for repeated real eigenvalues (boundary case); spec scenario λ₂ < 0 for rigid-body rotation; λ₂ on pure isotropic extension equals 1; λ₂ on diagonal strain matches the middle eigenvalue exactly; spec scenario swirling strength = 0 in irrotational (symmetric-gradient) flow; swirling strength = ω for rigid-body rotation; cross-check that Δ sign and λ_ci sign are consistent across representative inputs. **Δ-criterion bug caught during testing** — the original `(Q/3)³ + (R/2)²` form is only correct for incompressible flow; fixed the kernel to use the depressed-cubic discriminant `(p̃/3)³ + (q̃/2)²` and documented the convention.
- [x] 7.3 Added 4 causal wrappers + 4 wrapper smoke tests.
- [x] 7.4 Uncommented `pub use coherent_structures::*`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1270 tests pass, +22 since Group 6), `cargo fmt --check` all clean. No `#[allow]` suppressions added.

## 8. Compressible-flow thermodynamic kernels

- [x] 8.1 Implemented 6 compressible-flow thermodynamic kernels in `kernels/fluids/compressible.rs`: `speed_of_sound_ideal_gas_kernel`, `specific_enthalpy_kernel`, `total_enthalpy_kernel`, `total_pressure_isentropic_kernel`, `total_temperature_isentropic_kernel`, `entropy_production_rate_kernel`. Entropy production kernel includes both viscous-dissipation (`Φ/T`) and heat-conduction (`κ‖∇T‖²/T²`) contributions per the Clausius-Duhem inequality; errors on `T ≤ 0`. Isentropic kernels error on `γ ≤ 1`.
- [x] 8.2 Added 14 compressible_tests covering: speed of sound for air at 293.15 K (matches `(γ·R_s·T)^(1/2) ≈ 343 m/s` to within 0.1 m/s); zero-temperature and negative-γ error paths; specific enthalpy known value `c_p·T = 301,500 J/kg` and zero at zero Kelvin; total enthalpy at rest equals static; kinetic-energy contribution of total enthalpy; **spec scenario** `T_0 = T` at `M = 0`; total temperature at `M = 1` for air `T_0/T = 1.2`; γ ≤ 1 error paths for both isentropic kernels; total pressure at `M = 1` for air `p_0/p = 1.2^3.5 ≈ 1.893`; **spec scenario** entropy production ≥ 0 on a simple-shear Newtonian setup (verified against the closed-form `Φ = μγ̇²`); heat-conduction-only contribution at rest; zero-temperature error path; f32 precision sweep on speed of sound and total temperature.
- [x] 8.3 Added 6 causal wrappers + 8 wrapper tests (including error paths for speed of sound at T = 0 and entropy production at T = 0).
- [x] 8.4 Uncommented `pub use compressible::*` in `kernels/fluids/mod.rs`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1296 tests pass, +26 since Group 7), `cargo fmt --check` all clean. No `#[allow]` suppressions added.

## 9. Boundary-layer kernels

- [x] 9.1 Implemented 7 boundary-layer kernels in `kernels/fluids/boundary_layer.rs`: `wall_shear_stress_newtonian_kernel` (returns magnitude via `WallShearStress::new_unchecked` after explicit abs), `friction_velocity_kernel`, `viscous_length_scale_kernel`, `y_plus_kernel`, `viscous_sublayer_velocity_kernel` (identity `u⁺ = y⁺`), `log_law_velocity_kernel` (κ and B as caller inputs so the reference values can be picked per dataset), `skin_friction_coefficient_kernel`. `κ ≈ 0.41`, `B ≈ 5.0` documented as standard.
- [x] 9.2 Added 19 boundary_layer_tests covering: wall-shear known value, magnitude semantics, zero gradient; friction velocity known value and zero-density error; viscous length scale and its zero-u_τ error; **spec scenario** `y⁺` scales linearly with `y`; y⁺ known value; zero-ν error; **spec scenario** viscous sublayer law is identity; log law at `y⁺ = 100` with κ=0.41, B=5.0 recovers `≈16.231` to within `0.01`; log-law errors on `y⁺ ≤ 0` and `κ = 0`; **spec scenario** sublayer and log law differ in the buffer region (`y⁺ = 11.5`); skin friction known value and both error paths; f32 precision sweeps on y⁺ linearity and log-law value.
- [x] 9.3 Added 7 causal wrappers + 9 wrapper tests (including error paths for friction velocity and log law).
- [x] 9.4 Uncommented `pub use boundary_layer::*` in `kernels/fluids/mod.rs`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1326 tests pass, +30 since Group 8), `cargo fmt --check` all clean. No `#[allow]` suppressions added.

## 10. Ideal-flow primitive kernels

- [x] 10.1 Implemented 6 ideal-flow primitive kernels in `kernels/fluids/ideal_flow.rs`: `dynamic_pressure_kernel`, `bernoulli_total_head_kernel` (uses standard gravity `G = 9.80665 m/s²`; errors on `ρ = 0`), `stream_function_2d_kernel` and `velocity_potential_2d_kernel` (pure-scalar differential updates, infallible), `circulation_kernel` (line integral over a sampled loop; **spec-relaxed to `Result`** to surface length-mismatch errors explicitly), `kutta_joukowski_lift_kernel`.
- [x] 10.2 Added 18 ideal_flow_tests: dynamic pressure known value at sea-level air (q = 245 Pa), **spec scenario** "quadratic scaling with speed" (verified at k ∈ {0.5, 1, 2, 5}), zero-velocity case; Bernoulli head known value (free-fall conversion u² → 2gH), zero-density error, static-term-only sanity (`H = p/(ρg) = 10 m` for `p = 98066.5 Pa`); stream function and velocity potential known values; circulation = 0 for uniform flow on a closed square loop (Stokes' theorem); circulation = 4·ω·R for rigid-body rotation on a 4-point unit-radius loop; circulation length-mismatch error; circulation = 0 on empty loop; **spec scenario** Kutta–Joukowski lift = 0 at zero circulation (exact); Kutta–Joukowski known value `L' = ρ·u_∞·Γ = 612.5 N/m`; sign-follows-circulation symmetry test; f32 precision sweep on dynamic-pressure quadratic scaling.
- [x] 10.3 Added 6 causal wrappers + 8 wrapper tests (including error paths for Bernoulli head at ρ = 0 and circulation at length mismatch).
- [x] 10.4 Uncommented `pub use ideal_flow::*` in `kernels/fluids/mod.rs`. `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (1350 tests pass, +24 since Group 9), `cargo fmt --check` all clean. No `#[allow]` suppressions added.

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
