# Graded-metric MMS truncation study — CFD rung R1

Quantifies the **accuracy order** of the two operators of the incompressible march on
graded meshes — the **convective** `i_X ω` (interior product) and the **viscous** `Δ₀ = δd`
(Laplacian) — in **two norms**: max (pointwise truncation) and L2 (solution error). The
two-norm split matters because diagonal-DEC operators are often *supraconvergent* (1st-order
truncation but 2nd-order solution), so a max-norm-only study can overstate the practical
error. The heavy-verification companion to the fast CI gate
`cartan_formula_converges_under_smooth_grading`, per tests-fast / examples-verify.

```text
cargo run --release -p avionics_examples --example dec_graded_mms
```

## Method

On an `N × N` torus graded on axis 1 by a smooth periodic edge-length modulation
`ℓ(pos) = 1 + a·cos(2π·pos/N)` (smooth across the seam; sums to `N`, so the wavenumber is
unchanged):

- **Convective:** manufacture `ω = (sin ky, sin kx)`, `X = (cos kx, cos ky)`; the continuum
  Lie derivative `L_X ω = i_X dω + d i_X ω` is known. Evaluated at physical (cumulative-
  length) edge midpoints with the metric-correct flat `X♭ = X^axis·ℓ(edge)`.
- **Viscous:** manufacture `f = sin(kx)·sin(ky)` at physical vertices; the discrete `δd f`
  is compared to the analytic `2k²·f`.

## Measured outcome

| Operator | max-norm order (graded) | **L2 order (graded)** | Affected by grading? |
|---|---|---|---|
| **Viscous** `Δ₀ = δd` | ≈ 2 at all amplitudes | **≈ 2 at all amplitudes** | **No** — robust second order |
| **Convective** `i_X ω` | collapses → 0 | **collapses → 0** | **Yes** — genuine order loss |

Two conclusions, both honest:

1. **Structure is metric-free and exact at any grading** — divergence-freeness of the Leray
   projection is combinatorial, pinned independently by
   `leray_projection_stays_divergence_free_under_strong_grading`.
2. **Only the convective operator loses order, and it loses it in the solution (L2) norm
   too** — not a supraconvergence illusion. The interior product `±⋆(⋆ω ∧ X♭)` is not
   self-adjoint, so the off-centering error of the diagonal Hodge star has no symmetric
   cancellation (the self-adjoint Laplacian does cancel → stays 2nd order). Restoring
   convective order on graded meshes is the subject of the follow-up change
   `fix-graded-convective-consistency`; this example is its acceptance instrument (the
   convective L2-order column must return to ≈ 2).
