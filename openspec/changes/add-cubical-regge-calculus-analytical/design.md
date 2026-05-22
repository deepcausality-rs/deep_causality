## Context

`add-cubical-regge-calculus-core` shipped the geometric derivation layer for cubical Regge calculus on `LatticeComplex<D>` — cell volumes (R1), hinge enumeration with dihedral angles (R2), deficit angles and the Euclidean discrete Einstein–Hilbert action (R3). Three gaps remain before the cubical path is at parity with the simplicial one and before lattice quantum gravity simulations become tractable on this scaffold:

1. **The Hodge ⋆ on lattice complexes is not implemented**, which means `manifold/differential/{hodge,laplacian}.rs` still only works on `Manifold<SimplicialComplex<C>, F>`. Voxel-grid PDEs (heat, wave, Maxwell, FDTD) cannot use the existing differential machinery.
2. **The Euclidean / Lorentzian signature distinction is not tracked at the type level.** `CubicalReggeGeometry<D>` carries `timelike_axes: Option<[bool; D]>` as a runtime flag that R1–R3 ignores. Lattice gauge theory + GR, anisotropic spacetime work, and any Lorentzian-flavored research need a typed distinction.
3. **There is no way to perform Markov-chain Monte Carlo over edge-length configurations.** The Regge action is computable (per R3) but its gradient is not, and there is no Metropolis-update primitive.

The forward-looking design note [openspec/notes/CubicalReggeCalculus.md](../add-cubical-regge-calculus-core/CubicalReggeCalculus.md) §§3.R4–R6 lays out the implementation; this change set delivers all three phases as one reviewable unit because they share a non-trivial amount of infrastructure:

- R5's per-cell metric tensor (`metric_tensor_at`) is the same Gram-matrix machinery that R4's Hodge ⋆ relies on (dual/primal volume ratios fall out of the per-cell metric).
- R6's gradient consumes both R3's Euclidean Regge action and R5's Lorentzian variant.
- The `HasHodgeStar` capability trait (R4) is the natural home for the Hodge-dependent signature in R5 and for the `metric_tensor_at` method.

Stakeholders: anyone who needs lattice-native differential operators (voxel-grid PDE solvers, lattice gauge theory developers in `deep_causality_physics`, anyone implementing the upcoming `add-hodge-decomposition` change set, which depends critically on R4); anyone doing Lorentzian-flavored work on lattices (GRMHD, anisotropic-spacetime studies); anyone building Monte Carlo dynamics over edge lengths (lattice quantum gravity research).

The R1–R3 surface remains untouched. The R4–R6 additions are designed to be source-compatible with existing call sites via defaulted const generics and trait-bound widening.

## Goals / Non-Goals

**Goals:**

- Make `manifold/differential/{hodge,laplacian,codifferential}.rs` generic over `K: ChainComplex` where `K::Metric: HasHodgeStar`, with both `ReggeGeometry<T>` (simplicial) and `CubicalReggeGeometry<D, S>` (cubical) implementing the capability.
- Deliver a closed-form cubical Hodge ⋆ on `LatticeComplex<D>` for every grade `k`, with diagonal entries `volume(dual (D−k)-cell) / volume(primal k-cell)`, returned as a `CsrMatrix<f64>` from `deep_causality_sparse`.
- Track Euclidean vs. Lorentzian signature at the type level via `CubicalReggeGeometry<D, S>` with `S ∈ {Euclidean, Lorentzian}`, defaulting to `Euclidean` so R1–R3 call sites continue to compile unchanged.
- Compute per-cell metric tensors with the correct signature on demand.
- Detect light-cone violations at construction time for the Lorentzian variant.
- Compute the local edge-length gradient of the Regge action in O(2^D) per edge.
- Provide a one-shot Metropolis-Hastings update routine over edge lengths, parameterized by an inverse temperature β and an `Rng` from `deep_causality_rand`.
- Verify: discrete Hodge decomposition theorem holds on small lattices; per-cell metrics have the right signature; gradient agrees with finite differences to ~5 significant figures; equilibrium (unit-edge configuration) is a stationary point.

**Non-Goals:**

- The Hodge–Helmholtz decomposition (`hodge_decompose<K, F>`) and its application to vector-field denoising — separate change set `add-hodge-decomposition` (§7 of the design note). This change set provides the `HasHodgeStar` foundation; `add-hodge-decomposition` builds the decomposition on top.
- Causal-graph analysis of turbulent flows — separate change set `add-causal-flow-analysis` (§8 of the design note).
- The lattice-quantum-gravity *physics* program (sum over edge-length histories, computing observables, comparing with simplicial CDT). This change set delivers the *primitive operations* needed for that work; running and interpreting the physics is its own multi-month effort.
- Sparse cubical complexes (active-cubes-only representation).
- GPU paths.
- Non-cubical regular tilings (hex / kagome / triangular Regge analogs).
- Performance tuning beyond the algorithmically-natural O(2^D) per-edge gradient. Parallelization, SIMD, and GPU are a separate perf-track change set.
- Categorical coherence proofs (functoriality / monad laws / adjunction unit-counit equations) via `proptest`. Recommended as a follow-up validation pass, not part of this change set.

## Decisions

### Decision 1: One change set, three phases — R4 → R5 → R6 in dependency order

R4, R5, and R6 are shipped together because their infrastructure overlaps non-trivially (R5's `metric_tensor_at` is what R4's Hodge ⋆ uses for dual/primal volume ratios; R6's gradient consumes R3's action *and* R5's Lorentzian variant). Splitting into three separate change sets would require either landing the shared infrastructure twice (in R4 then again in R5) or shipping a R4-only "geometric scaffold for the metric tensor" change with no user-visible value.

Within this change set, tasks land in strict R4 → R5 → R6 order. Each phase is independently testable, and the property tests for an earlier phase MUST be passing before the next phase opens.

**Alternatives considered:**
- Three separate change sets (R4, R5, R6). Rejected: forces duplicating the per-cell metric machinery between R4 and R5, or shipping an intermediate change with no user value.
- Combine R4 with `add-hodge-decomposition`. Rejected: the Hodge decomposition is a separate scientific contribution with its own validation strategy (PyDEC parity, published-example reproduction); muddling the two would delay R4's downstream benefit (voxel-grid PDE one-liners) until the larger decomposition work is done.

### Decision 2: `HasHodgeStar` as a public capability trait, not an associated type on `ChainComplex`

The Hodge ⋆ is metric-dependent: it cannot live on `ChainComplex` itself because `CellComplex<C>` has no metric (`Metric = ()`). The natural shape is a separate capability trait:

```rust
pub trait HasHodgeStar {
    fn hodge_star_matrix(&self, k: usize) -> CsrMatrix<f64>;
}

impl<T: FloatType> HasHodgeStar for ReggeGeometry<T> { /* simplicial */ }
impl<const D: usize, S: SignatureMarker> HasHodgeStar for CubicalReggeGeometry<D, S> { /* cubical */ }
```

The generic differential operators then take a `where K::Metric: HasHodgeStar` bound. Complexes whose metric is `()` (e.g. `CellComplex<C>`) silently lose access to `manifold.hodge_star(k)` and `manifold.laplacian(k)` — correct behavior, since those operators are not defined without a metric.

**Why a trait, not an associated method on `CubicalReggeGeometry`:**
Both `ReggeGeometry<T>` (simplicial, already shipped) and `CubicalReggeGeometry<D, S>` (cubical, this change set) need to be usable through the same generic call. A trait is the only way to make `Manifold<K, F>::hodge_star(k)` resolve to either backend's implementation based on `K::Metric`.

**Alternatives considered:**
- Add `type HodgeStar` to `ChainComplex` directly. Rejected: forces `CellComplex<C>::HodgeStar = ()` with a `Never`-style impl, which is uglier than just bounding `K::Metric: HasHodgeStar` at the use site.
- Use dynamic dispatch (`Box<dyn HodgeBackend>`). Rejected: violates the AGENTS.md "static dispatch, no `dyn`" rule. Static dispatch via trait bound is the project convention.
- Make the trait method return `CausalTensor<f64>` instead of `CsrMatrix<f64>`. Rejected: the Hodge ⋆ on a lattice complex is diagonal but globally large (~num_cells(k) entries); `CsrMatrix` from `deep_causality_sparse` is the right representation, and the matrix integrates directly with the existing `coboundary_matrix` algebra used by the differential operators.

### Decision 3: Type-level signature marker `S` on `CubicalReggeGeometry<D, S>`, defaulted to `Euclidean`

Promote `CubicalReggeGeometry<D>` to `CubicalReggeGeometry<D, S = Euclidean>` with `S: SignatureMarker`. The marker types are unit structs (zero-sized):

```rust
pub struct Euclidean;
pub struct Lorentzian;

pub trait SignatureMarker: sealed::Sealed { /* signature methods */ }
impl SignatureMarker for Euclidean { /* (+, +, ..., +) */ }
impl SignatureMarker for Lorentzian { /* (-, +, ..., +) */ }
```

Source compatibility is preserved because `S` defaults to `Euclidean` — every R1–R3 call site (`CubicalReggeGeometry::<3>::unit_edge()`, etc.) continues to compile and produces the `Euclidean` variant.

The `Lorentzian` variant is constructed via a distinct builder (`with_timelike_axes_lorentzian` or similar) that requires `timelike_axes` to be `Some(...)` with at least one `true` entry and validates the construction at run time (returning `Err(LightConeViolation)` if edge lengths violate the cone).

**Some methods are only available on certain signatures:**
- `regge_action(&self, complex) -> f64` — available on `Euclidean` only. The Lorentzian variant returns `Complex<f64>` and is a distinct method.
- `regge_action_lorentzian(&self, complex) -> Complex<f64>` — available on `Lorentzian` only.
- `metric_tensor_at` — available on both, with signature determined by `S`.
- `hodge_star_matrix` — available on both. The Lorentzian Hodge ⋆ carries sign factors per the signature convention.

This is enforced by `impl` blocks bounded on `S`, not by run-time checks. Light-cone-violation detection happens at construction; once a `Lorentzian` value exists, the type system guarantees its consistency.

**Sealed trait:** `SignatureMarker` is sealed to prevent third parties from adding signature variants (e.g. degenerate / Kleinian) without coordinating with the differential-operator code. This can be relaxed later if a concrete need appears.

**Source-compatibility note on the R3 spec scenario "Lorentzian-marked metric is computed as Euclidean":** this scenario is REMOVED by this change set. Under the new typing, the question "what happens if `timelike_axes = Some(...)`?" doesn't arise — the type system tracks the choice. Migration: existing code that calls `regge_action` on a `CubicalReggeGeometry<D>` without specifying `S` continues to work (gets `S = Euclidean` by default). Code that sets `timelike_axes` via R1–R3's builder is unaffected because R1–R3's `with_timelike_axes` is repurposed as the entry to the `Lorentzian` constructor; the result type changes from `CubicalReggeGeometry<D>` to `CubicalReggeGeometry<D, Lorentzian>`. This is technically a signature shift, but R1–R3 ships with `with_timelike_axes` already documented as "the Lorentzian variant builder" — see Stage C of `add-cubical-complexes`.

**Alternatives considered:**
- Run-time `Signature` enum field, no type-level marker. Rejected: loses compile-time guarantees, requires every operator to branch on the variant, and breaks the "static dispatch" rule.
- Const-generic `const S: Signature` instead of marker types. Rejected: const generics over enums are unstable on the project's MSRV (last checked); marker types are stable Rust and equally expressive for our needs.
- Make `Euclidean` and `Lorentzian` distinct types entirely (no shared `CubicalReggeGeometry<D, S>`). Rejected: duplicates ~200 LOC of identical code; the type-parameterized version with bounded impls is the standard Rust pattern.

### Decision 4: Cubical Hodge ⋆ via dual/primal volume ratio

For each primal k-cell with a unique dual (D−k)-cell of complementary orientation, the diagonal Hodge ⋆ entry is:

```
star[k][cell_id] = volume(dual_cell(cell_id, D-k)) / volume(primal_cell(cell_id, k))
```

- Unit grid: both volumes are `1.0`, so ⋆ is the identity matrix for every grade.
- Per-axis grid in 2D with axes `[a, b]`: ⋆_0 (vertex → 2-cube) entry is `a · b`; ⋆_1 entry is `a/b` or `b/a` depending on the edge's axis; ⋆_2 (2-cube → vertex) entry is `1/(a · b)`. Closed-form, verifiable.
- Per-edge grid: closed form falls out of R1's per-edge cell volume and the dual-cell geometry. The per-edge derivation has a small piece of genuine novelty — the closed-form expression for the dual k-cell's edge lengths is folkloric in the lattice-physics literature but not in canonical published form. This is the same ~1-week derivation gap flagged in §7.8 of the design note as the "real research content" of the downstream Hodge-decomposition work.

**The derivation:** the dual cell of a primal k-cell in the cubical lattice is the (D−k)-cell formed by joining the centers of the top D-cubes incident to the primal cell. Its edge lengths are half-edge averages of the top cubes' edge lengths in the dual directions. On a unit grid this trivially gives unit dual volumes; on per-axis it gives axis-length products in the dual axes; on per-edge it gives the more general expression that lives in this change set's `hodge_star.rs`.

**Lorentzian sign factor:** on `S = Lorentzian`, the diagonal entry of ⋆_k carries an additional sign `(-1)^t` where `t` is the number of timelike axes in the primal cell's active dimensions, per the East-Coast metric signature convention. Documented in the doc comment.

**Alternatives considered:**
- Compute ⋆ from the local Cayley-Menger Gram matrix of each cell (the general DEC construction). Rejected: collapses to the diagonal closed form under axis alignment; no benefit, ~5× slower.
- Cache the ⋆ matrix once per `(complex, k)` behind a `Mutex`, similar to the coboundary cache. Deferred to a perf change set; not needed for correctness.

### Decision 5: Generic differential operators via trait-bound widening

The existing `manifold/differential/{hodge,laplacian,codifferential}.rs` files have impls of the form:

```rust
impl<C, F: FloatType> Manifold<SimplicialComplex<C>, F> {
    pub fn hodge_star(&self, k: usize) -> CsrMatrix<F> { /* uses ReggeGeometry */ }
}
```

After this change set, these become:

```rust
impl<K: ChainComplex, F: FloatType> Manifold<K, F>
where K::Metric: HasHodgeStar
{
    pub fn hodge_star(&self, k: usize) -> CsrMatrix<F> {
        self.metric.as_ref().expect("metric required").hodge_star_matrix(k)
    }
}
```

Call sites for existing simplicial users resolve identically because `ReggeGeometry<T>: HasHodgeStar` is added in this change set. Lattice users gain access for the first time.

**Note on call-site source compatibility:** any downstream code that has its own `impl Manifold<MyComplex, F>` blocks could in principle observe the trait-bound widening as a coherence change. We accept this. The trait `Manifold` is not user-implementable (no `unsafe impl` paths, no extension points), so the practical impact is zero in current downstream code.

**The `Laplacian` follows for free.** `Δ_k = δ_k d_k + d_{k-1} δ_{k-1}` is built from `d = coboundary_matrix` (already generic over `ChainComplex`) and `δ = ⋆⁻¹ d ⋆` (becomes generic once ⋆ does). The `Manifold::laplacian(k)` method ships generically as a side effect of R4.

### Decision 6: Regge gradient — local, O(2^D), closed-form

Edge `e` contributes to `∂S_R/∂(length_e)` only through hinges whose dihedral angles depend on `e`. By the structure of the cubical lattice, this is a constant number of hinges (the (D−2)-cells of every D-cube containing `e` — at most `2^(D-1)` cubes × `C(D-1, 2)` hinges per cube on the high side; in practice much smaller and `O(2^D)` total).

The closed form for `∂(dihedral)/∂(length)` on the per-axis / per-edge case is the derivative of an `arctan2` — a standard exercise (`d/dx arctan(y/x) = -y/(x²+y²)` etc.). This is the entire numerical content of R6 aside from bookkeeping.

The implementation produces `Vec<f64>` of length `num_edges()`, indexed by `edge_index` (the same private helper introduced in R1).

**Verification:** finite-difference check `(S(L + ε·δ_i) − S(L − ε·δ_i)) / (2ε)` ≈ analytical gradient to ~5 sig figs for ε ~ 1e-5. This is the strongest correctness test; ships as a property test.

**Alternatives considered:**
- Reverse-mode AD via a tape over the action computation. Rejected: pulls in either a tape library (external dep, against AGENTS.md) or a hand-rolled tape (~500 LOC, far more than the closed form). The closed form is small, transparent, and fast.
- Symbolic differentiation via `deep_causality_ast`. Rejected: that crate is for AST-driven causal reasoning, not numerical autodiff; using it here would be a layering violation.

### Decision 7: Metropolis update — single-edge, locality-exploiting

A Metropolis-Hastings update over `EdgeLengths { lengths: Vec<f64> }`:

1. Pick an edge `e` uniformly at random from `rng`.
2. Propose `length_e' = length_e + σ · normal(0, 1)` for a step size `σ` (passed in or held as a field).
3. Reject if `length_e'` violates the light cone (Lorentzian only) or goes non-positive.
4. Compute `ΔS_R` *locally* — only the hinges whose dihedral angles depend on `e` contribute. This is the same O(2^D) locality used in the gradient.
5. Accept with probability `min(1, exp(-β · ΔS_R))` (Euclidean) or `min(1, |exp(-β · ΔS_R · i)|)` with Wick rotation (Lorentzian — deferred subtlety).
6. Return `AcceptReject::{Accepted, Rejected}` (or richer struct with `delta_action`, `proposed_length`, etc.).

**RNG choice:** use `&mut R: Rng` from `deep_causality_rand`. The crate already exposes a Mersenne-Twister-compatible RNG and a `Normal` distribution.

**Step size σ:** not chosen automatically. The caller passes it. Adaptive step-size tuning (Robbins-Monro on the acceptance rate) is *not* part of this change set — it's a separate concern that downstream lattice-quantum-gravity simulations will handle.

**Alternatives considered:**
- Multi-edge / cluster updates. Rejected: out of scope. The user can call `metropolis_update` in a loop or implement their own cluster algorithm against the `regge_gradient` primitive.
- Hybrid Monte Carlo (HMC) with the gradient. Rejected: out of scope. The gradient primitive is provided; the caller can build HMC on top.

### Decision 8: `Complex<f64>` source — internal shim or external crate?

The Lorentzian Regge action returns `Complex<f64>` (real part = Euclidean action; imaginary part = Lorentzian phase under Wick rotation). The project already constrains us to "avoid the introduction of external crates unless necessary for testing" (AGENTS.md). Two options:

- **Internal shim:** a tiny `Complex<f64> { re: f64, im: f64 }` with the four arithmetic ops, exposed from `deep_causality_num`. ~50 LOC.
- **External `num-complex` crate:** mature, widely used. ~200 KB compile-time cost; adds one dep to the workspace.

**Recommendation:** internal shim under `deep_causality_num`, since complex numbers will be useful for several other future change sets (Hodge ⋆ in higher signatures, quantum-gravity observables, GR-spinor work in `deep_causality_physics`). Build it once, use it across the crate graph. This is a coordinated change to `deep_causality_num`, called out explicitly in the impact section.

**Open question:** if `deep_causality_num` already has a complex type, use it. If not, this change set lands the shim there. Verified in the open-questions section.

### Decision 9: Light-cone-violation detection at construction, not at action evaluation

The Lorentzian constructor `with_timelike_axes_lorentzian(lengths, timelike_axes)` validates that for every D-cube, the local metric tensor has exactly the right signature (one negative eigenvalue, D−1 positive). Violations return `Err(LightConeViolation { cell_id, eigenvalues })` at construction. Once a `CubicalReggeGeometry<D, Lorentzian>` value exists, the type system guarantees it's valid; no run-time check is needed at every action evaluation.

**Rationale:** light-cone violations are catastrophic (negative kinetic energy, ghost modes, action sign flips). Catching them at construction prevents propagating broken state through downstream code. The cost is one Gram-eigenvalue check per top D-cube at construction — `O(num_top_cubes · D^3)` once, vs. `O(num_top_cubes · D^3)` per Monte Carlo step otherwise.

**Alternatives considered:**
- Lazy check on first use. Rejected: hides the error far from where the bug was introduced.
- Check only on Lorentzian-flavored operations (`regge_action_lorentzian`, `metric_tensor_at`). Rejected: same propagation hazard.

## Risks / Trade-offs

- **[Risk] Per-edge cubical Hodge ⋆ derivation has small but real novelty.** The closed-form expression for ⋆_k on a general per-edge cubical metric is folkloric in the lattice-physics literature but, to my knowledge, not in canonical published form. A ~1-week derivation effort with cross-checks against the simplicial case (e.g. a unit square seen both as two triangles and as one 2-cube — both decompositions should give the same Hodge decomposition for a test field).
  → **Mitigation:** allocate explicit derivation time in the task list, and gate R4's per-edge implementation on a successful cross-check against the simplicial Hodge ⋆ on the unit square / cube. If the derivation slips, ship R4 with unit-edge + per-axis support only (already covers ~90% of use cases) and mark per-edge as an explicit follow-up.

- **[Risk] `Manifold<K, F>` users who hand-impl methods on the concrete type `Manifold<SimplicialComplex<C>, F>` could see coherence conflicts.**
  → **Mitigation:** audit the workspace before landing for downstream impls. Currently none exist outside `deep_causality_topology` itself; the trait-bound widening is safe in practice. If a future external crate hits this, the breakage is detectable at compile time and the fix is to use the generic `Manifold<K, F>` impl.

- **[Risk] `CubicalReggeGeometry<D>` → `CubicalReggeGeometry<D, S = Euclidean>` is a signature change.** R1–R3 documentation uses the bare `CubicalReggeGeometry<D>` form; turbofish call sites like `CubicalReggeGeometry::<3>::unit_edge()` continue to work because of the default, but explicit signatures `let g: CubicalReggeGeometry<3> = ...` break if the project enforces `#![deny(elided_lifetimes_in_paths)]` or similar lints on this kind of elision.
  → **Mitigation:** double-check workspace lint config; the `S = Euclidean` default is the standard Rust idiom for adding a parameter without breaking call sites. If a project-wide lint forbids elided generic arguments, add a `#[allow(...)]` at the type definition site or change R1–R3's call sites to be explicit. Audit happens during the implementation pass.

- **[Risk] Light-cone-violation detection on the per-edge Lorentzian variant requires a per-cell Gram-eigenvalue solve.** D=4 means a 4×4 symmetric eigenvalue problem per top cube. For a `[100; 4]` lattice that's 10⁸ eigenvalue solves at construction — slow.
  → **Mitigation:** for D ≤ 4 the eigenvalue solve has a closed-form sign-of-determinant + sign-of-trace shortcut (Sylvester's criterion); a closed-form signature check is O(D) per cube, not O(D^3). Implement the shortcut; full eigenvalue solve is only needed if D ≥ 5, and we don't currently target D ≥ 5. Documented in the task list.

- **[Risk] Metropolis updates need a careful proposal-symmetry argument.** Naive Gaussian proposals on edge lengths are not detailed-balance-preserving if the boundary `length > 0` is enforced by truncation. Standard fix: reject `length' ≤ 0` (already in the algorithm above), which preserves detailed balance.
  → **Mitigation:** documented in the doc comment; property test verifies that detailed balance holds on a 2-edge toy example by running ~10⁶ steps and checking that the equilibrium distribution matches the analytical `exp(-β S_R)`.

- **[Trade-off] One change set instead of three.** The benefit is shared infrastructure between R4, R5, R6. The cost is a larger review surface (~1000 LOC, ~37 tests, ~15 hours of focused work) than ideal. Mitigated by strict R4 → R5 → R6 staging within the change set, with property tests gating each phase before the next opens (per the `add-cubical-complexes` stage-gate protocol).

- **[Trade-off] Sealed `SignatureMarker` trait.** Prevents third parties from adding degenerate / Kleinian / split-signature variants. Acceptable because: (a) those signatures need their own Hodge ⋆ and differential-operator treatment, which is a separate scientific contribution; (b) the seal can be removed in a future change set without breaking existing impls.

- **[Trade-off] No HMC, no adaptive step-size tuning.** Pushes responsibility to downstream lattice-quantum-gravity simulations. Acceptable because: those simulations have their own conventions (NUTS, Riemannian HMC, replica exchange, etc.) and the `regge_gradient` primitive supports any of them.

## Migration Plan

This change is structurally non-breaking but semantically extends R1–R3's surface.

- **Source compatibility (R1–R3 call sites):**
  - `CubicalReggeGeometry::<D>::unit_edge()` etc. continue to compile and produce `CubicalReggeGeometry<D, Euclidean>`.
  - `regge_action(&self, complex) -> f64` is still available, now only on the `Euclidean` impl block. R1–R3 code that called `regge_action` on a Euclidean default continues to work.
  - `with_timelike_axes` (the R1–R3 Lorentzian-flag builder, documented in Stage C of `add-cubical-complexes`) is repurposed as the entry point to the `Lorentzian` constructor. Existing call sites are extremely rare (the feature was scaffold-only in R1–R3) and any that exist would need to be reviewed manually. Audit the workspace before landing.
- **API additions (the new surface):**
  - `HasHodgeStar` trait added to the crate root.
  - `Euclidean`, `Lorentzian` marker types and the sealed `SignatureMarker` trait added.
  - `LightConeViolation` error variant added.
  - `AcceptReject` enum added.
  - New methods on `CubicalReggeGeometry<D, S>`: `hodge_star_matrix`, `metric_tensor_at`, `regge_action_lorentzian` (Lorentzian only), `regge_gradient`, `metropolis_update`.
  - `Manifold::hodge_star`, `Manifold::laplacian`, `Manifold::codifferential` widen their trait bounds — source-compatible for all known downstream code.
  - `deep_causality_num` gains a `Complex<f64>` type (if not already present) — coordinated change to that crate.
- **Rollback:** revert the change set. No persisted state. The R1–R3 surface is preserved exactly. Downstream code that started using the new Hodge / Lorentzian / Metropolis surfaces would need to be reverted as well, but no R1–R3 user is affected.
- **Sequencing:** depends on `add-cubical-regge-calculus-core` having shipped. Unblocks `add-hodge-decomposition` (the uniform discrete Hodge–Helmholtz decomposition, §7 of the design note).

## Open Questions

1. **Does `deep_causality_num` already expose a `Complex<f64>`?** If yes, use it. If no, this change set lands a minimal `Complex<f64>` shim there. Verify before opening the implementation pass.
2. **Should the `S` parameter be a marker type (`Euclidean`, `Lorentzian`) or a const generic (`const S: SignatureKind`)?** Recommendation in design.md: marker types, because const generics over enums are unstable on the project's MSRV (verify) and marker types are equally expressive.
3. **Should `Manifold::laplacian(k)` return `CsrMatrix<F>` or `CausalTensor<F>`?** R4 ships it as `CsrMatrix<F>` because that's what the existing simplicial impl uses. If a future change set wants a tensor view, it can wrap.
4. **Adaptive step-size tuning for `metropolis_update`?** Recommendation: out of scope here; callers tune their own.
5. **Should `regge_gradient` return `Vec<f64>` or `CausalTensor<f64>`?** Recommendation: `Vec<f64>` indexed by `edge_index`. Conversion to a tensor is one line at the call site, and the gradient is intrinsically flat-indexed (not a multi-dimensional array).
6. **Cache invalidation for `hodge_star_matrix`?** If we cache the matrix per `(complex, grade)`, mutations to `CubicalReggeGeometry<D, S>` (which happen during `metropolis_update`) must invalidate the cache. Recommendation: don't cache in this change set. The matrix is cheap (sparse, diagonal) to construct. Revisit in a perf change.
7. **`HasHodgeStar` method shape.** Returning `CsrMatrix<f64>` forces `f64` precision. Should the trait be generic over a `FloatType` parameter? Recommendation: yes — `trait HasHodgeStar<F: FloatType> { fn hodge_star_matrix(&self, k: usize) -> CsrMatrix<F> }`. Aligns with the existing `Manifold<K, F>` parameterization. Confirm during implementation.
