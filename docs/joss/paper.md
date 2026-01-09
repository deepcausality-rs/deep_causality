---
title: 'DeepCausality: A Hypergeometric Computational Causality Library for Rust'
tags:
  - Rust
  - causality
  - hypergraph
  - causal-inference
  - physics
  - topology
authors:
  - name: Marvin Hansen
    orcid: 0000-0000-0000-0000 # Update ORCID
    equal-contrib: true
    affiliation: 1
affiliations:
  - name: DeepCausality Project, Linux Foundation for Data & AI
    index: 1
date: 09 January 2026
bibliography: paper.bib
---

# Summary

DeepCausality is a hypergeometric computational causality library written in Rust, designed to build systems that reason
about cause and effect. hosted by the Linux Foundation for Data & AI since 2023, it pioneers uniform reasoning across
deterministic and probabilistic modalities, supporting both static and dynamic contextual causal models. At its core,
the library implements the paradigm of "Causality as a spacetime-agnostic functional dependency" using **Causaloids** (
self-contained causal units) and **Contexts** (hypergraph environments). It bridges the gap between abstract causal
reasoning, physical laws, and programmable ethics, providing a comprehensive toolset for researchers and engineers
building complex, verifying autonomous systems.

# Statement of need

Causal inference has traditionally been the domain of statistical languages like R and Python, with a primary focus on
analyzing historical data to identify causal links (Pearl's Ladder of Causation). However, there is a growing need for
*computational* causality—systems that can reason about interventions and counterfactuals in real-time, within deployed
software.

DeepCausality fills this gap by providing a high-performance, type-safe framework in Rust. Unlike statistical packages
that primarily focus on *discovery* from datasets, DeepCausality focuses on *reasoning* and *execution*. It allows
developers to encode causal models directly into the system's logic, enabling autonomous agents to ask "What if?"
questions and verify the safety of their actions before execution. Furthermore, it integrates these causal models with
rigorous physics simulations and topological data analysis, making it uniquely suitable for applications in robotics,
aerospace, and complex system simulation where physical constraints and ethical guardrails are paramount.

# Software design

## Overview

DeepCausality is structured as a monorepo containing 20 crates, organized into five categories that layer functionality
from foundational mathematics to high-level research applications.

### 1. Foundational Layer

The foundation of the library rests on rigorous mathematical and functional primitives.

* **HAFT (Higher-Order Abstract Functional Traits)** implements Higher-Kinded Types (HKT) in Rust using a witness
  pattern, providing standard functional traits (Functor, Monad, Applicative) that allow for generic, abstract code
  across different container types.
* **NUM (Numerical Foundation)** defines a complete hierarchy of algebraic traits (from Magma to Division Algebras) and
  provides specialized numeric types such as `DoubleFloat` (double-double precision), `Complex`, `Quaternion`, and
  `Octonion`.
* **Rand** provides specialized random number generation and statistical distributions essential for stochastic
  simulations.

### 2. Dynamic Causality

This cluster encapsulates the core logic for causal reasoning and effect propagation.

* **DeepCausality (Main)** acts as the central integration point, modeling causality via **Causaloids** that compose
  into causal graphs.
* **Core** defines the **Causal Monad** pattern, offering `PropagatingEffect` (stateless) and `PropagatingProcess` (
  stateful) monads for composable effect propagation. It also includes a `ControlFlowBuilder` for constructing
  correct-by-construction, static execution graphs suitable for safety-critical and no-std environments.
* **Discovery (DSL)** provides a type-safe builder pipeline for the Causal Discovery Language (CDL), abstracting the
  complexity of data loading, feature selection, and algorithm execution.

### 3. Physics & Metrics

DeepCausality integrates abstract causal reasoning with physical verification.

* **Physics** constitutes a standard library of physics kernels and causal wrappers. It is organized by domain—spanning
  Astrophysics, Quantum Mechanics, Electromagnetism, Relativity, and Thermodynamics. Crucially, it leverages **Geometric
  Algebra** and **Gauge Fields** to model physical interactions coherently across theories.
* **Metric** defines foundational signatures to ensure consistent handling of geometric properties across the ecosystem.

### 4. Data Structures

Specialized data structures are implemented to optimize causal and geometric computation.

* **Topology** implements rigorous structures like `Graph` (sparse-matrix based), `Hypergraph`, `SimplicialComplex`, and
  `Manifold`, enabling Topological Data Analysis (TDA) algorithms such as Vietoris-Rips triangulation.
* **Tensor** and **Sparse** provide N-dimensional `CausalTensor` support with Einstein summation and Compressed Sparse
  Row matrices, respectively.
* **MultiVector** implements Clifford Algebra for relativistic geometry.
* **Ultragraph** provides a high-performance hypergraph backend for modeling complex connectivity.

### 5. Research & Applications

The top layer includes experimental components derived from active research.

* **Algorithms** implements advanced algorithms like **SURD** (Synergistic, Unique, and Redundant decomposition) and *
  *MRMR** feature selection.
* **Ethos** introduces a programmable deontic logic layer (`Teloid`), allowing systems to verify proposed actions
  against defined ethical or mission-critical objectives.
* **Uncertain** provides a first-order type, `Uncertain<T>`, which models values as probability distributions rather
  than scalar estimates, automatically propagating uncertainty through computations to prevent decision validity errors.

## Cross Crate Integration

The disparate components of DeepCausality are tightly coupled through three primary mechanisms that ensure type safety,
performance, and mathematical consistency across the entire ecosystem.

### 1. Shared Algebraic Structure

DeepCausality leverages a rigorous algebraic hierarchy defined in the `deep_causality_num` crate. Core data structures—
`CausalTensor` (mechanics), `CausalMultiVector` (geometric algebra), and `Manifold` (topology)—are generic over
underlying numeric types that implement the `Field` or `RealField` traits. This design decouples the algorithms from
specific numeric representations, effectively "paving the way" for advanced precision requirements. For instance, the
library seamlessly supports a 106-bit precision `DoubleFloat` type, enabling high-fidelity physics simulations that are
drop-in compatible with all tensor and topological operations.

### 2. Uniform Mathematical Foundation via HKT

The `deep_causality_haft` crate unifies the library's diverse domains through Higher-Kinded Types (HKT). By implementing
the "Witness Pattern," distinct types from different domains implement standard functional traits like `Functor` and
`Monad`. This enables a uniform mathetmatics where types interoperate within a single monadic flow:

* **Topology**: A `Manifold` provides the spatial context.
* **Mechanics**: Local data is extracted into a `CausalTensor` for numerical computation.
* **Algebra**: Tensors are lifted into `CausalMultiVector` format to apply geometric algebra manipulations (e.g.,
  rotations).

The unified math is a relatively new addition and not well documented yet. However, it serves as the foundation for the
physics crate.

### 3. Effect Propagation Process

The `deep_causality_core` crate provides the `PropagatingEffect` and `PropagatingProcess` monads which handle the
temporal aspect of causality—state transitions, error propagation,
and audit logging—without coupling the core logic to the specific data types being transformed. This separation of
concerns allows for the construction of complex, multi-stage causal models that are both robust and verifiable.

These mechanisms allow for the implementation of complex physical theories, such as General Relativity, in a concise,
type-safe, and highly performant manner. The `examples/physics_examples` directory demonstrates how these layers combine
to model sophisticated phenomena like spacetime curvature and quantum mechanical effects using a unified, high-level API
previously unattainable in systems programming languages.

# Research impact statement

DeepCausality enables a new class of research at the intersection of causal inference, functional programming, and
systems engineering. Hosted by the Linux Foundation since 2023, it has been instrumental in exploring:

1. **Causal AI Safety**: The `Ethos` layer allows for the formal verification of agent actions against safety protocols,
   a critical requirement for autonomous systems.
2. **Algorithm Development**: The implementation of the **SURD** algorithm provides researchers with open-source tools
   to decompose complex causal signals in high-dimensional data.
3. **Geometric Deep Learning**: By integrating `Topology` and `MultiVector` algebra, the library supports research into
   geometric priors for learning systems.
4. **Physics-Informed Causal Modeling**: The explicit linking of causal graphs with physical laws (via the `Physics`
   crate) facilitates the creation of "Digital Twins" that are both causally sound and physically valid.

# AI usage disclosure

The DeepCausality project embraces responsible artficial integlience (AI) usage in the following areas:

1) Software specification
2) Code review
3) Testing and QA

All code design and architectural decisions were made by human authors.

The project maintains a detailed document of its AI usage.

# Acknowledgements

We acknowledge the Linux Foundation for Data & AI for their stewardship of the project.

# References
