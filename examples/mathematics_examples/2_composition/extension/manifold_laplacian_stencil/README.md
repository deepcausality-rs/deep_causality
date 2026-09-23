# Tensor x Topology: Discrete Laplacian on a 1D Manifold

## Introduction

This example computes the discrete Laplacian of a scalar field on a 1D manifold with `ManifoldWitness::extend`. The closure reads neighbor values from `w.data()` and applies the stencil at each vertex.

The Laplacian measures, for each point, how far its value differs from the average of its neighbors. That number drives heat diffusion, fluid flow, electric potential, image blurring, edge detection and mesh smoothing.

The same code shape generalizes to 2D triangulated surfaces (the cotangent Laplacian used in computer graphics for mesh smoothing and texture synthesis), to irregular 3D meshes (finite element solvers), and to graphs (graph signal processing, GNNs, community detection). The manifold abstraction lets you write the stencil once and reuse it on any discretized space.

## How to Run

```bash
cargo run -p mathematics_examples --example manifold_laplacian_stencil_examples
```

## What It Demonstrates

The CoMonad pattern: `extend` walks every focused position of the manifold, and the closure reads the full underlying data. Stencil operators on a discretized space use this pattern.

The geometry (a 1D simplicial complex with 7 vertices and 6 edges) provides the context; the scalar field (a `CausalTensor<FloatType>`) provides the payload. Topology and tensor compose through the witness pattern, each crate working in its own terms.

## Mathematical Content

The discrete Laplacian on a 1D regular grid:

```
(Delta phi)(i) = phi(i-1) + phi(i+1) - 2 * phi(i)
```

Boundaries use a Neumann reflection (`phi outside = phi at boundary`), so the endpoints contribute zero curvature.

## What This Example Skips

This example uses the simplest discrete Laplacian: a uniform 1D three-point stencil with Neumann boundaries. It gives the right answer for this case; a production Laplacian carries machinery the example omits. Each piece below is a local change inside the `extend` closure; the manifold, the comonadic walk and the surrounding pipeline stay untouched. The example omits:

- **Non-uniform grid spacing.** A real grid has varying edge lengths `h_i`. The correct second-derivative stencil is `2 * (phi_r - phi_i) / (h_r * (h_l + h_r)) + 2 * (phi_l - phi_i) / (h_l * (h_l + h_r))`. The uniform-`h` simplification used here misestimates curvature wherever the mesh is graded.
- **Cotangent (Laplace-Beltrami) weights.** On 2D triangulated surfaces, the canonical discrete Laplacian weights each edge by `(cot(alpha) + cot(beta)) / 2` where `alpha, beta` are the angles opposite the edge. Computer graphics uses this operator for mesh smoothing, parametrization and spectral shape analysis. The example uses unit weights.
- **Mass matrix `M`.** The full discrete Laplace-Beltrami operator is `M^-1 L_c`, where `M` is the diagonal matrix of dual-cell areas (or volumes) at each vertex. Omitting `M` is fine for smoothing but wrong for anything quantitative such as solving a Poisson equation or extracting eigenvalues.
- **Use of the boundary operator `d1`.** The example builds `d1` and then ignores it. The metric-aware way to derive the Laplacian is `L = d1^T * M_edge * d1` (the discrete exterior calculus formula). Hand-rolling the stencil works on a regular line; on irregular complexes it does not.
- **Boundary condition richness.** Only Neumann reflection is implemented. Dirichlet (fixed value), Robin (mixed), and periodic (wrap-around) boundaries each require a different branch inside the closure. Absorbing boundaries for wave equations need a one-sided derivative.
- **Higher-order stencils.** The three-point stencil is second-order accurate. Five-point (`-phi(i-2) + 16 phi(i-1) - 30 phi(i) + 16 phi(i+1) - phi(i+2)) / 12` style) is fourth-order. Higher order matters when the field has fine features.
- **Higher-dimensional generalization.** The example is 1D. Two-dimensional rectangular grids need a five-point or nine-point stencil. Three-dimensional cubic grids need seven or twenty-seven. Triangulated surfaces and tetrahedral meshes need the cotangent formula above.
- **The inverse direction.** Most useful Laplacian computations are inversions: solving `L x = b` for the Poisson equation (electrostatics, incompressible-flow pressure, surface reconstruction), or solving the generalized eigenproblem `L v = lambda M v` for spectral mesh analysis and graph Laplacian methods. This example applies `L` forward.
- **Sparse linear-algebra kernels.** On large meshes one sparse matrix-vector product performs the whole apply. The boundary matrix is already a `CsrMatrix`, so assembling the full Laplacian once and reusing it runs faster per step.
- **Sign convention awareness.** Physics defines `Delta` as the negative-semi-definite operator that makes peaks negative. Graphics and graph theory often use the opposite sign so `L` is positive semi-definite (eigenvalues `0 = lambda_0 <= lambda_1 <= ...`). The example uses the physics convention; cross-check before plugging the output into a library that expects the other one.

Adding any of these is a local edit inside the `extend` closure plus, where relevant, a richer mesh data structure. The comonadic walk and the rest of the pipeline stay the same, so a pipeline can swap operators in and out on demand.

## Key APIs

- `SimplicialComplex`, `Skeleton`, `Simplex` for the topological skeleton
- `CsrMatrix::from_triplets` for the boundary operator `d1`
- `Manifold::new` to attach data to the complex
- `ManifoldWitness::extend` for the comonadic walk

## Adaptation

- Use the full `d1` boundary matrix to derive a metric-aware Laplacian on a non-regular complex.
- Replace the 1D line with a 2D triangulation; the stencil generalizes to the cotangent Laplacian.
- Chain a `Functor::fmap` to threshold or smooth the result.
