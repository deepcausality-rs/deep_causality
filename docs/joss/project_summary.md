# DeepCausality Project Summary

DeepCausality is a hypergeometric computational causality library designed to build systems that reason about cause and effect. It pioneers uniform reasoning across deterministic and probabilistic modalities, supporting both static and dynamic contextual causal models. The project has been hosted by the Linux Foundation for Data & AI since 2023.

## 0. Foundational
This cluster provides the mathematical and functional primitives required for the higher-level causal abstractions.

*   **HAFT (Higher-Order Abstract Functional Traits)**: A library enabling Higher-Kinded Types (HKT) in Rust using a witness pattern. It provides functional standard traits like `Functor`, `Applicative`, `Monad`, `Foldable`, and `Traversable` to write generic, abstract code over varying container types.
*   **NUM (Numerical Foundation)**: A comprehensive numerical library providing rigorous algebraic traits (Magma to Division Algebra), safe cast traits, and specialized numeric types including `DoubleFloat` (double-double precision), `Complex`, `Quaternion`, and `Octonion`.
*   **Rand**: Specialized random number generation and statistical distributions used throughout the physics and simulation engines.

## 1. Dynamic Causality
This cluster contains the core logic for causal reasoning, effect propagation, and system modeling.

*   **DeepCausality (Main)**: The central library that integrates all components. It implements the "Causality as a spacetime-agnostic functional dependency" paradigm using Causaloids (self-contained causal units) and Context (hypergraph environment).
*   **Core**: Defines the fundamental `CausalMonad` pattern. It provides:
    *   `PropagatingEffect<T>`: A stateless monad for composable effect propagation.
    *   `PropagatingProcess<T>`: A stateful monad dealing with state, context, error, and logs.
    *   `ControlFlowBuilder`: A tool for building static, type-safe, correct-by-construction execution graphs for safety-critical systems.
*   **Effects**: Effect types for heterogeneous graphs and causal collections, managing the flow of data and state changes.
*   **Discovery (DSL)**: Provides a Causal Discovery Language (CDL) pipeline. It uses a type-safe builder pattern to guide users through data loading, feature selection, causal discovery algorithms, and analysis, abstracting the complexity of the underlying statistical mechanisms.

## 2. Physics / Metrics
This cluster bridges abstract causal reasoning with physical laws and measurements.

*   **Physics**: A standard library of physics kernels and causal wrappers organized by domain (Astro, Quantum, Electromagnetism, Relativity, Thermodynamics, etc.). It leverages Geometric Algebra and Causal Tensors to model physical interactions with high fidelity. It distinguishes between pure "kernels" for computation and "theories" for coherent frameworks (e.g., General Relativity using Gauge Fields).
*   **Metric**: Defines foundational metric signatures used across tensor, multi-vector, and physics calculations, ensuring consistent handling of geometric properties like distance and curvature.

## 3. Data Structure
This cluster provides the specialized data structures optimized for causal and geometric computation.

*   **Topology**: Implements rigorous topological data structures including `Graph` (sparse-matrix based), `Hypergraph`, `SimplicialComplex`, and `Manifold`. It supports Topological Data Analysis (TDA) algorithms like Vietoris-Rips triangulation and Euler characteristic computation.
*   **Tensor**: N-dimensional `CausalTensor` implementation supporting Einstein summation (`ein_sum`) and linear algebra operations.
*   **Sparse**: Compressed Sparse Row (CSR) matrix implementation for efficient storage and manipulation of sparse data, primarily used in graph adjacency matrices.
*   **MultiVector**: Implementation of Geometric Algebra (Clifford Algebra) via `CausalMultiVector`, supporting operations essential for physics and relativistic geometry.
*   **AST**: Generic Abstract Syntax Tree used for representational structures within the system.
*   **Ultragraph**: A high-performance hypergraph backend used for complex connectivity modeling.

## 4. Research
This cluster contains experimental, advanced, or theoretical components derived from active research.

*   **Discovery (Algorithms)**: Implements cutting-edge causal discovery algorithms like SURD (Synergistic, Unique, and Redundant decomposition of causal influences) and MRMR (Minimum Redundancy Maximum Relevance) feature selection.
*   **Ethos**: A programmable deontic logic layer. It allows defining "Effect Ethos" to programmatically verify whether proposed actions align with safety, ethical, or mission-critical objectives before execution (`Teloid`).
*   **Uncertain**: A first-order type for uncertain data (`Uncertain<T>`). It models values as probability distributions (not just scalar estimates) and automatically propagates uncertainty through arithmetic and logical operations, preventing "uncertainty bugs" in decision-making processes.
