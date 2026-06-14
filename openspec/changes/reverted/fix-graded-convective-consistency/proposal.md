## Why

The graded-metric MMS study (`add-graded-metrics`, `examples/avionics_examples/dec_graded_mms`)
measured the order of the two march operators on smoothly graded meshes, in both the
max-norm and the L2 (solution) norm:

- **Viscous** `Δ₀ = δd` (Laplacian): **second order in both norms at every grading
  amplitude**, essentially unaffected by grading. It is self-adjoint, so its truncation
  errors cancel symmetrically. **No fix needed — out of scope.**
- **Convective** `i_X ω` (interior product): **loses second order in both norms** — the L2
  (solution) error stops converging and is ~5× the uniform error by spacing-ratio ≈ 1.11.
  It is *not* self-adjoint, so the off-centering error of the diagonal Hodge star (its dual
  cell is off-centre on a graded mesh) has no symmetric cancellation.

This is the **consistency** defect of the convective term, orthogonal to the already-fixed
**energy/skew** defect (`fix-dec-convective-instability`). Restoring second-order convective
accuracy on smooth graded meshes is the prerequisite for credible Re-10⁴ cavity and the
Stage-4 cylinder boundary layer — the whole economic case for graded meshes (cells where the
physics needs them *and* fast convergence) collapses if the convective term stays first order.

## What Changes

Per the ranked trade study (highest accuracy at highest speed): a **surgical correction as
default, a Galerkin star as fallback**, both **convective-only** (the viscous term is
verified out), and **gated by a prototype spike** so no full implementation lands on an
unverified mechanism.

- **Group A — gating spike (verify the mechanism before building).** Prototype the chosen
  correction on the 2D MMS and confirm the convective **L2 order returns to ≈ 2** across the
  amplitude sweep. If the surgical correction resists clean derivation, fall back to the
  Galerkin star (the textbook-guaranteed floor). This decides the default mechanism before
  any production code.
- **Default — surgical off-centering consistency correction (`graded-convective-order`).**
  Add the metric-gradient off-centering term to the **convective rate assembly only**; keep
  the diagonal Hodge star (and its memoized fast-path) for structure and the Leray/Poisson
  solve. Restores second order on smooth grading at near-diagonal cost. The correction
  **vanishes identically on uniform meshes**, so existing uniform results cannot move.
- **Fallback — Galerkin / Whitney (Q1) Hodge star (`graded-convective-order`).** An opt-in
  non-diagonal star for meshes where the smoothness assumption breaks (Stage-4 cut cells,
  Stage-5 AMR), where Galerkin's robustness earns its per-step cost. The diagonal star
  stays the default.
- **Validation.** The `dec_graded_mms` convective L2-order column returns to ≈ 2 with the
  correction on; the uniform-mesh validation ladder (Taylor–Green, Couette, Poiseuille) is
  unchanged; structure (divergence-freeness) stays exact at every grading. Composes with the
  vector-slot skew fix (the two convective defects are orthogonal and both apply).

## Impact

- **Affected specs (new capability):** `graded-convective-order`.
- **Affected code:** `deep_causality_physics` (the convective rate assembly — the correction
  term); `deep_causality_topology` (the optional Galerkin/Whitney Q1 star variant alongside
  the diagonal `has_hodge_star`). The diagonal star, the Leray/Poisson solve, and the viscous
  operator are untouched by default.
- **Out of scope (empirically excluded):** the viscous/Laplacian operator (verified second
  order on graded in both norms).
- **No regression risk on uniform meshes:** the correction is zero on uniform spacing; the
  Galerkin star is opt-in. Structure guarantees are unchanged (combinatorial, metric-free).
- **Performance:** the diagonal fast-path is preserved by default; only the convective rate
  gains a cheap local stencil term. The Galerkin star pays the banded-solve cost only when
  explicitly selected for rough meshes.
- **Unblocks:** credible high-Re wall-bounded accuracy on graded meshes (the R1 promise) and
  composes with Stage 4 (cut cells reuse the Galerkin fallback path).
