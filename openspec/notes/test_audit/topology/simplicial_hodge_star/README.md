<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The simplicial Hodge star is not a Hodge star at intermediate grades

Found while closing `unified-math-next` task 6.7u, which set out to reformulate
`ideal_induction_kernel` onto `Manifold::interior_product` by way of the star–wedge identity. That
route needs `⋆₂` on a simplicial complex. This is what `⋆₂` is.

**Nothing here is a proposal.** It is a measurement, its cause in the source, and what it reaches.

## The measurement

A regular tetrahedron of edge length `h`, built through `SimplicialComplex::with_geometry`, asking
`hodge_star_operators()` for its diagonal at `h = 1` and `h = 2`:

| operator | `h = 1` | `h = 2` | measured scaling | required, `⋆ₖ = \|⋆σ\|/\|σ\|` |
|---|---|---|---|---|
| `⋆₀` | 0.02946 | 0.23570 | `L³` | `L³` ✓ |
| `⋆₁` | 1.00000 | 2.00000 | `L¹` | `L¹` — right dimension, wrong value |
| `⋆₂` | 0.43301 | 1.73205 | **`L²`** | **`L⁻¹`** ✗ |
| `⋆₃` | 8.48528 | 1.06066 | `L⁻³` | `L⁻³` ✓ |

A Hodge star on `k`-forms in `n` dimensions scales as `L^{n−2k}`. In three dimensions that is
`L³, L¹, L⁻¹, L⁻³`. The middle two do not match, and `⋆₂` is out by a factor of `h³`.

## The cause

`lazy_hodge_star.rs`, `build_lumped_mass_hodge_star`. The diagonal entry is chosen by three cases:

* `k = 0` — the barycentric dual volume, `Σ|tets ∋ v| / (n+1)`. This *is* `|⋆v|/|v|`, since a
  vertex has unit volume. Correct.
* `k = n` — `1 / |σ|`. This *is* `|⋆σ|/|σ|`, since the dual of a top cell is a point. Correct.
* **everything in between — `mass_val = primal_vol`**, the primal volume `|σ|` itself.

The third case is not a dual/primal ratio, and nothing in the function computes a circumcentre or a
dual volume for an intermediate grade. `⋆₁ = |e|` has the right dimension by coincidence — `|⋆e|/|e|`
is `L²/L¹` and `|e|` is `L¹` — and is still not the same number. `⋆₂ = |f|` has neither.

## What it contradicts

`HasHodgeStar::hodge_star_matrix` documents its entries as "the dual / primal cell-volume ratios".
`ReggeGeometry` vends these operators verbatim. So the contract and the implementation disagree at
every grade strictly between `0` and `n`.

Note that the docstring was already corrected once, under task 6.7o, for saying the wrong *shape*.
This is a different error in the same operator, and it survived that correction because the shape
was what was being checked.

## What it reaches

Every generic differential operator on `Manifold<SimplicialComplex<R>, R>` goes through
`hodge_star_matrix`: `hodge_star`, `codifferential`, `laplacian`, `hodge_decomposition`, `leray`,
`neumann_poisson`. Each of them uses an intermediate grade whenever it is asked for one — which for
a 3D complex is `k = 1` and `k = 2`, and for a 2D complex is `k = 1`.

The cubical backend is unaffected: `CubicalReggeGeometry` computes its entries from per-cell volume
data and is a separate implementation.

`ideal_induction_kernel` no longer depends on it. Task 6.7u moved its contraction onto Whitney
interpolation, which needs no Hodge star, precisely so that the kernel would not be built on this.

## What a fix needs

The circumcentric dual volume of an intermediate-grade simplex — for a `k`-simplex `σ`, the volume
of the convex hull of circumcentres over every chain `σ = σ_k < σ_{k+1} < … < σ_n`, with signs where
the mesh is not well-centred. That is a real piece of DEC construction (Hirani, *Discrete Exterior
Calculus*, Caltech 2003, §2.5), and it comes with a well-centredness question the lumped-mass form
was presumably chosen to avoid.

The alternative, which avoids that question, is the Galerkin/Whitney **mass matrix** — `M_k` with
entries `∫ w_i · w_j` over the Whitney basis. It is not diagonal, which is a change to the operator
surface rather than to one function, but it needs no circumcentres and is the standard choice for
meshes that are not well-centred. Task 6.7h recorded it as the alternative for the same reason.

Either way this is a change of its own, with its own oracles: the dimensional scaling above is a
necessary condition and nowhere near a sufficient one.
