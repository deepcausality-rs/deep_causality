## ADDED Requirements

### Requirement: Typed vector and tensor newtype family

The crate SHALL expose the following vector and rank-2 tensor newtypes under `deep_causality_physics::kernels::fluids::quantities`, alongside the existing scalar newtypes. Each typed wrapper SHALL be generic over `R: RealField`, SHALL carry its documented invariant at construction (via `new(raw) -> Result<Self, PhysicsError>`), and SHALL provide `new_unchecked`, `value() -> &…`, `into_inner() -> …`, `impl From<Self> for [raw]` (always, drops the invariant), `Default`, `Debug`, `Clone`, `Copy`, `PartialEq`.

`impl From<[raw]> for Self` SHALL be provided **only when the type's invariant is finiteness alone** (the four vector newtypes and `VelocityGradient`). For invariant-bearing tensors (`StrainRateTensor`, `RotationRateTensor`, `CauchyStress`), `From<[[R; 3]; 3]> for Self` SHALL NOT exist — a silent bypass of the symmetry / antisymmetry invariant via `From` would defeat the purpose of the type. Callers with externally-supplied raw input SHALL use `new(raw)` (checked) or `new_unchecked(raw)` (explicit invariant bypass, visible at the call site).

**Vector newtypes (`[R; 3]` wrappers, finiteness-checked):**

- `Velocity3<R>` — fluid velocity vector (m/s).
- `VorticityVector<R>` — vorticity vector `ω = ∇ × u` (1/s); semantically a pseudovector that flips sign under spatial reflection.
- `AccelerationVector<R>` — acceleration (m/s²); the return type of momentum-equation RHS evaluators.
- `BodyForceDensity<R>` — body force per unit volume (N/m³).

**Rank-2 tensor newtypes (`[[R; 3]; 3]` wrappers):**

- `VelocityGradient<R>` — pins the Jacobian convention `[i][j] = ∂u_i/∂x_j` at construction. Finiteness-checked.
- `StrainRateTensor<R>` — symmetric tensor `S = 0.5·(∇u + ∇uᵀ)`. Construction-time check: `|S_ij − S_ji| ≤ ε` for all `i ≠ j`.
- `RotationRateTensor<R>` — antisymmetric tensor `Ω = 0.5·(∇u − ∇uᵀ)`. Construction-time check: `|Ω_ij + Ω_ji| ≤ ε` for all `i, j`.
- `CauchyStress<R>` — symmetric stress tensor (Pa), positive-in-tension sign convention. Construction-time check: symmetric.

Raw `[R; 3]` SHALL continue to be used for gradients of scalar fields (`grad_p`, `grad_rho`, `grad_phi`, `grad_T`) and for component-wise Laplacian results (`laplacian_u`, `laplacian_omega`). Raw `[[R; 3]; 3]` SHALL continue to be used for the velocity gradient of non-velocity vector fields (e.g. `grad_omega = ∂ω_i/∂x_j`), which appears in only one kernel and whose convention is pinned by docstring at the call site.

#### Scenario: VelocityGradient::new accepts a finite Jacobian matrix

- **WHEN** `VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [0.0, -1.0, 0.5], [4.0, 0.0, 0.0]])` is called
- **THEN** the call SHALL return `Ok(_)` and `into_inner()` SHALL recover the original matrix component-for-component

#### Scenario: StrainRateTensor::new rejects an asymmetric matrix

- **WHEN** `StrainRateTensor::<f64>::new(...)` is called with a matrix whose `[0][1]` and `[1][0]` entries differ by more than the construction-time symmetry tolerance
- **THEN** the call SHALL return `Err(PhysicsError::PhysicalInvariantBroken(_))` mentioning symmetry

#### Scenario: RotationRateTensor::new rejects a non-antisymmetric matrix

- **WHEN** `RotationRateTensor::<f64>::new(...)` is called with a matrix whose `[0][1] + [1][0]` exceeds the construction-time antisymmetry tolerance
- **THEN** the call SHALL return `Err(PhysicsError::PhysicalInvariantBroken(_))` mentioning antisymmetry

#### Scenario: From/Into round-trip preserves the raw representation

- **WHEN** a `Velocity3::<f64>::new([1.0, 2.0, 3.0])?` is converted to `[f64; 3]` via `into_inner()` and back via `Velocity3::from(...)`
- **THEN** the resulting `Velocity3` SHALL equal the original component-for-component

### Requirement: Pointwise kernel surface for governing equations

The crate SHALL expose pointwise, stateless, side-effect-free free functions under `deep_causality_physics::kernels::fluids::governing` that evaluate the RHS contributions of the classical conservation laws of fluid mechanics. Every kernel SHALL be generic over `R: RealField` (with `+ FromPrimitive` where literals are required) and SHALL NOT accept any non-algebraic input (no manifold, no context, no state). All typed vector and tensor inputs SHALL use the newtypes from `quantities`.

The surface SHALL include at minimum:

- `convective_acceleration_kernel<R>(u: &Velocity3<R>, grad_u: &VelocityGradient<R>) -> AccelerationVector<R>` returning `(u · ∇) u`.
- `viscous_diffusion_kernel<R>(nu: &KinematicViscosity<R>, laplacian_u: &[R; 3]) -> AccelerationVector<R>` returning `ν ∇²u`.
- `pressure_gradient_force_kernel<R>(rho: &Density<R>, grad_p: &[R; 3]) -> Result<AccelerationVector<R>, PhysicsError>` returning `−(1/ρ) ∇p` and erroring on `ρ ≤ 0`.
- `continuity_rhs_kernel<R>(rho: &Density<R>, u: &Velocity3<R>, grad_rho: &[R; 3], div_u: R) -> R` returning the RHS of `∂ρ/∂t = −∇·(ρu) = −(u·∇ρ + ρ ∇·u)`.
- `vorticity_transport_kernel<R>(omega: &VorticityVector<R>, u: &Velocity3<R>, grad_omega: &[[R; 3]; 3], laplacian_omega: &[R; 3], nu: &KinematicViscosity<R>) -> AccelerationVector<R>` returning `−(u·∇)ω + (ω·∇)u + ν∇²ω`. (Output type is `AccelerationVector` because it carries units of `(1/s)·(1/s) = 1/s²`, dimensionally identical to acceleration after vorticity scaling.)
- `scalar_advection_diffusion_kernel<R>(u: &Velocity3<R>, grad_phi: &[R; 3], laplacian_phi: R, diffusivity: R, source: R) -> R` returning `−u·∇φ + D ∇²φ + S`.
- `energy_rhs_kernel<R>(...)` for the compressible energy equation expressed in total-energy form.

#### Scenario: Convective acceleration is Galilean invariant

- **WHEN** `convective_acceleration_kernel(u, grad_u)` is called with any `Velocity3<R>` and `VelocityGradient<R>`, then called again with `Velocity3::new(u.into_inner() + c)` for any constant velocity offset `c: [R; 3]` and the same `grad_u`
- **THEN** both invocations SHALL return `AccelerationVector` values whose component-wise difference is at or below the precision backend's tolerance for `R ∈ {f32, f64, Float106}`

#### Scenario: Pressure gradient force errors on non-positive density

- **WHEN** `pressure_gradient_force_kernel(&Density::new(rho)?, &grad_p)` is invoked
- **THEN** a `Density::new(rho)` call with `rho ≤ 0` SHALL itself return `Err(PhysicsError::PhysicalInvariantBroken(_))`, and the kernel SHALL never observe a non-positive density at runtime

#### Scenario: Continuity equation reduces to incompressible divergence-free condition

- **WHEN** `continuity_rhs_kernel(&Density::new(rho)?, u, &[zero; 3], div_u)` is called with `rho` constant in space (`grad_rho = 0`) and `div_u = 0`
- **THEN** the returned RHS SHALL be exactly zero, reproducing `∂ρ/∂t = 0` for incompressible flow

#### Scenario: Vorticity transport reduces to Euler vortex stretching in inviscid limit

- **WHEN** `vorticity_transport_kernel` is invoked with `KinematicViscosity::new(0.0)`
- **THEN** the returned RHS SHALL equal `−(u·∇)ω + (ω·∇)u` to within precision tolerance, recovering the inviscid vorticity equation

### Requirement: Constitutive kernels for viscous stress

The crate SHALL expose kernels under `deep_causality_physics::kernels::fluids::constitutive` that evaluate the viscous stress tensor for Newtonian and power-law non-Newtonian fluids.

- `newtonian_viscous_stress_kernel<R>(mu: &Viscosity<R>, strain_rate: &StrainRateTensor<R>, div_u: R) -> CauchyStress<R>` returning `τ = 2μS − (2/3)μ(∇·u)I` (Stokes hypothesis: bulk viscosity = 0). Return type is `CauchyStress` because the viscous stress tensor is symmetric and follows the continuum-mechanics sign convention.
- `newtonian_viscous_stress_with_bulk_kernel<R>(mu, zeta, strain_rate, div_u) -> CauchyStress<R>` returning `τ = 2μS − (2/3)μ(∇·u)I + ζ(∇·u)I`.
- `power_law_apparent_viscosity_kernel<R>(consistency: R, flow_index: R, shear_rate: R) -> Result<Viscosity<R>, PhysicsError>` returning `μ_eff = K · γ̇^(n−1)`, erroring on `shear_rate < 0`.

Signs SHALL follow the continuum-mechanics convention: stress positive in tension.

#### Scenario: Newtonian stress vanishes in rigid-body motion

- **WHEN** `newtonian_viscous_stress_kernel` is called with a strain-rate tensor `S = 0` (rigid-body motion) and `div_u = 0`
- **THEN** the returned `CauchyStress` SHALL be the zero tensor to within precision tolerance

#### Scenario: Stokes hypothesis is the bulk-viscosity-zero special case

- **WHEN** `newtonian_viscous_stress_with_bulk_kernel(mu, zeta, S, div_u)` is called with `zeta = 0`
- **THEN** the result SHALL equal `newtonian_viscous_stress_kernel(mu, S, div_u)` to within precision tolerance

#### Scenario: Power-law reduces to Newtonian at flow_index = 1

- **WHEN** `power_law_apparent_viscosity_kernel(K, 1.0, gamma_dot)` is called for any non-negative `gamma_dot`
- **THEN** the returned apparent viscosity SHALL equal `K` to within precision tolerance, recovering the Newtonian limit

### Requirement: Kinematic kernels

The crate SHALL expose kinematic kernels under `deep_causality_physics::kernels::fluids::kinematics`:

- `strain_rate_tensor_kernel<R>(grad_u: &VelocityGradient<R>) -> StrainRateTensor<R>` returning `S = 0.5·(∇u + ∇uᵀ)`. Symmetric by construction; `new_unchecked` is acceptable internally because the algebra guarantees the invariant.
- `rotation_rate_tensor_kernel<R>(grad_u: &VelocityGradient<R>) -> RotationRateTensor<R>` returning `Ω = 0.5·(∇u − ∇uᵀ)`. Antisymmetric by construction.
- `vorticity_from_gradient_kernel<R>(grad_u: &VelocityGradient<R>) -> VorticityVector<R>` returning `ω = ∇ × u` from the antisymmetric part of `∇u`.
- `velocity_gradient_invariants_kernel<R>(grad_u: &VelocityGradient<R>) -> (R, R, R)` returning `(P, Q, R)` invariants of the velocity gradient tensor in the Chong–Perry–Cantwell (1990) convention.
- `helicity_density_kernel<R>(u: &Velocity3<R>, omega: &VorticityVector<R>) -> R` returning `h = u · ω`.
- `enstrophy_density_kernel<R>(omega: &VorticityVector<R>) -> R` returning `0.5 · ‖ω‖²`.

#### Scenario: Strain and rotation decomposition reconstructs the gradient

- **WHEN** `strain_rate_tensor_kernel(grad_u)` and `rotation_rate_tensor_kernel(grad_u)` are summed component-wise (via `into_inner()`) for any `grad_u: VelocityGradient<R>`
- **THEN** the result SHALL equal `grad_u.into_inner()` to within precision tolerance

#### Scenario: Helicity density flips sign under spatial reflection

- **WHEN** a velocity `u: Velocity3<R>` and its vorticity `ω: VorticityVector<R>` are reflected along one axis (flipping the appropriate components)
- **THEN** `helicity_density_kernel` SHALL return a value whose sign is opposite to the unreflected case

#### Scenario: Enstrophy density is non-negative

- **WHEN** `enstrophy_density_kernel(&omega)` is called with any `omega: VorticityVector<R>`
- **THEN** the returned scalar SHALL be `≥ 0` exactly (algebraic identity, not tolerance-bound)

### Requirement: Dimensionless number kernels

The crate SHALL expose kernels under `deep_causality_physics::kernels::fluids::dimensionless` computing the following dimensionless numbers: Reynolds, Mach, Froude, Weber, Prandtl, Peclet, Strouhal, Knudsen, Richardson, Rayleigh, Grashof, Eckert, Schmidt, Lewis, Stokes (particle Stokes), Capillary, Bond, Nusselt. Each kernel SHALL return `Result<R, PhysicsError>` where appropriate (erroring on zero denominators or non-physical inputs) or `R` when the formula has no failure mode.

Each kernel SHALL document its formula with units in the docstring.

#### Scenario: Reynolds number is correctly composed from kinematic viscosity

- **WHEN** `reynolds_number_kernel(&Speed::new(u)?, &Length::new(L)?, &KinematicViscosity::new(nu)?)` is called
- **THEN** the returned value SHALL equal `u · L / nu` to within precision tolerance, and the kernel SHALL return `Err(PhysicsError::PhysicalInvariantBroken(_))` if `nu ≤ 0` via the `KinematicViscosity::new` constructor

#### Scenario: Peclet equals Reynolds times Prandtl

- **WHEN** `peclet_number_kernel`, `reynolds_number_kernel`, and `prandtl_number_kernel` are evaluated on a consistent set of inputs
- **THEN** the relation `Pe = Re · Pr` SHALL hold to within precision tolerance

#### Scenario: Lewis number equals Schmidt over Prandtl

- **WHEN** all three numbers are computed for the same fluid
- **THEN** `Le = Sc / Pr` SHALL hold to within precision tolerance

### Requirement: Turbulence quantities

The crate SHALL expose kernels under `deep_causality_physics::kernels::fluids::turbulence`:

- `turbulent_kinetic_energy_kernel<R>(u_prime: &Velocity3<R>) -> R` returning `k = 0.5 · (u'·u')`.
- `dissipation_rate_kernel<R>(nu: &KinematicViscosity<R>, grad_u_prime: &VelocityGradient<R>) -> R` returning `ε = 2ν · S':S'` (or equivalent gradient form documented in the kernel).
- `kolmogorov_length_kernel<R>(nu, epsilon) -> Result<Length<R>, PhysicsError>` returning `η = (ν³/ε)^(1/4)`.
- `kolmogorov_time_kernel<R>(nu, epsilon) -> Result<R, PhysicsError>` returning `τ_η = (ν/ε)^(1/2)`.
- `kolmogorov_velocity_kernel<R>(nu, epsilon) -> Result<Speed<R>, PhysicsError>` returning `u_η = (νε)^(1/4)`.
- `taylor_microscale_kernel<R>(k, epsilon, nu) -> Result<Length<R>, PhysicsError>` returning `λ = √(15 ν k / ε)`.
- `integral_length_scale_kernel<R>(k, epsilon) -> Result<Length<R>, PhysicsError>` returning `L = k^(3/2) / ε`.
- `reynolds_stress_kernel<R>(u_prime_outer_u_prime: &StrainRateTensor<R>) -> CauchyStress<R>` returning the Reynolds-stress tensor `R_ij = u'_i u'_j` (already-averaged input; the input is symmetric by physical construction, output is the corresponding stress).
- `eddy_viscosity_boussinesq_kernel<R>(reynolds_stress: &CauchyStress<R>, strain_rate_mean: &StrainRateTensor<R>, k: R) -> Result<Viscosity<R>, PhysicsError>` returning the scalar eddy viscosity that closes the Boussinesq hypothesis at the given strain.

#### Scenario: Kolmogorov scales recover the standard ν/ε scaling

- **WHEN** `kolmogorov_length_kernel`, `kolmogorov_time_kernel`, and `kolmogorov_velocity_kernel` are called on the same `(ν, ε)`
- **THEN** the products SHALL satisfy `η · u_η / ν = 1` and `η / (u_η · τ_η) = 1` to within precision tolerance

#### Scenario: Taylor and integral scales satisfy the algebraic identity from B5

- **WHEN** `taylor_microscale_kernel` and `integral_length_scale_kernel` are evaluated on the same `(k, ε, ν)`
- **THEN** the identity `λ² · ε = 15 · ν · k` SHALL hold to within precision tolerance

#### Scenario: Dissipation rate is non-negative

- **WHEN** `dissipation_rate_kernel(&nu, &grad_u_prime)` is called with any positive `nu` and any `VelocityGradient`
- **THEN** the returned `ε` SHALL be `≥ 0` to within precision tolerance

### Requirement: Coherent-structure detector kernels

The crate SHALL expose four coherent-structure detector kernels under `deep_causality_physics::kernels::fluids::coherent_structures`:

- `q_criterion_kernel<R>(grad_u: &VelocityGradient<R>) -> R` returning `Q = 0.5 · (‖Ω‖² − ‖S‖²)`.
- `lambda2_kernel<R>(grad_u: &VelocityGradient<R>) -> R` returning the second-largest eigenvalue of `S² + Ω²` (Jeong–Hussain criterion).
- `delta_criterion_kernel<R>(grad_u: &VelocityGradient<R>) -> R` returning `Δ = (Q/3)³ + (R/2)²` from the velocity gradient invariants.
- `swirling_strength_kernel<R>(grad_u: &VelocityGradient<R>) -> R` returning `λ_ci`, the imaginary part of the complex eigenvalue pair of `∇u` when one exists, and zero otherwise.

These kernels SHALL satisfy the B5 extraction-equivalence test. Block B5 of `3DCausalFluidDynamics.md` publishes raw-array signatures; this typed surface interoperates via `VelocityGradient::from([[R; 3]; 3])` at the call site.

#### Scenario: Q-criterion satisfies the algebraic identity in the docstring

- **WHEN** `q_criterion_kernel(grad_u)`, `strain_rate_tensor_kernel(grad_u)`, and `rotation_rate_tensor_kernel(grad_u)` are evaluated
- **THEN** the identity `Q + 0.5 · ‖S‖² − 0.5 · ‖Ω‖² = 0` SHALL hold to within precision tolerance for `R ∈ {f32, f64, Float106}`

#### Scenario: Lambda2 is negative inside a known vortex tube

- **WHEN** `lambda2_kernel` is evaluated on the analytical velocity gradient of a Burgers vortex at a point inside its core
- **THEN** the returned value SHALL be strictly negative

#### Scenario: Swirling strength vanishes in irrotational flow

- **WHEN** `swirling_strength_kernel` is called on a `VelocityGradient` whose vorticity is zero (irrotational flow)
- **THEN** the returned `λ_ci` SHALL be zero to within precision tolerance

### Requirement: Compressible-flow thermodynamic kernels

The crate SHALL expose kernels under `deep_causality_physics::kernels::fluids::compressible`:

- `speed_of_sound_ideal_gas_kernel<R>(gamma: R, R_specific: R, temperature: &Temperature<R>) -> Result<Speed<R>, PhysicsError>` returning `a = √(γ R_s T)`.
- `specific_enthalpy_kernel<R>(cp: R, temperature: &Temperature<R>) -> SpecificEnthalpy<R>` returning `h = c_p T`.
- `total_enthalpy_kernel<R>(h: &SpecificEnthalpy<R>, u: &Velocity3<R>) -> SpecificEnthalpy<R>` returning `h_0 = h + 0.5·‖u‖²`.
- `total_pressure_isentropic_kernel<R>(p: &Pressure<R>, mach: R, gamma: R) -> Result<Pressure<R>, PhysicsError>` returning `p_0 = p · (1 + (γ−1)/2 · M²)^(γ/(γ−1))`.
- `total_temperature_isentropic_kernel<R>(T: &Temperature<R>, mach: R, gamma: R) -> Result<Temperature<R>, PhysicsError>` returning `T_0 = T · (1 + (γ−1)/2 · M²)`.
- `entropy_production_rate_kernel<R>(...)` returning the local entropy-production density `σ ≥ 0` for a Newtonian fluid.

#### Scenario: Total temperature equals static temperature at zero Mach

- **WHEN** `total_temperature_isentropic_kernel(&T, 0.0, gamma)` is called
- **THEN** the returned temperature SHALL equal `T` to within precision tolerance

#### Scenario: Entropy production is non-negative

- **WHEN** `entropy_production_rate_kernel` is evaluated on any physically valid input (positive temperatures and densities, finite gradients)
- **THEN** the returned value SHALL be `≥ 0` to within precision tolerance

### Requirement: Boundary-layer kernels

The crate SHALL expose kernels under `deep_causality_physics::kernels::fluids::boundary_layer`:

- `wall_shear_stress_newtonian_kernel<R>(mu: &Viscosity<R>, du_dy_wall: R) -> WallShearStress<R>` returning `τ_w = μ · |∂u/∂y|_wall` (magnitude).
- `friction_velocity_kernel<R>(tau_w: &WallShearStress<R>, rho: &Density<R>) -> Result<Speed<R>, PhysicsError>` returning `u_τ = √(τ_w / ρ)`.
- `viscous_length_scale_kernel<R>(nu: &KinematicViscosity<R>, u_tau: &Speed<R>) -> Result<Length<R>, PhysicsError>` returning `δ_ν = ν / u_τ`.
- `y_plus_kernel<R>(y: &Length<R>, u_tau: &Speed<R>, nu: &KinematicViscosity<R>) -> Result<R, PhysicsError>` returning `y⁺ = y · u_τ / ν`.
- `viscous_sublayer_velocity_kernel<R>(y_plus: R) -> R` returning `u⁺ = y⁺` (valid for `y⁺ ≲ 5`).
- `log_law_velocity_kernel<R>(y_plus: R, kappa: R, B: R) -> Result<R, PhysicsError>` returning `u⁺ = (1/κ)·ln(y⁺) + B` (valid for `30 ≲ y⁺ ≲ 300`).
- `skin_friction_coefficient_kernel<R>(tau_w, rho, u_inf) -> Result<R, PhysicsError>` returning `C_f = τ_w / (0.5 · ρ · u_∞²)`.

#### Scenario: y⁺ scales linearly with wall distance

- **WHEN** `y_plus_kernel` is called with `y = k · y_ref` for any positive scalar `k`
- **THEN** the returned `y⁺` SHALL equal `k · y_plus_ref` to within precision tolerance

#### Scenario: Viscous sublayer law equals log law at no specific point

- **WHEN** the viscous sublayer and log laws are compared at `y⁺ = 11.5` (the buffer-layer transition region)
- **THEN** the two laws SHALL give different velocities; this scenario documents that the kernels do not impose a unified law-of-the-wall and that the caller selects the appropriate one for the `y⁺` range

### Requirement: Ideal-flow primitive kernels

The crate SHALL expose kernels under `deep_causality_physics::kernels::fluids::ideal_flow`:

- `dynamic_pressure_kernel<R>(rho: &Density<R>, u: &Speed<R>) -> Pressure<R>` returning `q = 0.5 · ρ · u²`.
- `bernoulli_total_head_kernel<R>(p: &Pressure<R>, rho: &Density<R>, u: &Speed<R>, h: &Length<R>) -> Result<Length<R>, PhysicsError>` returning `H = p/(ρg) + u²/(2g) + h`.
- `stream_function_2d_kernel<R>(u: R, v: R, dx: R, dy: R) -> R` returning the differential update `dψ = u·dy − v·dx` (caller integrates along a path).
- `velocity_potential_2d_kernel<R>(u: R, v: R, dx: R, dy: R) -> R` returning `dφ = u·dx + v·dy`.
- `circulation_kernel<R>(velocity_at_loop_points: &[Velocity3<R>], tangents: &[[R; 3]]) -> R` returning the discrete line integral `Γ = ∮ u·dl`.
- `kutta_joukowski_lift_kernel<R>(rho: &Density<R>, u_inf: &Speed<R>, circulation: R) -> R` returning `L' = ρ · u_∞ · Γ`.

#### Scenario: Dynamic pressure scales quadratically with speed

- **WHEN** `dynamic_pressure_kernel` is evaluated at `(ρ, u)` and again at `(ρ, k·u)` for any positive `k`
- **THEN** the second result SHALL equal `k² ·` the first to within precision tolerance

#### Scenario: Kutta-Joukowski lift vanishes at zero circulation

- **WHEN** `kutta_joukowski_lift_kernel(&rho, &u_inf, 0.0)` is called
- **THEN** the returned lift SHALL be exactly zero (algebraic, not tolerance-bound)

### Requirement: Precision robustness across backends

Every kernel introduced by this change SHALL compile and pass its property tests when instantiated with each of `f32`, `f64`, and `Float106`. Per-kernel tolerance constants SHALL be defined in the corresponding test module, generic over the precision backend, matching the existing pattern in `kernels/em/` and `kernels/relativity/`.

#### Scenario: Q-criterion identity holds across precision backends

- **WHEN** the Q-criterion algebraic identity test is run with `R ∈ {f32, f64, Float106}`
- **THEN** for each backend, the identity SHALL hold within the backend's documented tolerance

### Requirement: Causal wrappers shadow every kernel

For every `*_kernel` introduced under `deep_causality_physics::kernels::fluids::<group>`, the crate SHALL provide a corresponding wrapper function under `deep_causality_physics::kernels::fluids::wrappers` that has the same input signature, returns `PropagatingEffect<T>` where `T` is the kernel's output type, and lifts a successful kernel call via `PropagatingEffect::pure` and an error via `PropagatingEffect::from_error(CausalityError::from(physics_error))`. Infallible kernels (return type not `Result<…>`) are wrapped via a direct `PropagatingEffect::pure`.

#### Scenario: Wrapper lifts kernel success into PropagatingEffect::Value

- **WHEN** a kernel returns `Ok(value)` (or its infallible value) and its wrapper is invoked with the same inputs
- **THEN** the wrapper SHALL return a `PropagatingEffect` whose `value` field is `EffectValue::Value(value)`

#### Scenario: Wrapper lifts kernel error into PropagatingEffect error channel

- **WHEN** a kernel returns `Err(PhysicsError)` and its wrapper is invoked with the same inputs
- **THEN** the wrapper SHALL return a `PropagatingEffect` whose error channel carries a `CausalityError` constructed from the underlying `PhysicsError`

### Requirement: Test discipline and AGENTS.md conformance

Every new source file SHALL achieve 100% test coverage per AGENTS.md §"Code testing". Tests SHALL live under `deep_causality_physics/tests/kernels/fluids/<group>_tests.rs` mirroring the src tree, with each test file registered in its parent `mod.rs` and in `deep_causality_physics/tests/BUILD.bazel`. No `#[allow(dead_code)]` or `#[allow(clippy::...)]` suppressions are permitted to close coverage or lint gates.

#### Scenario: Coverage tooling reports 100% on every new src file

- **WHEN** the project's coverage tooling is run after this change ships
- **THEN** every new source file under `deep_causality_physics/src/kernels/fluids/` introduced by this change SHALL report 100% line coverage, or SHALL have any unreachable code explicitly justified per AGENTS.md §"Code testing"
