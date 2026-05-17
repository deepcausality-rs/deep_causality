# Tensor x Topology: Discrete Laplacian on a 1D Manifold

## Introduction

The Laplacian is the workhorse operator behind most numerical simulations you have ever heard of. It answers the question: "for each point, how much does its value differ from the average of its neighbors?" That single number drives heat diffusion, fluid flow, electric potential, image blurring, edge detection, and mesh smoothing.

This example computes the Laplacian on a discretized line. The same code shape generalizes to 2D triangulated surfaces (the cotangent Laplacian used in computer graphics for mesh smoothing and texture synthesis), to irregular 3D meshes (finite element solvers), and to graphs (graph signal processing, GNNs, community detection). The "manifold" abstraction lets you write the stencil once and reuse it on whatever discretized space your problem lives on.

A scalar field on the vertices of a `Manifold` is processed with `ManifoldWitness::extend`. The closure reads neighbor values from `w.data()` and computes the discrete Laplacian stencil at each vertex.

## How to Run

```bash
cargo run -p mathematics_examples --example tensor_x_topology_laplacian
```

## What It Demonstrates

The CoMonad pattern: `extend` walks every focused position of the manifold and the closure has access to the full underlying data. This is exactly the pattern used for stencil operators on a discretized space.

The geometry (a 1D simplicial complex with 7 vertices and 6 edges) provides the context; the scalar field (a `CausalTensor<f64>`) provides the payload. Topology and tensor compose through the witness pattern without either crate knowing about the other.

## Mathematical Content

The discrete Laplacian on a 1D regular grid:

```
(Delta phi)(i) = phi(i-1) + phi(i+1) - 2 * phi(i)
```

Boundaries use a Neumann reflection (`phi outside = phi at boundary`), so the endpoints contribute zero curvature.

## What This Example Skips

This example uses the simplest possible discrete Laplacian: a uniform 1D three-point stencil with Neumann boundaries. It runs end-to-end and produces the right answer for the case at hand, but a production Laplacian operator carries machinery this example deliberately omits. The good news is that adding any of the pieces below is a local change inside the `extend` closure; the manifold, the comonadic walk, and the surrounding pipeline stay untouched.

Specifically, the example omits:

- **Non-uniform grid spacing.** A real grid has varying edge lengths `h_i`. The correct second-derivative stencil is `2 * (phi_r - phi_i) / (h_r * (h_l + h_r)) + 2 * (phi_l - phi_i) / (h_l * (h_l + h_r))`. The uniform-`h` simplification used here would over- or under-estimate curvature anywhere the mesh is graded.
- **Cotangent (Laplace-Beltrami) weights.** On 2D triangulated surfaces, the canonical discrete Laplacian weights each edge by `(cot(alpha) + cot(beta)) / 2` where `alpha, beta` are the angles opposite the edge. This is the operator that powers mesh smoothing, parametrization, and spectral shape analysis in computer graphics. The example uses unit weights.
- **Mass matrix `M`.** The full discrete Laplace-Beltrami operator is `M^-1 L_c`, where `M` is the diagonal matrix of dual-cell areas (or volumes) at each vertex. Omitting `M` is fine for smoothing but wrong for anything quantitative such as solving a Poisson equation or extracting eigenvalues.
- **Use of the boundary operator `d1`.** The example builds `d1` and then ignores it. The metric-aware way to derive the Laplacian is `L = d1^T * M_edge * d1` (the discrete exterior calculus formula). Hand-rolling the stencil works on a regular line; on irregular complexes it does not.
- **Boundary condition richness.** Only Neumann reflection is implemented. Dirichlet (fixed value), Robin (mixed), and periodic (wrap-around) boundaries each require a different branch inside the closure. Absorbing boundaries for wave equations need a one-sided derivative.
- **Higher-order stencils.** The three-point stencil is second-order accurate. Five-point (`-phi(i-2) + 16 phi(i-1) - 30 phi(i) + 16 phi(i+1) - phi(i+2)) / 12` style) is fourth-order. Higher order matters when the field has fine features the coarse stencil cannot resolve.
- **Higher-dimensional generalization.** The example is 1D. Two-dimensional rectangular grids need a five-point or nine-point stencil. Three-dimensional cubic grids need seven or twenty-seven. Triangulated surfaces and tetrahedral meshes need the cotangent formula above.
- **The inverse direction.** Most useful Laplacian computations are inversions, not applications: solving `L x = b` for the Poisson equation (electrostatics, incompressible-flow pressure, surface reconstruction), or solving the generalized eigenproblem `L v = lambda M v` for spectral mesh analysis and graph Laplacian methods. The example only applies `L` forward.
- **Sparse linear-algebra kernels.** Even when applying `L`, large meshes benefit from a single sparse matrix-vector product instead of a per-vertex closure. The boundary matrix is already stored as `CsrMatrix`; assembling the full Laplacian as a sparse matrix once is faster than running `extend` per step.
- **Sign convention awareness.** Physics defines `Delta` as the negative-semi-definite operator that makes peaks negative. Graphics and graph theory often use the opposite sign so `L` is positive semi-definite (eigenvalues `0 = lambda_0 <= lambda_1 <= ...`). The example uses the physics convention; cross-check before plugging the output into a library that expects the other one.

Adding any of these is a local edit inside the `extend` closure plus, where relevant, a richer mesh data structure. The comonadic walk does not change. That is the property the example exposes: the operator definition is one swap-out away, the rest of the pipeline stays the same.
This also allows for parametric operator pipelines that can swap in and out operators on demand.

## Key APIs

- `SimplicialComplex`, `Skeleton`, `Simplex` for the topological skeleton
- `CsrMatrix::from_triplets` for the boundary operator `d1`
- `Manifold::new` to attach data to the complex
- `ManifoldWitness::extend` for the comonadic walk

## Adaptation

- Use the full `d1` boundary matrix to derive a metric-aware Laplacian on a non-regular complex.
- Replace the 1D line with a 2D triangulation; the stencil generalizes to the cotangent Laplacian.
- Chain a `Functor::fmap` to threshold or smooth the result.
