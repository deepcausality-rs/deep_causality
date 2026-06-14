# Graded-metric MMS truncation study — CFD rung R1

Quantifies how the **accuracy order** of the discrete interior product `i_X ω` — the
building block of the convective term `i_u(du)` — behaves as the **grading strength** of
the mesh rises. It is the heavy-verification companion to the fast CI gate
`cartan_formula_converges_under_smooth_grading` (which only asserts that mild grading does
not destroy convergence), per the project's tests-fast / examples-verify split.

```text
cargo run --release -p avionics_examples --example dec_graded_mms
```

## Method

On an `N × N` torus we manufacture `ω = (sin ky, sin kx)`, `X = (cos kx, cos ky)`,
`k = 2π/N`, whose continuum Lie derivative `L_X ω = i_X dω + d i_X ω` is known in closed
form. The metric is graded on axis 1 by a smooth, periodic edge-length modulation
`ℓ(pos) = 1 + a·cos(2π·pos/N)` (smooth across the torus seam; sums to `N`, so the
wavenumber is unchanged). The manufactured solution is evaluated at the **physical**
(cumulative-length) edge midpoints, with `X` supplied as the flat `X♭ = X^axis·ℓ(edge)`.
The relative sup-error of the discrete `i_X dω + d i_X ω` against the analytic Lie
derivative over a refinement sweep gives the observed order `p = log₂(E_N / E_{2N})`.

## What it shows (representative run)

| amplitude `a` | spacing ratio | finest-grid order |
|---|---|---|
| 0.00 (uniform) | 1.00 | ≈ 2.0 |
| 0.05 | 1.11 | < 1.5 |
| ≥ 0.10 | ≥ 1.22 | → 0 (plateau) |

Two honest conclusions:

1. **Structure is metric-free and exact at any grading.** `d∘d = 0`, the discrete Stokes
   theorem, and the divergence-free-by-construction property of the Leray projection are
   combinatorial — proven exact on a *strongly* graded lattice by the topology test
   `leray_projection_stays_divergence_free_under_strong_grading`, independent of this
   accuracy study.

2. **The convective operator loses formal second order under grading.** The discrete
   interior product `±⋆(⋆ω ∧ X♭)` is still *convergent* under refinement (the error keeps
   decreasing), but it is only ~first order on anisotropic/graded cells and the error
   plateaus at modest grading. This is the same consistency class as the convective-term
   *form-slot* issue (`fix-dec-convective-instability`), and a candidate for the same
   vector-slot M-adjoint correction. It means graded meshes are sound for *structure*
   today, but recovering high-order *accuracy* of the convective term on them needs that
   follow-up fix — a finding this study exists to surface, not hide.
