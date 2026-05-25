## Why

The `deep_causality_physics` crate covers electromagnetism, relativity, MHD, quantum, and condensed matter, but its `kernels/fluids/` module ships only two formulas (hydrostatic pressure, Bernoulli). The crate has no coherent treatment of the Navier–Stokes equations, no kinematic primitives (vorticity, strain rate), no turbulence-scale formulas, no coherent-structure detectors, and no dimensionless numbers beyond ad-hoc usage. This blocks Block B1b and Block B5 of [`openspec/notes/3DCausalFluidDynamics.md`](../../notes/3DCausalFluidDynamics.md) — both depend on these primitives being in the kernel surface before the SURD pipeline can extract `FluidPhysicsInvariants<R>`. It also leaves the physics crate unable to express any of the standard textbook fluid-dynamics workflows that downstream causal-discovery and avionics examples will need.

This change codifies textbook fluid dynamics — governing equations, constitutive relations, kinematics, dimensionless numbers, turbulence quantities, coherent-structure detectors, boundary-layer relations, compressible-flow thermodynamics, and ideal-flow primitives — as a coherent kernel + theory layer, fully generic over `R: RealField`, matching the existing crate conventions.

## What Changes

- Add a comprehensive pointwise-kernel surface under `deep_causality_physics/src/kernels/fluids/` covering:
  - **Governing equations** (pointwise RHS contributions): continuity, momentum (full Navier–Stokes), energy, vorticity-transport, scalar advection-diffusion.
  - **Constitutive relations**: Newtonian viscous stress tensor with Stokes hypothesis, bulk-viscosity-aware form, power-law non-Newtonian variant.
  - **Kinematics**: strain-rate tensor `S`, rate-of-rotation tensor `Ω`, vorticity vector `ω`, deformation-gradient-derived invariants `(P, Q, R)` of `∇u`, helicity density, enstrophy density.
  - **Dimensionless numbers**: Reynolds, Mach, Froude, Weber, Prandtl, Peclet, Strouhal, Knudsen, Richardson, Rayleigh, Grashof, Eckert, Schmidt, Lewis, Stokes, Capillary, Bond, Nusselt.
  - **Turbulence quantities**: turbulent kinetic energy `k`, dissipation rate `ε`, Kolmogorov length / time / velocity scales, Taylor microscale, integral length scale, Reynolds-stress tensor reconstruction, eddy viscosity (Boussinesq closure).
  - **Coherent-structure detectors**: Q-criterion, λ₂ (Jeong–Hussain), Δ-criterion, swirling-strength `λ_ci`.
  - **Compressible-flow thermodynamics**: speed of sound (ideal-gas + isentropic), specific enthalpy, total enthalpy, total pressure / temperature, entropy production rate, isentropic-flow relations.
  - **Boundary-layer relations**: wall shear stress, friction velocity `u_τ`, viscous length scale, dimensionless wall distance `y⁺`, viscous-sublayer + log-law profiles, skin-friction coefficient.
  - **Ideal-flow primitives**: 2D stream function `ψ`, velocity potential `φ`, circulation, Kutta–Joukowski lift per span, Bernoulli total head, dynamic pressure.
- Promote the four kernels currently slated as private helpers inside Block B1b (`q_criterion`, `taylor_microscale`, `integral_length_scale`, `kolmogorov_scale`) into this public kernel surface, satisfying the Block B5 "extraction equivalence" property test up front rather than after B1b ships.
- Add fluid-specific physical-quantity newtypes under `deep_causality_physics/src/units/` only where one is missing and structurally distinct from existing units: `DynamicViscosity<R>`, `KinematicViscosity<R>`, `Vorticity<R>`, `StrainRate<R>`, `MassFlux<R>`, `SpecificEnthalpy<R>`, `WallShearStress<R>`. Existing newtypes (`Density`, `Pressure`, `Speed`, `Length`, `Temperature`) are reused; no parallel hierarchy.
- Add a coherent `theories/fluid_dynamics/` module that assembles the kernels into the four classical regimes:
  - **incompressible Newtonian Navier–Stokes** (pointwise RHS assembly)
  - **compressible Navier–Stokes** (with energy equation)
  - **Euler equations** (inviscid limit)
  - **Stokes flow** (creeping-flow limit)
  Each regime is a free function that composes the kernel surface; no `&self`, no state.
- Every new kernel is generic over `R: RealField` (with `+ FromPrimitive` where literals are needed). No `f64` appears in any new public signature. No `unsafe`, no `dyn`, no macros in library code.
- Property tests cover Galilean invariance (for objective kernels), dimensional consistency via the `units::*` newtypes, limiting cases (`Re → ∞` recovers Euler; `Re → 0` recovers Stokes; incompressible limit of compressible NS), and precision robustness across `f32`, `f64`, and `Float106`.

This change is **non-breaking**: it adds modules, free functions, and newtypes. Existing `hydrostatic_pressure_kernel` and `bernoulli_pressure_kernel` remain in place and are referenced from the new theory layer.

## Capabilities

### New Capabilities
- `fluid-dynamics-kernels`: Pointwise, stateless kernels covering governing equations, constitutive relations, kinematics, dimensionless numbers, turbulence quantities, coherent-structure detectors, compressible-flow thermodynamics, boundary-layer relations, and ideal-flow primitives — all generic over `R: RealField`.
- `fluid-dynamics-theory`: Coherent regime assemblies (incompressible NS, compressible NS, Euler, Stokes) that compose the kernel surface into full RHS evaluators for the four classical Navier–Stokes regimes.

### Modified Capabilities
<!-- None. This change introduces new modules under existing crate scaffolding without modifying existing requirement-level behaviour. -->

## Impact

- **Affected crate:** `deep_causality_physics` only. No changes to `deep_causality_topology`, `deep_causality_core`, or any other crate. No new external dependencies.
- **Affected files (new):**
  - `deep_causality_physics/src/kernels/fluids/` — new submodules `governing.rs`, `constitutive.rs`, `kinematics.rs`, `dimensionless.rs`, `turbulence.rs`, `coherent_structures.rs`, `compressible.rs`, `boundary_layer.rs`, `ideal_flow.rs` (one file per kernel group, sub-split per AGENTS.md if any file exceeds reasonable size).
  - `deep_causality_physics/src/theories/fluid_dynamics/` — new module with `incompressible_ns.rs`, `compressible_ns.rs`, `euler.rs`, `stokes.rs`, `mod.rs`.
  - `deep_causality_physics/src/units/` — new files for the missing fluid quantity newtypes.
  - `deep_causality_physics/tests/kernels/fluids/`, `deep_causality_physics/tests/theories/fluid_dynamics/`, `deep_causality_physics/tests/units/` — tests mirroring the src tree per AGENTS.md §"Test structure".
- **Affected files (modified):**
  - `deep_causality_physics/src/kernels/fluids/mod.rs` — register new submodules.
  - `deep_causality_physics/src/theories/mod.rs` — register `fluid_dynamics` submodule.
  - `deep_causality_physics/src/units/mod.rs` — register new unit files.
  - `deep_causality_physics/src/lib.rs` — re-export new public types and free functions per the project's "all public types exported from lib.rs" rule.
  - `deep_causality_physics/BUILD.bazel` and `deep_causality_physics/tests/BUILD.bazel` — register new modules per AGENTS.md §"Test structure".
- **Downstream unlocks:**
  - Block B1b of `3DCausalFluidDynamics.md` consumes `q_criterion_kernel`, `helicity_density_kernel`, `taylor_microscale_kernel`, `integral_length_scale_kernel` from this surface directly, eliminating the planned private-helper-then-extract path.
  - Block B5 collapses to a "verify already-public" step instead of a fresh kernel-build step.
  - Future avionics wake-vortex example and any compressible-flow examples land against this surface without further kernel work.
- **Risk profile:** Low. Additive, no API removals, no behavioural change to existing kernels. The main correctness risk is dimensional / sign-convention drift across formulas; addressed by property tests for Galilean invariance, limiting cases, and dimensional consistency via the `units::*` newtypes.
