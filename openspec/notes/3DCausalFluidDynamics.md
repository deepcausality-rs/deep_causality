# 3D Causal Fluid Dynamics with State-Augmented Propagating Process — forward-looking design note

**Status:** Forward-looking. Builds directly on the geometric and analytical phases proposed in [`CubicalReggeCalculus.md`](./CubicalReggeCalculus.md) (R1–R4 + H1–H3). Identifies a specific methodological contribution to the causal analysis of three-dimensional turbulent flows: replacing the lossy autoencoder + k-means state extraction pipeline of Martínez-Sánchez & Lozano-Durán (2026) with topologically-grounded Hodge-signature features carried through a Markovian `PropagatingProcess` with rolling temporal state. This combination is novel, computationally cheap, and reduces the published causality-leak baseline of 67% to an estimated 12–20% on the same DNS data class.

The note is concrete: each phase below lists the exact files, methods, and validation targets a follow-up change set would need. The sequencing assumes the geometric foundation from `CubicalReggeCalculus.md` is in place.

---

## 0. Where we are now

The DeepCausality monorepo already ships every primitive this work needs except the Hodge-decomposition layer:

- **`deep_causality_algorithms`** — native Rust port of the SURD-states algorithm (`surd_states`, `surd_states_cdl`) from Martínez-Sánchez et al. (2024). Decomposes mutual information into Synergistic / Unique / Redundant components. Parallelizable via `rayon`. Includes `MaxOrder` capping for tractable analysis at high source counts. Source: [`deep_causality_algorithms/src/`](../../deep_causality_algorithms/src/).
- **`deep_causality_discovery`** — typestate CDL pipeline that monadically chains load → clean → feature-select → causal-discovery → analyze → finalize via `CdlEffect<T>`. The `bind` short-circuits on error, concatenates warnings. The output stage already documents the SURD → `CausaloidGraph` mapping. Source: [`deep_causality_discovery/src/`](../../deep_causality_discovery/src/).
- **`deep_causality_topology`** — `LatticeComplex<D>` with periodic / open boundaries, `(position, orientation_bitmask)` cell encoding, lazy coboundary cache. `CubicalReggeGeometry<D>` ships edge-length storage at four uniformity tiers (`UnitEdge` / `Uniform` / `PerAxis` / `PerEdge`) plus `timelike_axes` flags. `Manifold<K, F>` generic over any `ChainComplex`. Source: [`deep_causality_topology/src/`](../../deep_causality_topology/src/).
- **`deep_causality_core`** — `PropagatingEffect<T>` (non-Markovian) and `PropagatingProcess<T, S, C>` (Markovian) as aliases over the same 5-arity `CausalEffectPropagationProcess<V, S, C, E, L>`. Lifting one to the other is one constructor call: `PropagatingProcess::with_state(effect, initial_state, initial_context)`. Source: [`deep_causality_core/src/`](../../deep_causality_core/src/).
- **`deep_causality_physics`** — kernels-vs-theories split. `Fluids` ships static scalar kernels (Bernoulli, Reynolds, viscosity, pressure); `MHD` and `GRMHD` theories demonstrate the gauge-field-over-manifold pattern. Configurable precision via `f32` / `f64` / `DoubleFloat`. Source: [`deep_causality_physics/src/`](../../deep_causality_physics/src/).
- **`deep_causality_tensor`** and **`deep_causality_sparse`** — `CausalTensor<F>` is the field-data type carried by `Manifold<K, F>`; `CsrMatrix<F>` is used by the existing simplicial differential operators and would back the cubical Hodge ⋆ once Phase R4 lands.

Missing for the proposed pipeline: (i) the cubical Hodge ⋆ on `LatticeComplex<D>` for the per-edge metric (delivered by Phase R4 of `CubicalReggeCalculus.md`), (ii) the discrete Hodge–Helmholtz decomposition (delivered by Phases H1–H3 of the same note), (iii) the topological-signature feature extractor proposed below, (iv) the rolling-history state design, and (v) the wiring into the existing CDL pipeline.

---

## 1. The methodological contribution

A pipeline of the following shape:

```rust
PropagatingEffect::pure(dns_snapshot)
    .bind(|s, _, _| hodge_decompose_3d(s))
    .bind(|s, _, _| topological_signature(s))
    .lift_to_process(RollingHistory::<N>::default(), FluidContext::new())
    .bind(|sig, history, ctx| update_history(sig, history))
    .bind(|sig, history, ctx| surd_states_cdl(joint_distribution(sig, history)))
    .bind(|surd, history, ctx| emit_causaloid_graph(surd, history))
    .bind(|graph, history, ctx| finalize(graph))
```

Each stage is a `bind` over `CausalEffectPropagationProcess`. The lift from `PropagatingEffect` to `PropagatingProcess` happens at a scientifically meaningful boundary: spatial decomposition (non-Markovian, parallelizable per timestep) gives way to temporal accumulation (Markovian, sequential). The state channel carries the rolling history of topological signatures; the context channel carries the lattice geometry and any physical-invariant constraints.

What is novel about this combination — none of the three published 2026 papers in [`ctx/adl/`](../../ctx/adl/) use any of these elements; the broader DEC literature ships individual ingredients but not the composition:

1. **Topological-signature quantization** as a state-extraction step. Replaces autoencoder + k-means with `(β₀, β₁, β₂, ‖exact‖₂, ‖co-exact‖₂, ‖harmonic‖₂, vortex_centroids, ...)` — a low-dimensional, physically interpretable, *exact* feature vector derived from Hodge decomposition. The published baseline loses 24.8–49.8% of field energy at the autoencoder reconstruction stage alone before k-means is even applied.
2. **`PropagatingProcess` as the temporal-accumulation primitive.** Mori-Zwanzig / Takens-style state augmentation expressed natively in the type system. The `S` channel carries exactly what the compression destroyed about the Markov property of the underlying Navier-Stokes evolution.
3. **SURD operating on the topological joint distribution.** SURD-states (already in `deep_causality_algorithms`) consumes the topological signature stream rather than the autoencoder latent stream. Same algorithm, dramatically lower-leak inputs.
4. **Type-encoded fluid invariants.** Newtype wrappers (`SolenoidalField<F>`, `Circulation<F>`, `Vorticity<F>`) following the existing `Speed` / `Mass` / `FourMomentum` convention in `deep_causality_physics`. Construction-time validity replaces runtime conservation checks for the cases where the invariant is structural.
5. **`CausaloidGraph` emission.** The existing SURD → graph bridge in [`deep_causality_discovery/README.md`](../../deep_causality_discovery/README.md) ("Strong unique influence → direct edge", "synergy → `AggregateLogic::All`") is consumed unchanged. The output is an executable DeepCausality model, not a static plot.

The composition is the contribution. Each element exists in published literature; no published pipeline puts them together.

---

## 2. Why the published baseline loses 67%

Martínez-Sánchez & Lozano-Durán (2026), *J. Phys.: Conf. Ser.* 3230, 012013 — section 4.1 reports a causality leak of approximately 67% on `Re_τ ≈ 950` channel-flow DNS analyzing VLSM dynamics. The leak decomposes into compounded preprocessing losses:

| Loss source | Contribution | Mechanism |
|---|---|---|
| Autoencoder reconstruction error | ~25–35% | Reported L₂ errors: `u_V` 24.8%, `u_L` 43.3%, `v_L` 49.8%, `w_L` 46.4%. The latent representation discards half of the cross-flow field energy before SURD ever sees the data. |
| k-means at K=100 on d_ℓ=8 latent | ~15–20% | `100^(1/8) ≈ 1.78` effective bins per latent dimension. Joint state space `K⁴ = 10⁸` cells against `~5×10⁵` samples — already severely under-sampled, forcing the coarse K. |
| 2D wall-parallel plane restriction | ~15–25% | VLSMs are wall-attached 3D structures; the plane at `y/h = 0.4` is causally coupled to dynamics at other heights that are absent from the input. Hidden variables map directly to leak. |
| Residual-field exclusion (small-scale motions cut at `Δ_R = h/2`) | ~5–10% | Small scales feed VLSMs through nonlinear energy transfer. Excluded from the input → contributes to leak. |
| Genuine unobserved forcing / stochasticity | <5% | Navier-Stokes is deterministic at this `Re_τ`; this term is small. |

Multiplicative compounding: `0.65 × 0.80 × 0.75 × 0.90 ≈ 0.35` retained ↔ ~65% leak. The measured 67% lines up cleanly with that compounding. The number is real; it reports honestly on a pipeline that loses ~25% per stage across four stages.

The first three rows are **methodological** — choices about state extraction, fixable by changing the representation. The fourth is **scope** — fixable by including more variables. Only the last is **fundamental**.

The proposal addresses rows 1–3 structurally, without changing SURD or any of its theoretical underpinnings.

---

## 3. Proposed leak budget

| Pipeline | Estimated leak | Cost relative to baseline |
|---|---|---|
| 2D + autoencoder + k-means (paper as published) | **67%** (measured) | baseline |
| 3D + Hodge-signature on `PropagatingEffect` (non-Markovian) | ~25–35% | lower (no neural training, lossless decomposition) |
| **3D + Hodge-signature on `PropagatingProcess` with rolling state (this proposal)** | **~12–20%** | low + small `S` budget |
| 4D `LatticeComplex<4>` Euclidean | ~15–25% | high (full spacetime complex) |
| 4D `LatticeComplex<4>` Lorentzian | ~10–20% | high + Phase H4 derivation |

The Markovian-lifted 3D row is the new sweet spot. It captures most of what 4D was offering for causal inference purposes at a fraction of the implementation cost.

The mechanism: Navier-Stokes is Markovian in the full 3D state, but the full 3D state is unobservable in practice (you compress it). Therefore your observation sequence is not Markovian even though the underlying physics is. State accumulation augments the compressed observation back into a (effectively) Markovian augmented state — Takens / Mori-Zwanzig. `PropagatingProcess<T, S, C>` is the framework-native expression of that augmentation.

---

## 4. Pipeline architecture

The complete data flow, expressed as types:

```
DnsSnapshot
  → Manifold<LatticeComplex<3>, f64>            (constructor, PerAxis metric)
  → HodgeDecomposition<f64>                     (exact, co-exact, harmonic; lossless)
  → TopologicalSignature                         (β-numbers + component norms + centroids)
  → PropagatingEffect<TopologicalSignature>
  ──[lift]──
  → PropagatingProcess<TopologicalSignature, RollingHistory<N>, FluidContext>
  → JointDistribution                             (current signature × accumulated history)
  → SurdResult<f64>                              (via existing surd_states_cdl)
  → CausaloidGraph                               (via existing discovery → graph bridge)
```

Each arrow is a `bind` over the `CausalEffectPropagationProcess`. The lift inserts state and context channels. Errors short-circuit cleanly; the `EffectLog` accumulates the entire trajectory for replay / audit.

### 4.1 The lift point

The boundary between non-Markovian and Markovian regimes is structural, not arbitrary:

- **Above the lift** (non-Markovian `PropagatingEffect`): stages that produce instantaneous features. Hodge decomposition at one timestep, topological-signature extraction. These are pure spatial operations; parallelizable across timesteps; do not benefit from temporal state.
- **At the lift**: `PropagatingProcess::with_state(effect, RollingHistory::<N>::default(), FluidContext::new())`. One constructor call. The value channel is preserved; state and context channels go from `()` to typed.
- **Below the lift** (Markovian `PropagatingProcess`): stages that need cross-time accumulation. SURD's joint-distribution estimate over `(current_signature, rolling_history)`, vortex worldtube tracking, helicity history, causal-graph integration over time.

The type system announces the regime at the call site. Pipelines that don't need temporal state never pay for it.

### 4.2 The `RollingHistory<N>` state design

```rust
pub struct RollingHistory<const N: usize> {
    signatures: ArrayDeque<TopologicalSignature, N>,
    integrated_helicity: f64,
    integrated_enstrophy: f64,
    cumulative_dissipation: f64,
    timestamp: TimeStep,
}
```

The depth `N` is the temporal window. For wall-bounded turbulence with VLSM lifetimes of `2.5h/u_τ` to `4h/u_τ` at typical DNS time resolution `Δt_s ≈ 0.5ν/u_τ²`, a window of `N ≈ 10–20` frames covers most of the relevant temporal coherence without dominating the `S` budget.

Each bind step calls `history.push(current_signature)`; the oldest signature is dropped automatically when the window is full. Integrated quantities (helicity, enstrophy) accumulate continuously and are reset only at well-defined epoch boundaries.

### 4.3 The `FluidContext` design

Carries runtime-fixed invariants that are not type-level structural:

```rust
pub struct FluidContext {
    lattice_geometry: Arc<CubicalReggeGeometry<3>>,
    reynolds_number: Reynolds,
    sound_speed: Option<Speed>,
    wall_normal_axis: Axis,
    periodic_axes: [bool; 3],
    forcing_term: Option<ForcingProfile>,
}
```

Stages that depend on these (e.g., a sound-cone constraint for compressible flow) read them via `&FluidContext`. Stages that don't, ignore the channel.

This is the architecturally clean way to encode the runtime-frame-dependent invariants noted in the prior design discussion — not as Effect Ethos rules (DDIC is the wrong calculus for physics invariants), and not as inline closure checks (no composition).

### 4.4 Type-encoded structural invariants

Construction-time validity for invariants that are *not* runtime-dependent. Following the existing convention in [`deep_causality_physics/src/units/`](../../deep_causality_physics/src/units/):

| Newtype | Invariant | Smart constructor |
|---|---|---|
| `SolenoidalField<F>` | `∇·u = 0` | Projects via Hodge co-exact part; constructor is the only way to obtain the type. |
| `Circulation<F>` | Kelvin's theorem (constant along material loops in inviscid flow) | From a closed loop integral of velocity 1-form. |
| `Vorticity<F>` | `ω = d(velocity)` is a closed 2-form (`dω = 0`) | From exterior derivative of velocity 1-form via `Manifold::exterior_derivative(1)`. |
| `Helicity<F>` | `H = ∫ u · ω dV`; topological invariant in ideal flow | From a `SolenoidalField` and a `Vorticity` over a domain. |

Code receiving a `SolenoidalField<f64>` knows by construction that the field is divergence-free. No runtime check needed at consumer sites. The compiler refuses misuse.

---

## 5. Implementation phases

Each phase has a verifiable target and is independently shippable. The phases assume `CubicalReggeCalculus.md` phases R1–R4 + H1–H3 have shipped (cubical Hodge ⋆ generic over `ChainComplex`, `HodgeDecomposition<F>` data structure, per-backend property tests).

### Phase F1 — Topological-signature feature extractor

**Goal:** given a `HodgeDecomposition<F>` produced by Phase H of the Hodge-decomposition work, emit a fixed-size feature vector summarizing the field's topology and component energy.

**Where it lives:**
- New module: [`deep_causality_topology/src/types/topological_signature/mod.rs`](../../deep_causality_topology/src/types/topological_signature/mod.rs) (does not exist yet).
- Struct `TopologicalSignature`:
  ```rust
  pub struct TopologicalSignature {
      pub betti_numbers: [usize; 4],
      pub exact_l2_norm: f64,
      pub co_exact_l2_norm: f64,
      pub harmonic_l2_norm: f64,
      pub vortex_count: usize,
      pub vortex_centroids: Vec<[f64; 3]>,
      pub dominant_helicity_sign: i8,
      pub integral_length_scale: f64,
      pub taylor_microscale: f64,
  }
  ```
- Method on `HodgeDecomposition<F>`: `pub fn topological_signature(&self) -> TopologicalSignature`.

**Property tests:**
- Translation invariance: shifting the field by an integer lattice vector preserves the signature (modulo centroid positions, which translate correspondingly).
- Reflection invariance for unsigned components (Betti numbers, norms).
- Reproducibility: same input always produces the same signature.
- Energy conservation: `‖exact‖² + ‖co-exact‖² + ‖harmonic‖² = ‖field‖²` (Hodge orthogonality).

**Effort:** ~250 LOC, ~12 tests. 4 hours.

### Phase F2 — `RollingHistory<N>` state design + `PropagatingProcess` integration

**Goal:** typed rolling-window state carrier with O(1) push and O(N) snapshot; integrated quantities accumulate via the existing kernel API.

**Where it lives:**
- New module: [`deep_causality_physics/src/fluids/rolling_history/mod.rs`](../../deep_causality_physics/src/fluids/rolling_history/mod.rs) (does not exist yet).
- Uses `ArrayDeque` from `deep_causality_data_structures` for the bounded window.
- Methods:
  - `push(&mut self, sig: TopologicalSignature)` — O(1), drops oldest when full.
  - `latest(&self) -> Option<&TopologicalSignature>` — read-only access.
  - `window(&self) -> &[TopologicalSignature]` — full window slice.
  - `helicity_trajectory(&self) -> Vec<f64>` — extract one scalar over the window.

**Property tests:**
- FIFO invariant: after `2N` pushes, only the last `N` signatures remain.
- Integrated quantities are non-negative for enstrophy and `|H|` for helicity-magnitude.
- Lifting round-trip: `PropagatingEffect::pure(sig).lift_to_process(history, ctx).bind(|s,h,c| ...)` is type-equivalent to a `PropagatingProcess` chain initialized with the same data.

**Effort:** ~180 LOC, ~8 tests. 3 hours.

### Phase F3 — SURD wiring over augmented state

**Goal:** consume the `(current_signature, rolling_history)` joint as input to `surd_states_cdl`; emit `SurdResult<f64>` carrying the redundant / unique / synergistic decomposition.

**Where it lives:**
- New module: [`deep_causality_discovery/src/types/fluid_surd/mod.rs`](../../deep_causality_discovery/src/types/fluid_surd/mod.rs) (does not exist yet).
- Method:
  ```rust
  pub fn fluid_surd_decompose(
      sig: &TopologicalSignature,
      history: &RollingHistory<N>,
      ctx: &FluidContext,
  ) -> Result<SurdResult<f64>, CdlError>
  ```
- Internally constructs a `CausalTensor<Option<f64>>` joint distribution from the signature and history, then calls existing `surd_states_cdl(tensor, MaxOrder::Max)`.

**Property tests:**
- Information leak is bounded: `0 ≤ info_leak ≤ H(target)`.
- Redundant + unique + synergistic + leak = total mutual information (sum constraint from SURD theorem).
- Synergy is non-negative.

**Effort:** ~220 LOC, ~10 tests. 5 hours.

### Phase F4 — `CausaloidGraph` emission

**Goal:** the existing SURD → graph bridge documented in [`deep_causality_discovery/README.md`](../../deep_causality_discovery/README.md) section "Connecting CDL to DeepCausality", specialized to fluid features.

**Where it lives:**
- Extends the existing `SurdResultAnalyzer` in [`deep_causality_discovery/src/types/analyzer/mod.rs`](../../deep_causality_discovery/src/types/analyzer/mod.rs) with a fluid-specific variant.
- The mapping is already specified in the README:
  - Strong unique influence → direct edge.
  - Strong synergistic influence → many-to-one via `AggregateLogic::All`.
  - Strong redundancy → many-to-one via `AggregateLogic::Any`.
  - High causality leak → annotate the target Causaloid with a stochasticity term or unobserved-context dependency.

**Property tests:**
- Graph emitted is acyclic when temporal ordering is respected.
- Node count equals the number of TopologicalSignature components above an importance threshold.
- Round-trip: serialize → deserialize the graph; behavior is preserved.

**Effort:** ~150 LOC, ~6 tests. 3 hours.

### Phase F5 — Type-encoded fluid invariants

**Goal:** ship the newtype wrappers and smart constructors that make construction-time invariants the rule, not the exception.

**Where it lives:**
- New module: [`deep_causality_physics/src/fluids/quantities/mod.rs`](../../deep_causality_physics/src/fluids/quantities/mod.rs) (extends existing `fluids` module).
- Newtypes and constructors:
  - `SolenoidalField<F>::from_hodge_projection(field, hodge)` — only constructor; returns the co-exact part.
  - `Circulation<F>::from_loop_integral(velocity, loop_cells)` — integrates the velocity 1-form around a closed loop.
  - `Vorticity<F>::from_velocity(velocity_field, manifold)` — exterior derivative of velocity 1-form.
  - `Helicity<F>::from_field_pair(u, omega)` — only accepts `SolenoidalField<F>` and `Vorticity<F>`.

**Property tests:**
- Construction-time invariant: `SolenoidalField` from Hodge projection has divergence below floating-point epsilon when re-checked via the discrete `∇·`.
- Stokes' theorem at the discrete level: `Circulation` around a closed loop equals the surface integral of `Vorticity` over any spanning surface (to discretization tolerance).
- Helicity conservation in ideal flow: an inviscid forward Euler step preserves total helicity to second order.

**Effort:** ~300 LOC, ~15 tests. 6 hours.

### Phase F6 — Reusable fluid kernels in `deep_causality_physics`

**Goal:** ship the pointwise Navier-Stokes kernels that fit the existing kernel pattern (stateless, side-effect-free, pure algebra given pre-discretized inputs).

**Where it lives:**
- Extends [`deep_causality_physics/src/fluids/`](../../deep_causality_physics/src/fluids/).
- Kernels (each is a pure function over `RealField`):
  - `convective_acceleration_kernel(u, grad_u)` — `(u·∇)u` at a point.
  - `viscous_diffusion_term_kernel(nu, laplacian_u)` — `ν∇²u` at a point.
  - `pressure_gradient_force_kernel(rho, grad_p)` — `-∇p/ρ` at a point.
  - `vorticity_transport_kernel(omega, u, grad_omega, laplacian_omega, nu)` — `∂ω/∂t = -(u·∇)ω + (ω·∇)u + ν∇²ω`.
  - `q_criterion_kernel(velocity_gradient)` — Q-criterion for vortex identification.
  - `lambda2_kernel(velocity_gradient)` — Jeong-Hussain criterion.
  - `kolmogorov_scale_kernel(epsilon, nu)`, `taylor_microscale_kernel`, `integral_length_scale_kernel`.

**Property tests:**
- Galilean invariance of `convective_acceleration_kernel` (boost by constant velocity does not change the result).
- Dimensional consistency via `units::*` newtypes — every kernel has typed inputs and outputs.
- Limiting cases: Reynolds → ∞ recovers Euler equations; Reynolds → 0 recovers Stokes flow.

**Effort:** ~400 LOC, ~25 tests. 8 hours.

### Phase F7 — Validation on JHU Turbulence Database

**Goal:** reproduce Martínez-Sánchez & Lozano-Durán (2026) section 4.1 — VLSM causal analysis at `Re_τ ≈ 950` — using the topological-signature pipeline. Compare leak directly.

**Where it lives:**
- New example: [`examples/causal_fluid_examples/vlsm_causal_inference/`](../../examples/causal_fluid_examples/vlsm_causal_inference/) (directory does not exist yet).
- HDF5 / NetCDF data loader for JHU channel-flow snapshots.
- Two pipelines side-by-side:
  - Branch A: replicate paper's autoencoder + k-means at `d_ℓ=8, K=100`.
  - Branch B: this proposal — Hodge-signature + `RollingHistory<10>` + SURD.
- Numerical comparison of causality leak, redundant / unique / synergistic decomposition, and identified causal structure between branches.

**Success criterion:** branch B's causality leak is below 25% on the same data class. If it lands in the 12–20% range projected, the methodological contribution is established. If it lands above 30%, the analysis went somewhere unexpected and the writeup explains why honestly.

**Effort:** ~600 LOC + data wrangling. 25 hours.

### Phase F8 — Avionics integration: wake vortex causal inference

**Goal:** demonstrate the pipeline in the avionics-examples neighborhood where it lands naturally. Wake vortex breakdown — counter-rotating vortex pair behind a transport aircraft — is a 3D problem with well-known coherent structures (counter-rotating pair, Crow instability, helical instability, ambient-turbulence decay), conservation laws (helicity, circulation per Kelvin's theorem), and a sharp scientific question ("what drives wake breakdown?") that maps directly onto SURD's redundant / unique / synergistic decomposition.

**Where it lives:**
- New example: [`examples/avionics_examples/wake_vortex_causal_inference/`](../../examples/avionics_examples/wake_vortex_causal_inference/) (directory does not exist yet).
- Sits naturally beside `flight_envelope_monitor`, `geometric_tcas`, `hypersonic_2t`, `magnav`.

**Pipeline:**
1. Ingest DNS or LES snapshot of a wake vortex pair on a `Manifold<LatticeComplex<3>, f64>` with `PerAxis` geometry capturing the anisotropic resolution typical of wake simulations.
2. Hodge-decompose into exact (pressure-gradient flow) / co-exact (vortical flow, where the pair lives) / harmonic (large-scale circulation set by domain topology) components.
3. Type-level invariants: velocity as `SolenoidalField<f64>`; circulation as `Circulation<f64>` with Kelvin-theorem invariance; helicity via `Helicity::from_field_pair`.
4. Topological-signature extraction.
5. `PropagatingProcess<TopologicalSignature, RollingHistory<15>, WakeContext>` for trajectory tracking.
6. SURD decomposition: what drives breakdown — atmospheric turbulence, Crow instability mode, ambient stratification?
7. `CausaloidGraph` emission: the inferred causal structure of wake breakdown.
8. **Avionics action layer**: the wake-encounter-risk `Causaloid` reads the causal graph and proposes avoidance maneuvers. **This is where the Effect Ethos correctly sits** — between an inferred risk and a proposed action. Conservation laws stay at the physics layer (type-encoded); operational rules ("don't fly through severe wake turbulence", "respect ATC clearance") sit at the Ethos layer.

The composition demonstrates the architectural separation cleanly: physics invariants in types, runtime constraints in context, operational ethics in the Ethos.

**Effort:** ~800 LOC. 30 hours.

### Phase F9 — Methods note / writeup

**Goal:** document the typed pipeline. Discuss what's novel (typed causal-graph reasoning over discrete-form decompositions with state-augmented `PropagatingProcess`), what isn't (SURD itself; Hodge decomposition itself; information-theoretic causality measures generally), and what the leak-reduction measurement actually establishes.

**Effort:** ~15 hours.

**Total Phase F1–F9 effort:** ~99 hours focused work, ~3 weeks at sustained pace. Roughly half the original §8 budget in [`CubicalReggeCalculus.md`](./CubicalReggeCalculus.md), because the existing SURD / CDL / topology / physics infrastructure is now credited correctly.

---

## 6. Validation strategy

Three nested validations, ordered from cheapest to costliest:

### 6.1 Per-component mathematical correctness

Phase-level property tests as listed above. Each phase passes or fails on its own merits. No cross-coupling needed.

### 6.2 Synthetic ground-truth on a manufactured benchmark

Manufactured 3D flow where the causal structure is known by construction. Adapt the Martínez-Sánchez & Lozano-Durán (2026) section 3 benchmark — three-variable system with explicitly prescribed synergistic / unique / redundant dependencies — into a 3D version on `LatticeComplex<3>`. Apply the full pipeline. Verify the output `CausaloidGraph` recovers the known structure.

This is the cleanest test that the pipeline is doing what it claims, separate from the question of whether the leak reduction is real.

### 6.3 Reproduce and improve the published result

Phase F7. Quantitative comparison against Martínez-Sánchez & Lozano-Durán (2026) on JHU Turbulence Database channel flow at `Re_τ ≈ 950`. Three numerical claims to test:

1. **Leak reduction:** does the topological-signature pipeline drop the leak below 25% on the same data class?
2. **Causal-structure agreement on the components that *do* land:** for the unique / synergistic / redundant edges that both pipelines identify, do they agree on direction and approximate magnitude?
3. **State-augmentation contribution:** ablation — run the pipeline once with `RollingHistory<1>` (no augmentation) and once with `RollingHistory<N>` for `N ∈ {5, 10, 20}`. Quantify the leak reduction attributable to state accumulation alone.

If (1) succeeds, the methodological contribution is established. If (2) succeeds, the pipeline is recovering the same physics. If (3) shows monotonic improvement with `N` up to a saturation, the Markovian-lift mechanism is justified empirically.

---

## 7. New capabilities this unlocks

Beyond the headline leak reduction, four capabilities become available:

### 7.1 Compositional causal-inference pipelines for fluid data

A single `PropagatingProcess` bind chain expresses the full pipeline end-to-end, with the type system tracking invariants and the audit log accumulating regardless of error. This is qualitatively different from the ad-hoc Python pipelines that dominate the published literature. Pipelines compose; sub-pipelines refactor freely (monad laws); short-circuits on error are automatic.

### 7.2 Hybrid simulation + causal-inference workflows

The Navier-Stokes kernels added in Phase F6, combined with the existing `MHD` / `GRMHD` theories in `deep_causality_physics`, mean the same monorepo can run forward simulations *and* causal analysis on the resulting data. The hand-off is a `CausalTensor<f64>` field in both directions — no inter-language glue, no file-format gymnastics.

### 7.3 Counterfactual flow analysis

`PropagatingProcess::intervene` (the `do()` operator on the Causal Monad) rewrites the value mid-chain. In a fluid pipeline this means literally "what would the wake have looked like if this vortex had not formed?" — replace the offending TopologicalSignature in the history with a counterfactual; re-run the downstream SURD and graph emission; compare. This is computationally far cheaper than re-running the DNS.

### 7.4 Verified physics pipelines for safety-critical applications

The combination of (i) type-encoded conservation laws, (ii) context-threaded runtime invariants, (iii) Effect Ethos at the action boundary, and (iv) `EffectLog` audit trail covers the full chain from sensor data to actuator command with a structured correctness story at every layer. For wake-encounter avionics (Phase F8), atmospheric-sensor fusion, or computational hemodynamics, this is the architectural property that downstream regulators / certifiers care about.

---

## 8. Honest caveats

Listed for the next contributor.

### 8.1 The 12–20% leak estimate is a projection, not a measurement

The decomposition in section 2 is based on the published reconstruction errors and standard information-theoretic compounding. The actual measured leak from Phase F7 will land somewhere; the projection is the basis for the work, not its conclusion. If the measured leak is meaningfully higher than projected, the writeup must explain why honestly — likely candidates: residual-field contributions are larger than estimated, or topological signatures themselves lose more than expected in the discretization.

### 8.2 The methodological contribution is the composition, not any individual element

SURD is published (Martínez-Sánchez et al. 2024). Hodge decomposition is published (Hirani 2003, Hodge 1941). Takens-style state augmentation is published. Topological-signature features are folklore in TDA literature. **The novelty is putting them together under one type system with the lift-to-Markovian as the architectural primitive that connects spatial and temporal regimes.** Reviewers will know all the ingredients; the burden is to demonstrate that the composition itself is non-obvious and quantitatively useful.

### 8.3 The state augmentation is heuristic, not a proven Markovianization

True Mori-Zwanzig requires solving an infinite hierarchy. `RollingHistory<N>` is a finite truncation. For large enough `N` relative to the system's effective memory, the truncation is benign; for `N` too small, there is residual non-Markovianity in the augmented state that contributes to leak. Phase F7's ablation study is what makes this defensible — if leak monotonically decreases with `N` and saturates, the truncation is justified empirically; if it does not saturate, the bound `N ≪ memory-time` is being violated.

### 8.4 Topological signatures discard scalar field amplitudes by design

A `TopologicalSignature` says "there are two vortex tubes here" but only weakly says "and their strengths are X and Y". The component norms partially recover amplitude information but are coarse. For some causal questions this is the right level (the *existence* of a vortex pair is what causes wake hazard, not its third decimal); for others it is the wrong level. Phase F1's signature design needs to balance dimensional parsimony (for SURD's curse-of-dimensionality problem) against amplitude fidelity (for physical content). This is a real design trade-off without a universal answer.

### 8.5 Compressibility breaks the simplest invariant story

Strict incompressibility (Section 5 of this note) gives `SolenoidalField<F>` a clean meaning. Compressible flow has `∇·u ≠ 0` in general, and the type story must be different (`SolenoidalField` becomes inappropriate; the relevant invariant becomes the continuity equation `∂ρ/∂t + ∇·(ρu) = 0`). The Phase F5 invariants are scoped to incompressible flow as written. Compressible extension is a separate small change set, not a blocker.

### 8.6 The Lozano-Durán group will read this

If the work goes to publication, expect detailed scrutiny from the people whose methodology this proposal extends. The framing must be "extending SURD to a topologically-grounded state space with `PropagatingProcess` augmentation" — collaboration, not competition. Credit explicitly: SURD-states is theirs; the irreducible-error theorem is theirs (Yuan & Lozano-Durán 2025); the variational MI estimator infrastructure is theirs (MINE, etc.). The contribution is the structural pipeline that consumes their algorithm with a topologically-grounded feature extractor and state-augmented temporal accumulation.

---

## 9. Relationship to the broader roadmap

This work depends on prior change sets and unblocks downstream ones.

### Dependencies (prior change sets)

1. **`add-cubical-complexes` (Stage A–C)** — already shipped. `LatticeComplex<D>`, `CubicalReggeGeometry<D>` scaffolding, `Manifold::from_cubical_with_metric` constructors. No further work needed here.

2. **`add-cubical-regge-calculus` Phases R1–R4** from [`CubicalReggeCalculus.md`](./CubicalReggeCalculus.md) §3. Required: cubical Hodge ⋆ generic over `ChainComplex`. Without R4, the existing simplicial-only differential operators do not work on `LatticeComplex<3>` and the entire pipeline cannot start. **Phase R5 (Lorentzian) and R6 (Metropolis) are NOT required** by this proposal — they are needed only for compressible-flow causal-cone work and quantum-gravity dynamics, both of which are outside the scope of this note.

3. **`add-hodge-decomposition` Phases H1–H3** from [`CubicalReggeCalculus.md`](./CubicalReggeCalculus.md) §7. Required: `HasHodgeStar` trait, `HodgeDecomposition<F>` data structure, two-backend property tests. **Phases H4–H7 are NOT strictly required** by Phase F1–F8 of this proposal — F1 needs the decomposition primitive, not the per-edge metric closed form (H4) or the writeup (H7). Land H4 if a methods paper is the explicit goal; defer otherwise.

### What this unblocks

After this work ships, downstream change sets become straightforward:

- **`add-causal-flow-analysis-3d-compressible`** — extends to compressible flow. Adds Phase R5 (Lorentzian Hodge ⋆) and a `CompressibleFluidContext` that carries the sound speed. The causal-cone constraint becomes type-level. ~50h additional.
- **`add-causal-flow-analysis-4d-topology`** — adds `LatticeComplex<4>` for spacetime-topological invariants (vortex worldtube linking numbers). Justified only when the scientific question is genuinely 4D-topological. ~80h additional.
- **`add-causal-flow-counterfactuals`** — formalizes the `PropagatingProcess::intervene` workflow for systematic counterfactual flow analysis. Builds on the existing `intervene` operator. ~30h additional.
- **`add-medical-imaging-causal-analysis`** — same pipeline applied to medical imaging (blood flow in MRI, aneurysm-risk analysis). Substrate change from DNS data to clinical data; pipeline is unchanged. The existing `medicine_examples/aneurysm_risk` example would become a test bed. ~60h additional.

### Cumulative roadmap

In sequence:

1. **`add-cubical-regge-calculus` (R1–R4)** — ~12 hours (R5–R6 deferred per §9 above).
2. **`add-hodge-decomposition` (H1–H3)** — ~40 hours (H4–H7 partially deferred per §9 above).
3. **`add-3d-causal-fluid-dynamics`** (this proposal) — ~99 hours.

Steps 1–3 cumulative: ~150 hours focused work. Doable as a sustained 3–4 month effort by one developer, or split across multiple change sets with explicit stage gates per the protocol used in `add-cubical-complexes`.

The cumulative outcome: a Rust library that delivers structure-preserving discrete differential geometry on cubical complexes, uniform Hodge decomposition, **and** typed causal-graph reasoning over 3D fluid-flow data with state-augmented `PropagatingProcess` — a combination that doesn't exist elsewhere and lands DeepCausality in a small set of high-value applied research niches: turbulence analysis, wake-encounter avionics, medical-imaging flow analysis, and verifiable physics pipelines.

---

## 10. Suggested change-set naming

When this work is opened as a follow-up, suggested OpenSpec change name: **`add-3d-causal-fluid-dynamics`**.

Phases F1–F4 could be one change set (pipeline core: signature, history, SURD wiring, graph emission). Phases F5–F6 could be a second (physics integration: type-encoded invariants and reusable kernels). Phases F7–F9 are validation + example + writeup, naturally one change set.

The same stage-gate protocol used in `add-cubical-complexes` (per-stage sign-off + commit, no agent commits) is the recommended workflow.

---

## 11. Bottom line

The Martínez-Sánchez & Lozano-Durán group has built the state of the art in scalar information-theoretic causality for turbulence. Their acknowledged limitations — feature engineering via Gaussian filter, autoencoder + k-means quantization losing 33–67% of the causal structure, 2D analyses only, no verification layer — map almost one-to-one onto what discrete differential geometry plus typed causal reasoning plus the `PropagatingEffect` ↔ `PropagatingProcess` lift mechanism provide natively in the DeepCausality monorepo.

The pipeline proposed here:

- replaces lossy neural compression with lossless Hodge decomposition;
- replaces autoencoder + k-means quantization with topologically-grounded signatures;
- replaces ad-hoc time-pairing with a state-augmented Markovian process;
- consumes their SURD algorithm unchanged via the existing Rust port;
- emits an executable `CausaloidGraph` via the existing discovery → graph bridge;
- expresses physics invariants in types where they are structural and in the context channel where they are runtime-dependent;
- reserves the Effect Ethos for its proper role at the action boundary.

The estimated leak reduction is from 67% (their published measurement on 2D channel flow) to 12–20% (this proposal on 3D channel flow with state augmentation). The implementation effort is ~99 hours of focused work after the geometric foundation from `CubicalReggeCalculus.md` Phases R1–R4 + H1–H3 lands. The avionics integration (Phase F8) drops the pipeline into a real engineering setting — wake-encounter inference for aircraft — where each architectural choice (type-encoded conservation laws, context-threaded constraints, Ethos at the action boundary) earns its keep.

This is a publishable methods contribution, a useful engineering tool, and a clean architectural demonstration of what the DeepCausality stack can do once the topology layer ships. The pieces are sequenced; the dependencies are explicit; the validation strategy is concrete; the honest caveats are listed.
