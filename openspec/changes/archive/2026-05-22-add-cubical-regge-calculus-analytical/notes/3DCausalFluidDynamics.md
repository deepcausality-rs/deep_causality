# 3D Causal Fluid Dynamics with State-Augmented Propagating Process

**Status:** Forward-looking design note. Replaces the prior `f64`-typed sketch with an `R: RealField` parameterisation that aligns with the existing `CubicalReggeGeometry<const D, R>`, `ReggeGeometry<R>`, and `Manifold<K, R>` surface in `deep_causality_topology`.

**Scope of this note:** the *pipeline core*, *physics integration*, and *type-encoded invariants*. Reproduction of the published Martínez-Sánchez & Lozano-Durán (2026) measurement on JHU channel-flow DNS (the F7–F9 work in the prior draft) is **explicitly deferred** to a follow-up note `3DCausalFluidDynamicsValidation.md` to be opened after Blocks B1–B5 ship. The methodology stands or falls on the synthetic ground-truth test in B3; the JHU reproduction is a publishability concern, not a correctness concern.

**Prerequisites (must ship before B1 opens):**

1. `add-cubical-regge-calculus-analytical` (R4 + R5 + R6) — `HasHodgeStar<R>` capability trait, generic differential operators on `Manifold<K, R> where K::Metric: HasHodgeStar<R>`, signature marker `S ∈ {Euclidean, Lorentzian}` on `CubicalReggeGeometry<const D, R, S>`. **Proposed; not yet shipped.**
2. `add-hodge-decomposition` — `HodgeDecomposition<R>` carrier type and `Manifold::hodge_decompose(field, k) -> Result<HodgeDecomposition<R>, ManifoldError>`. **Does not exist; must be opened as a separate change set.** ~40h effort. This is the single largest dependency and the only piece with genuine mathematical novelty (per-edge cubical Hodge ⋆ closed form).

Without both prerequisites in place, Block B1 cannot start.

---

## 1. The methodological contribution

A typed pipeline of the following shape, with `R: RealField` flowing end-to-end:

```rust
PropagatingEffect::pure(dns_snapshot)
    .bind(|s, _, _| hodge_decompose_3d::<R>(s))
    .bind(|s, _, _| topological_signature::<R>(s))
    .lift_to_process(RollingHistory::<N, R>::new(), FluidContext::<R>::new(...))
    .bind(|sig, history, ctx| update_history(sig, history))
    .bind(|sig, history, ctx| surd_states_cdl(joint_distribution(sig, history)))
    .bind(|surd, history, ctx| emit_causaloid_graph(surd, history))
```

Each stage is a `bind` over `CausalEffectPropagationProcess`. The lift from `PropagatingEffect` to `PropagatingProcess` happens at a scientifically meaningful boundary: spatial decomposition (non-Markovian, parallelisable per timestep) gives way to temporal accumulation (Markovian, sequential). The state channel carries the rolling history of topological signatures; the context channel carries the lattice geometry and runtime physical invariants.

Novel composition (none of these elements is individually new; the combination under one type system with `R: RealField` precision is):

1. **Topological-signature quantisation** as state extraction. Replaces autoencoder + k-means with `(β₀, β₁, β₂, ‖exact‖, ‖co-exact‖, ‖harmonic‖, vortex_centroids, ...)` derived losslessly from `HodgeDecomposition<R>`.
2. **`PropagatingProcess` as temporal-accumulation primitive.** Mori–Zwanzig / Takens state augmentation expressed natively in the type system.
3. **SURD on the topological joint distribution.** Existing `surd_states_cdl` consumed unchanged; only the boundary cast from `R` to `f64` (SURD's required type) happens at the JointDistribution build step.
4. **Type-encoded fluid invariants** parameterised over `R`, matching the existing `Speed<R>` / `Mass<R>` / `FourMomentum<R>` convention in `deep_causality_physics/src/units/`.
5. **`CausaloidGraph` emission** via the existing SURD → graph bridge.

---

## 2. Precision parameterisation

Hard rule across all blocks: **no `f64` in any new public signature except at the documented SURD boundary in B3.** Every new type, trait, method, kernel, and newtype is parameterised over `R: RealField` (with `+ FromPrimitive` where literal construction is needed), mirroring the convention already established in [`CubicalReggeGeometry<const D, R>`](../../../../deep_causality_topology/src/types/cubical_regge_geometry/mod.rs#L88), [`ReggeGeometry<R>`](../../../../deep_causality_topology/src/types/regge_geometry/mod.rs#L15), and the wider topology / physics surface.

The single permitted lossy boundary: `TopologicalSignature<R>` → `JointDistribution<f64>` at the SURD input. SURD-states uses information-theoretic quantities whose precision saturates well within `f64`; the conversion is one-way and documented at the call site.

---

## 3. Block structure

Five independently-shippable blocks, each gated by three sequential checkpoints:

1. **Compilation gate.** All affected crates compile clean (`cargo build -p <crate>`) under release and debug profiles, with no new clippy warnings (`cargo clippy -p <crate> --all-targets -- -D warnings`). Fix lints at root cause; never suppress them with `#[allow(clippy::...)]` (per `feedback_clippy_lints`).
2. **Coverage gate.** 100% test coverage on every new or modified source file in the block, verified by the project's coverage tooling. Property tests, not just point tests, where the math admits them. Unreachable code is explicitly annotated and justified per AGENTS.md §"Code testing".
3. **Review gate.** The block is committed by the user (per AGENTS.md golden rule: agents never `git commit`). The user reviews the diff, runs `make format && make fix && make build && make test` if more than one crate changed, and explicitly signs off.

**No block opens until the prior block's three gates are closed.** Skipping ahead is forbidden; a gate failure rolls the block back to the in-progress state until the failure is addressed at root cause (no `#[allow(dead_code)]` workarounds for coverage gaps, no `#[allow(clippy::...)]` for lint failures — fix the code).

Total in-scope work: ~1300 LOC of library code, ~75 tests, ~26 hours of focused work after both prerequisites ship.

---

## Block B1 — Topological-signature feature extractor

**Goal.** Given a `HodgeDecomposition<R>` produced by `add-hodge-decomposition`, emit a fixed-size feature vector summarising the field's topology and per-component energy. This is the input shape consumed by every downstream block.

**Crate affected:** `deep_causality_topology` only.

**Where it lives:**

- New module: `deep_causality_topology/src/types/topological_signature/mod.rs` plus per-trait files following the one-type-one-module convention.
- Struct:
  ```rust
  pub struct TopologicalSignature<R: RealField> {
      betti_numbers: [usize; 4],
      exact_l2_norm: R,
      co_exact_l2_norm: R,
      harmonic_l2_norm: R,
      vortex_count: usize,
      vortex_centroids: Vec<[R; 3]>,
      dominant_helicity_sign: i8,
      integral_length_scale: R,
      taylor_microscale: R,
  }
  ```
- Constructor:
  ```rust
  impl<R: RealField + FromPrimitive> HodgeDecomposition<R> {
      pub fn topological_signature(&self) -> Result<TopologicalSignature<R>, ManifoldError>;
  }
  ```
- Fields are private per the AGENTS.md visibility rule; getters per field follow the project convention.

**Property tests:**

- **Translation invariance.** Shifting the field by an integer lattice vector preserves Betti numbers and component norms; centroids translate correspondingly. Tested on `LatticeComplex<3>` at unit-edge geometry.
- **Reflection invariance for unsigned components.** Betti numbers and L2 norms are reflection-invariant; helicity sign flips.
- **Hodge orthogonality.** `exact_l2_norm² + co_exact_l2_norm² + harmonic_l2_norm² = ‖field‖²` to numerical tolerance, verified on at least three lattice sizes and on both `Euclidean` cubical and simplicial backends.
- **Reproducibility.** Same input always produces a bit-identical signature.

**Effort:** ~250 LOC + ~12 tests. ~4 hours.

**Block gates:**

- [ ] B1-G1 Compilation: `cargo build -p deep_causality_topology` clean (release + debug), `cargo clippy -p deep_causality_topology --all-targets -- -D warnings` clean.
- [ ] B1-G2 Coverage: 100% on every file under `src/types/topological_signature/` and any modified file in `src/types/manifold/`.
- [ ] B1-G3 Review: user reviews, runs `make format && make fix`, signs off, commits.

---

## Block B2 — `RollingHistory<N, R>` state + `PropagatingProcess` integration

**Goal.** Typed rolling-window state carrier with O(1) push and O(N) snapshot, plus the lift-to-Markovian boundary that turns the per-timestep signature stream into a `PropagatingProcess`.

**Crate affected:** `deep_causality_physics` (state carrier) + integration shim against `deep_causality_core` (no changes to core).

**Decision: where does `RollingHistory` live?** It is fluid-domain-specific. Recommendation: `deep_causality_physics/src/fluids/rolling_history/`, accepting the new dependency edge from `deep_causality_physics` to `deep_causality_topology` (for `TopologicalSignature<R>`) and to `deep_causality_data_structures` (for `ArrayDeque`). If the inbound dep on `deep_causality_topology` is rejected at B2-G3 review, fall back to a new `deep_causality_fluid_state` crate carrying only the state carrier. Decide at B2 kickoff before any code is written.

**Where it lives:**

```rust
pub struct RollingHistory<const N: usize, R: RealField> {
    signatures: ArrayDeque<TopologicalSignature<R>, N>,
    integrated_helicity: R,
    integrated_enstrophy: R,
    cumulative_dissipation: R,
    timestamp: TimeStep,
}

impl<const N: usize, R: RealField + FromPrimitive> RollingHistory<N, R> {
    pub fn new() -> Self;
    pub fn push(&mut self, sig: TopologicalSignature<R>);
    pub fn latest(&self) -> Option<&TopologicalSignature<R>>;
    pub fn window(&self) -> &[TopologicalSignature<R>];
    pub fn helicity_trajectory(&self) -> Vec<R>;
    pub fn enstrophy_trajectory(&self) -> Vec<R>;
}
```

The `PropagatingProcess` integration is one constructor call against existing `deep_causality_core` API:

```rust
PropagatingProcess::with_state(
    effect,
    RollingHistory::<N, R>::new(),
    FluidContext::<R>::new(geometry, reynolds, ...),
)
```

No changes to `deep_causality_core`.

**Property tests:**

- **FIFO invariant.** After `2N` pushes, only the last `N` signatures remain.
- **Integrated quantities non-negativity.** Enstrophy is always ≥ 0; `|helicity|` is bounded by the Cauchy-Schwarz product of `‖u‖` and `‖ω‖`.
- **Lift round-trip.** `PropagatingEffect::pure(sig).lift_to_process(history, ctx).bind(...)` is type-equivalent to a `PropagatingProcess` chain initialised with the same data.
- **Window slice stability.** `window()` returns a slice consistent with insertion order across N pushes.

**Effort:** ~200 LOC + ~10 tests. ~3 hours.

**Block gates:**

- [ ] B2-G1 Compilation: every affected crate clean. If a new dep edge is added to `deep_causality_physics`, document it in the crate's `Cargo.toml` and `BUILD.bazel`.
- [ ] B2-G2 Coverage: 100% on every new file.
- [ ] B2-G3 Review: user signs off on the crate-boundary decision (physics-hosted vs. new crate) at this gate.

---

## Block B3 — `FluidContext<R>`, SURD wiring, and `CausaloidGraph` emission

**Goal.** Stand up the runtime-invariant context, wire the augmented `(signature, history)` joint into `surd_states_cdl`, and emit a `CausaloidGraph` via the existing discovery → graph bridge. This is the block where the methodology becomes end-to-end testable on synthetic ground truth.

**Crates affected:** `deep_causality_physics` (FluidContext) + `deep_causality_discovery` (SURD wiring + graph emission).

**Where it lives:**

- `deep_causality_physics/src/fluids/fluid_context/mod.rs`:
  ```rust
  pub struct FluidContext<R: RealField> {
      lattice_geometry: Arc<CubicalReggeGeometry<3, R, Euclidean>>,
      reynolds_number: Reynolds<R>,
      sound_speed: Option<Speed<R>>,
      wall_normal_axis: Axis,
      periodic_axes: [bool; 3],
      forcing_term: Option<ForcingProfile<R>>,
  }
  ```
  `Reynolds<R>`, `ForcingProfile<R>`, `Axis` are introduced here following the existing `Speed<R>` newtype convention from `deep_causality_physics/src/units/`.

- `deep_causality_discovery/src/types/fluid_surd/mod.rs`:
  ```rust
  pub fn fluid_surd_decompose<const N: usize, R: RealField + FromPrimitive>(
      sig: &TopologicalSignature<R>,
      history: &RollingHistory<N, R>,
      ctx: &FluidContext<R>,
  ) -> Result<SurdResult<f64>, CdlError>;
  ```
  Internally builds a `CausalTensor<Option<f64>>` joint distribution by casting `R → f64` once at the tensor-build step (the documented lossy boundary). Calls existing `surd_states_cdl(tensor, MaxOrder::Max)` unchanged.

- `deep_causality_discovery/src/types/analyzer/` extended with `FluidSurdResultAnalyzer` that consumes a `SurdResult<f64>` and emits a `CausaloidGraph` per the existing mapping documented in [`deep_causality_discovery/README.md`](../../../../deep_causality_discovery/README.md):
  - Strong unique influence → direct edge.
  - Strong synergy → many-to-one via `AggregateLogic::All`.
  - Strong redundancy → many-to-one via `AggregateLogic::Any`.
  - High causality leak → annotate target Causaloid with a stochasticity term.

**Synthetic ground-truth test (the critical correctness check):**

Adapt the Martínez-Sánchez & Lozano-Durán (2026) §3 three-variable benchmark — explicitly prescribed synergistic / unique / redundant dependencies — into a 3D version on `LatticeComplex<3>`. Apply the full B1+B2+B3 pipeline. The emitted `CausaloidGraph` must recover the prescribed structure to within the test tolerance. **This test is the single most important correctness gate in the entire roadmap.** If it fails, the methodology is broken regardless of how the deferred JHU reproduction lands.

**Property tests:**

- **Information-leak bound.** `0 ≤ info_leak ≤ H(target)` for every emitted SURD result.
- **Sum constraint.** Redundant + unique + synergistic + leak = total mutual information to numerical tolerance (the SURD theorem).
- **Synergy non-negativity.** Synergistic component is always ≥ 0.
- **Graph acyclicity.** When temporal ordering is respected, the emitted graph is a DAG.
- **Synthetic ground-truth recovery.** Per above; full end-to-end test on the 3D-adapted §3 benchmark.

**Effort:** ~400 LOC + ~18 tests. ~7 hours.

**Block gates:**

- [ ] B3-G1 Compilation: both crates clean.
- [ ] B3-G2 Coverage: 100% on every new file. The synthetic-ground-truth test counts as a regression test, not as coverage of the analyzer itself; the analyzer needs its own unit tests in addition.
- [ ] B3-G3 Review: user signs off, with explicit acknowledgement that the synthetic-ground-truth test passed. If that test fails, B3 does not close and the methodology is reopened for design review before B4 starts.

---

## Block B4 — Type-encoded fluid invariants

**Goal.** Ship the newtype wrappers and smart constructors that make conservation laws structural rather than runtime-checked. Construction-time validity replaces runtime conservation checks for the cases where the invariant is structural.

**Crate affected:** `deep_causality_physics` only.

**Where it lives:**

`deep_causality_physics/src/fluids/quantities/` with one file per newtype following the one-type-one-module rule:

```rust
pub struct SolenoidalField<R: RealField> { /* private */ }
pub struct Circulation<R: RealField> { /* private */ }
pub struct Vorticity<R: RealField> { /* private */ }
pub struct Helicity<R: RealField> { /* private */ }
```

Smart constructors (the only way to construct each type):

| Newtype | Invariant | Constructor signature |
|---|---|---|
| `SolenoidalField<R>` | `∇·u = 0` | `from_hodge_projection(field: CausalTensor<R>, hodge: &HodgeDecomposition<R>) -> Self` (returns the co-exact part) |
| `Circulation<R>` | Kelvin's theorem | `from_loop_integral(velocity: &CausalTensor<R>, loop_cells: &[CellId]) -> Self` |
| `Vorticity<R>` | `dω = 0` (closed 2-form) | `from_velocity<K>(velocity_1form: &CausalTensor<R>, manifold: &Manifold<K, R>) -> Self where K: ChainComplex, K::Metric: HasHodgeStar<R>` |
| `Helicity<R>` | `H = ∫ u · ω dV` | `from_field_pair<K>(u: &SolenoidalField<R>, omega: &Vorticity<R>, domain: &Manifold<K, R>) -> Self` |

The compiler refuses constructions that bypass the invariants. Consumers receiving `&SolenoidalField<R>` know the field is divergence-free by construction.

**Property tests:**

- **Construction-time divergence.** `SolenoidalField` from Hodge projection has `‖∇·u‖ < ε_R` re-checked via the discrete divergence operator, with `ε_R` derived from `R`'s machine epsilon via `FromPrimitive`.
- **Discrete Stokes' theorem.** `Circulation` around a closed loop equals the surface integral of `Vorticity` over any spanning surface to discretisation tolerance.
- **Helicity in ideal flow.** Forward Euler step under inviscid evolution preserves total helicity to second order in the timestep.
- **Type safety (compile-fail tests).** The wrong-type construction path does not exist (e.g. you cannot construct `Helicity` from a raw `CausalTensor<R>`; only from `&SolenoidalField<R>` and `&Vorticity<R>`).

**Effort:** ~300 LOC + ~15 tests. ~5 hours.

**Block gates:**

- [ ] B4-G1 Compilation: clean, including the compile-fail tests in `tests/`.
- [ ] B4-G2 Coverage: 100% on every new file. Smart constructors are the only constructor path; coverage of all error returns is mandatory.
- [ ] B4-G3 Review: user signs off.

---

## Block B5 — Reusable pointwise Navier–Stokes kernels

**Goal.** Ship the pointwise Navier–Stokes kernels that fit the existing `deep_causality_physics` kernel pattern: stateless, side-effect-free, pure algebra given pre-discretised inputs, generic over `R: RealField`.

**Crate affected:** `deep_causality_physics` only.

**Where it lives:** Extends `deep_causality_physics/src/fluids/`. Each kernel is a free `pub fn` following the existing `Fluids` kernel convention; no kernel takes `&self`.

**Kernel signatures (all generic over `R: RealField`, with `+ FromPrimitive` where literals are needed):**

```rust
pub fn convective_acceleration_kernel<R: RealField>(
    u: &[R; 3], grad_u: &[[R; 3]; 3],
) -> [R; 3];

pub fn viscous_diffusion_kernel<R: RealField>(
    nu: R, laplacian_u: &[R; 3],
) -> [R; 3];

pub fn pressure_gradient_force_kernel<R: RealField>(
    rho: R, grad_p: &[R; 3],
) -> [R; 3];

pub fn vorticity_transport_kernel<R: RealField>(
    omega: &[R; 3], u: &[R; 3], grad_omega: &[[R; 3]; 3],
    laplacian_omega: &[R; 3], nu: R,
) -> [R; 3];

pub fn q_criterion_kernel<R: RealField + FromPrimitive>(
    velocity_gradient: &[[R; 3]; 3],
) -> R;

pub fn lambda2_kernel<R: RealField + FromPrimitive>(
    velocity_gradient: &[[R; 3]; 3],
) -> R;

pub fn kolmogorov_scale_kernel<R: RealField + FromPrimitive>(epsilon: R, nu: R) -> R;
pub fn taylor_microscale_kernel<R: RealField + FromPrimitive>(k_energy: R, epsilon: R, nu: R) -> R;
pub fn integral_length_scale_kernel<R: RealField + FromPrimitive>(k_energy: R, epsilon: R) -> R;
```

No kernel takes a manifold, a context, or any non-algebraic input. Discretisation and assembly happen outside the kernel.

**Property tests:**

- **Galilean invariance.** `convective_acceleration_kernel(u + c, grad_u) == convective_acceleration_kernel(u, grad_u)` for any constant velocity `c`. Tested across `R ∈ {f32, f64, DoubleFloat}`.
- **Dimensional consistency.** Every kernel is exercised with `units::*` newtypes wrapping `R`; the type system catches dimensional mismatches at compile time.
- **Limiting cases.** As `Re → ∞`, viscous diffusion vanishes (recovers Euler). As `Re → 0`, convective acceleration is negligible relative to viscous diffusion (recovers Stokes flow). Tested numerically with prescribed ratios.
- **Precision robustness.** The Q-criterion algebraic identity `Q + 0.5 * ‖S‖² = 0.5 * ‖Ω‖²` holds for `f32`, `f64`, and `DoubleFloat` to each backend's expected tolerance.

**Effort:** ~400 LOC + ~25 tests. ~7 hours.

**Block gates:**

- [ ] B5-G1 Compilation: clean across all three precision backends (`f32`, `f64`, `DoubleFloat`).
- [ ] B5-G2 Coverage: 100% on every new file. Precision-backend coverage is enforced by parameterised tests.
- [ ] B5-G3 Review: user signs off. After B5-G3, the roadmap of this note is complete; validation (the deferred F7–F9) becomes its own change set.

---

## 4. Deferred work (was F7–F9 in the prior draft)

The following is **explicitly out of scope** for this note and must be opened as a separate follow-up note `3DCausalFluidDynamicsValidation.md` only after B5-G3 closes:

- **Validation on JHU Turbulence Database** (was F7). Reproduce Martínez-Sánchez & Lozano-Durán (2026) §4.1 at `Re_τ ≈ 950`. Two-branch comparison against the published autoencoder + k-means pipeline. ~25 hours plus data wrangling. Success criterion: leak below 25% on the same data class.
- **Avionics wake-vortex example** (was F8). Subsonic, incompressible wake only; compressible extension is a further downstream change set. ~30 hours. Lives under `examples/avionics_examples/wake_vortex_causal_inference/` as its own workspace crate depending on `deep_causality`, `deep_causality_physics`, `deep_causality_topology`, `deep_causality_discovery`, `deep_causality_ethos`.
- **Methods writeup** (was F9). ~15 hours.

Rationale for deferral: validation reproduces a published measurement and is a publishability concern, not a correctness concern. The methodology stands or falls on the **synthetic ground-truth test in B3** (the 3D-adapted §3 benchmark of the same paper), which is fully self-contained and lands inside this note's scope. If B3-G3 passes, the methodology is established; if it fails, no amount of JHU reproduction would recover it.

---

## 5. Cumulative effort and ordering

| Order | Change set | Status | Effort | Gates |
|---|---|---|---|---|
| 1 | `add-cubical-regge-calculus-analytical` (R4 + R5 + R6) | Proposed; needs `R: RealField` refinement (see proposal review) | ~15h | Prerequisite |
| 2 | `add-hodge-decomposition` (H1–H3) | **Does not exist; must be opened** | ~40h | Prerequisite |
| 3 | Block B1 — TopologicalSignature | This note | ~4h | B1-G1, B1-G2, B1-G3 |
| 4 | Block B2 — RollingHistory + lift | This note | ~3h | B2-G1, B2-G2, B2-G3 |
| 5 | Block B3 — FluidContext + SURD + graph | This note | ~7h | B3-G1, B3-G2, B3-G3 |
| 6 | Block B4 — Type-encoded invariants | This note | ~5h | B4-G1, B4-G2, B4-G3 |
| 7 | Block B5 — Pointwise NS kernels | This note | ~7h | B5-G1, B5-G2, B5-G3 |
| — | `3DCausalFluidDynamicsValidation.md` (was F7–F9) | **Deferred** | ~70h | Outside this note |

Total in-scope after prerequisites: **~26 hours focused work, ~1300 LOC, ~75 tests, 15 gates**. Plus ~55 hours of prerequisite work (R4–R6 + H1–H3) before B1 can open.

The gating discipline is strict: no block opens with an unclosed gate from the prior block; no gate closes without compilation, full coverage, and explicit user review. Per AGENTS.md, agents never commit; every G3 sign-off is the user committing the block.

---

## 6. Honest caveats

- **The synthetic ground-truth test in B3 is the single load-bearing correctness gate.** If it fails, the methodology is broken and no later block recovers it. Allocate explicit design review time at B3-G3 if the result is borderline.
- **`RollingHistory<N>` is a finite Mori–Zwanzig truncation, not a proven Markovianisation.** For large enough `N` relative to the system's effective memory the truncation is benign; the ablation study that would defend `N` empirically lives in the deferred validation note, not here. B1–B5 ship without that empirical defence.
- **Topological signatures discard amplitude information by design.** A `TopologicalSignature<R>` says "two vortex tubes exist" with their L2 norms but is coarse on per-vortex amplitude. For the causal questions this pipeline targets (existence and interaction of coherent structures), this is the right level. For other questions it is the wrong level. The signature design in B1 trades dimensional parsimony (for SURD's curse-of-dimensionality problem) against amplitude fidelity.
- **The SURD precision boundary is a one-way cliff.** Converting `R → f64` at the JointDistribution build step in B3 is the single permitted lossy cast. Information-theoretic quantities saturate `f64` well within sample-noise floors; the loss is real but does not affect SURD's outputs at any tested precision backend. Document at the call site.
- **Compressibility breaks `SolenoidalField<R>`.** B4's invariants are scoped to incompressible flow. Compressible extension (`∇·u ≠ 0`; continuity equation as the relevant invariant) is a separate small change set, not a blocker.

---

## 7. Bottom line

Five gated blocks. Each generic over `R: RealField`. Each one independently shippable and reviewable. The full path from `HodgeDecomposition<R>` to executable `CausaloidGraph` lands in ~26 hours of focused work after the two prerequisite change sets ship. Validation against the published 67%-leak baseline is deferred to a separate note; the synthetic ground-truth test inside B3 is the correctness gate that this work stands or falls on.
