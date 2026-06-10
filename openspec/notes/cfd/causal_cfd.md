# Causal CFD — pre-specification note

**Status.** Exploratory. This is not a proposal. It is a starting point for *deriving* a proper proposal once the scope, dependencies, and implications are better understood.

**Revision 2026-06-10.** Brought in line with `cfd-gap.md`, which is the ground-truth foundation note for the solver core. This note now *assumes the cfd-gap program (G1–G6) is closed and its APIs are available*: the DEC wedge and interior product, the de Rham/♯ isos, the typed-form carriers (including the shared `SolenoidalField<R>` type-state), pinned conventions, the one-solve `leray_project` entry point, and harmonic-kernel deflation for full `hodge_decompose` on periodic lattices. Sections §4.1, §4.3–§4.5, §7, and §10 were updated accordingly; the sequencing across all three CFD notes lives in `cfd-roadmap.md`.

**Working title.** *Causal CFD*. A structure-preserving, cut-cell-capable, incompressible and compressible Navier–Stokes solver built on the existing `deep_causality_topology` cubical-Regge stack and the `deep_causality_physics` fluids surface. Integrated end-to-end with the `PropagatingEffect` / `Intervenable` framework, so that probabilistic data fusion, multiphysics composition, structured corrective control, and forensic logging are first-class capabilities rather than bolt-ons.

---

## 0. Value proposition

A Causal CFD solver built on the DeepCausality stack offers three capabilities that no current CFD code combines, and which would be structurally hard for any incumbent to replicate without rebuilding their platform.

**One: selective probabilistic typing of the field.** Sensor patches, data-assimilation regions, interface zones, and moving-surface boundaries can be typed as `MaybeUncertain<R>` while the rest of the domain remains plain `R`. The kernels handle both transparently because they are generic over the trait, not the concrete type. Industrial codes either ignore sensor uncertainty entirely, or wrap heavy bespoke covariance machinery around a deterministic core. Neither approach composes. This one does.

**Two: type-checked multiphysics composition.** The DeepCausality physics crate already ships kernel surfaces for fluids, electromagnetism, MHD, thermodynamics, materials, and general relativity, all generic over `R: RealField`. Conjugate heat transfer, MHD, fluid-structure interaction, and aeroelasticity become *type-level compositions* of those kernels rather than C++ glue code. OpenFOAM has a multiphysics story; it works, but is brittle and requires substantial bespoke engineering per coupling pair.

**Three: a structured library of corrective interventions.** Industrial solvers contain roughly a dozen ad-hoc inline corrective patterns: CFL adaption, divergence rescue, BC fallback, mass-conservation enforcement, turbulence-bounds clipping, hybrid-scheme switching, geometric robustness fixes, fault recovery. They are scattered across the source, untested in isolation, and untraceable in production. The `.intervene` mechanism makes each one a named, logged, compositional unit. For regulated industries this is potentially the load-bearing certification story.

Underneath all three sits the existing `Intervenable` and `EffectLog` machinery: counterfactual geometry exploration, fault injection, cascading-failure analysis, and the forensic-provenance artefact for certification. That is necessary infrastructure. It is not, by itself, a competitive moat.

### 0.1 Why this is structurally hard to replicate

The "structurally hard to replicate" claim deserves justification.

Selective probabilistic typing works only because every kernel is generic over `R: RealField` and the broader algebraic trait hierarchy in `deep_causality_num`. Adding that hierarchy to a 30-year-old C++ codebase like OpenFOAM or Fluent is not a release-cycle effort. It is a multi-year rewrite of the entire numerical core.

Multiphysics composition works only because the kernel surfaces for fluids, EM, MHD, thermal all live in one crate with one trait set. Bolting that on to an architecture where each physics package is a separate executable is harder than it sounds.

The corrective intervention library works only because the chain primitive is already monadic. Bolting structured interventions onto a procedural solver requires inventing the chain semantics first.

In each case the platform property doing the work was not added for CFD. It was already there. That is the unique fabric the rest of this note unpacks.

### 0.2 Mapping to NASA CFD Vision 2030

The canonical strategic document for the future of computational fluid dynamics is *CFD Vision 2030 Study: A Path to Revolutionary Computational Aerosciences* by Slotnick, Khodadoust, Alonso, Darmofal, Gropp, Lurie, and Mavriplis, NASA/CR-2014-218178 (2014). The study was commissioned by NASA Langley with industry input from Boeing, Pratt & Whitney, the major US aerospace primes, and academic CFD groups. It remains the most-cited strategic document in the field and is the working framework for funding priorities at NASA, AIAA, ATI, and most national aerospace programmes. (Source: NASA Technical Reports Server, document ID 20150007533, ntrs.nasa.gov.)

A 2024 follow-up progress report by Slotnick and Heller explicitly notes that progress against the 2014 targets has been slower than projected on the structural problems, particularly uncertainty quantification, validation infrastructure, and multidisciplinary integration. The structural problems remain unsolved after a decade of focused industry investment. (Source: AIAA Aviation Forum 2024 proceedings, paper AIAA 2024-4501.)

Vision 2030 identifies six structural problems with current CFD. The mapping to DeepCausality capabilities is unusually tight. Five of the six have conceptual solutions already built in the platform; the sixth is operationally manageable through the platform's other capabilities. This is the substantive reason the Causal CFD pitch is application of an existing platform rather than invention of new methods.

#### The mapping

| Vision 2030 structural problem | DeepCausality capability | Status |
|---|---|---|
| **1. Mesh generation bottleneck** (30-60% of engineer time per the Vision 2030 estimate) | `LatticeComplex`, `DualLatticeComplex`, `CubicalReggeGeometry` in `deep_causality_topology` | Conceptually built. The cubical mesh structurally sidesteps unstructured tetrahedral meshing. The cut-cell extension is engineering work in Phase 2, not a research problem. |
| **2. HPC scaling plateau and adoption brittleness** | HKT witness pattern in `deep_causality_haft`; per-region typing via `R: RealField` generic kernels in `deep_causality_num` | Conceptually built. GPU and cluster backends become additional witnesses rather than algorithm rewrites. The constraint from the MLX revert (no trait leakage through user-facing APIs) is documented and understood; the Candle path in §11.4 is consistent with it. |
| **3. Turbulence modeling for separated and unsteady flows** | No new closure model. The platform provides: `MaybeUncertain<R>` for RANS-failure regions, `deep_causality_discovery` for causal source identification on flow data, the intervention framework for fallback when closure models fail, and type-checked composition for swapping closure models without rewriting downstream code | Not solved at the turbulence-modeling level. Honestly: this is the one Vision 2030 problem where the platform does not provide a structural answer. The platform makes RANS limitations operationally manageable rather than solving them at the physics level. This is the correct positioning to take in any grant pitch. |
| **4. Uncertainty quantification essentially absent from industrial practice** | `Uncertain<R>` and `MaybeUncertain<R>` in `deep_causality_uncertain`, with selective per-region typing as detailed in §2.7 | Conceptually built. Five years of platform work on the uncertain crate is the substantive solution. The CFD application is wiring, not invention. |
| **5. Multidisciplinary integration brittleness** | Shared `R: RealField` trait bound across the fluids, electromagnetism, MHD, thermodynamics, materials, and general-relativity kernel surfaces in `deep_causality_physics`. Type-checked composition. | Conceptually built. No coupling pair requires bespoke glue. Composition is type-level. The §3.2 multiphysics amplifier follows directly. |
| **6. Validation infrastructure inadequacy** | `EffectLog` produced by the `Intervenable` framework, plus the §10 structured corrective intervention library | Conceptually built. Every simulation produces an auditable forensic record. Validation is integrated into the simulation chain rather than performed as a separate post-hoc activity. |

The Vision 2030 study additionally identifies several second-tier problems: autonomous error estimation, certification-credit pathways for replacing physical tests with simulation, and the "knowledge-to-insight pipeline." The DeepCausality `EffectLog`, intervention framework, and `deep_causality_discovery` crate also map onto these. They are subsidiary to the six primary structural problems above but worth knowing about for grant applications targeting aerospace certification specifically.

#### Why this changes the proposal's strategic frame

The mapping changes the proposal's posture in three specific ways. Each is worth being explicit about in grant applications and commercial conversations.

**One: technical risk is materially lower than a standard CFD grant proposal.** The standard pattern is "fund invention of solutions to the structural problems." The Causal CFD pitch is that the conceptual solutions are already built, peer-reviewed by the Linux Foundation Technical Advisory Committee with industry sponsor approval, deployed across 18 crates and 250,000 lines of code, and verified through 10,000 tests at 95% coverage published per pull request. The proposed work is application engineering against canonical reference data. That converts the project from speculative platform research to scoped application engineering.

For grant assessors, this is a fundamentally different risk profile. Innovate UK Smart Grants and the ATI both score risk explicitly. A project with the platform already de-risked scores meaningfully higher than one where the platform is part of the proposal.

**Two: time to credible validation is shorter than the field's standard.** An "invent and validate" CFD grant typically runs 3-5 years before the first benchmark result of weight. An "apply existing platform" project is plausibly 9-12 months to lid-driven cavity (Phase 1), 18-24 months to 3D cylinder (Phase 2), and 3 years to NASA CRM RANS (Phase 3). The shorter timeline reflects the absence of platform-layer research risk and is consistent with the validation-first sequencing recommended in prior turns of the strategic discussion.

For commercial partners and anchor customers, this matters because the gap between commitment and demonstrable results is smaller than they would expect for a project of this scope.

**Three: competitive defensibility runs deeper than the CFD code itself.** A competitor wanting to replicate Causal CFD would first need to replicate `deep_causality_uncertain`, `deep_causality_haft`, `deep_causality_topology`, `deep_causality_num`, the physics kernels, the intervention framework, the supply-chain-security posture, and the Linux Foundation TAC-reviewed governance position. That is more than a decade of person-years before the CFD wiring can begin. The moat is the platform, not the CFD application.

For commercial defensibility, this is the strongest version of the argument. Competitors who build CFD on top of conventional platforms cannot easily acquire the §3 amplifiers because the amplifiers are platform properties they do not have. The OpenFOAM-style services business adapted for MIT licensing (per the prior turns of the discussion) is therefore not just a viable business model; it is a business model whose moat is structural rather than incidental.

#### Why the alignment is structural rather than coincidental

The five-of-six Vision 2030 alignment is not the product of designing the platform around Vision 2030. The DeepCausality platform was built over five years against an independent set of motivations: causal reasoning, type-safe scientific computing, multiphysics composition, uncertainty propagation, and forensic provenance. Those motivations happen to be the same abstract problems Vision 2030 identifies as the structural blockers of next-generation CFD. The convergence reflects something real: the structural problems in scientific computing across multiple domains (CFD, climate, computational physics generally) have common roots in the platform-layer abstractions, and a platform built right addresses many of them simultaneously.

The pitch is honest because the alignment is structural rather than retrofitted. Grant assessors and technical reviewers who probe this will find the alignment holds under scrutiny.

---

## 1. Context

Industrial CFD is a multi-billion-dollar market. ANSYS Fluent, STAR-CCM+, Siemens, OpenFOAM, Cadence Fidelity, ConvergeCFD. Its dominant pain point is **mesh generation on complex geometry**. Engineers routinely spend more time on the mesh than on the simulation, and bad meshes silently kill solution quality. The industry response over the last decade has been to move toward **cartesian cut-cell solvers with adaptive mesh refinement**; Cadence Fidelity and ConvergeCFD's autonomous AMR are the two clearest examples. Cubical complexes with cut-cell handling at boundaries sidestep most of the unstructured-tet pain.

Separately, structure-preserving (mimetic) discretisations based on **discrete exterior calculus** have been a 20-year research topic (Bossavit, Hirani, Desbrun, Marsden, and others). Nobody has shipped this cleanly at the open-source level. The two trends, cartesian cut-cell and mimetic DEC, should converge. In practice they have not.

The DeepCausality stack happens to have most of the structural pieces for both, plus the three differentiators named in §0.

---

## 2. What we already have

### 2.1 `deep_causality_topology` — DEC and cubical-Regge stack

The pieces below are already public types in the topology crate:

- **`LatticeComplex<D, R>`**. A D-dimensional cubical lattice. Shape, periodic or open boundary conditions, primal cells, sparse boundary operator (CSR). Constructors for square/cubic/hypercubic torus and open variants.
- **`DualLatticeComplex`**. Dual lattice with coboundary operator. Required for any staggered (MAC-style) discretisation.
- **`CellComplex` + `BoundaryOperator`**. Discrete chain complex with sparse boundary matrices. Gives the discrete exterior derivative `d` directly.
- **`CubicalReggeGeometry<D, R, S>`**. Metric on the lattice (Euclidean or Lorentzian via `SignatureMarker`), edge lengths, metric tensor at hinges, `HasHodgeStar` impl, cell volumes, deficit and dihedral angles, Regge gradient, Metropolis update.
- **`hodge_decomposition`**. Discrete Helmholtz decomposition. *Decisive for incompressible NS*: pressure projection is a Helmholtz decomposition.
- **`differential_form`, `chain`, `manifold`, `curvature_tensor`, `topological_invariants`**. Generic DEC building blocks.

### 2.2 `deep_causality_physics::kernels::fluids` — pointwise NS surface

Already shipped. 1431 tests, verified against textbook reference solutions:

- Sixty-plus pointwise kernels: kinematics (S, Ω, vorticity, CPC invariants, helicity, enstrophy); governing equations (continuity, convective acceleration, viscous diffusion, pressure-gradient force, vorticity transport, scalar advection-diffusion, kinetic energy density, viscous dissipation, pressure work); constitutive (Newtonian and power-law); 18 dimensionless numbers; turbulence quantities (TKE, ε, Kolmogorov scales, Reynolds stress, Boussinesq eddy viscosity); coherent-structure detectors (Q, Δ, λ₂, swirling strength); compressible thermodynamics (speed of sound, isentropic stagnation, entropy production); wall functions; ideal-flow primitives (Bernoulli, stream function, circulation, Kutta–Joukowski).
- Four NS regime evaluators: `incompressible_ns_rhs_kernel`, `euler_momentum_rhs_kernel`, `stokes_momentum_rhs_kernel`, `compressible_ns_{continuity,momentum,energy}_rhs_kernel`.
- Typed quantities (`Velocity3`, `VelocityGradient`, `StrainRateTensor`, `ViscousStress`, `ReynoldsStress`, `CauchyStress`, `Density`, `Pressure`, …) with invariant-enforcing constructors.
- All kernels wrapped into `PropagatingEffect<T>` causal wrappers.

The kernels deliberately stop at *pointwise RHS given local inputs*. They will not compute their own inputs. That is the assembly layer's job.

### 2.3 `deep_causality_core::Intervenable` — the chain primitive

This is the part the rest of the CFD world doesn't have. From inspection of the intervention examples:

- **Corrective control loops** (`corrective_lane_keeping`, `corrective_glucose_pump`, `corrective_decompression_stops`, `corrective_network_failover`). A `PropagatingProcess` chain runs as a sequence of `bind` operations representing simulation ticks. A monitor watches the value channel after each tick. When an anomaly threshold fires, the controller computes a corrected value and applies `.intervene(corrected)`. The next `bind` sees the corrected state. The full event timeline is recorded in an `EffectLog`.
- **Counterfactual analysis** (`counterfactual_envelope_fault`, `counterfactual_treatment_options`, `counterfactual_cascade_failure`, `counterfactual_resection_intervention`). A factual chain runs against the baseline. A counterfactual chain takes the same upstream state to some point, then `.intervene` substitutes a different value mid-chain, and the downstream stages run against the substituted value. Context state (covariance, accumulated history) is untouched; only the value channel is swapped.
- **Cascading failure** (`counterfactual_cascade_failure`, which is literally a *fluid distribution network* with pipe-failure cascades). Each cascade step is an `.intervene` composed onto the chain that already carries the previous intervention. Interventions compose. The `EffectLog` is the forensic timeline.

### 2.4 Existing DEC-on-cubes worked example

`examples/mathematics_examples/topology/cubical_heat_diffusion.rs` is the assembly-layer pattern needed for a fluid solver. A `Manifold<CubicalComplex<2, R>, R>` carries a scalar field, evolved by explicit-Euler stepping with a Moore-neighborhood discrete Laplacian. The loop reads the current field, computes the per-cell RHS via the discrete Laplacian, time-steps, writes back. Swap the heat Laplacian for the NS RHS and you have a fluid solver of identical shape.

The simplicial analogue is in `differential_field.rs` (heat equation on a triangle with Hodge ⋆ via `ReggeGeometry`).

### 2.5 `deep_causality_haft` — the HKT engine room

HAFT is the abstract type-theoretic substrate that makes every monadic composition in the rest of the ecosystem possible. It contains no causal logic and no physics. It supplies the *language*: Higher-Kinded Types via the Witness pattern with GATs, plus the standard `Functor` / `Applicative` / `Monad` / `Foldable` / `Traversable` traits. Three points matter for causal CFD.

**Why `PropagatingEffect<T>` is `.bind`-chainable.** The fluid wrappers shipped in `kernels::fluids::wrappers` all return `PropagatingEffect<T>`, and `PropagatingEffect` is a `Monad` via the HAFT witness pattern. That is why a fluid time step composes as a `bind`, why corrective control is `.intervene` on the same chain, and why counterfactual analysis is structurally clean. None of this is *added by* the CFD project. It comes for free with the existing wrappers.

**The `Effect5` bridge fixes four type parameters.** The intervention examples use a five-parameter causal monad (`Value, State, Context, Error, Log`); `HKT5` fixes four of them so the chain reads as a single-parameter monad over `Value`. For CFD the natural binding is:

- **Value**: the fluid field at the current step
- **State**: accumulated solver state (cut-cell registry, refinement history, time-step controller's history, BC-patch activation)
- **Context**: solver configuration (mesh, BCs, regime selector, turbulence-model constants)
- **Error**: `PhysicsError` / `CausalityError`
- **Log**: `EffectLog` (the forensic record)

The solver does not need to invent a new monadic stack; it slots into the existing `Effect5` bridge.

**The uniform-API claim is operationally a test strategy.** From the HAFT documentation: *"`map`, `bind`, and `pure` work exactly the same for a simple `Option` as they do for a complex `CausalEffectPropagationProcess`."* The concrete CFD consequence: the assembly layer and time-stepper can be unit-tested against the `OptionWitness` or `ResultWitness` carrier first, then validated against the full `CausalEffectPropagationProcess` in integration tests, without code duplication. A real schedule de-risker for Phase 1.

HAFT also ships **Cybernetic Loops**, **Adjunctions**, **Comonads**, **Bifunctors**, **Profunctors**, **Promonads**, and natural-isomorphism machinery (`iso::natural_iso{,_2,_3,_4,_5}`). Most are orthogonal to CFD. The one that directly applies is **Cybernetic Loops**.

#### 2.5.1 `CyberneticLoop` for genuine closed-loop control

`deep_causality_haft::traits::cybernetic_loop::CyberneticLoop` exists, and models a five-component feedback system: **Sensor (S)**, **Belief (B)**, **Context (C)**, **Action (A)**, **Entropy (E)**. The trait's `control_step` method takes the agent, a sensor input, an `observe_fn: (S, C) → B`, and a `decide_fn: (B, C) → A`, and returns `Result<A, E>`. The doc explicitly calls out OODA loops, quantum error correction, and PID controllers as use cases.

`CyberneticLoop` is the right tool when control-theoretic invariants (stability margins, observability, controllability) are load-bearing. That mostly means active flow control with closed-loop dynamics that can themselves destabilise the flow. For routine corrective patterns (CFL adaption, divergence rescue, BC fallback) the simpler `.intervene` mechanism is sufficient and cheaper to implement. The library in §10 distinguishes the two.

For the active-flow-control demo in §6 item 1, the `CyberneticLoop` binding is:

| HAFT slot | CFD binding |
|---|---|
| **Sensor (S)** | Wall shear stress, surface pressure, vorticity probe at the actuator station |
| **Belief (B)** | Estimated separation state: bubble length, transition status, shock position |
| **Context (C)** | Actuator capabilities, geometry, regime constants, current time-step |
| **Action (A)** | Actuator command: synthetic-jet velocity, plasma duty cycle, flap deflection angle |
| **Entropy (E)** | Sensor noise, model-error budget, fail-safe trigger |
| **`observe_fn`** | Estimator turning probe readings plus context into a separation-state estimate (Kalman, EKF, or a low-pass filter for the first demo) |
| **`decide_fn`** | Controller (PID first; replaceable with anything that returns an Action) |

Other HAFT traits worth naming briefly:

- **`Foldable` / `Traversable`**. Result aggregation across a time-window (mean drag, RMS pressure, vorticity histograms). Standard for postprocessing.
- **`Comonad`**. Relevant if a future stencil-operator abstraction is introduced; comonadic extension is a clean way to express "value at every cell as a function of the local neighbourhood". Speculative.
- **`Adjunction` / `Profunctor` / `Promonad` / natural isomorphisms**. Theoretical machinery; out of practical scope for Phases 1–3.

### 2.6 `deep_causality_num` — precision parametricity for free

The numeric crate provides the trait hierarchy that every quantity and every kernel in the physics crate is generic over. The relevant bound is `R: RealField`, which delivers:

- Ordered-field semantics (`+`, `-`, `*`, `/`, `PartialOrd`, `Neg`)
- Transcendentals (`sqrt`, `exp`, `ln`, `sin`, `cos`, `tan`, `powf`, `acos`, …)
- Constants (`pi()`, `e()`, `epsilon()`)
- `Copy + PartialOrd` so kernel internals can do `if val < R::zero()` branching without trait-object overhead

`RealField` is implemented for `f32`, `f64`, and `Float106` (double-double extended precision). Consequence for CFD: the same solver source can run at `f32` for memory-bandwidth-limited problems, at `f64` for numerical stability (transonic wing buffet), and at `Float106` to verify that `f64` truncation is not masking a physical effect (academic verification studies, condition-number-sensitive Poisson solves). Most commercial CFD codes pin to `double`. Switching precision there is a multi-month refactor. Here it is a type parameter at the call site.

Two further trait families are relevant:

- **`ComplexField<R>`**. Needed if any future spectral pressure solver is wired in; FFT-based Poisson solves on periodic cubical meshes operate in `Complex<R>`.
- **`Integer` / `UnsignedInt` / `SignedInt`**. Relevant to Phase 4 AMR work. Octree refinement levels, cell IDs, neighborhood-index arithmetic all use these primitives; `UnsignedInt::is_power_of_two` and `next_power_of_two` are the operations octree codes need.

### 2.7 `deep_causality_uncertain` — `Uncertain<R>` and `MaybeUncertain<R>`

The uncertain crate ships two types relevant to CFD:

- **`Uncertain<R>`**. A value drawn from a probability distribution. Arithmetic propagates the distribution (analytically where possible, numerically otherwise). Standard UQ semantics.
- **`MaybeUncertain<R>`**. A value that is *probabilistically present*. Carries a probability of existence plus a conditional distribution. The natural type for sensor feeds that occasionally drop, for sparse measurements within a dense field, and for cells near a moving or stochastic surface.

Worked usage of both is in `examples/causal_uncertain_examples/` (clinical_trial, gps_navigation, sensor_processing). The GPS example in particular is structurally close to what a CFD inflow BC consuming weather-station data looks like: a sensor stream with intermittent dropout, a fusion step, a downstream consumer.

For CFD the key idea is **selective application**. Uniform `MaybeUncertain<R>` over a 100M-cell domain is unaffordable in memory (single-precision floats grow from 4 bytes to roughly 64-128 bytes per cell, an order of magnitude or more). The kernels do not need to be uniform.

The natural layout is a hybrid storage scheme:

- **Interior cells**: plain `R`. The vast majority of any production simulation.
- **Sensor-input zones**: `MaybeUncertain<R>` for cells within N units of a sensor port consuming external data.
- **Interface zones**: `MaybeUncertain<R>` for cells within N units of a multiphase interface, a level-set zero, or a moving cut surface.
- **Multi-fidelity overlap zones**: `MaybeUncertain<R>` where high-fidelity DNS or experimental data overlaps a low-fidelity RANS background.

The boundary between zones is a kernel that lifts `R` into `MaybeUncertain<R>` (cheap; constant probability of presence, point distribution) when fields cross from deterministic into uncertain regions, and another that collapses the other direction (extracts the mean, drops the distribution) when uncertainty has decayed below a threshold or for visualisation.

The 8-16x memory blowup applies to the uncertain zones only. If those are 5-10% of the domain (typical for sensor patches and interfaces), total memory cost is 1.4-2.5x. Tractable.

This selective-typing pattern is structurally impossible to retrofit onto a non-generic CFD code. It is the strongest single argument for the platform.

---

## 3. The composition story — four amplifiers

The note groups capabilities into four amplifier categories, each rooted in a different crate of the DeepCausality fabric.

### 3.1 Probabilistic data zones (`MaybeUncertain`)

The capabilities that follow from §2.7 selective probabilistic typing.

1. **Sensor-driven boundary conditions with native dropout handling.** Wind-tunnel pressure taps, atmospheric soundings, anemometer feeds, GPS-tagged measurements. A BC that consumes `MaybeUncertain<Speed>` knows when input is missing and how often, and propagates that knowledge into the solve instead of silently substituting last-known values.

2. **Data assimilation from sparse measurements.** PIV (particle image velocimetry) only covers part of the domain. The velocity at unmeasured cells is `MaybeUncertain`. Industrial 4D-Var explicitly models the covariance; here it is a type.

3. **Multi-fidelity coupling.** A high-fidelity DNS patch embedded in a low-fidelity RANS field. In the overlap region each cell has *maybe* a DNS sample. The conventional treatment is co-Kriging; this is its type-level expression.

4. **Multiphase interface tracking.** A cell near a water/air interface is "maybe water, maybe air". VOF and level-set are the conventional tricks. `MaybeUncertain<MaterialProperties>` is the direct expression.

5. **Moving / compliant surfaces.** A compliant wing, a flapping fish tail, a heart valve. Cut-cell apertures near the surface are `MaybeUncertain<R>` because the instantaneous surface position itself is.

The commercial story here is the largest single market: weather and ocean modeling, biomedical flow, multiphase chemical engineering, wind-energy site assessment. All currently rely on bespoke covariance machinery wrapped around a deterministic core.

### 3.2 Multiphysics composition

The capabilities that follow from the shared `R: RealField` bound across all physics kernels.

1. **Conjugate heat transfer.** Fluid kernel in the gas path, thermal kernel in the metal, coupled at the cut-cell interface. Composition is type-level.

2. **MHD and GR-MHD.** The MHD kernels are already in `deep_causality_physics::kernels::mhd`. Coupling fluid to electromagnetic fields is one `bind` chain composed with another. The Lorentzian signature in `CubicalReggeGeometry` makes GR-MHD on a curved background not a rewrite.

3. **Fluid-structure interaction.** The materials crate has stress / strain / Hooke's law kernels. FSI is a fluid chain plus a solid chain plus a coupling `.intervene` at the interface. Industrial codes treat this as a separate executable per side with file-based handoff.

4. **Aeroelasticity.** FSI at high Reynolds plus a compressible regime evaluator plus an unsteady structural solver. All ingredients exist.

5. **Combustion-adjacent flows (without full chemistry).** Variable-density flows with prescribed reaction rates are tractable using existing thermodynamic and dimensionless-group kernels. Full chemistry stays out of scope.

OpenFOAM's multiphysics is a meaningful capability but it is brittle and requires substantial bespoke engineering per coupling pair. Here, type-checking does most of the work.

### 3.3 Structured corrective interventions (`.intervene`)

The capabilities that follow from §10's library of named corrective patterns.

1. **CFL-adaptive timestepping.** Monitor max velocity, halve Δt on threshold cross. Logged.
2. **Solver divergence rescue.** Monitor residual, restart from last-good state with smaller Δt.
3. **BC fallback on dropped sensor data.** Composes with `MaybeUncertain` from §3.1.
4. **Mass conservation enforcement.** Monitor ∮ u·n drift, apply global correction.
5. **Turbulence-model bounds protection.** Clip k, ω, or ε to physical ranges, log every clip.
6. **Shock-capturing fallback.** Hybrid scheme switching per cell, per step.
7. **Cut-cell geometric robustness.** Berger-Helzel merging as a structured intervention.
8. **Conjugate heat-flux runaway protection.** Sub-cycle the thermal solver when ΔT/Δt exceeds threshold.
9. **Multiphysics relaxation.** Aitken or partitioned-implicit fix on FSI added-mass divergence.
10. **Checkpoint-restart on unhandled error.** Fault tolerance as a named intervention.

The full list and phase tags are in §10. Each one is currently implemented in industrial codes as scattered inline logic; here each is a named, logged, compositional unit.

### 3.4 Counterfactual analysis and forensic provenance (`Intervenable` + `EffectLog`)

The capabilities that hang off the existing intervention framework.

1. **Counterfactual geometry exploration.** "What if this turbine blade has this fillet instead?" Run the factual chain to the geometry-dependent stage, `.intervene` on the geometry channel, continue.

2. **Fault and degradation analysis.** "What if the wing develops ice accretion at minute 12?" Factual: clean wing. Counterfactual: `.intervene` at minute 12 to substitute degraded surface roughness. The two trajectories share the pre-fault state exactly.

3. **Cascading failure analysis** (already demonstrated in `counterfactual_cascade_failure` for a fluid distribution network). Pipe X fails, re-solve, find overloaded neighbors, intervene to fail them, iterate. Direct industrial application: cooling-system resilience in IC engines, gas-turbine fuel-line redundancy, refinery process-pipe networks.

4. **Forensic provenance for certification.** Aerospace (DO-178C, ARP4754), nuclear (NRC 50.46), medical-device (IEC 62304) certification all require traceable artefacts. The `EffectLog` produced by an intervention chain *is* such an artefact.

5. **Adjoint-style design optimisation without writing adjoints.** Finite-difference / surrogate-gradient design exploration directly: factual plus counterfactual at perturbed parameter, surrogate-gradient, next design candidate. Not a replacement for true adjoints where they are required, but cheaper to implement.

This category is the most mature of the four (the intervention framework is already shipped). The pitch here is that the existing machinery applies *unchanged* to CFD; the work is wiring, not invention.

---

## 4. Gap inventory

What is *not* in the existing crates and would need to be built:

### 4.1 Assembly layer (CLOSED for the incompressible path by `cfd-gap.md` G1–G3)
The incompressible assembly layer is delivered by the cfd-gap program, in a stronger
(DEC-native) form than this section originally sketched. Velocity is an edge 1-form
throughout; the operators are `d`, `δ`, `⋆`, `Δ`, the wedge, and the interior product
(G1); transfer to and from pointwise representations goes through the de Rham/♯ isos
(G2); and the field carrier is the typed-form layer (G3: `VelocityOneForm<R>`,
`VorticityTwoForm<R>`, `SolenoidalField<R>`, …), which **supersedes the
`FluidField<D, R>` container proposed here** — do not build a duplicate container.

What remains of 4.1 for later phases: assembly of the compressible terms (`∇·τ`,
`∇·q`, `∇·(ρuE)` from lattice data) and the cut-cell variants of the operators
(Phase 2+).

### 4.2 Cut-cell geometry (the industrial moat)
The cubical lattice is uniform. Real geometry intersects it. Required:

- `CutCell<D>`: partial cube with clipped volume, modified face areas and normals, wetted-area fractions, list of cut-face fragments tagged with the source geometry
- Apertures (face-area fractions) for flux computations
- Cube ↔ triangle intersection (for STL inputs); cube ↔ analytic surface (for primitives like cylinders, spheres)
- Small-cut-cell stabilisation (Berger–Helzel cell merging, or flux redistribution). Small cells violate the CFL condition catastrophically. This is the canonical reason cut-cell solvers are hard.
- Wall boundary handling on cut faces: no-slip Dirichlet, slip, wall functions for high-Re

The hardest single component of the project. It is what differentiates a research toy from something that can mesh a turbine blade.

### 4.3 Pressure-velocity coupling (CLOSED by `cfd-gap.md` §2 — Leray, not Chorin-with-pressure)
Superseded. The original sketch here (predictor step, explicit pressure-Poisson solve,
correction) is replaced by the Leray formulation in `cfd-gap.md` §2: the projector
needs only the gradient half of the Hodge decomposition,

```
P(u*) = u* − d(Δ₀⁻¹ δ u*)
```

one gauge-fixed grade-0 CG solve per evaluation (`leray_project`), no pressure
variable in the time loop at all. Pressure is recovered **on demand** as an opt-in
diagnostic from the same decomposition (Bernoulli vs. static convention documented at
the call site). The β-step singularity on periodic lattices never arises in the
solver core; full `hodge_decompose` (with G6 harmonic deflation) is needed only by
the causal-analysis tap (§4.11 of `cfd-roadmap.md` sequencing).

### 4.4 Linear solver stack (audit RESOLVED; preconditioning remains future work)
The audit question is answered: a matrix-free CG (`deep_causality_sparse::cg_solve`)
ships, is generic over `RealField` with per-precision tolerance clamping, and is
already wired into `hodge_decompose` / `leray_project`. Phase 1 needs nothing more.

Remaining for later phases (performance, not correctness):

- Preconditioning (diagonal first, then geometric multigrid on the cubical complex)
- AMG fallback for cut-cell regions where geometric MG breaks down

### 4.5 Time integration (explicit path CLOSED; implicit paths remain)
Stale as originally written: `Rk4` and `Euler` ship in `deep_causality_calculus` as
Arrow endomorphisms over any state with `Clone + Add + Mul<R>` — the whole typed-form
field rides them directly — and `EndoArrow` supplies `iterate_n` /
`iterate_to_fixpoint` / `iterate_until` (the run loop with event stop). The march is
the arrow; the fallible projection and CFL check are `bind` steps in the causal monad
(`cfd-gap.md` §5.4). Pure numerics in the arrow, fallible plumbing in the monad.

Still needed for later phases:

- Semi-implicit IMEX: viscous diffusion implicit, convection explicit. The workhorse for moderate-Re incompressible.
- Implicit BDF2: high-Re, RANS, stiff regimes

### 4.6 Boundary conditions
Periodic is in `LatticeComplex` already, and the periodic solver core ships before any
wall work starts. Inflow (Dirichlet), outflow (Neumann or convective), wall (no-slip / slip / wall function), and symmetry all need a BC layer that modifies the assembly stencils near boundary cells. `MaybeUncertain`-typed BCs (§3.1 item 1) require a dropout-handling extension on top of the basic BC layer.

Wall staging follows `cfd-gap.md` G5: boundary-corrected Hodge star duals, a
Neumann–Poisson projection path, and no-slip Laplacian rows — validated analytic-first
on **laminar Poiseuille channel flow** (periodic x, walls y; constructible today,
exact parabolic steady state) before the lid-driven cavity's Ghia-table comparison.

### 4.7 Turbulence model closure
The fluid kernels include TKE, dissipation rate, Boussinesq eddy viscosity, Reynolds stress. Missing is the *composition* into a closed RANS model (k-ε, k-ω SST):

- Production term `P = -⟨u'u'⟩ : ∇⟨u⟩`
- Transport equations for k and ε (or ω); scalar advection-diffusion with source terms. The kernel exists (`scalar_advection_diffusion_kernel`).
- Model constants and the wall-function blend
- μ_t feedback into the momentum equation

### 4.8 Adaptive mesh refinement
Cadence Fidelity and ConvergeCFD are AMR by default. Octree-style refinement and coarsening on the lattice complex, refinement criteria (gradient-based, vorticity-based, Q-criterion-based; we have Q already), inter-level flux conservation. Large piece. Could be deferred to a later phase.

### 4.9 I/O
STL and STEP readers for cut-cell geometry input. VTK, HDF5, CGNS writers for output. Out of scope for the physics layer but blocking for any real-world demo. Probably belongs in a separate `deep_causality_io` crate, or as example-level utilities.

### 4.10 Selective uncertain-zone typing
The hybrid storage layout from §2.7. Specifically: a `FluidField` that admits per-region typing, the lift kernels (`R → MaybeUncertain<R>`) at zone boundaries, the collapse kernels for visualisation, and a registry of which cells live in which zone. This is largely engineering on top of the existing `Uncertain<R>` / `MaybeUncertain<R>` machinery, but it does require the assembly layer to support type-heterogeneous storage.

---

## 5. Proposed three-phase roadmap

### Phase 1: walls + corrective library on top of the shipped periodic core
Smallest deliverable that exercises the whole chain end-to-end on a real problem.
The DEC-native periodic solver (assembly, Leray projection, Rk4 arrow march,
Taylor–Green validation ladder) ships *before* Phase 1, per `cfd-gap.md` and
`cfd-roadmap.md` Stages 0–1; Phase 1 adds walls and the corrective-pattern
infrastructure.

- Wall BC layer per `cfd-gap.md` G5 (4.6): boundary star, Neumann projection path, no-slip rows
- Validate analytic-first on **laminar Poiseuille channel flow** (periodic x, walls y)
- **Ship at least three corrective interventions from §10**: CFL-adaptive timestepping (10.1), divergence rescue (10.2), checkpoint-restart on error (10.10). These establish the corrective-pattern infrastructure early. (10.1 and 10.2 are upgrades of the `cfl_check` bind and the `CgFailure` short-circuit already present in the periodic chain.)
- Validate against **lid-driven cavity** at Re = 1000 and Re = 10000. Reference: Ghia, Ghia & Shin 1982. *No cut cells required.*

Phase 1 alone is a meaningful research deliverable. A structure-preserving DEC incompressible NS solver on a cubical mesh, with a clean intervention chain and three named corrective patterns wired in. Publishable as such; useful as a teaching tool; demonstrates three of the four amplifiers from §3 (corrective library, counterfactual, forensic logging).

### Phase 2: cut cells + first probabilistic zone
Validate against geometry the solver can't avoid, and stand up the `MaybeUncertain` story.

- `CutCell<D>` type (4.2)
- Cube–triangle intersection routine
- Small-cut-cell stabilisation
- Wall BC on cut faces
- **First `MaybeUncertain` zone (4.10)**: a sensor-fed inflow BC with dropout handling. Composes with the BC-fallback corrective intervention (10.3).
- Validate against **flow around a 3D cylinder** at Re = 100–3900. Reference: Lehmkuhl et al. 2013, plus the Williamson lineage back to the 1990s.

Phase 2 makes this competitive with research cartesian solvers and demonstrates the §3.1 probabilistic-zone amplifier on a concrete case.

### Phase 3: compressible + RANS + flagship validation
The industrial-relevance proof.

- Compressible solver wiring (the kernels exist; need the assembly plus shock-capturing scheme, likely AUSM or HLLC)
- k-ω SST turbulence model closure (4.7)
- Wall functions on cut faces
- **Add corrective interventions from §10** for shock-capturing fallback (10.6) and turbulence-bounds protection (10.5).
- Validate against **NASA Common Research Model transonic wing buffet** (5th–7th AIAA Drag Prediction Workshops; NASA Ames experimental data).

Phase 3 makes this directly relevant to industry. It also demonstrates the type-encoded regime advantage at scale.

### Phase 4 (provisional): multiphysics + AMR
Once Phases 1-3 establish credibility, the §3.2 multiphysics amplifier becomes the next leg.

- Conjugate heat transfer demo (fluid + thermal). Validates §3.2 item 1.
- AMR on the cubical complex (4.8).
- Aeroelastic FSI demo (validates §3.2 item 3).

This phase is deliberately provisional; the proposal that derives from this note should defer detailed scoping of Phase 4 until Phase 3 measurement data is in hand.

---

## 6. Demonstration scenarios (after Phase 2)

Each maps to an intervention pattern already shown in the existing examples.

1. **Active flow control via `CyberneticLoop`.** Flow over a backward-facing step. Open loop: a separation bubble develops; mean reattachment length matches published data (Driver & Seegmiller 1985). Closed loop: a `CyberneticLoop::control_step` runs each tick. Sensor reads wall shear stress at a downstream probe. When the reversed-flow indicator fires, `.intervene` applies a synthetic-jet velocity boundary condition at the step lip. Compare reattachment length open versus closed.

2. **Counterfactual fault injection.** Internal-combustion engine intake port flow. Factual: clean port surface. Counterfactual: mid-chain `.intervene` at t = T to substitute degraded (rough) surface conditions on a subset of cut cells. Quantify the impact on tumble or swirl ratio. Structurally identical to `counterfactual_envelope_fault`.

3. **Cascading failure analysis on a cooling network.** Multi-pass turbine-blade cooling passage. Factual: nominal mass-flow split across passages. Counterfactual: `.intervene` to mark passage X as blocked. Re-solve, find passages now exceeding thermal limits, intervene again, iterate. Output: final blocked footprint plus `EffectLog` of the cascade.

4. **Sensor-driven inflow with dropout** (new, exercises §3.1). Atmospheric boundary-layer simulation. Inflow BC consumes a `MaybeUncertain` weather-station stream. Dropouts in the stream trigger the corrective intervention from §10.3. The `EffectLog` records every dropout and every fallback. Compare to a deterministic-inflow control run.

---

## 7. Open questions to resolve before a proper proposal

1. **~~Staggered vs. co-located storage.~~ Resolved** by `cfd-gap.md` decision 2: velocity as an edge 1-form *is* the staggered/mimetic (MAC-like) choice, by construction of the lattice complex. No co-located stage exists.

2. **~~Sparse linear-solver provenance.~~ Resolved.** Matrix-free `cg_solve` ships in `deep_causality_sparse`, precision-generic, already wired into `hodge_decompose` / `leray_project`. Preconditioning is future performance work (§4.4).

3. **Geometry input format.** STL is universal but lossy (no curved surfaces). STEP is high-fidelity but parser-heavy. Recommend STL first, defer STEP.

4. **~~Scope of `Intervenable` integration.~~ Resolved** by `cfd-gap.md` §5.4: coarse-grained binds — the whole-field march step is the arrow, and projection plus CFL check are the `bind` stages. Pure numerics never enter the monad; per-cell wrapping never happens.

5. **Cut-cell algorithm provenance.** Berger–Helzel cell-merging is the textbook approach, but adds complexity. Flux-redistribution (Colella, Graves, Modiano) is simpler but less accurate near small cuts. **Decision required during Phase 2 design.** Prototype both on the cylinder case if time allows.

6. **Turbulence model scope.** k-ω SST is the industrial default, but has many tunable constants. Spalart–Allmaras is simpler and almost as good for aerospace. Decision can be deferred to Phase 3.

7. **AMR vs. uniform cubical (4.8).** Phase 1 and 2 can be uniform. Phase 3 probably needs AMR to be competitive on the CRM wing case. Postpone the decision until Phase 2 is complete and runtime cost on uniform meshes is measured.

8. **Relationship to existing `cubical_heat_diffusion` example.** Should Phase 1 deliver `cubical_lid_cavity` as the next entry in `examples/mathematics_examples/topology/`? Likely yes; it makes the composition story walkable for new readers.

9. **Concrete `Effect5` type-parameter binding.** Per §2.5, the five-parameter causal monad needs `Value, State, Context, Error, Log` types pinned for the CFD use case. The proposed binding is *Value = fluid field*, *State = solver state*, *Context = configuration*, *Error = PhysicsError*, *Log = EffectLog*. The shape of `State` (whether the cut-cell registry, refinement-level table, and time-step history live in one struct or are split) needs to be decided before the assembly layer is built. **Decision required during Phase 1 design.**

10. **HAFT-witness test strategy.** Per §2.5, the assembly layer and time-stepper can be developed against the `OptionWitness` or `ResultWitness` carrier first and promoted to `CausalEffectPropagationProcess` later. **Decision: adopt unless a specific reason emerges to skip it.**

11. **~~Cybernetic Loops traits availability.~~ Resolved.** `deep_causality_haft::traits::cybernetic_loop::CyberneticLoop` exists. Use it for closed-loop active control (§6 item 1); use plain `.intervene` for routine corrective patterns (§10).

12. **Hybrid storage layout for `MaybeUncertain` zones.** §2.7 sketches the per-region typing approach. The concrete representation (zone registry, lift/collapse kernel placement, traversal order) needs design work. **Decision required during Phase 2 design.** Could prototype on a 1D toy first.

13. **Granularity of `EffectLog` entries.** Per-tick is cheap; per-cell-correction is expensive. The §10 corrective patterns fire per-cell or per-step depending on the pattern. A configurable verbosity level seems right. **Decision required during Phase 1 design.**

---

## 8. Out of scope (for the proposal that derives from this note)

- Multiphase / VOF / level-set methods as a primary deliverable. The `MaybeUncertain` interface story in §3.1 item 4 covers a real subset of this, but full VOF / level-set is a separate topic.
- Combustion / reactive flow. Couples to chemistry kernels we don't yet have.
- Particle / DEM coupling.
- Free-surface flow (sloshing tanks, marine hydrodynamics).
- Non-Newtonian rheology beyond the existing power-law kernel.
- Compressible LES. Compressible RANS is hard enough for Phase 3.
- True adjoint optimisation. Covered indirectly by intervention-based surrogate gradients (§3.4 item 5).
- GPU acceleration and cluster support. Both are deferred and treated in §11. The HAFT "new container = a new witness" claim does *not* automatically save GPU integration from trait leakage; the previously reverted MLX backend is the cautionary tale (see `openspec/changes/reverted/revert_mlx_backend.md`). Phase 1-3 explicitly targets workstation deployment. Cluster is a separate change-set entirely.
- Real-time and coupled control hardware loops.
- ~~Causal discovery on flow data~~ — **no longer distant research territory.** The Leray projection step computes (half of) the Hodge decomposition of the velocity field at every time step; `3DCausalFluidDynamics.md`'s `FluidSignature` → `RollingHistory` → SURD pipeline consumes exactly a `HodgeDecomposition<R>` on the same `Manifold`, the same `R`. Causal attribution of simulated flow is therefore a *tap on the solve chain* (one β-step solve per sampled snapshot, enabled by G6), not a separate pipeline. It remains out of scope for *this* note's proposal, but it is scheduled — see `cfd-roadmap.md` Stage 2. No other CFD code can offer this, because no other code's projection step is a Hodge decomposition.

---

## 9. Summary

The DeepCausality stack contains, to an unusual degree, the algebraic prerequisites for a structure-preserving cartesian cut-cell CFD solver (DEC on cubical complexes via `deep_causality_topology`) and the physics prerequisites at the pointwise level (the fluid kernel surface in `deep_causality_physics`). The missing pieces are mostly engineering: an assembly layer wiring them together, cut-cell geometry handling, a sparse linear solver, time integration, boundary conditions.

What makes the resulting solver *different* from every other open-source or commercial CFD tool is not one thing. It is the four amplifiers of §3, each rooted in a different crate of the existing fabric:

- **`MaybeUncertain`-typed sensor and interface zones** (§3.1). The largest commercial story.
- **Type-checked multiphysics composition** (§3.2). The strongest technical moat.
- **Structured corrective interventions** (§3.3, §10). The certification story.
- **Counterfactual analysis and forensic provenance** (§3.4). The baseline differentiator.

Each is structurally hard to replicate in a codebase that wasn't built generic-over-`R: RealField` from day one with a monadic chain primitive.

The external strategic validation for the proposal is the NASA CFD Vision 2030 alignment documented in §0.2. Of the six structural problems Vision 2030 identifies as the blockers of next-generation industrial CFD, five have conceptual solutions already built into the DeepCausality platform across five years of independent platform development. The sixth (turbulence modeling for separated flows) is operationally manageable through the platform's other capabilities even though it is not solved at the closure-model level. This alignment is structural rather than retrofitted, and is the load-bearing reason the proposal can credibly position itself as application engineering rather than platform research.

Phase 1 (uniform cubical incompressible NS, lid-driven cavity, three corrective interventions wired in) is a self-contained deliverable that proves three of the four amplifiers end-to-end. It should be the entry point. Phase 2 adds cut cells and the first probabilistic-zone demo, validating the fourth amplifier (§3.1) on a concrete case.

---

## 10. Corrective intervention library

A candidate library of named, logged, compositional corrective patterns. Each one is implemented in industrial codes today as scattered inline logic; here each is a structured `.intervene` (or, where load-bearing closed-loop dynamics demand it, a `CyberneticLoop`). The table tags each pattern by recommended phase, mechanism, and the trigger condition.

| ID | Pattern | Mechanism | Phase | Trigger |
|---|---|---|---|---|
| 10.1 | **CFL-adaptive timestepping** | `.intervene` | Phase 1 (MVP) | `max(\|u\|) · Δt / Δx` exceeds threshold |
| 10.2 | **Solver divergence rescue** | `.intervene` + checkpoint restore | Phase 1 (MVP) | residual norm exceeds previous-step value by N× |
| 10.3 | **BC fallback on dropped sensor data** | `.intervene` (composes with `MaybeUncertain`) | Phase 2 | `MaybeUncertain::is_present` returns false |
| 10.4 | **~~Mass conservation enforcement~~ Obsoleted upward** | type-state, not `.intervene`: `SolenoidalField<R>` is divergence-free by construction (`cfd-gap.md` G3); drift is unrepresentable, not monitored | — | n/a |
| 10.5 | **Turbulence-model bounds protection** | `.intervene` (clip k, ω, ε) | Phase 3 | `k < 0`, `ω < 0`, or `ε < 0` |
| 10.6 | **Shock-capturing scheme fallback** | `.intervene` per cell, per step | Phase 3 | local Mach gradient or limiter activation |
| 10.7 | **Cut-cell geometric robustness** | `.intervene` (cell merge or flux redistribute) | Phase 2 | cell aperture below threshold |
| 10.8 | **Conjugate heat-flux runaway protection** | `.intervene` (sub-cycle thermal) | Phase 4 | `\|ΔT/Δt\|` exceeds threshold |
| 10.9 | **Multiphysics relaxation (FSI added-mass)** | `.intervene` (Aitken relax) | Phase 4 | FSI residual oscillates |
| 10.10 | **Checkpoint-restart on unhandled error** | `.intervene` from `Result::Err` | Phase 1 (MVP) | any kernel returns `Err` |
| 10.11 | **Active flow control** (`CyberneticLoop`, not plain `.intervene`) | `CyberneticLoop::control_step` | Phase 3 | per-tick, per the controller |

Three of these (10.1, 10.2, 10.10) ship in Phase 1 as the minimum-viable corrective library. They establish the pattern, log into the `EffectLog`, and provide a concrete demonstration of the §3.3 amplifier even before cut cells exist.

The library is open-ended. Production deployments will add domain-specific patterns; the framework supports them by composition, not by source-tree privilege.

---

## 11. Computational scale and deployment

### 11.1 What industrial CFD jobs actually look like

The honest distribution of compute load across industrial CFD users:

- **Workstation-scale jobs**, roughly 70-80% of industrial work. Parametric studies, design-of-experiments runs, simple RANS on representative geometry. Problem size 1M-50M cells. Minutes to overnight on 16-64 CPU cores, or 1-2 GPUs. ConvergeCFD and Cadence Fidelity built market share by serving this segment well.
- **Moderate cluster jobs**, roughly 15-20%. Eight to thirty-two nodes, 256-1024 cores, multi-day runs. Full-geometry RANS, simple LES on subsets, multipoint design optimisation. ANSYS Fluent and STAR-CCM+ dominate here.
- **Flagship jobs**, roughly 5%. Thousands to tens of thousands of cores, week-long runs. DDES and full LES of aircraft or turbomachinery, certification campaigns, multidisciplinary optimization. National labs and the largest aerospace companies.

The 80% point is the commercially valuable target. A workstation-deployable solver with optional GPU acceleration covers it. Clusters become necessary for the top 20%, mostly for high-end aerospace and turbomachinery.

### 11.2 Phase-by-phase scale targets

- **Phase 1 (lid-driven cavity)**: 65k-1M cells. A single-threaded laptop handles the smaller meshes; a workstation with 8-16 cores handles the larger ones in minutes. No acceleration needed.
- **Phase 2 (3D cylinder)**: 1M-20M cells. Workstation. CPU is fine for validation; GPU would speed up parametric Re sweeps meaningfully, but is not a blocker.
- **Phase 3 (NASA CRM transonic wing)**: 50M-200M cells for RANS, 500M+ for DDES. A 64-core workstation with one or two server-class GPUs is enough to publish credible RANS results against the AIAA Drag Prediction Workshop reference data. DDES at full resolution is a Phase 4+ ambition; cluster strongly preferred there.

Conclusion: **Phases 1-3 ship on a workstation**. Cluster is a strategic decision orthogonal to the core solver, deferred to Phase 5+.

### 11.3 The MLX lesson

DeepCausality previously experimented with an MLX (Apple Metal) GPU backend. It was reverted (see `openspec/changes/reverted/revert_mlx_backend.md`). The post-mortem is short and load-bearing for any future GPU work.

The backend trait propagated through every high-level API, forcing GPU-specific type-level constraints onto user-facing call sites where they had no business being. Two specific failure modes were documented:

- A `TensorData` trait that leaked through topology and multivector APIs, forcing `Send + Sync + 'static` bounds on user-facing types.
- An `f32`-only constraint that broke `f64` and `Float106` precision paths for physics applications.

Any future GPU integration must satisfy two non-negotiables:

1. The acceleration backend must not appear in user-facing trait bounds. Higher-level code must remain `R: RealField` only.
2. Precision must remain a free parameter. `f64` and `Float106` paths must remain available alongside any `f32`-only GPU path.

The HAFT "new container is a new witness" claim does *not* automatically satisfy these. It saves you from rewriting algorithms; it does not save you from designing the witness boundary carefully enough that GPU concerns do not leak.

### 11.4 GPU acceleration via Candle (candidate path)

Candle is a Rust tensor library with a more contained abstraction than MLX. A single `Tensor` type wraps backend selection internally; the user does not see backend traits unless they go looking. That is structurally a better fit for the non-leakage requirement in §11.3.

Candidate path for GPU acceleration once Phase 3 is shipping:

- A new HKT witness wrapping `candle::Tensor` (or whatever Rust GPU tensor library is chosen at the time)
- The witness exposes the same `Monad` / `Functor` interface as `OptionWitness` or `PropagatingEffect`
- Fluid kernels remain generic over `R: RealField`; the witness is responsible for shipping the kernel evaluations onto the GPU device
- Precision policy stays at the kernel level: `f64` paths run on CPU, `f32` paths can run on GPU, mixed-precision can run partially on each

If this works cleanly, GPU acceleration is additive. It does not change the solver design.

Honest caveat: the MLX experience proves that "this should work cleanly" is not automatic. A small prototype on a single non-trivial kernel (say, the Poisson solve from Phase 1) is the right way to verify the witness boundary before committing. The decision to adopt Candle, some other Rust GPU library, or to defer GPU work entirely should be made on the basis of that prototype, not on architectural enthusiasm.

### 11.5 Cluster support is a different category

Cluster support is structural, not additive. It changes:

- The lattice complex (needs `partition()` returning sub-complexes per rank)
- The boundary conditions (halo exchange between ranks at sub-complex interfaces, expressible as cross-partition `.intervene` or as a specialised `bind`)
- The linear solver (the Poisson solve becomes distributed; this is the hardest single piece, and the literature on parallel multigrid is deep)
- The time integration (collective operations per step)
- The intervention chain (becomes a distributed object; fault tolerance becomes a real concern, not a theoretical one)
- The cut-cell decomposition (geometry crossing rank boundaries needs careful handling)

None of this is structurally impossible on the DeepCausality fabric. The lattice complex already has sparse-matrix infrastructure. MPI bindings exist for Rust. The chain primitive is monadic, which composes naturally with distributed semantics. But each piece is real engineering, and the cumulative cost places cluster support firmly in "phase 5-6" territory rather than "incremental Phase 4 work".

Strategic position for the proposal that derives from this note: **cluster support is a separate change-set entirely**. It is out of scope for Phase 1-3. If and when it becomes a priority, it gets its own pre-spec note and its own roadmap. Promising cluster support before workstation deployment is solid would over-extend the proposal.

### 11.6 Deployment recommendation

For the proposal that derives from this note, the deployment baseline should read:

- **Phases 1-3**: workstation. 16-64 CPU cores. Single-node memory in the 32GB-1TB range, depending on phase. The solver must be performant on this configuration. GPU is optional acceleration and may be deferred.
- **Phase 4** (provisional): workstation plus optional Candle (or equivalent) GPU witness, prototyped against the MLX-lesson requirements in §11.3.
- **Phase 5+** (explicitly out of scope here): cluster support as a separate change-set.

This is honest about what we can deliver and what we should not promise. The 80% commercial point is reachable on a workstation. The top 20% is cluster territory and a different conversation entirely.

---

**Next step.** Use this note to scope a formal OpenSpec proposal. Likely starting with Phase 1 only, with Phases 2 and 3 as follow-on change sets. The open questions in §7 should be resolved, or explicitly deferred with rationale, before the proposal is written. The deployment recommendation in §11.6 should be reflected in the proposal's stated success criteria.
