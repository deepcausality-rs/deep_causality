## ADDED Requirements

### Requirement: Coherent regime assemblies under `theories/fluid_dynamics`

The crate SHALL expose a `deep_causality_physics::theories::fluid_dynamics` module that assembles the pointwise kernels from `fluid-dynamics-kernels` into the four classical regimes of Navier–Stokes fluid mechanics. Each regime SHALL be expressed as one or more free functions taking algebraic inputs and returning the pointwise RHS of the corresponding evolution equation in `∂/∂t = …` form. No regime function SHALL re-implement algebra that a kernel from `fluid-dynamics-kernels` already exposes; each regime SHALL compose kernels.

The four regimes SHALL each live in their own submodule (`incompressible_ns`, `compressible_ns`, `euler`, `stokes`) and SHALL be re-exported from `theories/fluid_dynamics/mod.rs`.

All regime functions SHALL be generic over `R: RealField` (with `+ FromPrimitive` where literals appear), SHALL NOT use `unsafe` / `dyn` / macros, and SHALL match the existing crate convention of returning `Result<T, PhysicsError>` where any kernel they compose can fail.

#### Scenario: Regime functions compose kernels, never re-derive algebra

- **WHEN** the implementation of any function under `theories/fluid_dynamics/` is inspected
- **THEN** every algebraic operation that corresponds to a published kernel from `fluid-dynamics-kernels` SHALL invoke that kernel rather than reimplement the formula inline

### Requirement: Incompressible Newtonian Navier–Stokes regime

The `theories::fluid_dynamics::incompressible_ns` module SHALL expose a function with substantively the following signature:

```rust
pub fn incompressible_ns_rhs_kernel<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    laplacian_u: &[R; 3],
    grad_p: &[R; 3],
    rho: &Density<R>,
    nu: &KinematicViscosity<R>,
    body_force: &BodyForceDensity<R>,
) -> Result<AccelerationVector<R>, PhysicsError>
where R: RealField + FromPrimitive;
```

It SHALL return the pointwise RHS of the incompressible momentum equation `∂u/∂t = −(u·∇)u − (1/ρ)∇p + ν∇²u + f` as an `AccelerationVector<R>`, composed from `convective_acceleration_kernel`, `pressure_gradient_force_kernel`, `viscous_diffusion_kernel`, and a body-force addition. The `BodyForceDensity<R>` input carries N/m³; combining with `1/ρ` produces an acceleration contribution.

#### Scenario: Inviscid limit recovers Euler

- **WHEN** `incompressible_ns_rhs_kernel` is invoked with `KinematicViscosity::new(0.0)`
- **THEN** the returned RHS SHALL equal the result of `euler::euler_momentum_rhs_kernel` invoked on the same `(u, grad_u, grad_p, rho, body_force)` inputs, to within precision tolerance

#### Scenario: Creeping-flow limit recovers Stokes

- **WHEN** `incompressible_ns_rhs_kernel` is invoked with `u = Velocity3::default()` (zero vector) and `grad_u = VelocityGradient::default()` (zero matrix) — zero convective contribution by construction
- **THEN** the returned RHS SHALL equal the result of `stokes::stokes_momentum_rhs_kernel` invoked on the same `(laplacian_u, grad_p, rho, nu, body_force)` inputs, to within precision tolerance

#### Scenario: Body-force injection is additive and linear

- **WHEN** `incompressible_ns_rhs_kernel` is called twice with the same hydrodynamic inputs but two different body forces `f_1` and `f_2`
- **THEN** the difference of the two returned RHS values SHALL equal `f_2 − f_1` to within precision tolerance

### Requirement: Compressible Navier–Stokes regime

The `theories::fluid_dynamics::compressible_ns` module SHALL expose three functions covering the three conservation laws of compressible Newtonian flow:

- `compressible_ns_continuity_rhs_kernel<R>(...)` returning the RHS of `∂ρ/∂t = −∇·(ρu)`, composed from `continuity_rhs_kernel`.
- `compressible_ns_momentum_rhs_kernel<R>(...)` returning the RHS of the compressible momentum equation including the Stokes-hypothesis viscous-stress divergence, composed from `convective_acceleration_kernel`, `pressure_gradient_force_kernel`, and a stress-divergence term computed from `newtonian_viscous_stress_kernel`.
- `compressible_ns_energy_rhs_kernel<R>(...)` returning the RHS of the total-energy equation `∂(ρE)/∂t = …` including pressure work, viscous dissipation, and heat conduction.

The energy equation SHALL use the total-energy form `E = e + 0.5·‖u‖²`. The conserved variable identity SHALL be stated in each kernel's docstring.

#### Scenario: Continuity reduces to the incompressible case when density is constant

- **WHEN** `compressible_ns_continuity_rhs_kernel` is invoked with a spatially constant density and a divergence-free velocity
- **THEN** the returned RHS SHALL equal zero to within precision tolerance

#### Scenario: Compressible momentum reduces to incompressible in the low-Mach divergence-free limit

- **WHEN** `compressible_ns_momentum_rhs_kernel` is invoked with `∇·u = 0` and a constant density
- **THEN** the returned RHS SHALL equal `incompressible_ns_rhs_kernel(...)` evaluated on the consistent set of inputs, to within precision tolerance

#### Scenario: Energy equation dissipation term is non-negative for Newtonian fluids

- **WHEN** the viscous dissipation contribution of `compressible_ns_energy_rhs_kernel` is computed in isolation
- **THEN** for any strain-rate tensor and positive dynamic viscosity, the dissipation term SHALL be `≥ 0` to within precision tolerance

### Requirement: Euler regime (inviscid limit)

The `theories::fluid_dynamics::euler` module SHALL expose:

- `euler_momentum_rhs_kernel<R>(u: &Velocity3<R>, grad_u: &VelocityGradient<R>, grad_p: &[R; 3], rho: &Density<R>, body_force: &BodyForceDensity<R>) -> Result<AccelerationVector<R>, PhysicsError>` returning `∂u/∂t = −(u·∇)u − (1/ρ)∇p + f`, composed from `convective_acceleration_kernel` and `pressure_gradient_force_kernel`.

The Euler regime SHALL NOT accept or use any viscosity input.

#### Scenario: Euler equals incompressible NS with zero viscosity

- **WHEN** `euler_momentum_rhs_kernel` and `incompressible_ns_rhs_kernel` are invoked on the same `(u, grad_u, grad_p, rho, body_force)` inputs, with `nu = 0` and `laplacian_u = [0; 3]` for the latter
- **THEN** the two returned RHS values SHALL be equal to within precision tolerance

#### Scenario: Euler has no viscosity-related dependency

- **WHEN** the signature of `euler_momentum_rhs_kernel` is inspected
- **THEN** it SHALL contain no `KinematicViscosity<R>`, `Viscosity<R>`, or `laplacian_u` parameter

### Requirement: Stokes regime (creeping flow limit)

The `theories::fluid_dynamics::stokes` module SHALL expose:

- `stokes_momentum_rhs_kernel<R>(laplacian_u: &[R; 3], grad_p: &[R; 3], rho: &Density<R>, nu: &KinematicViscosity<R>, body_force: &BodyForceDensity<R>) -> Result<AccelerationVector<R>, PhysicsError>` returning the RHS of `0 = −(1/ρ)∇p + ν∇²u + f` rearranged as a residual whose zero defines Stokes flow. Composed from `viscous_diffusion_kernel` and `pressure_gradient_force_kernel`.

The Stokes regime SHALL NOT accept or use any convective-acceleration input.

#### Scenario: Stokes equals incompressible NS with zero convective term

- **WHEN** `stokes_momentum_rhs_kernel` is invoked with the same `(laplacian_u, grad_p, rho, nu, body_force)` as a corresponding incompressible-NS call that uses `u = [0; 3]` and `grad_u = [[0; 3]; 3]`
- **THEN** the two RHS values SHALL be equal to within precision tolerance

#### Scenario: Stokes has no convective-term dependency

- **WHEN** the signature of `stokes_momentum_rhs_kernel` is inspected
- **THEN** it SHALL contain no `u` or `grad_u` parameter

### Requirement: Sign convention and momentum-form discipline

Every regime function under `theories::fluid_dynamics::` SHALL express its return value as the RHS of `∂u/∂t = …` (Eulerian acceleration form, not material-derivative form), with the convective term `−(u·∇)u` explicit on the RHS where applicable. The Newtonian viscous stress SHALL follow the continuum-mechanics convention `τ = 2μS − (2/3)μ(∇·u)I + ζ(∇·u)I` (positive in tension). Each regime function's docstring SHALL restate the sign convention and the form of the equation it returns.

#### Scenario: Docstrings restate convention

- **WHEN** the rustdoc of any regime function under `theories::fluid_dynamics::` is read
- **THEN** it SHALL explicitly state (a) the form of the equation being evaluated and (b) the sign convention of any stress or pressure-gradient term involved

### Requirement: Test discipline and AGENTS.md conformance

Every new source file under `deep_causality_physics/src/theories/fluid_dynamics/` SHALL achieve 100% test coverage per AGENTS.md §"Code testing". Tests SHALL live under `deep_causality_physics/tests/theories/fluid_dynamics/<regime>_tests.rs` mirroring the src tree, with each test file registered in its parent `mod.rs` and in `deep_causality_physics/tests/BUILD.bazel`. No `#[allow(dead_code)]` or `#[allow(clippy::...)]` suppressions are permitted.

#### Scenario: Coverage tooling reports 100% on every new theory file

- **WHEN** the project's coverage tooling is run after this change ships
- **THEN** every new file under `deep_causality_physics/src/theories/fluid_dynamics/` SHALL report 100% line coverage, or SHALL have any unreachable code explicitly justified
