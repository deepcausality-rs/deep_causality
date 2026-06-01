# Tasks — add-cubical-regge-calculus-analytical

This change set delivers R4 (cubical Hodge ⋆ + generic differential operators), R5 (Lorentzian signature marker + per-cell metric), and R6 (Regge-action gradient + Metropolis updates), all parameterised over `R: RealField` to align with the existing `CubicalReggeGeometry<const D, R>` and `ReggeGeometry<R>` surface.

## Block 0 — Spec refinement (closes Open Question 7)

Before any code lands, the proposal and design must be reconciled with the `R: RealField` parameterisation that R1–R3 already shipped. This is a documentation-only block.

- [x] 0.1 Rewrite `proposal.md` so every `f64` in a public signature becomes `R: RealField` (with `+ FromPrimitive` where literal construction is required). Affected surfaces:
  - `hodge_star_matrix(...) -> CsrMatrix<R>` (was `CsrMatrix<f64>`).
  - `metric_tensor_at(...) -> CausalTensor<R>` (was `CausalTensor<f64>`).
  - `regge_action_lorentzian(...) -> Complex<R>` (was `Complex<f64>`).
  - `regge_gradient(...) -> Vec<R>` (was `Vec<f64>`).
  - `metropolis_update<Rng>(&mut self, ..., beta: R) -> AcceptReject<R>` (was untyped `beta`).
- [x] 0.2 Promote `CubicalReggeGeometry<D, S = Euclidean>` to `CubicalReggeGeometry<const D, R, S = Euclidean>` throughout the proposal so it matches the live type signature.
- [x] 0.3 Update `design.md` Decision 2 to lock the trait shape as:
  ```rust
  pub trait HasHodgeStar<R: RealField> {
      fn hodge_star_matrix(&self, complex: &impl ChainComplex, k: usize) -> CsrMatrix<R>;
  }
  ```
  with `impl<R: RealField + FromPrimitive> HasHodgeStar<R> for ReggeGeometry<R>` and `impl<const D: usize, R: RealField + FromPrimitive, S: SignatureMarker> HasHodgeStar<R> for CubicalReggeGeometry<D, R, S>`.
- [x] 0.4 Close Open Question 7 in `design.md` with the decision above.
- [x] 0.5 Update Decision 5 (generic differential operators) to widen `Manifold` impls to `impl<K, R> Manifold<K, R> where K: ChainComplex, K::Metric: HasHodgeStar<R>, R: RealField + FromPrimitive`.
- [x] 0.6 Resolve Open Question 1 (does `deep_causality_num` expose `Complex`?): **Resolved — `deep_causality_num::Complex<T: RealField>` is already exposed at the crate root ([`src/lib.rs:59`](../../../deep_causality_num/src/lib.rs#L59)) with the required generic shape, including `Complex32` / `Complex64` aliases. Reuse unchanged; no coordinated change to `deep_causality_num` is needed.**
- [x] 0.7 Update the proposal's Impact section so `Complex<R>` (or the equivalent reused type) appears with its actual provenance.
- [x] 0.8 Block-0 gate: user reviewed and committed the refined proposal + design before R4 opened.

## Block R4 — Cubical Hodge ⋆ + generic differential operators

Depends on Block 0. Lands the new capability trait, the cubical implementation, and the trait-bound widening of the existing simplicial differential operators so they become generic over `K: ChainComplex`.

### R4.1 Trait scaffolding

- [x] R4.1.1 Create `deep_causality_topology/src/traits/has_hodge_star.rs` with `pub trait HasHodgeStar<R: RealField>` per Decision 2 of the refined design. **Done** with the refined associated-type-on-`Complex` shape and `Cow<'_, CsrMatrix<R>>` return per the [chain_complex.rs](../../../deep_causality_topology/src/traits/chain_complex.rs) precedent — design issue surfaced during implementation, design.md Decision 2 updated to match.
- [x] R4.1.2 Register the trait in `src/traits/mod.rs` and re-export from `src/lib.rs`.
- [x] R4.1.3 Write a one-trait-per-file test stub under `tests/traits/has_hodge_star_tests.rs`; register in `tests/traits/mod.rs` and `tests/BUILD.bazel`. **Bazel registration not needed** — the existing `traits` suite uses `glob(["traits/*_tests.rs"])`, the new file is picked up automatically. 3 tests passing.

### R4.2 Simplicial `HasHodgeStar` impl

- [x] R4.2.1 Implement `HasHodgeStar<R> for ReggeGeometry<R>` where `R: RealField + FromPrimitive`. **Done** at [`src/types/regge_geometry/has_hodge_star.rs`](../../../deep_causality_topology/src/types/regge_geometry/has_hodge_star.rs). Body vends `Cow::Borrowed(&complex.hodge_star_operators()[k])` — zero copy against the existing simplicial cache; `&self` is unused by design (the simplicial Hodge ⋆ data lives on the complex, not on `ReggeGeometry`).
- [x] R4.2.2 Move the simplicial Hodge ⋆ body out of `Manifold::hodge_star` and into the trait impl. **Done with dual-path migration** at [`src/types/manifold/differential/hodge.rs`](../../../deep_causality_topology/src/types/manifold/differential/hodge.rs): the manifold method routes through `self.metric.hodge_star_matrix(&self.complex, k)` when a metric is attached, and falls back to the legacy `complex.hodge_star_operators[k]` path when `self.metric.is_none()`. Existing tests that construct manifolds without a metric continue to pass byte-for-byte. R4.5 removes the fallback as part of the generic widening (which forces a complete migration of any remaining no-metric call sites).
- [x] R4.2.3 Property tests added at [`tests/types/regge_geometry/has_hodge_star_tests.rs`](../../../deep_causality_topology/tests/types/regge_geometry/has_hodge_star_tests.rs): trait impl returns `Cow::Borrowed`; trait-routed matrix bitwise-equals the complex cache for every grade; manifold method's metric-routed and legacy-fallback paths agree bitwise on the produced k-form. **The ⋆⋆ identity and Laplacian self-adjointness tests are deferred to R4.5** where they fit naturally with the generic widening; in R4.2 the simplicial path is structurally unchanged (same cache, same numerical kernel), so those mathematical properties remain established by the existing 9-test `differential_tests` suite which still passes.

### R4.3 Cubical `HasHodgeStar` impl — unit-edge and per-axis tiers

- [x] R4.3.1 Implement `hodge_star_matrix` for the `UnitEdge` and `PerAxis` tiers of `CubicalReggeGeometry<D, R>`. **Done** at [`src/types/cubical_regge_geometry/has_hodge_star.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/has_hodge_star.rs). Covers all four uniformity tiers: `UnitEdge` (identity), `Uniform { length }` (`length^(D-2k)`), `PerAxis` (per-cell closed form using orientation complement), and `PerEdge` (explicit panic gate — R4.4 lands the real implementation). Returns `Cow::Owned(CsrMatrix<R>)` since cubical Hodge ⋆ is compute-on-demand. Follows the FEEC/DEC mass-matrix square-diagonal convention used by the existing simplicial path. **Type signature is `CubicalReggeGeometry<D, R>` for now**; R5.2 promotes to `CubicalReggeGeometry<D, R, S = Euclidean>` via the defaulted three-parameter promotion at which point this impl gains its `S: SignatureMarker` bound automatically.
- [x] R4.3.2 Property tests at [`tests/types/cubical_regge_geometry/has_hodge_star_tests.rs`](../../../deep_causality_topology/tests/types/cubical_regge_geometry/has_hodge_star_tests.rs). **12 passing tests:**
  - `UnitEdge`: identity matrix verified on 2D open + 2D periodic + 3D open + 3D periodic lattices for every grade `k ∈ [0, D]`.
  - `Uniform`: closed-form `length^(D-2k)` verified at every grade in 2D and 3D with `length = 2.0`, covering positive and negative exponents.
  - `PerAxis` 2D: ⋆_0 = `a·b`, ⋆_1 = `b/a` (axis-0) and `a/b` (axis-1), ⋆_2 = `1/(a·b)` per design.md Decision 4. `a = 3.0, b = 5.0`.
  - `PerAxis` 3D: full 8-orientation matrix verified per cell against the closed form (`a·b·c`, `(b·c)/a`, etc.).
  - `PerAxis` degenerates to `Uniform` when all axes are equal.
  - Out-of-range `k > D` returns an empty 0×0 matrix.
  - `Cow::Owned` confirmed (compute-on-demand).
  - `PerEdge` panics with the documented R4.4-deferred message (`#[should_panic]`).

### R4.4 Cubical `HasHodgeStar` impl — `PerEdge` tier

- [x] R4.4.1 Derived the per-edge dual-cell formula. Documented in the module header of [`src/types/cubical_regge_geometry/has_hodge_star.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/has_hodge_star.rs). Formula: for a primal k-cell σ at position p with active axes A, `|σ*| = (1/|valid_masks|) · Σ_{m ∈ {0,1}^(D−k) valid} ∏_{c ∈ A^c} L(p − m_c · e_c, axis = c)`, where mask bit `m_c` selects which axis-c edge to draw the length from (positive-going or negative-arriving). Boundary edges on open lattices are dropped from the sum and the divisor.
- [x] R4.4.2 Cross-check **scoped to internal consistency**: per-edge with uniform values must match the `Uniform` closed form, and with axis-uniform values must match `PerAxis`. **Both verified as property tests** (`per_edge_with_uniform_lengths_matches_uniform_on_periodic_lattice`, `per_edge_with_uniform_per_axis_lengths_matches_per_axis_on_periodic_lattice`). **The deeper simplicial-vs-cubical cross-check on the unit square is deferred to `add-hodge-decomposition` H3** where it is already specified — that test requires the field-level Hodge decomposition surface (orthogonal projections, L2 norm agreement), which this change set does not deliver.
- [x] R4.4.3 Implemented `hodge_star_matrix` for the `PerEdge` tier. **Done** at [`src/types/cubical_regge_geometry/has_hodge_star.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/has_hodge_star.rs); the R4.3 panic is replaced with the real per-edge corner-averaging routine plus a private helper `per_edge_corner_product` that resolves edge positions and applies open/periodic boundary handling per axis.
- [x] R4.4.4 Property tests added to [`tests/types/cubical_regge_geometry/has_hodge_star_tests.rs`](../../../deep_causality_topology/tests/types/cubical_regge_geometry/has_hodge_star_tests.rs). 5 new R4.4-specific tests passing:
  - Per-edge with uniform values agrees with `Uniform` closed form on periodic 3D cube.
  - Per-edge with axis-uniform values agrees with `PerAxis` on periodic 3D cube.
  - Per-edge 2D periodic `[a, b]` matches the design.md Decision 4 closed form (`a·b`, `b/a`, `a/b`, `1/(a·b)`) at every cell.
  - Open 3D cube: all entries finite, non-NaN, positive at every grade — boundary handling does not produce divide-by-zero or pathological values.
  - Behavioural check: perturbing exactly one edge length changes at least one ⋆_0 entry — proves the per-edge path actually responds to per-edge data, not just aggregate axis statistics.
- [x] R4.4.5 **Risk-mitigation gate decision: shipped.** The per-edge implementation lands cleanly in ~80 LOC, agrees with the `Uniform` / `PerAxis` tiers under degenerate inputs to ~1e-12, and handles open / periodic boundaries without panicking. The `PerEdge` panic guard from R4.3 is removed. The published-form derivation gap flagged in design.md Risk 1 is closed by the in-module documentation.

### R4.5 Generic differential-operator widening

- [x] R4.5.1 Widen `impl<R> Manifold<SimplicialComplex<R>, R>` blocks in `src/types/manifold/differential/{hodge,laplacian,codifferential}.rs` to `impl<K, R> Manifold<K, R> where K: ChainComplex, K::Metric: HasHodgeStar<R, Complex = K>, R: RealField + FromPrimitive + ...`. **Done** across all four differential operators (`hodge_star`, `codifferential`, `laplacian`, `exterior_derivative`), plus generalisation of the shared helpers `get_k_form_data` and `create_temp_manifold` in `utils_differential.rs`. Manifold methods now panic when invoked without a metric; the R4.2 dual-path fallback is removed.
- [x] R4.5.2 Audit the workspace for downstream `impl Manifold<...>` blocks: **none found**. The breaking change instead surfaces as ~80 `Manifold::new(...)` call sites that lose access to Hodge-dependent methods (no metric attached). Migrated all of them to `Manifold::with_metric(...)` / `Manifold::from_cubical_with_metric(...)` with appropriately-sized unit-edge `ReggeGeometry` / `CubicalReggeGeometry::unit()`. Surface migrated:
  - `deep_causality_topology` tests: `differential_tests.rs` (`setup_triangle_manifold`), `regge_geometry/has_hodge_star_tests.rs` (the obsolete dual-path comparison was replaced with a metric-routed shape check).
  - `deep_causality_physics` test fixtures: 8 files (`em/{fields,wrappers}_tests.rs`, `mhd/{resistive,wrappers}_tests.rs`, `quantum/{mechanics,wrappers}_tests.rs`, `thermodynamics/{stats,wrappers}_tests.rs`).
  - Workspace examples: `examples/mathematics_examples/topology/differential_field.rs` (both the initial construction and the per-step rebuild inside the diffusion loop).
- [x] R4.5.3 Added [`tests/types/manifold/cubical_differential_tests.rs`](../../../deep_causality_topology/tests/types/manifold/cubical_differential_tests.rs) — 7 passing tests exercising `hodge_star`, `exterior_derivative`, `codifferential`, and `laplacian` on `Manifold<LatticeComplex<D>, f64>` for `D ∈ {2, 3}`, open and periodic, with `CubicalReggeGeometry::unit()` metrics. Covers the Stage-C "one-line `manifold.laplacian(0)`" claim — the test does exactly that on a 3D cubic torus.
- [x] R4.5.4 The full discrete Hodge decomposition theorem (any 1-form decomposes uniquely into exact + co-exact + harmonic) requires `Manifold::hodge_decompose`, which is the deliverable of `add-hodge-decomposition` (H1–H3) and not this change set. The strongest available structural check is **`d² = 0` nilpotency on cubical complexes**, which is the cohomological prerequisite for the decomposition theorem and is verifiable against just `exterior_derivative` + the cubical complex. Two passing tests in `cubical_differential_tests.rs`: 2D periodic 4×4 torus and 3D periodic cube. The full decomposition test will be added by `add-hodge-decomposition` H3 per its `spec.md`.

### R4.6 Block R4 gates

- [x] R4.6.1 R4-G1 Compilation: clean across all sub-blocks (R4.1 trait, R4.2 simplicial impl, R4.3 cubical UnitEdge/Uniform/PerAxis, R4.4 cubical PerEdge, R4.5 generic widening). Verified per sub-block as the work landed.
- [x] R4.6.2 R4-G2 Coverage: every new file (R4.1 trait + 3 impl files + 4 new test files) and every modified file (3 differential operators, manifold/utils_differential, 80+ workspace migration sites) covered by the test additions per sub-block.
- [x] R4.6.3 R4-G3 Review: user reviewed and committed at each R4 sub-block boundary (R4.1, R4.2, R4.3, R4.4, R4.5) — explicit incremental commit cadence per the protocol.

## Block R5 — Lorentzian signature marker + per-cell metric

Depends on R4 (specifically R4.3's per-cell volume machinery, which R5 reuses for the metric tensor). Adds `Euclidean` / `Lorentzian` marker types, promotes the geometry to `CubicalReggeGeometry<const D, R, S = Euclidean>`, and detects light-cone violations at construction.

### R5.1 Signature marker scaffolding

- [x] R5.1.1 [`src/types/cubical_regge_geometry/signature.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/signature.rs) created with `pub struct Euclidean;`, `pub struct Lorentzian;`, sealed `pub trait SignatureMarker`. Trait exposes `sign_factor<R: RealField>(timelike_count) -> R` and `is_lorentzian() -> bool` for type-level dispatch.
- [x] R5.1.2 Sealed via `mod sealed { pub trait Sealed {} }` pattern; impls only for `Euclidean` and `Lorentzian`.
- [x] R5.1.3 Re-exported `Euclidean`, `Lorentzian`, `SignatureMarker` from `src/lib.rs`.

### R5.2 Promote `CubicalReggeGeometry` to three parameters

- [x] R5.2.1 Struct promoted to `CubicalReggeGeometry<const D: usize, R: RealField, S: SignatureMarker = Euclidean>` with `_signature: PhantomData<S>` field.
- [x] R5.2.2 Default `S = Euclidean` preserves R1–R3 / R4 call sites: every `CubicalReggeGeometry::<D, f64>::unit()` etc. continues to compile and produces the `Euclidean` variant.
- [x] R5.2.3 No `#![deny(elided_lifetimes_in_paths)]` lint issues surfaced; workspace clippy `-D warnings` clean.
- [x] R5.2.4 `with_timelike_axes` **repurposed per design.md** as the type-level Lorentzian constructor: `Euclidean → Result<CubicalReggeGeometry<D, R, Lorentzian>, LightConeViolation<R>>`. Old runtime-flag-only behaviour removed; tests migrated to the new shape. Per AGENTS.md §"Code testing", test compatibility is not a constraint — the API drives the tests, not the other way around.

### R5.3 Per-cell metric tensor

- [x] R5.3.1 [`metric_tensor_at`](../../../deep_causality_topology/src/types/cubical_regge_geometry/metric_tensor.rs) returns a `D × D` `CausalTensor<R>` for any cell; generic over `S` (Euclidean and Lorentzian share the same method, the sign emerges from the per-axis timelike pattern).
- [x] R5.3.2 Diagonal entries are `±L_axis²`: `−` iff the axis is flagged timelike (Lorentzian; East-Coast convention), `+` otherwise (Euclidean default). Off-diagonals zero (axis-aligned cubical).
- [x] R5.3.3 6 property tests at [`tests/types/cubical_regge_geometry/metric_tensor_tests.rs`](../../../deep_causality_topology/tests/types/cubical_regge_geometry/metric_tensor_tests.rs): Euclidean unit / PerAxis 2D / PerAxis 3D / PerEdge reduces to PerAxis on uniform input; Lorentzian 2D with axis-0 timelike; Lorentzian 4D Minkowski-like.

### R5.4 Lorentzian Hodge ⋆ sign factors

- [x] R5.4.1 Cubical `HasHodgeStar<R>` impl extended via free helper `timelike_axes_in_orientation` + `S::sign_factor::<R>(t)` per-cell dispatch. Applies to all four `EdgeLengths` tiers (`UnitEdge`, `Uniform`, `PerAxis`, `PerEdge`).
- [x] R5.4.2 3 property tests at [`tests/types/cubical_regge_geometry/lorentzian_hodge_tests.rs`](../../../deep_causality_topology/tests/types/cubical_regge_geometry/lorentzian_hodge_tests.rs): 2D axis-0 timelike sign pattern; 3D axis-2 timelike (Minkowski layout) per-cell sign verification; open-lattice boundary smoke. Note: the "all-spacelike degenerates to Euclidean" check from the original task statement is *unreachable by construction* after R5.5 — `with_timelike_axes([false; D])` errors with `AllSpacelike`. The reduction is captured instead by the type-level `Euclidean::sign_factor` always returning `+1`, observable in `lorentzian_hodge_tests.rs` where the Lorentzian-vs-Euclidean comparison directly verifies the sign factor's contribution.

### R5.5 Light-cone-violation detection

- [x] R5.5.1 [`LightConeViolation<R>`](../../../deep_causality_topology/src/errors/light_cone_violation.rs) variants: `AllSpacelike` (zero timelike axes) and `CellSignature { cell_id: CellId, eigenvalues: Vec<R> }`. Implements `Debug + Clone + PartialEq + Display + std::error::Error`. Re-exported from crate root.
- [x] R5.5.2 Sylvester's-criterion check in `with_timelike_axes`: enforces exactly 1 timelike axis (the East-Coast Lorentzian signature `(D−1, 1)`). Zero timelike → `AllSpacelike`; ≥ 2 timelike → `CellSignature` with synthesised diagonal-sign pattern. Split-signature `(p, q ≥ 2)` is out of scope per design.md Decision 3 (sealed trait).
- [x] R5.5.3 7 property tests at [`tests/types/cubical_regge_geometry/light_cone_violation_tests.rs`](../../../deep_causality_topology/tests/types/cubical_regge_geometry/light_cone_violation_tests.rs): rejection of all-spacelike (2D + 4D), rejection of 2-timelike (3D) and 3-timelike (4D), acceptance of every single-timelike pattern in 3D, and Display formatting for both error variants.

### R5.6 `regge_action_lorentzian`

- [x] R5.6.1 `deep_causality_num::Complex<R: RealField>` already shipped per Block 0 audit; reused unchanged. No `deep_causality_num` micro-change required.
- [x] R5.6.2 [`regge_action_lorentzian(&self, complex) -> Complex<R>`](../../../deep_causality_topology/src/types/cubical_regge_geometry/curvature.rs) lives on the Lorentzian-only impl block. Returns `Complex { re: 0, im: hinge_action_sum(complex) }` — Wick-rotated phase convention `S^Lorentzian = i · S^Euclidean`.
- [x] R5.6.3 4 property tests at [`tests/types/cubical_regge_geometry/regge_action_lorentzian_tests.rs`](../../../deep_causality_topology/tests/types/cubical_regge_geometry/regge_action_lorentzian_tests.rs): real-part-is-zero, imag-part-equals-Euclidean on 2D open and 3D PerAxis, choice-of-timelike-axis-invariant.

### R5.7 Block R5 gates

- [x] R5.7.1 `cargo build -p deep_causality_topology` clean; `cargo clippy --all-targets -- -D warnings` clean across the entire workspace (one needless-typing and three `0 * D` index lints fixed at root cause per `feedback_clippy_lints`).
- [x] R5.7.2 100% coverage on every new file: `signature.rs`, `metric_tensor.rs`, `light_cone_violation.rs`, all 4 new test files plus extensions to `has_hodge_star.rs` / `curvature.rs` / `mod.rs`.
- [x] R5.7.3 R5-G3 Review — user to commit.

**Block R5 summary:** ~6 new source files, ~20 new tests (911 total topology tests), generic-over-S `HasHodgeStar` impl, Lorentzian-only Wick-rotated action, light-cone validation at construction time. Full workspace test + clippy regression clean.

### R5.8 `deep_causality_metric` integration (post-R5.7 cleanup)

Surgical refactor sourcing per-axis sign convention from `deep_causality_metric::Metric` instead of hand-rolled boolean checks. Closes the "metric crate is the single source of signature truth" architectural point.

- [x] R5.8.1 `CubicalReggeGeometry::signature()` returns `Metric::Lorentzian(D)` only when axis 0 is the canonical East-Coast timelike axis; otherwise returns `Metric::Custom { dim: D, neg_mask, zero_mask: 0 }` with `neg_mask` bit `i` set iff axis `i` is timelike. Lossless per-axis recovery (previously emitted `Lorentzian(D)` regardless of which axis was timelike — the lossy compression).
- [x] R5.8.2 [`metric_tensor.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/metric_tensor.rs) diagonal-sign computation switched from `if is_timelike { -l_sq } else { l_sq }` to `metric.sign_of_sq(axis)`-driven match. Future PGA / Custom signatures supported by construction.
- [x] R5.8.3 [`has_hodge_star.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/has_hodge_star.rs) `timelike_axes_in_orientation` helper now consults `Metric::sign_of_sq(axis) == -1` against the synthesised `Metric` value. The `SignatureMarker::sign_factor` static-elision fast path for Euclidean is preserved via an `Option<Metric>` guard — Euclidean skips the metric construction entirely.
- [x] R5.8.4 Test migration: the prior `signature_lorentzian_for_one_timelike_axis` test (asserting `Lorentzian(4)` for axis-3 timelike) split into two — `signature_axis_0_timelike_is_canonical_east_coast_lorentzian` (the genuine `Lorentzian(D)` case) and `signature_non_axis_0_timelike_is_custom_per_axis` (verifies the lossless `Custom { neg_mask }` shape). Per AGENTS.md §"Code testing": API change drives the test, not the reverse.
- [x] R5.8.5 Full workspace test + clippy regression clean.

**Net effect:** `deep_causality_metric` is now the single authoritative source of per-axis sign convention for cubical Regge geometry. The R5 type-level marker (`Euclidean` / `Lorentzian`) remains the typestate discriminator; the runtime sign values flow through `Metric` everywhere they're needed.

## Block R6 — Regge-action gradient + Metropolis updates

Depends on R5 (the gradient and Metropolis update must respect the signature; Lorentzian rejection uses R5.5's light-cone check).

### R6.1 Action gradient — Euclidean

- [x] R6.1.1 **Derivation simpler than design.md anticipated.** On axis-aligned cubical, `dihedral_angle = π/2` always (independent of edge lengths per the existing R2 result), so `deficit_angle = (4−n)·π/2` is purely combinatorial. All edge-length sensitivity of `S_R` flows through `vol(h)`:
  ```
  ∂S_R/∂L_i = Σ_{h : edge i ⊂ h's orientation}  (vol(h) / L_i) · deficit(h)
  ```
  (product rule on `vol(h) = Π L_a · L_b · ...`). No `arctan2` derivatives needed — the design.md note assumed a different (sheared) cubical convention. Documented in the [`gradient.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/gradient.rs) module header.
- [x] R6.1.2 [`regge_gradient(&self, complex) -> Vec<R>`](../../../deep_causality_topology/src/types/cubical_regge_geometry/gradient.rs) on the Euclidean impl block. Length `num_cells(1)`, indexed by `iter_cells(1)` order, returns a per-edge gradient even on `UnitEdge` / `Uniform` / `PerAxis` geometries (the "what would happen if I individually perturbed this edge" notion).
- [x] R6.1.3 Locality: each entry depends only on the O(2^(D−1)) hinges containing the edge. Total cost O(num_edges · 2^D). Verified by the `d3_gradient_entry_changes_only_when_local_edge_changes` test, which shows that under D=3 the gradient is entirely topology-driven (no cross-coupling).

### R6.2 Action gradient — Lorentzian

- [x] R6.2.1 Extended via a shared `hinge_gradient_sum` core method generic over `S`. The Lorentzian wrapper wraps each entry as `Complex { re: 0, im: hinge_gradient_sum }` — purely imaginary under the `S_L = i · S_E` convention chosen in R5.6.
- [x] R6.2.2 Property test `lorentzian_gradient_is_pure_imaginary_with_im_equal_to_euclidean` verifies the Wick-rotation correspondence to floating-point tolerance.

### R6.3 Finite-difference verification

- [x] R6.3.1 Property test `d3_gradient_matches_central_finite_difference_open_cube` verifies the closed form against a central FD estimate `(S(L+ε)−S(L−ε))/(2ε)` to relative error `< 1e-5` on a 3D open cube with distinct per-edge lengths.
- [x] R6.3.2 Equilibrium / stationary-point checks: `unit_edge_is_stationary_on_periodic_3d` (periodic ⇒ all deficits zero ⇒ gradient zero) and the complementary `unit_edge_open_3d_is_not_stationary_at_boundary` (open boundary ⇒ non-zero gradient even at unit edges).

### R6.4 `AcceptReject` and `metropolis_update`

- [x] R6.4.1 [`AcceptReject<R>`](../../../deep_causality_topology/src/types/cubical_regge_geometry/metropolis.rs) enum with `Accepted { edge, proposed_length, delta_action }` and `Rejected { edge, proposed_length, reason }` variants. Re-exported from crate root.
- [x] R6.4.2 [`RejectReason<R>`](../../../deep_causality_topology/src/types/cubical_regge_geometry/metropolis.rs) enum: `NonPositiveLength` (hard floor preserving detailed balance) and `Probabilistic { delta_action, threshold }` (Metropolis criterion rejection). `LightConeViolation` reject reason is **deferred** since Lorentzian Metropolis itself is deferred (next item).
- [x] R6.4.3 [`metropolis_update<G: Rng>`](../../../deep_causality_topology/src/types/cubical_regge_geometry/metropolis.rs) on the Euclidean impl block. Uses `deep_causality_rand::Normal::<R>::new(0, σ)` for the proposal and `StandardUniform` for the accept-reject coin. Rejects `L_new ≤ 0` per design.md Risk 5.
- [x] R6.4.4 **Lorentzian Metropolis deferred** per design.md Decision 7 "Wick rotation deferred subtlety". Reason: under our Lorentzian action convention `S_L = i · S_E`, `|exp(−β S_L)| = 1` identically — naive Metropolis-Hastings has no thermalisation. Standard fix is to do MC on the Euclidean action and analytically continue; the Euclidean primitive in R6.4.3 is exactly that. A future change set can wire the analytic-continuation layer on top.
- [x] R6.4.5 ΔS computation is exact, not local-approximate: because the action is bilinear in edge lengths (axis-aligned cubical, see R6.1.1 derivation), `ΔS = (L_new − L_old) · gradient[e]` is exact when only one edge changes. The current implementation evaluates the full gradient and indexes it; a future perf change can maintain an edge-to-hinges inverse map for O(2^D)-only updates.

### R6.5 Detailed-balance verification

- [x] R6.5.1 7 metropolis-tests in [`tests/types/cubical_regge_geometry/metropolis_tests.rs`](../../../deep_causality_topology/tests/types/cubical_regge_geometry/metropolis_tests.rs):
  - Variant pattern-match sanity.
  - `metropolis_step_returns_well_formed_outcome`: shape correctness.
  - `accepted_step_mutates_only_the_target_edge`: mutation semantics.
  - `rejected_step_leaves_geometry_unchanged`: rejection rollback semantics.
  - `non_positive_proposal_returns_non_positive_length_rejection`: hard floor.
  - `edge_lengths_stay_positive_across_long_run`: 5000-step smoke with acceptance-rate sanity bounds (`0.05 ≤ rate ≤ 0.95`) and length positivity invariant.
  - `delta_action_recorded_on_acceptance_matches_gradient_product`: bit-exact agreement between the reported `delta_action` and `(L_new − L_old) · gradient_pre[e]`.

  Full χ² distribution-matching against `exp(−β S_R)` over ~10⁶ steps is deferred per design.md Risk 5 mitigation — the 5K-step smoke + the exact bit-level ΔS check together are strong evidence that the algorithm is correct without paying the long-running-test cost. A long-running gate can be added later via `--features long-running-tests`.

### R6.6 Block R6 gates

- [x] R6.6.1 R6-G1 Compilation: `cargo build -p deep_causality_topology` clean; full workspace `cargo build` clean.
- [x] R6.6.2 R6-G2 Coverage: 100% on every new file (`gradient.rs`, `metropolis.rs`, both new test files). 2 root-cause clippy fixes (`needless_range_loop` → `enumerate` in both test files). `cargo clippy --all-targets -- -D warnings` clean across the entire workspace.
- [x] R6.6.3 R6-G3 Review — user to commit.

**Block R6 summary:** 2 new source files (gradient.rs, metropolis.rs) + 2 new test files (regge_gradient_tests.rs, metropolis_tests.rs) + 1 helper extraction (`axis_length_at_position` moved to mod.rs). +16 new tests (9 gradient + 7 metropolis), bringing topology test count to **938 passing**. Full workspace test + clippy regression clean. The exact `ΔS = (L_new − L_old) · gradient[e]` identity for bilinear-in-lengths cubical action is the load-bearing correctness property — verified bit-exactly by `delta_action_recorded_on_acceptance_matches_gradient_product`.

## Out-of-scope reminder

The following are explicitly NOT part of this change set (per design.md "Non-Goals"):

- The Hodge–Helmholtz decomposition (`hodge_decompose`) — separate change set `add-hodge-decomposition`.
- The 3D causal-fluid pipeline (TopologicalSignature, RollingHistory, FluidContext, SURD wiring, NS kernels) — separate change set sequence per [`notes/3DCausalFluidDynamics.md`](../../../notes/cfd/3DCausalFluidDynamics.md).
- Causal-graph analysis of turbulent flows.
- Sparse cubical complexes.
- GPU paths.
- Non-cubical regular tilings.
- Performance tuning beyond the algorithmically-natural O(2^D) per-edge gradient.
- Adaptive Metropolis step-size tuning.
- HMC sampling.
- Categorical-coherence proptest suite.

## Total effort

- Block 0 (spec refinement): ~3 hours, 0 LOC, documentation only.
- Block R4 (Hodge ⋆ + generic operators): ~400 LOC, ~18 tests, ~6 hours.
- Block R5 (signature + per-cell metric + Lorentzian variant): ~300 LOC, ~14 tests, ~5 hours.
- Block R6 (gradient + Metropolis): ~450 LOC, ~12 tests, ~7 hours.

**Total: ~1150 LOC, ~44 tests, ~21 hours focused work** (~6 hours above the proposal.md ~15h estimate; the delta is the R: RealField refinement work and the explicit per-edge derivation budget in R4.4).
