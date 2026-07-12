---
title: Topology
description: Machine-checked Riemann curvature symmetries (antisymmetry, first Bianchi, linearity), bound to Rust law-tests on the CurvatureTensor.
sidebar:
  order: 6
---

The curvature laws at the concrete `CurvatureTensor`, proved in [`lean/DeepCausalityFormal/Topology/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal/Topology). Reference: do Carmo, *Riemannian Geometry*, Ch. 4.

Rust witnesses live in `deep_causality_topology/tests/types/curvature_tensor/curvature_tensor_law_tests.rs`. Every row is `proved` in Lean and checked by a passing law-test. This layer has no per-row Rust-witness column: one test file carries all three.

| id | statement | Lean proof | Test |
|---|---|---|---|
| `topology.curvature.antisymmetry` | `R(u,v)w = −R(v,u)w` | `RiemannCurvature.lean` | ✓ |
| `topology.curvature.bianchi_first` | `R(u,v)w + R(v,w)u + R(w,u)v = 0` (needs `g` symmetric) | `RiemannCurvature.lean` | ✓ |
| `topology.curvature.linearity` | additivity + homogeneity in the transported slot | `RiemannCurvature.lean` | ✓ |
