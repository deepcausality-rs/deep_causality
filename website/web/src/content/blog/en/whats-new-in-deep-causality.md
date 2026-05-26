---
title: "What's New in Deep Causality 0.13.7"
description: "Release highlights: precision as a parameter, fluid dynamics kernels, Hodge decomposition and cubical Regge calculus, biconnectivity in ultragraph, and a witness-typed isomorphism trait family."
date: 2026-05-26
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

This release touches 16 crates across the workspace. Four ship breaking changes: `ultragraph` (0.8 → 0.9), `deep_causality_multivector` (0.4 → 0.5), `deep_causality_topology` (0.5 → 0.6), and `deep_causality_physics` (0.5 → 0.6). The rest reamain fully API-compatible.

Five themes stand out.

## 1. Precision is now a parameter

The biggest cross-crate change. `deep_causality_physics`, `deep_causality_multivector`, and `deep_causality_topology` were generalized over `R: RealField`. Hard-coded `f64` is gone from kernels, unit types, and state carriers. That includes astro, fluids, MHD, photonics, EM fields, relativity spacetime, dynamics, thermodynamics, nuclear, materials, quantum gates, and `MaxwellSolver`. `HilbertState` and `HopfState` are now generic over `R` with no `f64` default.

In practice, you can run the same kernel with `f32` for throughput, `f64` for production, or extended-precision types such as `Float106` when stability matters, all by Changing just one type alias. Take a look at the [mathematics examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples) for how precision as a parameter is used in practice.

This is the breaking change behind the major bumps in physics, multivector, and topology crates. Migration is mechanical: add an `R: RealField` parameter, or pin a concrete float type at the call site.

## 2. Fluid dynamics, end to end

`deep_causality_physics` gained a complete fluid-dynamics module, added in 15 staged groups:

- **Governing equations**: Navier–Stokes residual, conservation forms.
- **Constitutive kernels**: viscous stress. `ViscousStress` and `ReynoldsStress` were promoted from a shared `CauchyStress` to dedicated newtypes.
- **Dimensionless numbers**: Reynolds, Mach, Prandtl, and friends.
- **Turbulence**: TKE, dissipation, eddy quantities.
- **Coherent-structure detectors**: Q-criterion, λ₂, vorticity-based detectors.
- **Compressible-flow thermodynamics** and **boundary-layer kernels**.
- **Regime evaluators**: incompressible Newtonian NS, Stokes, Euler, and compressible Newtonian NS. Each is a distinct regime with its own validity checks.
- **Reference-solution verification tests** for the NS regimes (Poiseuille, Couette, etc.).

Raw-scalar inputs in three fluid kernels are now validated rather than trusted, and the test tree was restructured to mirror `src/kernels/`.

## 3. Topology: Hodge decomposition and cubical Regge calculus

`deep_causality_topology` added real differential geometry."

**Hodge decomposition** lands as a first-class operation:

- New `HodgeDecomposition` carrier type and matching error variant.
- A `hodge_decompose` algorithm using matrix-free conjugate gradient.
- A `HasHodgeStar<R>` capability trait, with simplicial and cubical implementations (UnitEdge, Uniform, PerAxis, and PerEdge tiers).
- Property-based tests covering orthogonality and idempotence.
- Lazy ⋆ population via `OnceLock`, so the star operator is built once and cached.

**Cubical Regge calculus** ships as a working core:

- Cell volumes, hinges, dihedrals, deficit angles, and the discrete Einstein–Hilbert action on cubical lattices.
- A Lorentzian signature marker and Wick-rotated action. Signature truth is routed through `deep_causality_metric`.
- Regge action gradient plus a Metropolis–Hastings sampler, with a single-edge gradient on the hot path.
- A `Manifold` generic widening over `ChainComplex`, so the same manifold types work simplicially or cubically.

**Other topology additions**:

- `TopologicalInvariants` extractor (B1a). Betti numbers and friends in one pass.
- `PointCloud::triangulate_delaunay` with discriminating errors for degenerate input, capped at ambient dimension.
- `LatticeComplex` plus cubical aliases and Neighborhood strategies.
- A new `TopologyErrorEnum::HodgeDecompositionFailed` variant.

The breaking changes here include the removal of the old `Lattice`, `LatticeWitness`, and `DualLattice` structs, plus the `CWComplex` trait. All four are superseded by the new `LatticeComplex`-based design.

## 4. Ultragraph 0.9: biconnectivity

`ultragraph` gained a biconnectivity decomposition:

- `articulation_points`, `bridges`, and `biconnected_components` are now methods on `StructuralGraphAlgorithms`.
- Benchmarks for the decomposition path.


## 5. A witness-typed isomorphism trait family

`deep_causality_num` got a three-tier isomorphism trait stack:

- **Tier 1**: marker subtraits that declare "these two carriers are iso."
- **Tier 2**: witness-typed `Iso` traits with `StandardIso` as the canonical witness.
- **Tier 3**: the integration layer, including `From` and witness round-trip checks in both directions.

Six concrete iso instances ride on top. One of them is a structural iso between `CausalMultiField` and its tuple carrier in `deep_causality_multivector`. Along the way, six pre-existing bugs blocking `no_std` were fixed in `deep_causality_num`.

## Smaller things worth knowing

- `deep_causality_effects` was retired; the `generalize-effects-over-realfield` spec was archived alongside it.
- CI: codecov, Miri, and OSV GitHub Actions were updated. Several test modules now skip cleanly under Miri (CDL SURD, transcendental floats, `rusty-fork` modules, stat and float doctests).
- `deep_causality_core` pinned Functor/Monad consistency between propagating-effect witnesses.
- `deep_causality_physics` flattened its unit-type folder and consolidated examples into a dedicated examples folder.

## Upgrading

If you only depend on the API-compatible crates (`deep_causality`, `deep_causality_core`, `deep_causality_num`, `deep_causality_uncertain`, `deep_causality_tensor`, `deep_causality_algorithms`, `deep_causality_discovery`, `deep_causality_ethos`, `deep_causality_macros`, `deep_causality_sparse`, `deep_causality_rand`, `deep_causality_haft`), this is a drop-in upgrade.

For the four breaking crates, the migration story is:

- **physics, multivector, topology**: thread an `R: RealField` parameter through your code, or pin it to `f64` at the boundary. Drop any `<f64>` turbofish; it is no longer accepted where the kernel is now generic.
- **topology**: replace `Lattice`, `DualLattice`, and `CWComplex` usage with the new `LatticeComplex` API.
- **ultragraph**: if you implement `StructuralGraphAlgorithms` yourself, add the three new methods. If you only call it, no action needed.

Full per-crate detail is in each crate's `CHANGELOG.md`.
