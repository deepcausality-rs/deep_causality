## Context

A graded mesh is a `PerEdge` metric on an unchanged lattice. The combinatorial guarantees
(`d∘d = 0`, Stokes, divergence-free-by-construction) hold exactly at any grading — only
accuracy order is at stake. The `add-graded-metrics` MMS study measured the order of both
march operators on smoothly graded meshes, in max- and L2-norms:

| Operator | self-adjoint? | order on graded (max / L2) | verdict |
|---|---|---|---|
| Viscous `Δ₀ = δd` | yes | ≈ 2 / ≈ 2 at all amplitudes | unaffected — out of scope |
| Convective `i_X ω` | no | collapses / collapses | the defect to fix |

**Root cause.** The diagonal Hodge star stores one scalar per cell (averaged dual-volume /
primal-volume ratio); on a graded mesh the dual cell is off-centre relative to the primal
centroid by `O(Δℓ) ≈ O(h·∇ℓ)`. A scalar cannot encode that displacement, so the star is
first-order on graded meshes. The self-adjoint Laplacian cancels this symmetrically (stays
2nd order); the non-self-adjoint interior product does not, so the convective term loses
order. This is the **consistency** defect — distinct from the energy/skew defect fixed in
`fix-dec-convective-instability`.

## Goals / Non-Goals

**Goals**
- Restore ≈ 2nd-order convective accuracy (L2) on *smooth* graded meshes.
- Preserve the diagonal star, the Leray/Poisson fast-path, and all uniform-mesh results.
- Provide a robust fallback for non-smooth meshes (cut cells, AMR).
- Gate the build on an empirical prototype, not theory alone.

**Non-Goals**
- Touching the viscous/Laplacian operator (verified 2nd order on graded).
- Changing the structure guarantees (already exact, metric-free).
- High-order accuracy on arbitrary/degenerate meshes (a research problem, out of scope).
- Replacing the diagonal star globally (only the convective assembly and the opt-in
  fallback are affected).

## Decisions

### D1: Convective-only scope, fixed by measurement
The viscous term is excluded because the MMS measured it second order in both norms at every
amplitude. The change touches only the convective rate assembly (default) and an opt-in
star variant (fallback). This is the single largest scope decision and it is empirical, not
assumed.

### D2: Default mechanism — surgical off-centering correction (Option 3)
Add the leading off-centering consistency term — a local, metric-gradient (`∇ℓ`) stencil
correction — to the convective rate `i_u(du)` only. The diagonal star, the codifferential's
`M⁻¹`, and the Leray/Poisson CG solve are untouched, so the memoized diagonal fast-path
survives. The correction is **identically zero on uniform spacing** (`∇ℓ = 0`), guaranteeing
no movement of existing uniform results.

### D3: Fallback mechanism — Galerkin / Whitney (Q1) Hodge star (Option 1)
A non-diagonal Hodge star assembled from the cubical Whitney (bilinear/trilinear Q1) basis,
`⋆_ij = ∫ Wᵢ ∧ ⋆Wⱼ`, which is second-order consistent on arbitrary smooth meshes. Opt-in,
for meshes where D2's smoothness assumption breaks (cut cells, AMR). It is the
textbook-guaranteed floor: if D2's correction cannot be derived cleanly or stably, the
default escalates to D3.

### D4: Gate the build on a prototype spike (no unverified mechanism in production)
Before production code, prototype the chosen mechanism on the 2D MMS and confirm the
convective **L2 order returns to ≈ 2** across the amplitude sweep. The spike decides D2-vs-D3
as the default. (The viscous measurement and the both-norms convective measurement that
scoped this change are already done — see the example.)

### D5: Composition with the energy/skew fix
The two convective defects are orthogonal: skew-symmetrization fixes *energy* (stability);
this fix addresses *consistency* (order). Both apply to the same `i_u(du)` assembly; the
correction is added to the already-skew-symmetrized term, and the energy-neutrality and
stability gates of `fix-dec-convective-instability` must stay green.

### D6: Acceptance is the MMS instrument plus the uniform ladder
- `dec_graded_mms`: convective L2-order column returns to ≈ 2 across amplitudes (the headline
  acceptance).
- Uniform-mesh ladder (Taylor–Green convergence, Couette/Poiseuille exactness, the energy
  budget) unchanged — the correction vanishes on uniform.
- Structure: divergence-freeness exact at every grading (already pinned, must stay green).

## Risks / Trade-offs

- **The correction may resist clean/stable derivation (D2).** Mitigated by D4 (prototype
  first) and D3 (Galerkin floor): the change still lands at 2nd order via the fallback, just
  paying the matrix-star tax on the solve.
- **Galerkin star inflates the Leray/Poisson solve** (banded mass matrix, loses the diagonal
  fast-path). Mitigated by keeping it opt-in (D3) and convective-scoped; the default path
  never pays it.
- **Interaction with the skew fix (D5).** Mitigated by re-running the energy-budget and
  stability gates as part of acceptance.
- **Cut-cell meshes (Stage 4) are not smooth.** The Galerkin fallback (D3) is exactly the
  path cut cells will use; this change builds it once and Stage 4 reuses it.

## Open Questions

1. **Exact form of the off-centering correction (D2).** Derive from the truncation analysis
   of `±⋆(⋆ω ∧ X♭)` on a graded diagonal metric; the prototype confirms it empirically.
2. **Default selection.** D2 if the prototype recovers L2 order cleanly and stably; else D3.
   Decided by the Group A spike, recorded here before production code.
3. **Whitney star structure on mixed-periodicity / walled lattices** — confirm the Q1
   assembly composes with the boundary-corrected star from Stage 3.
