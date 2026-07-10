# 3D Causal Fluid Dynamics with State-Augmented Propagating Process

**Status:** Forward-looking design note. Replaces the prior conflated-`TopologicalSignature` sketch with a clean topology/physics split aligned to crate-responsibility boundaries, and an explicit typed pipeline through `CausalEffectPropagationProcess`.

**Revision 2026-06-10.** Brought in line with `cfd-gap.md` (ground truth for the DEC/solver foundation, assumed closed) — see prerequisite 4, the B1b/B5 kernel-status corrections, the B4 type unification, the §1.1 solver synergy, and the §7 periodic-lattice caveat update. Sequencing across the three CFD notes lives in `cfd-roadmap.md`.

**Scope of this note:** the *pipeline core*, *physics integration*, and *type-encoded invariants*. Reproduction of the published Martínez-Sánchez & Lozano-Durán (2026) measurement on JHU channel-flow DNS is **explicitly deferred** to a follow-up note `3DCausalFluidDynamicsValidation.md` to be opened after Blocks B1b–B5 ship. The methodology stands or falls on the synthetic ground-truth test in B3; the JHU reproduction is a publishability concern, not a correctness concern.

**Prerequisites (must ship before B1b opens):**

1. `add-cubical-regge-calculus-analytical` (R4 + R5 + R6) — `HasHodgeStar<R>` capability trait, generic differential operators on `Manifold<K, R> where K::Metric: HasHodgeStar<R>`, signature marker `S ∈ {Euclidean, Lorentzian}` on `CubicalReggeGeometry<const D, R, S>`. **Shipped 2026-05-22.**
2. `add-hodge-decomposition` — `HodgeDecomposition<R>` carrier type and `Manifold::hodge_decompose(field, k) -> Result<HodgeDecomposition<R>, TopologyError>`. **Shipped 2026-05-22**, archived as `openspec/changes/archive/2026-05-22-add-hodge-decomposition/`.
3. `B1a: TopologicalInvariants extractor` — pure-topology product of the Hodge decomposition (Betti numbers + per-component L2 norms). **Shipped 2026-05-22** as part of this note. `pub fn HodgeDecomposition::topological_invariants<K>(&self, &Manifold<K, R>) -> Result<TopologicalInvariants<R>, TopologyError>` is live in `deep_causality_topology`.
4. **The `cfd-gap.md` foundation (G1–G6)** — *assumed closed as of the 2026-06-10 revision of this note.* Relevant APIs: the DEC wedge and interior product (G1), the de Rham/♯ isos (G2), the typed-form carriers in the physics crate (G3, including the shared `SolenoidalField<R>` — see B4), pinned sign/orientation conventions (G4), and **G6 harmonic-kernel deflation**, which lifts this note's original periodic-lattice restriction (see §7).

Without all four prerequisites in place, B1b cannot start.

---

## 1. The methodological contribution

A typed pipeline of the following shape, with `R: RealField` flowing end-to-end and every `bind` producing one typed value. The pipeline splits cleanly along the API surface exposed by `deep_causality_core`: stage 1 uses the trait-based `deep_causality_haft::Monad::bind` (clean closure, raw value in, no `EffectValue` boilerplate); stage 2 uses the inherent `bind` on `CausalEffectPropagationProcess` (closure receives `(EffectValue<Value>, State, Option<Context>)` so it can read state and context).

```rust
use deep_causality_haft::{Monad, Pure};
use deep_causality_core::{
    PropagatingEffect, PropagatingEffectWitness, PropagatingProcess,
    EffectValue, CausalityError, EffectLog,
};

type EffectW = PropagatingEffectWitness<CausalityError, EffectLog>;

// ── Stage 1 — per-timestep decomposition + dual extraction ──
// PropagatingEffect, non-Markovian, parallelisable per snapshot.
// External invariants (manifold, nu) captured by reference in the closure environment.

let snapshot_eff: PropagatingEffect<CausalTensor<R>>      = EffectW::pure(snapshot);
let hodge_eff:    PropagatingEffect<HodgeDecomposition<R>> = EffectW::bind(snapshot_eff,
    |raw| EffectW::pure(hodge_decompose(raw, &manifold)));
let signature_eff: PropagatingEffect<FluidSignature<R>>    = EffectW::bind(hodge_eff,
    |hd|  EffectW::pure(compose_fluid_signature(&hd, &manifold, nu)?));

// ── Lift Effect → Process at the spatial/temporal boundary ──
// Associated function on PropagatingProcess; not a fluent method on Effect.

let process = PropagatingProcess::<FluidSignature<R>, RollingHistory<N, R>, FluidContext<R>>::with_state(
    signature_eff,
    RollingHistory::new(),
    Some(FluidContext::new(lattice_geometry, reynolds, ...)),
);

// ── Stage 2 — temporal accumulation + SURD attribution ──
// PropagatingProcess, Markovian, sequential. Closure receives EffectValue + State + Option<Context>
// so it can read history and context as the rolling state advances.

let surd_proc = process.bind(|ev, history, ctx| {
    let sig = match ev { EffectValue::Value(s) => s, _ => return PropagatingProcess::from_error(...) };
    let mut new_history = history.clone();
    new_history.push(sig.clone());
    let surd = fluid_surd_decompose(&sig, &new_history, ctx.as_ref().unwrap())?;
    PropagatingProcess { value: EffectValue::Value(surd), state: new_history, context: ctx,
                         error: None, logs: EffectLog::new() }
});

// ── Consume ──
// The SurdResult<f64> is consumed by the existing SurdResultAnalyzer in
// deep_causality_discovery, which produces a ProcessAnalysis (Vec<String>) of
// human-readable categorisations against caller-supplied thresholds. No new
// SURD-output consumer is built in this note; downstream consumers (causal
// graph emission, dashboarding, ablation studies) attach to the same SurdResult
// via their own analyzers in their own change sets.

use deep_causality_discovery::{SurdResultAnalyzer, ProcessResultAnalyzer, AnalyzeConfig};

let analyzer = SurdResultAnalyzer;
let report = match surd_proc.value {
    EffectValue::Value(ref surd) => analyzer.analyze(surd, &AnalyzeConfig::default())?,
    _ => return Err(/* propagate the process's error state */),
};
```

Why the two-API split:

- The trait-based `Monad::bind` from `deep_causality_haft` is the idiomatic clean-closure API: `Func: FnOnce(A) -> Self::Type<B>`, the closure receives raw `A`, no `EffectValue` pattern matching, no `Default` bound on `B` (both witnesses set `Constraint = NoConstraint`). Stage 1 is context-free, so this is the natural fit.
- The inherent `bind` on `CausalEffectPropagationProcess` is the API that gives the closure read access to `State` and `Option<Context>`. Stage 2 needs that — the rolling-history push and the SURD-context lookup both happen inside the closure body — so the heavier closure shape `(EffectValue<Value>, State, Option<Context>) -> CausalEffectPropagationProcess<NewValue, …>` is the correct ergonomic cost for what state accumulation actually does.

The consistency tests at [`deep_causality_core/tests/iso/effect_process_consistency_tests.rs`](../../../../deep_causality_core/tests/iso/effect_process_consistency_tests.rs) pin the iso between the two witnesses on the shared carrier `CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`. The same operation produces bit-identical output through either dispatch path; this guarantees that the trait-based stage 1 and the inherent-bind stage 2 compose without semantic surprises.

The lift from `PropagatingEffect` to `PropagatingProcess` happens at a scientifically meaningful boundary: spatial decomposition (non-Markovian, parallelisable per timestep) gives way to temporal accumulation (Markovian, sequential). The state channel carries the rolling history of signatures; the context channel carries the lattice geometry and runtime physical invariants.

`FluidSignature<R>` is the *combined* per-timestep carrier:

```rust
pub struct FluidSignature<R: RealField> {
    topology: TopologicalInvariants<R>,    // from deep_causality_topology (B1a, shipped)
    physics:  FluidPhysicsInvariants<R>,   // from deep_causality_physics  (B1b, this note)
}
```

The two halves come from two different crates because they describe two different things:

- **`TopologicalInvariants<R>`** lives in `deep_causality_topology` because Betti numbers and Hodge-component L2 norms are properties of the discretisation, not of the velocity field. Pure topology.
- **`FluidPhysicsInvariants<R>`** lives in `deep_causality_physics` because vortex centroids, helicity sign, and turbulence length scales are properties of the velocity field viewed as a physical state, not properties of the discretisation. Pure physics.

The original B1 design conflated these into a single `TopologicalSignature` in the topology crate, which was a crate-boundary violation. This rewrite restores the boundary.

### Novel composition under one type system

None of these elements is individually new. The combination under one type system with `R: RealField` precision, with explicit responsibility boundaries between crates, is:

1. **Lossless decomposition-based feature extraction**, replacing autoencoder + k-means with `(β₀..β₃, ‖α‖, ‖β‖, ‖h‖, vortex_count, vortex_centroids, helicity_sign, length_scales)` derived from `HodgeDecomposition<R>`.
2. **`PropagatingProcess` as temporal-accumulation primitive.** Mori–Zwanzig / Takens state augmentation expressed natively in the type system.
3. **SURD on the augmented joint distribution.** Existing `surd_states_cdl` consumed unchanged; only the boundary cast from `R` to `f64` (SURD's required type) happens at the `JointDistribution` build step inside `fluid_surd_decompose`.
4. **Type-encoded fluid invariants** parameterised over `R`, matching the existing `Speed<R>` / `Mass<R>` / `FourMomentum<R>` convention in `deep_causality_physics/src/units/`.

### 1.1 Solver synergy: the decomposition is a free byproduct of the solve

With the `cfd-gap.md` solver in place, this pipeline gains an in-house data source.
The solver's Leray projection evaluates the gradient half of the Hodge decomposition
of the velocity 1-form at **every time step**, on the same `Manifold`, at the same
`R`. Stage 1 of this pipeline consumes exactly a `HodgeDecomposition<R>` of a
velocity 1-form. Tapping the solve chain therefore costs one β-step solve per
*sampled* snapshot (G6 makes that solve well-posed on tori) rather than a full
standalone decomposition pipeline — and the snapshot cadence is a free parameter of
the tap, not of the solver.

Consequences:

- The synthetic ground-truth test in B3 can be driven by solver-generated flows with
  *prescribed* causal structure (forced regimes on the torus), not only by the
  adapted three-variable benchmark.
- The deferred JHU validation note gains a second branch: external DNS (JHU) and
  in-house DNS (the cfd-gap solver at Taylor–Green/Re-1600 scale) through one
  pipeline.
- `simulate → decompose (free) → attribute` is the program's flagship loop; no other
  CFD code has a projection step that *is* a Hodge decomposition. Scheduling lives in
  `cfd-roadmap.md` Stage 2.

---

## 2. Precision parameterisation

Hard rule across all blocks: **no `f64` in any new public signature except at the documented SURD boundary in B3.** Every new type, trait, method, kernel, and newtype is parameterised over `R: RealField` (with `+ FromPrimitive` where literal construction is needed), mirroring the convention already established in `CubicalReggeGeometry<const D, R>`, `ReggeGeometry<R>`, `Manifold<K, R>`, `HodgeDecomposition<R>`, and `TopologicalInvariants<R>`.

The single permitted lossy boundary: `FluidSignature<R>` → `JointDistribution<f64>` at the SURD input. SURD-states uses information-theoretic quantities whose precision saturates well within `f64`; the conversion is one-way and documented at the call site in `fluid_surd_decompose`.

---

## 3. Block structure

Six independently-shippable blocks. Each block is gated by three sequential checkpoints:

1. **Compilation gate.** All affected crates compile clean (`cargo build -p <crate>`) under release and debug profiles, with no new clippy warnings (`cargo clippy -p <crate> --all-targets -- -D warnings`). Fix lints at root cause; never suppress them with `#[allow(clippy::...)]` (per `feedback_clippy_lints`).
2. **Coverage gate.** 100% test coverage on every new or modified source file in the block, verified by the project's coverage tooling. Property tests, not just point tests, where the math admits them. Unreachable code is explicitly annotated and justified per AGENTS.md §"Code testing".
3. **Review gate.** The block is committed by the user (per AGENTS.md golden rule: agents never `git commit`). The user reviews the diff, runs `make format && make fix && make build && make test` if more than one crate changed, and explicitly signs off.

**No block opens until the prior block's three gates are closed.** Skipping ahead is forbidden; a gate failure rolls the block back to the in-progress state until the failure is addressed at root cause (no `#[allow(dead_code)]` workarounds for coverage gaps, no `#[allow(clippy::...)]` for lint failures — fix the code).

Total in-scope work after prerequisites: ~1000 LOC of library code, ~78 tests, ~22 hours of focused work.

---

## Block B1a — Topological invariants extractor (SHIPPED)

**Status:** ✅ Shipped 2026-05-22 in `deep_causality_topology` as part of this note's prerequisite chain. Documented here for completeness; no work remains.

`TopologicalInvariants<R>` carries the four Betti numbers `[β_0, β_1, β_2, β_3]` (zero-padded beyond `max_dim`) and the three component L2 norms `(‖α‖, ‖β‖, ‖h‖)` of a Hodge decomposition.

```rust
impl<R: RealField> HodgeDecomposition<R> {
    pub fn topological_invariants<K>(
        &self,
        manifold: &Manifold<K, R>,
    ) -> Result<TopologicalInvariants<R>, TopologyError>
    where K: ChainComplex;
}
```

33 tests cover the Hodge orthogonality identity, Betti-number consistency on both contractible and torus-topology lattices, reproducibility, sign-flip invariance, and the multi-precision compile-pass surface.

---

## Block B1b — Fluid physics invariants extractor

**Goal.** Given a `HodgeDecomposition<R>` of a velocity 1-form on a manifold and a kinematic viscosity, emit a `FluidPhysicsInvariants<R>` carrying field-theoretic invariants of the velocity field: vortex count and centroids, helicity sign, integral length scale, Taylor microscale.

**Crate affected:** `deep_causality_physics` only. First dependency edge to `deep_causality_topology` (consuming `HodgeDecomposition<R>` and `Manifold<K, R>` types). Document the new dep in `Cargo.toml` and `BUILD.bazel`.

**Where it lives:**

- New module: `deep_causality_physics/src/fluids/physics_invariants/mod.rs` plus per-trait files following the one-type-one-module convention.
- Struct:
  ```rust
  pub struct FluidPhysicsInvariants<R: RealField> {
      vortex_count: usize,
      vortex_centroids: Vec<[R; 3]>,
      dominant_helicity_sign: i8,
      integral_length_scale: R,
      taylor_microscale: R,
  }
  ```
- Constructor (free function for ergonomic crate-boundary):
  ```rust
  pub fn fluid_physics_invariants<K, R>(
      velocity_decomposition: &HodgeDecomposition<R>,
      manifold: &Manifold<K, R>,
      kinematic_viscosity: R,
  ) -> Result<FluidPhysicsInvariants<R>, FluidExtractionError>
  where
      K: ChainComplex,
      K::Metric: HasHodgeStar<R, Complex = K>,
      R: RealField + FromPrimitive + Display;
  ```
- Fields are private; getters per field follow the project convention.

**Computation (consumes the public kernels already shipped in
`deep_causality_physics/src/kernels/fluids/` — Q-criterion, λ₂, turbulence scales,
vorticity transport all exist there with tests; B1b adds no private formula helpers,
only the extraction/labelling logic around the shipped kernels. See the revised B5.)**

- `vortex_count`, `vortex_centroids` — Q-criterion `Q = 0.5·(‖Ω‖² − ‖S‖²)` evaluated at every cell from the velocity gradient `∇u` (recovered from the decomposition's solenoidal part `β + h`). Connected-component labelling on cells where `Q > threshold` yields count + centroids.
- `dominant_helicity_sign` — sign of integrated helicity `∫ u · ω dV` where `ω = ∇ × u` is the vorticity 2-form computed by `Manifold::exterior_derivative` on `β + h`.
- `integral_length_scale` — `L = k_energy^(3/2) / ε` where `k_energy = 0.5·‖u‖²` and `ε = ν·‖∇u‖²`.
- `taylor_microscale` — `λ = sqrt(15·ν·k_energy / ε)`.

The 3D-only `vortex_centroids: Vec<[R; 3]>` field is checked at construction time; non-3D manifolds return `FluidExtractionError::Non3DManifold`.

**Robustness caveat and comparison target (references.md: Abolholl-2024).**
Teschner's group documents that Q, Δ, and swirling-strength criteria all detect
spurious vortices (false positives *and* negatives) — "vortex core detection
remains an unsolved problem" — and validates a hybrid CNN+DNN detector on the
Taylor–Green vortex, the same benchmark this deck uses everywhere. Two
consequences for B1b: (a) the Q-threshold + connected-components extractor must
treat its threshold as a declared, tested parameter, not a constant — the property
tests should include a threshold-sensitivity sweep on the TG case; (b) this
pipeline holds a structural alternative no pointwise criterion has: the vortex
information enters through the *Hodge decomposition* (global, topological — the
solenoidal component and the Betti numbers), so "topological vortex detection vs.
local criteria, on the shared TG benchmark, against the Abolholl hybrid-ML
baseline" is a publishable comparison that lands precisely on a problem the field
itself calls unsolved. Flag it as a candidate section of the deferred validation
note.

**Property tests:**

- **Galilean invariance of vortex detection.** Adding a constant velocity to the field preserves the Q-criterion field; vortex_count and vortex_centroids are unchanged.
- **Helicity sign flips under spatial reflection.** Reflecting the velocity field along one axis flips `dominant_helicity_sign`.
- **Length-scale relation.** `taylor_microscale² = 15 · ν² / (ε / k_energy)` holds to numerical tolerance, exercised on prescribed input pairs.
- **Construction-time dimension check.** Non-3D manifolds produce `FluidExtractionError::Non3DManifold`.
- **Reproducibility.** Same input always produces a bit-identical result.

**Effort:** ~300 LOC + ~15 tests. ~5 hours.

**Block gates:**

- [ ] B1b-G1 Compilation: `cargo build -p deep_causality_physics` clean (release + debug). New dep edge to `deep_causality_topology` documented in `Cargo.toml` + `BUILD.bazel`. Clippy clean.
- [ ] B1b-G2 Coverage: 100% on every new file under `src/fluids/physics_invariants/`.
- [ ] B1b-G3 Review: user signs off on the new dep edge at this gate.

---

## Block B1c — FluidSignature composition

**Goal.** Combine `TopologicalInvariants<R>` and `FluidPhysicsInvariants<R>` into a single `FluidSignature<R>` carrier that flows through the `PropagatingEffect` / `PropagatingProcess` pipeline.

**Crate affected:** `deep_causality_physics` only.

**Where it lives:**

- `deep_causality_physics/src/fluids/signature/mod.rs`:
  ```rust
  pub struct FluidSignature<R: RealField> {
      topology: TopologicalInvariants<R>,
      physics:  FluidPhysicsInvariants<R>,
  }
  ```
  With getters per field, Debug, Display, PartialEq following the project convention.

- Constructor that fans into both extractors:
  ```rust
  pub fn compose_fluid_signature<K, R>(
      velocity_decomposition: &HodgeDecomposition<R>,
      manifold: &Manifold<K, R>,
      kinematic_viscosity: R,
  ) -> Result<FluidSignature<R>, FluidExtractionError>
  where K: ChainComplex, K::Metric: HasHodgeStar<R, Complex = K>, R: RealField + FromPrimitive + Display;
  ```
  Internally:
  ```rust
  let topology = velocity_decomposition.topological_invariants(manifold)?;
  let physics  = fluid_physics_invariants(velocity_decomposition, manifold, kinematic_viscosity)?;
  Ok(FluidSignature { topology, physics })
  ```

**Property tests:**

- **Composition orthogonality.** `compose_fluid_signature(...)` produces a signature whose `.topology()` exactly equals the standalone `topological_invariants(...)` and whose `.physics()` exactly equals the standalone `fluid_physics_invariants(...)`.
- **Error propagation.** If either sub-extractor returns an error, `compose_fluid_signature` returns the same error wrapped in `FluidExtractionError`.

**Effort:** ~80 LOC + ~6 tests. ~1.5 hours.

**Block gates:**

- [ ] B1c-G1 Compilation clean.
- [ ] B1c-G2 Coverage: 100% on every new file.
- [ ] B1c-G3 Review.

---

## Block B2 — `RollingHistory<N, R>` state + `PropagatingProcess` lift

**Goal.** Typed rolling-window state carrier with O(1) amortised push, O(N) snapshot, and a documented FIFO cap. Plus the lift-to-Markovian boundary that turns the per-timestep `FluidSignature<R>` stream into a `PropagatingProcess`.

**Crate affected:** `deep_causality_physics` (state carrier) + integration shim against `deep_causality_core` (no changes to core).

**Where it lives:** `deep_causality_physics/src/fluids/rolling_history/`. The new dep edge to `deep_causality_topology` was established by B1b; no further dep changes.

**Storage choice:** wrap `std::collections::VecDeque<FluidSignature<R>>` with FIFO-cap semantics. This matches the project's existing idiomatic pattern of extending standard collections via traits (see `deep_causality/src/extensions/causable/mod.rs` where `MonadicCausableCollection` is implemented for `[T]`, `VecDeque<T>`, `HashMap<K, V>`, `BTreeMap<K, V>`).

Why not `deep_causality_data_structures::SlidingWindow<S, T>`: the existing `WindowStorage<T>` trait requires `T: PartialEq + Copy + Default`. `FluidSignature<R>` contains `FluidPhysicsInvariants<R>::vortex_centroids: Vec<[R; 3]>`, which owns heap storage and therefore cannot be `Copy`. A `Copy`-free sliding-window primitive in `deep_causality_data_structures` would be a useful general infrastructure addition, but it is out of scope for B2 — wrapping `VecDeque` is ~30 LOC and lands the carrier without crate-boundary infrastructure work.

Why not roll a new `ArrayDeque<T, N>`: nothing under that name exists in `deep_causality_data_structures` today, and `VecDeque` plus a cap check is mathematically equivalent for the bounded-window semantics this block needs.

```rust
use std::collections::VecDeque;

pub struct RollingHistory<const N: usize, R: RealField> {
    signatures: VecDeque<FluidSignature<R>>,   // capacity N maintained by push()
    integrated_helicity: R,
    integrated_enstrophy: R,
    cumulative_dissipation: R,
    timestamp: TimeStep,
}

impl<const N: usize, R: RealField + FromPrimitive> RollingHistory<N, R> {
    pub fn new() -> Self;

    /// Push a new signature. If the window is at capacity, drop the oldest first.
    pub fn push(&mut self, sig: FluidSignature<R>) {
        if self.signatures.len() == N {
            self.signatures.pop_front();
        }
        self.signatures.push_back(sig);
    }

    pub fn latest(&self) -> Option<&FluidSignature<R>>;
    pub fn window(&self) -> impl Iterator<Item = &FluidSignature<R>>;
    pub fn helicity_trajectory(&self) -> Vec<R>;
    pub fn enstrophy_trajectory(&self) -> Vec<R>;
}
```

`window()` returns an iterator (not a `&[...]`) because `VecDeque`'s storage is two slices, not one contiguous slice. Callers needing a contiguous view can collect into a `Vec`; downstream consumers that just iterate (every consumer in this pipeline) take the iterator unchanged.

The `PropagatingProcess` integration is one associated-function call against existing `deep_causality_core` API. Note the signature: `with_state` is an associated function on the target process type, not a fluent method on the source effect, and `initial_context` is wrapped in `Option<Context>`:

```rust
let process = PropagatingProcess::<FluidSignature<R>, RollingHistory<N, R>, FluidContext<R>>::with_state(
    effect,
    RollingHistory::<N, R>::new(),
    Some(FluidContext::<R>::new(geometry, reynolds, ...)),
);
```

No changes to `deep_causality_core`. `PropagatingEffect<T>` and `PropagatingProcess<T, S, C>` are confirmed type aliases of the same `CausalEffectPropagationProcess<...>` carrier; the iso tests in [`deep_causality_core/tests/iso/effect_process_consistency_tests.rs`](../../../../deep_causality_core/tests/iso/effect_process_consistency_tests.rs) pin that consistency.

**Property tests:**

- **FIFO invariant.** After `2N` pushes, only the last `N` signatures remain.
- **Integrated quantities non-negativity.** Enstrophy is always ≥ 0; `|helicity|` is bounded by the Cauchy-Schwarz product of `‖u‖` and `‖ω‖`.
- **Lift round-trip.** `PropagatingProcess::with_state(PropagatingEffect::pure(sig), history, Some(ctx))` followed by `.bind(...)` is type-equivalent to a `PropagatingProcess` chain initialised with the same data. The associated-function lift is the only public path from `PropagatingEffect<T>` (state = `()`, context = `()`) to `PropagatingProcess<T, S, C>` with non-trivial state/context.
- **Window slice stability.** `window()` returns a slice consistent with insertion order across N pushes.

**Effort:** ~200 LOC + ~10 tests. ~3 hours.

**Block gates:**

- [ ] B2-G1 Compilation: every affected crate clean.
- [ ] B2-G2 Coverage: 100% on every new file.
- [ ] B2-G3 Review.

---

## Block B3 — `FluidContext<R>` + SURD wiring

**Goal.** Stand up the runtime-invariant context and wire the augmented `(signature, history)` joint into `surd_states_cdl`. The output is a `SurdResult<f64>` — the same type the existing `SurdResultAnalyzer` already knows how to interpret. This is the block where the methodology becomes end-to-end testable on synthetic ground truth.

**Crates affected:** `deep_causality_physics` (`FluidContext`) + `deep_causality_discovery` (SURD wiring). No changes to `SurdResultAnalyzer` or to any graph-emission code; the existing report-style analysis is the consumer.

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
      sig: &FluidSignature<R>,
      history: &RollingHistory<N, R>,
      ctx: &FluidContext<R>,
  ) -> Result<SurdResult<f64>, CdlError>;
  ```
  Internally builds a `CausalTensor<Option<f64>>` joint distribution by casting `R → f64` once at the tensor-build step (the documented lossy boundary). Calls existing `surd_states_cdl(tensor, MaxOrder::Max)` unchanged.

The `SurdResult<f64>` returned by `fluid_surd_decompose` is consumed by the existing `SurdResultAnalyzer` ([`deep_causality_discovery/src/types/analysis/surd_result_analyzer.rs`](../../../../deep_causality_discovery/src/types/analysis/surd_result_analyzer.rs)) via `analyzer.analyze(&surd, &AnalyzeConfig::default())`, which produces a `ProcessAnalysis(Vec<String>)` of human-readable categorisations against the configured synergy / unique / redundancy / info-leak thresholds. No new analyzer is built in this note.

The joint-distribution feature selection — *which* fields of `FluidSignature<R>` and `RollingHistory<N, R>` are projected into the `CausalTensor<Option<f64>>` and with what discretisation buckets — is the load-bearing design decision of B3 and is documented in this block's preflight notes when it opens. The shape of the joint is not fixed by the surrounding pipeline; it is the feature engineering that B3 commits to and that B3's synthetic ground-truth test validates.

**Synthetic ground-truth test (the critical correctness check):**

Adapt the Martínez-Sánchez & Lozano-Durán (2026) §3 three-variable benchmark — explicitly prescribed synergistic / unique / redundant dependencies — into a 3D version on `LatticeComplex<3>`. Apply the full B1a + B1b + B1c + B2 + B3 pipeline. The returned `SurdResult<f64>` must report the prescribed synergy, unique, and redundancy components within the test tolerance, and the `SurdResultAnalyzer.analyze(...)` output must categorise the strong relationships above their respective thresholds. **This test is the single most important correctness gate in the entire roadmap.** If it fails, the methodology is broken regardless of how the deferred JHU reproduction lands.

**Property tests:**

- **Information-leak bound.** `0 ≤ info_leak ≤ H(target)` for every emitted `SurdResult`.
- **Sum constraint.** Redundant + unique + synergistic + leak = total mutual information to numerical tolerance (the SURD theorem).
- **Synergy non-negativity.** Synergistic component is always ≥ 0.
- **Analyzer threshold consistency.** Every relationship that `SurdResultAnalyzer.analyze(...)` flags as "strong" has its component value at or above the configured threshold; relationships below threshold are not flagged.
- **Synthetic ground-truth recovery.** Per above; full end-to-end test on the 3D-adapted §3 benchmark.

**Effort:** ~300 LOC + ~16 tests. ~7 hours. The drop from the prior estimate (~400 LOC) reflects removing the planned `FluidSurdResultAnalyzer` → `CausaloidGraph` bridge; the synthetic ground-truth test budget remains the dominant cost.

**Block gates:**

- [ ] B3-G1 Compilation: both crates clean.
- [ ] B3-G2 Coverage: 100% on every new file. The synthetic-ground-truth test counts as a regression test, not as coverage of the analyzer itself; the analyzer needs its own unit tests in addition.
- [ ] B3-G3 Review: user signs off, with explicit acknowledgement that the synthetic-ground-truth test passed. If that test fails, B3 does not close and the methodology is reopened for design review before B4 starts.

---

## Block B4 — Type-encoded fluid invariants

**Goal.** Ship the newtype wrappers and smart constructors that make conservation laws structural rather than runtime-checked.

**Type unification (2026-06-10).** `SolenoidalField<R>` is **the same invariant** as
the solver's projected-velocity carrier in `cfd-gap.md` G3 (working name
`ProjectedVelocityOneForm<R>`). One type ships, in this crate, under the
`SolenoidalField<R>` name, with exactly two construction paths: the solver's
`leray_project` (per-step, the type-state that makes "you cannot time-step an
unprojected field" a compile-time fact) and `from_hodge_projection` (per-snapshot,
this pipeline's entry). B4 must not define a duplicate; the solver spec must consume
this type. Likewise `Vorticity<R>::from_velocity` is a thin wrapper over the G1
`exterior_derivative` API, not a reimplementation.

**Crate affected:** `deep_causality_physics` only.

**Where it lives:** `deep_causality_physics/src/fluids/quantities/` with one file per newtype following the one-type-one-module rule:

```rust
pub struct SolenoidalField<R: RealField> { /* private */ }
pub struct Circulation<R: RealField> { /* private */ }
pub struct Vorticity<R: RealField> { /* private */ }
pub struct Helicity<R: RealField> { /* private */ }
```

Smart constructors (the only way to construct each type):

| Newtype | Invariant | Constructor signature |
|---|---|---|
| `SolenoidalField<R>` | `∇·u = 0` | `from_hodge_projection(field: CausalTensor<R>, hodge: &HodgeDecomposition<R>) -> Self` (returns the co-exact + harmonic part) |
| `Circulation<R>` | Kelvin's theorem | `from_loop_integral(velocity: &CausalTensor<R>, loop_cells: &[CellId]) -> Self` |
| `Vorticity<R>` | `dω = 0` (closed 2-form) | `from_velocity<K>(velocity_1form: &CausalTensor<R>, manifold: &Manifold<K, R>) -> Self where K: ChainComplex, K::Metric: HasHodgeStar<R>` |
| `Helicity<R>` | `H = ∫ u · ω dV` | `from_field_pair<K>(u: &SolenoidalField<R>, omega: &Vorticity<R>, domain: &Manifold<K, R>) -> Self` |

The compiler refuses constructions that bypass the invariants.

**Property tests:**

- **Construction-time divergence.** `SolenoidalField` from Hodge projection has `‖∇·u‖ < ε_R` re-checked via the discrete divergence operator.
- **Discrete Stokes' theorem.** `Circulation` around a closed loop equals the surface integral of `Vorticity` over any spanning surface to discretisation tolerance.
- **Helicity in ideal flow.** Forward Euler step under inviscid evolution preserves total helicity to second order in the timestep.
- **Type safety (compile-fail tests).** The wrong-type construction path does not exist.

**Effort:** ~300 LOC + ~15 tests. ~5 hours.

**Block gates:**

- [ ] B4-G1 Compilation: clean, including the compile-fail tests in `tests/`.
- [ ] B4-G2 Coverage: 100% on every new file.
- [ ] B4-G3 Review.

---

## Block B5 — Reusable pointwise Navier–Stokes kernels (STATUS CORRECTED: largely shipped)

**Status correction (2026-06-10).** The kernel surface this block planned to build
already ships in `deep_causality_physics/src/kernels/fluids/` (governing, kinematics,
turbulence, coherent-structure modules — `convective_acceleration_kernel`,
`viscous_diffusion_kernel`, `pressure_gradient_force_kernel`, Q-criterion, λ₂,
Kolmogorov/Taylor/integral scales — with the four regime evaluators on top in
`theories/fluid_dynamics/`). B5's remaining scope reduces to: an audit of the
signature list below against the shipped surface, plus any missing property tests
(notably the cross-precision Q-criterion identity and Galilean invariance, if not
already present). Budget accordingly — hours, not the original ~7h of construction.
B1b consumes the shipped kernels directly; the original "extract B1b private helpers
into B5" plan is void.

**Goal (original).** Ship the pointwise Navier–Stokes kernels that fit the existing `deep_causality_physics` kernel pattern: stateless, side-effect-free, pure algebra given pre-discretised inputs, generic over `R: RealField`.

**Crate affected:** `deep_causality_physics` only.

**Where it lives:** Extends `deep_causality_physics/src/fluids/`. Each kernel is a free `pub fn` following the existing `Fluids` kernel convention; no kernel takes `&self`.

**Kernel signatures (all generic over `R: RealField`, with `+ FromPrimitive` where literals are needed):**

```rust
// Full Navier-Stokes RHS terms
pub fn convective_acceleration_kernel<R: RealField>(u: &[R; 3], grad_u: &[[R; 3]; 3]) -> [R; 3];
pub fn viscous_diffusion_kernel<R: RealField>(nu: R, laplacian_u: &[R; 3]) -> [R; 3];
pub fn pressure_gradient_force_kernel<R: RealField>(rho: R, grad_p: &[R; 3]) -> [R; 3];
pub fn vorticity_transport_kernel<R: RealField>(omega: &[R; 3], u: &[R; 3], grad_omega: &[[R; 3]; 3], laplacian_omega: &[R; 3], nu: R) -> [R; 3];

// Coherent-structure detection
pub fn q_criterion_kernel<R: RealField + FromPrimitive>(velocity_gradient: &[[R; 3]; 3]) -> R;
pub fn lambda2_kernel<R: RealField + FromPrimitive>(velocity_gradient: &[[R; 3]; 3]) -> R;

// Turbulence scales 
pub fn kolmogorov_scale_kernel<R: RealField + FromPrimitive>(epsilon: R, nu: R) -> R;
pub fn taylor_microscale_kernel<R: RealField + FromPrimitive>(k_energy: R, epsilon: R, nu: R) -> R;
pub fn integral_length_scale_kernel<R: RealField + FromPrimitive>(k_energy: R, epsilon: R) -> R;
```

The `q_criterion`, `lambda2`, `taylor_microscale`, and `integral_length_scale` kernels are *extracted* from the private helpers that B1b will land inline. The B1b API does not change when this extraction happens; only the location of the formulas moves.

No kernel takes a manifold, a context, or any non-algebraic input. Discretisation and assembly happen outside the kernel.

**Property tests:**

- **Galilean invariance.** `convective_acceleration_kernel(u + c, grad_u) == convective_acceleration_kernel(u, grad_u)` for any constant velocity `c`. Tested across `R ∈ {f32, f64, Float106}`.
- **Dimensional consistency.** Every kernel is exercised with `units::*` newtypes wrapping `R`; the type system catches dimensional mismatches at compile time.
- **Limiting cases.** As `Re → ∞`, viscous diffusion vanishes (recovers Euler). As `Re → 0`, convective acceleration is negligible relative to viscous diffusion (recovers Stokes flow).
- **Precision robustness.** The Q-criterion algebraic identity `Q + 0.5 * ‖S‖² = 0.5 * ‖Ω‖²` holds for `f32`, `f64`, and `Float106` to each backend's expected tolerance.
- **Extraction equivalence.** The B5 `q_criterion_kernel`, `taylor_microscale_kernel`, and `integral_length_scale_kernel` produce bit-identical output to the B1b private helpers they replace, on a battery of prescribed inputs.

**Effort:** ~400 LOC + ~25 tests. ~7 hours.

**Block gates:**

- [ ] B5-G1 Compilation: clean across all three precision backends (`f32`, `f64`, `Float106`).
- [ ] B5-G2 Coverage: 100% on every new file. Precision-backend coverage is enforced by parameterised tests.
- [ ] B5-G3 Review. After B5-G3, the roadmap of this note is complete; validation becomes its own change set.

---

## 4. Deferred work (was F7–F9 in the original draft)

The following is **explicitly out of scope** for this note and must be opened as a separate follow-up note `3DCausalFluidDynamicsValidation.md` only after B5-G3 closes:

- **Validation on JHU Turbulence Database.** Reproduce Martínez-Sánchez & Lozano-Durán (2026) §4.1 at `Re_τ ≈ 950`. Two-branch comparison against the published autoencoder + k-means pipeline. ~25 hours plus data wrangling. Success criterion: leak below 25% on the same data class.
- **Avionics wake-vortex example.** Subsonic, incompressible wake only; compressible extension is a further downstream change set. ~30 hours. Lives under `examples/avionics_examples/wake_vortex_causal_inference/` as its own workspace crate depending on `deep_causality`, `deep_causality_physics`, `deep_causality_topology`, `deep_causality_discovery`, `deep_causality_ethos`.
- **Methods writeup.** ~15 hours.

Rationale for deferral: validation reproduces a published measurement and is a publishability concern, not a correctness concern. The methodology stands or falls on the **synthetic ground-truth test in B3** (the 3D-adapted §3 benchmark of the same paper), which is fully self-contained and lands inside this note's scope. If B3-G3 passes, the methodology is established; if it fails, no amount of JHU reproduction would recover it.

---

## 5. Cumulative effort and ordering

| Order | Block | Crate | Status | Effort | Gates |
|---|---|---|---|---|---|
| — | `add-cubical-regge-calculus-analytical` (R4 + R5 + R6) | topology | ✅ Shipped 2026-05-22 | — | Prerequisite |
| — | `add-hodge-decomposition` (H1–H3) | topology | ✅ Shipped 2026-05-22 | — | Prerequisite |
| 1 | B1a — TopologicalInvariants | topology | ✅ Shipped 2026-05-22 | — | Prerequisite |
| 2 | B1b — FluidPhysicsInvariants | physics | This note | ~5h | B1b-G1, B1b-G2, B1b-G3 |
| 3 | B1c — FluidSignature composition | physics | This note | ~1.5h | B1c-G1, B1c-G2, B1c-G3 |
| 4 | B2 — RollingHistory + lift | physics | This note | ~3h | B2-G1, B2-G2, B2-G3 |
| 5 | B3 — FluidContext + SURD wiring | physics + discovery | This note | ~7h | B3-G1, B3-G2, B3-G3 |
| 6 | B4 — Type-encoded invariants | physics | This note | ~5h | B4-G1, B4-G2, B4-G3 |
| 7 | B5 — Pointwise NS kernels | physics | This note | ~7h | B5-G1, B5-G2, B5-G3 |
| — | `3DCausalFluidDynamicsValidation.md` (was F7–F9) | mixed | **Deferred** | ~70h | Outside this note |

Total in-scope after prerequisites: **~28.5 hours focused work, ~1000 LOC, ~78 tests, 18 gates**. The B1 split into three small blocks (B1a + B1b + B1c) trades one block-G3 review for three, which is the right tradeoff because each block now has a sharp single-crate scope and reviewable diff size. B3's prior planned `FluidSurdResultAnalyzer` → `CausaloidGraph` bridge is no longer in scope; the existing `SurdResultAnalyzer.analyze(...)` consumes the `SurdResult<f64>` directly, and any downstream graph-emission work attaches in its own change set.

The gating discipline is strict: no block opens with an unclosed gate from the prior block; no gate closes without compilation, full coverage, and explicit user review. Per AGENTS.md, agents never commit; every G3 sign-off is the user committing the block.

---

## 6. Crate-responsibility boundaries

The architectural lesson from the original B1 conflation: every type and function in this pipeline belongs in exactly one crate, chosen by what its math describes, not by which downstream consumer reads it.

| Crate | Owns |
|---|---|
| `deep_causality_topology` | `Manifold<K, R>`, `HodgeDecomposition<R>`, `ChainComplex`, Hodge ⋆, differential operators (d, δ, Δ), `TopologicalInvariants<R>`, Betti numbers. |
| `deep_causality_physics` | Physical units (`Speed<R>`, `Mass<R>`, `Reynolds<R>`, `ForcingProfile<R>`), fluid newtypes (`SolenoidalField<R>`, `Vorticity<R>`, etc.), `FluidPhysicsInvariants<R>`, `FluidSignature<R>`, `RollingHistory<N, R>`, `FluidContext<R>`, pointwise NS kernels. |
| `deep_causality_discovery` | SURD wiring (`fluid_surd_decompose`), `CausalTensor<Option<f64>>` joint-distribution assembly at the SURD input boundary, consumption of the resulting `SurdResult<f64>` via the existing `SurdResultAnalyzer`. No new analyzer; no graph-emission code in scope for this note. |
| `deep_causality_core` | `CausalEffectPropagationProcess<V, S, C, E, L>` (the underlying carrier), `PropagatingEffect<T>` and `PropagatingProcess<T, S, C>` as type aliases, inherent `bind` (closure receives `(EffectValue<V>, S, Option<C>)`), associated `with_state` lift function, and the trait-based `deep_causality_haft::Monad`/`Functor`/`Pure` impls on `PropagatingEffectWitness` / `PropagatingProcessWitness`. **No changes from this work.** |

Anything that consumes `HodgeDecomposition<R>` to produce a velocity-field invariant (helicity, vortex centroids, turbulence scales) is physics and lives in the physics crate. Anything that consumes `Manifold<K, R>` to produce a discretisation invariant (Betti numbers, component L2 norms) is topology and lives in the topology crate. The combined `FluidSignature<R>` lives in physics because physics is the domain crate; topology is the supporting numerical infrastructure.

---

## 7. Honest caveats

- **The synthetic ground-truth test in B3 is the single load-bearing correctness gate.** If it fails, the methodology is broken and no later block recovers it. Allocate explicit design review time at B3-G3 if the result is borderline.
- **`RollingHistory<N>` is a finite Mori–Zwanzig truncation, not a proven Markovianisation.** For large enough `N` relative to the system's effective memory the truncation is benign; the ablation study that would defend `N` empirically lives in the deferred validation note, not here.
- **`FluidSignature` discards amplitude information by design.** It says "two vortex tubes exist with these L2 norms" but is coarse on per-vortex amplitude. For the causal questions this pipeline targets (existence and interaction of coherent structures), this is the right level. For other questions it is the wrong level.
- **The SURD precision boundary is a one-way cliff.** Converting `R → f64` at the `JointDistribution` build step in B3 is the single permitted lossy cast. Information-theoretic quantities saturate `f64` well within sample-noise floors; the loss is real but does not affect SURD's outputs at any tested precision backend. Documented at the call site.
- **Compressibility breaks `SolenoidalField<R>`.** B4's invariants are scoped to incompressible flow. Compressible extension (continuity equation as the relevant invariant) is a separate small change set.
- **~~`Manifold::hodge_decompose` does not project out the harmonic kernel of `Δ_k` for `k > 0`.~~ Resolved by `cfd-gap.md` G6** (prerequisite 4): harmonic-kernel deflation makes the β-step well-posed on periodic lattices, so this pipeline runs on torus data — which is exactly where the solver tap (§1.1) produces it. Note the asymmetry for context: the *solver* never needed the fix, because `leray_project` uses only the gauge-fixed grade-0 half of the decomposition; G6 exists for *this* pipeline's full-decomposition consumption.

---

## 8. Bottom line

Six gated blocks, three already shipped, five remaining. Each block is generic over `R: RealField` and lives in exactly one crate chosen by its mathematical responsibility. The full path from raw DNS snapshot to a SURD attribution consumable by the existing `SurdResultAnalyzer` lands in ~28.5 hours of focused work. The synthetic ground-truth test inside B3 is the correctness gate this work stands or falls on; the JHU reproduction is deferred to a separate validation note. Graph-emission work (turning the `SurdResult<f64>` into a `CausaloidGraph`) is out of scope for this note and lands in its own change set if and when a downstream consumer needs it.
