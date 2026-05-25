## Context

`deep_causality_physics` already follows a strict two-layer convention:

- `kernels/<domain>/` — pointwise, stateless, side-effect-free free functions that consume scalars or fixed-size arrays of `R: RealField` and produce a `Result<T, PhysicsError>`. No `&self`, no manifold, no context, no I/O. Each kernel encodes one textbook formula.
- `theories/<domain>/` — coherent assemblies that compose kernels into larger physical models (full RHS evaluators, conservation laws, regime-specific systems). Also stateless and free-function-based.

The fluids kernel surface today is exactly two functions: `hydrostatic_pressure_kernel` and `bernoulli_pressure_kernel`, plus a thin `wrappers.rs` that lifts each into a `PropagatingEffect`. There is no `theories/fluid_dynamics/`.

Block B5 of [`openspec/notes/3DCausalFluidDynamics.md`](../../notes/3DCausalFluidDynamics.md) defines a minimal kernel set (convective acceleration, viscous diffusion, pressure gradient force, vorticity transport, Q-criterion, λ₂, three turbulence scales). This change set is the natural place to land that minimal set plus the rest of the standard textbook surface, so the physics crate has a complete and coherent fluid-dynamics layer instead of a partial one that future change sets keep extending.

Inputs to all kernels are *already discretised*: the caller supplies velocity vectors, gradient tensors, Laplacians, and scalar fields evaluated at a point. Discretisation lives in `deep_causality_topology` (cubical or simplicial manifolds, `Manifold::exterior_derivative`, `Manifold::hodge_decompose`). The kernel/theory split keeps the physics crate independent of `deep_causality_topology` — no new dependency edge.

## Goals / Non-Goals

**Goals:**

- Land the complete textbook surface of classical fluid dynamics as pointwise kernels: governing equations (continuity, momentum, energy, vorticity transport, scalar advection-diffusion), constitutive relations (Newtonian + power-law), kinematics (`S`, `Ω`, `ω`, `∇u` invariants, helicity, enstrophy), dimensionless numbers (18 of them), turbulence quantities (TKE, ε, Kolmogorov / Taylor / integral scales, Reynolds stress, eddy viscosity), coherent-structure detectors (Q, λ₂, Δ, λ_ci), compressible-flow thermodynamics, boundary-layer relations, ideal-flow primitives.
- Assemble those kernels into four coherent regime evaluators under `theories/fluid_dynamics/`: incompressible Newtonian NS, compressible NS, Euler, Stokes.
- Maintain the crate's existing precision discipline: every new public signature generic over `R: RealField` (`+ FromPrimitive` where literals appear), no `f64` in new public surfaces, no `unsafe`, no `dyn`, no macros in library code.
- Honour the existing kernel pattern: `*_kernel` suffix for raw algebra, paired causal wrapper in `wrappers.rs` returning `PropagatingEffect<T>`. Reuse existing units (`Pressure`, `Density`, `Speed`, `Length`, `Temperature`) wherever they already model the quantity.
- Front-load the four B5-extracted kernels (`q_criterion`, `taylor_microscale`, `integral_length_scale`, `kolmogorov_scale`) so Block B1b consumes them directly and Block B5 becomes a verify-only step.
- Provide property tests for Galilean invariance, dimensional consistency, limiting cases, and precision robustness across `f32` / `f64` / `Float106`.

**Non-Goals:**

- Discretisation, spatial assembly, time integration. Kernels never see a `Manifold`, a stencil, or a timestep. Time-stepping schemes (Runge-Kutta, semi-implicit, projection methods) are out of scope.
- LES / RANS subgrid-scale closure models (Smagorinsky, dynamic Smagorinsky, k-ε, k-ω, SST). The proposal lands the *quantities* a closure consumes (Reynolds stress, eddy viscosity, dissipation rate) but not the closures themselves; closure models live in a follow-up.
- Multiphase flow, non-Newtonian rheology beyond the power-law placeholder, granular flow, viscoelastic models.
- Coupling to electromagnetism (MHD is its own kernels directory) or to relativistic flow. The compressible-flow path stops at classical thermodynamics.
- Topology dependency. No `use deep_causality_topology::*`, no new dependency edge in `Cargo.toml` or `BUILD.bazel`.
- Reproduction of any published measurement. Validation is the deferred concern called out in `3DCausalFluidDynamics.md` §4.
- A `PropagatingProcess`-based fluid-dynamics monad. The causal `wrappers.rs` layer lifts each kernel into `PropagatingEffect<T>` for compositional use; stateful process-based pipelines belong in `3DCausalFluidDynamics.md`'s B2.

## Decisions

### D1. Kernel signatures take arrays of `R`, not units, by default.

The convective acceleration kernel signature is `fn(&[R; 3], &[[R; 3]; 3]) -> [R; 3]`, not `fn(&Velocity<R>, &VelocityGradient<R>) -> AccelerationVector<R>`. Three reasons:

1. The existing fluids kernels (`hydrostatic_pressure_kernel`, `bernoulli_pressure_kernel`) take `Pressure<R>` / `Density<R>` / `Speed<R>` because their inputs are *scalar physical quantities with finite-positivity constraints*. A 3-component velocity vector has no such constraint — any real triple is a valid velocity. Wrapping `[R; 3]` in `Velocity3<R>` would add a newtype whose constructor only validates `is_finite()`, which is dead weight.
2. Velocity-gradient tensors `[[R; 3]; 3]` have no positivity / sign constraint either. The constitutive and kinematic kernels manipulate `S = 0.5·(∇u + ∇uᵀ)` and `Ω = 0.5·(∇u − ∇uᵀ)` as raw tensors; wrapping them in newtypes serves no checking purpose and breaks ergonomic composition.
3. The Block B5 signatures in `3DCausalFluidDynamics.md` already use `[R; 3]` / `[[R; 3]; 3]`. Matching that surface lets B5 close as "verify equivalence" rather than as a second API redesign.

Scalar physical quantities with finite-positivity invariants (kinematic viscosity, density, pressure, temperature, wall shear stress) **do** use newtypes — both because the invariants matter and because that's the existing convention for scalar fluid inputs.

**Alternative considered:** wrap every input in a units newtype. Rejected for the above reasons; would force a parallel `Velocity3<R>` / `VelocityGradient<R>` / `StrainRateTensor<R>` hierarchy with no checking value.

### D2. One file per kernel group, not one file per kernel.

The crate's existing one-type-one-module convention from AGENTS.md targets *types*. Free functions group naturally by physical concept: all six dimensionless numbers in the Newton family belong together in `dimensionless.rs` because the reader looking for "is there a Strouhal here" reads the same file as one looking for "is there a Reynolds here". Splitting each formula into its own file under `kernels/fluids/dimensionless/reynolds.rs` etc. would explode the module count without helping discoverability.

Kernel modules are: `governing.rs`, `constitutive.rs`, `kinematics.rs`, `dimensionless.rs`, `turbulence.rs`, `coherent_structures.rs`, `compressible.rs`, `boundary_layer.rs`, `ideal_flow.rs`. Each is sub-split only if it grows past ~600 LOC, matching existing crate practice.

The existing `mechanics.rs` (hydrostatic + Bernoulli) is renamed neither; it stays as `mechanics.rs` and the new modules sit alongside it. `Mechanics` is the right home for the static-fluid and steady-incompressible primitives it already contains.

**Alternative considered:** one file per kernel. Rejected — would create ~70 files for ~70 kernels, with no navigational benefit because the formulas are short.

### D3. The `theories/fluid_dynamics/` regimes are free functions, not types.

A regime evaluator like `incompressible_ns_rhs<R>(u, grad_u, lap_u, grad_p, rho, nu, body_force) -> [R; 3]` returns the pointwise RHS of `∂u/∂t = …`. There is no state to carry, no configuration object, no method dispatch. A free function exactly matches the contents.

The four regimes:

- `incompressible_ns.rs::incompressible_ns_rhs_kernel` — `(∂u/∂t) = −(u·∇)u − (1/ρ)∇p + ν∇²u + f`
- `compressible_ns.rs::compressible_ns_momentum_rhs_kernel`, `compressible_ns_continuity_rhs_kernel`, `compressible_ns_energy_rhs_kernel` — the three conservation laws written so they can be composed into a system solver downstream.
- `euler.rs::euler_momentum_rhs_kernel` — `(∂u/∂t) = −(u·∇)u − (1/ρ)∇p + f` (inviscid limit)
- `stokes.rs::stokes_momentum_rhs_kernel` — `0 = −(1/ρ)∇p + ν∇²u + f` rearranged as `ν∇²u + f = (1/ρ)∇p` (creeping flow limit)

Each regime function is built by composing the relevant kernels from `kernels/fluids/`. No regime function reimplements algebra that a kernel already covers — this is the load-bearing invariant of the kernel/theory split.

**Alternative considered:** a `FluidRegime` enum with a method that dispatches to the appropriate RHS. Rejected — the crate prefers static dispatch (AGENTS.md "Static Dispatch"), and the four regimes have different input arities (compressible needs temperature + total energy, Stokes drops the convective term, etc.), so a single enum with a uniform method signature would either force unused parameters or `Option<…>` wrappers.

### D4. The B5 extraction lands here, not later.

`3DCausalFluidDynamics.md` Block B5 says: *"The `q_criterion`, `lambda2`, `taylor_microscale`, and `integral_length_scale` kernels are extracted from the private helpers that B1b will land inline. The B1b API does not change when this extraction happens; only the location of the formulas moves."*

This change set lands those four kernels publicly from the start. Block B1b then consumes them directly rather than reimplementing them privately and waiting for B5 to extract them. Block B5 collapses to a verify-only step: "these four kernels exist with the published signatures, and B1b uses them."

This is strictly better than the original plan: the extraction-equivalence property test in B5 becomes vacuous (no extraction happened), B1b ships ~80 LOC lighter, and the kernels get unit-test coverage on day one rather than after a second migration round.

**Alternative considered:** preserve the B5 extraction step. Rejected — the only reason to defer was that `add-fluid-dynamics-kernels` did not yet exist. It does now.

### D5. Causal wrappers shadow every kernel.

The existing pattern is `mechanics::hydrostatic_pressure_kernel` (raw) + `wrappers::hydrostatic_pressure` (returns `PropagatingEffect<Pressure<R>>`). The new change preserves that pattern: every `*_kernel` gets a corresponding `wrappers::*` function that lifts a `Result<T, PhysicsError>` into `PropagatingEffect<T>` via `PropagatingEffect::pure` / `PropagatingEffect::from_error`.

For kernels that return `[R; 3]` instead of a units newtype (per D1), the wrapper returns `PropagatingEffect<[R; 3]>` — same convention, no special casing.

**Alternative considered:** only wrap the "regime" composites in `wrappers.rs` and leave the leaf kernels unwrapped. Rejected — it would split the public API surface inconsistently and break the existing convention. The wrappers are cheap and uniform.

### D6. Sign convention: stress-tensor positive in tension, RHS expressed as `∂u/∂t = …`.

Two conventions appear in fluid-dynamics textbooks. This change set commits to:

- **Stress tensor positive in tension** (continuum-mechanics convention). The Newtonian viscous stress is `τ = 2μS − (2/3)μ(∇·u)I + ζ(∇·u)I` where `S` is the symmetric strain-rate tensor and `ζ` is bulk viscosity (Stokes hypothesis ⇒ `ζ = 0`).
- **Momentum RHS written as `∂u/∂t = …` (Eulerian acceleration form)**, not as `D u/Dt = …` (material derivative form). The convective term `−(u·∇)u` appears explicitly on the RHS. This matches the Block B5 signatures and is the form a time-integrator consumes directly.

Each regime function's docstring states the sign convention it assumes and the form it returns. Property tests assert the convention by exercising prescribed inputs whose analytical answer the convention pins.

**Alternative considered:** material-derivative form. Rejected — the kernel-level convective acceleration `(u·∇)u` is a separate kernel callers can use directly; forcing `D/Dt` form at the theory layer would hide it.

### D7. SI units throughout.

Every kernel assumes SI inputs (m, s, kg, K, Pa, ...). The existing units newtypes are SI; no conversion logic enters the kernel layer. Non-SI consumers convert at their boundary.

### D8. Errors propagate via existing `PhysicsError`; no new variants needed.

The kernel layer surfaces three failure modes: non-finite output (numerical instability), invariant violation in a units constructor (e.g. negative density), and `R::from_f64(constant)` failure. All three are already covered by `PhysicsError::NumericalInstability(String)` and `PhysicsError::PhysicalInvariantBroken(String)`. No new error variants.

### D9. Dimensional consistency is a test discipline, not a type-level guarantee.

Compile-time dimensional analysis (à la `uom` or `dimensioned`) is out of scope. The crate's existing newtypes (`Pressure`, `Density`, `Speed`, `Length`, `Temperature`) check finiteness and positivity, not dimensional algebra. Property tests exercise dimensional consistency by constructing inputs whose units are well-defined and asserting outputs lie in the expected unit class.

**Alternative considered:** adopt a dimensional-analysis crate. Rejected — would add an external dependency that AGENTS.md §"Safety and security style" advises against, and the existing newtype discipline already catches the failure modes that matter.

### D10. Test layout mirrors src tree per AGENTS.md.

Test files live under `tests/kernels/fluids/<group>_tests.rs` and `tests/theories/fluid_dynamics/<regime>_tests.rs`. Each is registered in its `mod.rs` parent and in `BUILD.bazel`. The 100% coverage rule from AGENTS.md §"Code testing" applies to every new src file.

Shared test fixtures (e.g. a prescribed `(u, grad_u, lap_u)` triple with known Q-criterion value) live in `src/utils_tests/fluid_fixtures.rs` per AGENTS.md's Bazel-imposed rule that test utilities sit under `src/`.

## Risks / Trade-offs

- **[Risk] Sign / convention drift between kernel formulas as the surface grows to ~70 kernels.** → Mitigation: D6 fixes the conventions at the design level, every kernel docstring restates them, and property tests for Galilean invariance + limiting cases pin behaviour. The compressible-NS limit-to-incompressible test and the Re → ∞ recovery of Euler test catch drift in the regime layer.
- **[Risk] Newtype-vs-array boundary inconsistency makes the API feel jagged.** → Mitigation: D1 fixes a clear rule (scalars with positivity constraints get newtypes; vectors and tensors are raw arrays). The rule is stated in the spec and applied uniformly. The pattern is already the existing convention for the `mechanics.rs` kernels.
- **[Risk] Float106 tolerance choice across ~70 kernels is non-trivial.** → Mitigation: each kernel's test module defines its own tolerance constant generic over the precision backend, following the precision-policy pattern already used in `kernels/em/` and `kernels/relativity/`. No central tolerance constant; per-kernel tolerances reflect each formula's conditioning.
- **[Risk] Kernel count balloons LOC and review burden.** → Mitigation: kernels are short (median ~10 lines of algebra), grouped 5–10 per file, with co-located unit tests. The total estimate (see tasks.md) is ~2500 LOC of library code + ~3500 LOC of tests, which the existing kernels in `kernels/em/` and `kernels/thermodynamics/` benchmark as a reasonable size.
- **[Risk] Choosing power-law as the only non-Newtonian variant locks out callers needing Bingham, Carreau-Yasuda, Herschel-Bulkley.** → Mitigation: the power-law kernel is shipped as `power_law_viscous_stress_kernel`; future non-Newtonian rheologies attach as sibling kernels with their own names. The constitutive module is open to extension.
- **[Trade-off] No time integration, no spatial discretisation.** → Consequence: kernels are not directly runnable as a CFD solver. This is the right level: the existing crate convention puts discretisation in `deep_causality_topology` and statefulness in `deep_causality_core` via `PropagatingProcess`. Composing all three into a full solver lives in `3DCausalFluidDynamics.md` and downstream examples, not here.
- **[Trade-off] B5 collapses to a verify-only step rather than performing the extraction.** → Consequence: the original "extraction equivalence" property test in B5 becomes vacuous. The corresponding kernels are unit-tested here; the B5 review gate downgrades from "extraction verified" to "kernels exist and B1b uses them." This is an improvement (fewer migrations) but it does change the B5 deliverable shape.

## Migration Plan

Non-breaking, additive. No existing API removed or renamed.

1. Land kernel modules group-by-group (see tasks.md) behind their `mod.rs` registration. Each group's tests must pass before its `mod.rs` line is uncommented in `kernels/fluids/mod.rs`. This keeps the public surface consistent at every commit.
2. Land `theories/fluid_dynamics/` after all kernel groups are in place.
3. Update `lib.rs` re-exports in a single final commit so the public surface lands atomically.
4. No rollback step required — the change is additive. If a kernel group is rejected at review, revert its commit; no other group is affected.

## Open Questions

- **Q1.** Should the compressible-NS energy equation use total energy `E = ρ(e + 0.5·u²)` or internal energy `e` as the conserved variable? Decision: total energy, matching most compressible-flow textbooks and finite-volume solver conventions. Documented in each compressible kernel's docstring.
- **Q2.** Power-law rheology constant convention: `μ_eff = K · γ̇^(n−1)` with consistency index `K` and flow-behaviour index `n`. Confirm units of `K` are `Pa·sⁿ` in the kernel docstring. Decision: yes, this is the standard convention; documented.
- **Q3.** Should `lambda2_kernel` and `q_criterion_kernel` share an internal helper that computes `S` and `Ω` from `∇u`, or duplicate the algebra for clarity? Decision: extract a private helper in `kinematics.rs` (`strain_and_rotation_tensors_kernel`) since `S` and `Ω` are themselves part of the public kernel surface — both `lambda2_kernel` and `q_criterion_kernel` call them as public kernels, no duplication needed.
