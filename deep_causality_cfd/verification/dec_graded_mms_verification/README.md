# Graded-metric MMS truncation study — CFD rung R1

Measures the **accuracy order** of the two operators of the incompressible march on graded
meshes — the **convective** `i_X ω` (interior product) and the **viscous** `Δ₀ = δd`
(Laplacian) — in **two norms** (max + L2). The heavy-verification companion to the fast CI
gate, per tests-fast / examples-verify.

```text
cargo run --release -p deep_causality_cfd --example dec_graded_mms_verification
```

## Method

On an `N × N` torus graded on axis 1 by a smooth periodic edge-length modulation
`ℓ(pos) = 1 + a·cos(2π·pos/N)` (smooth across the seam; sums to `N`, so the wavenumber is
unchanged):

- **Convective:** the Cartan magic-formula MMS `i_X dω + d i_X ω → L_X ω` for
  `ω = (sin ky, sin kx)`, `X = (cos kx, cos ky)`.
- **Viscous:** `δd f` against `2k²·f` for `f = sin(kx)·sin(ky)`.

**Cochain convention (load-bearing).** DEC operators act on **cochains = integrals over
cells**: a discrete 1-form on an edge is `∫ ω ≈ (tangential midpoint value) · ℓ_edge`. Both
`ω` and `X♭` carry that `ℓ` factor, and the 1-form output is normalised by `ℓ` before
comparison to the pointwise analytic. Omitting `ℓ` is invisible on a uniform mesh (`ℓ = 1`)
but `O(ℓ)`-wrong on a graded one.

## Measured outcome

| Operator | max-norm order (graded) | L2 order (graded) | Affected by grading? |
|---|---|---|---|
| **Convective** `i_X ω` | ≈ 2 (to amp 0.5 = 3:1 ratio) | **≈ 2** | only the error *constant* grows mildly |
| **Viscous** `Δ₀ = δd` | ≈ 2 | **≈ 2** | only the error *constant* grows mildly |

Two conclusions:

1. **Smooth grading retains second order — for both operators.** Even at strong grading
   (a 3:1 spacing ratio) the order holds at ≈ 2; only the error constant grows. The R1
   promise — resolve walls cheaply *and* keep fast convergence — holds today, no follow-up
   needed.
2. **Structure is metric-free and exact at any grading** — divergence-freeness of the Leray
   projection is combinatorial, pinned independently by the topology exactness test.

> **History / caveat.** An earlier revision of this study mis-measured a convective
> "order collapse" on graded meshes. The cause was a *measurement* bug — feeding pointwise
> 1-form values instead of edge-integrals (omitting the `ℓ` factor above), inconsistently
> between `ω` and `X♭`. With consistent cochains the convective operator is second order,
> like the self-adjoint viscous operator that was always measured correctly (0-forms carry
> no length factor). This example enforces the correct convention so the mistake cannot
> recur.
