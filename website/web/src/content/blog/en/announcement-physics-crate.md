---
title: "Announcing DeepCausality Physics: A Library for Scientific Computing in Rust"
description: "Introducing deep_causality_physics: a modular, type-safe, and composable physics library powered by Uniform Mathematics."
date: 2025-12-12
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

We are thrilled to announce the release of `deep_causality_physics`, a new crate designed to accelerate complex physics simulations, advanced engineering, and scientific computing in Rust.

While Rust has an exploding ecosystem for linear algebra (`ndarray`, `nalgebra`) and game physics (`rapier`), there has been a gap in high-fidelity **theoretical physics**. We built `deep_causality_physics` to fill this gap, providing a comprehensive collection of kernels spanning from Quantum Mechanics to General Relativity.

## 🏗️ The Foundation: Uniform Mathematics

Scientific computing is often fragmented. You use one library for Tensors, another for Geometric Algebra, and a third for Topology. Connecting them requires writing brittle "glue code" that swallows errors and hides physical meaning.

This crate is built on the **[Uniform Mathematical Foundation](/blog/announcement-uniform-math/)** introduced in DeepCausality. Powered by our **[Higher-Order Abstract Functional Traits (HAFT)](/blog/announcement-haft-hkt/)**, it treats Tensors, MultiVectors, and Topological Manifolds as composable, monadic structures.

This means you can take a **Tensor** representing a gravitational field, map it into a **MultiVector** to compute electromagnetic interactions, and project the result onto a **Topological Manifold** all within a single, type-safe, and mathematically rigorous flow.

## 📦 Modular Architecture

The library is organized into modular domains, each providing highly optimized computation kernels:

*   **🌌 Astro**: Schwarzschild radius, orbital mechanics, Hubble's law.
*   **⚛️ Quantum**: Wavefunctions, operators, gates, fidelity, and Haruna's Gauge Field gates.
*   **🕰️ Relativity**: Einstein tensors, geodesic deviation, spacetime intervals.
*   **⚡ Electromagnetism**: Maxwell's equations, Lorentz force, Poynting vectors using Geometric Algebra.
*   **🔥 Thermodynamics**: Heat diffusion, entropy, partition functions.
*   **🧱 Materials**: Stress/Strain tensors, Hooke's Law, Von Mises stress.
*   **📏 Units**: A rigorous type-safe unit system to prevent dimensional errors.

## ⚡ Dual Usage: Standalone or Causal

We designed this crate to be versatile. You can use it in two distinct ways:

### 1. Standalone (Pure Rust)
If you just need to calculate a value, use the **Kernels**. These are pure, stateless functions that take primitive types or tensors and return a `Result`.

```rust
use deep_causality_physics::astro::mechanics::schwarzschild_radius_kernel;
use deep_causality_physics::Mass;

let mass = Mass::new(1.989e30).unwrap(); // Solar mass
let radius = schwarzschild_radius_kernel(&mass).unwrap();
println!("Schwarzschild Radius: {} m", radius.value());
```

### 2. Causal (Monadic Simulation)
If you are building a complex simulation with state, logging, and error propagation, use the **Wrappers**. These lift the kernels into the **Causal Monad** (`PropagatingEffect`), allowing you to chain physics operations into a pipeline.

```rust
use deep_causality_physics::schwarzschild_radius;

// Returns a PropagatingEffect that carries audit logs and error context
let effect = schwarzschild_radius(&mass)
    .bind(|r, _, _| {
        // ... chain next step ...
    });
```

## 🧪 Case Study: Multi-Physics Pipeline

To demonstrate the power of this approach, we included a **[Multi-Physics Pipeline example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples/examples/multi_physics_pipeline)** in the repository.

This example simulates the decay of a Higgs-like particle, requiring **zero-copy data transformation** across three distinct branches of physics:

1.  **Quantum Field Theory**: It starts with a scalar field $\phi$ evolving on a Topological Manifold via the **Klein-Gordon equation**.
2.  **Particle Physics**: As energy density spikes, it triggers **Hadronization**, converting field energy into discrete momentum vectors (Jets).
3.  **Thermodynamics**: The particle jets thermalize, and we solve the **Heat Diffusion** equation to model the expansion of the resulting quark-gluon plasma.
4.  **Quantum Measurement**: Finally, we calculate the **Born Probability** of detecting specific final states.

```rust
// A simplified view of the pipeline
let result = klein_gordon(&phi_manifold, mass)
    .bind_or_error(
        |evolved_tensor, _, _| hadronization(&evolved_tensor, ...),
        "Hadronization failed"
    )
    .bind_or_error(
        |jets, _, _| heat_diffusion(&jets, ...),
        "Thermalization failed"
    )
    .bind_or_error(
        |final_temp, _, _| born_probability(&final_temp, ...),
        "Detection failed"
    );
```

In a traditional setup, passing data from a QFT solver to a Fluid Dynamics solver would involve writing files to disk or complex memory mapping. Here, because Tensors and Manifolds share the same **HKT** foundation, the data flows seamlessly through the pipeline.

## 🤝 Call for Collaboration

We invite **physicists, engineers, and scientific computing experts** to collaborate with us. Whether you are working on Lattice QCD, Orbital Mechanics, or Condensed Matter, your expertise can help expand this library into a universal tool for the scientific community.

*   **[Explore the Code](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_physics)**
*   **[Run the Examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples)**
*   **[Join the Discussion](https://deepcausality.com/community/)**

Let's build the future of scientific simulation together.

## About

[DeepCausality](https://deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware causal reasoning in Rust. Please give us
a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of
the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
