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
    orcid: 0009-0000-1159-8173
    corresponding: true
    affiliation: 1
affiliations:
  - name: Center for Dynamic Causality
    index: 1
date: 14 May 2026
bibliography: paper.bib
---

# Summary

DeepCausality is a hypergeometric computational causality library written in Rust, designed to build systems that
reason about cause and effect. Hosted by the Linux Foundation for Data & AI since 2023, it supports uniform reasoning
across deterministic and probabilistic modalities, including static and dynamic contextual causal models. At its core,
the library implements the paradigm of "Causality as a spacetime-agnostic functional dependency", inspired by the
causaloid formalism of @hardy2005causaloid, using **Causaloids** (self-contained causal units) and **Contexts**
(hypergraph environments). It connects abstract causal reasoning, physical laws, and programmable ethics in a single
toolset for researchers and engineers building complex, verifiable autonomous systems.

# Statement of need

Causal inference toolkits are usually specialized along one axis at a time: a library handles Markovian dynamics *or*
non-Markovian memory, deterministic structural models *or* probabilistic graphical models, static causal graphs *or*
time-varying ones, offline discovery from datasets *or* online reasoning inside a running system. Combining these
regimes in a single model—say, a deterministic physical law nested inside a probabilistic Markov layer that itself sits
inside a non-Markovian, context-dependent envelope, evaluated at runtime under uncertainty—typically requires gluing
several disparate packages together, with no shared type system and no shared semantics for composition.

DeepCausality is, to the authors' knowledge, among the first packages to treat these regimes as orthogonal axes of a
single framework rather than as separate libraries. Within one type-safe Rust API it supports: (i) **deterministic and
probabilistic** modalities (the latter via `Uncertain<T>` first-class uncertainty propagation), (ii) **static and
dynamic** causal structure (Causaloids over fixed graphs or over time-evolving contexts), (iii) **Markovian and
non-Markovian** processes (the Causal Monad threads explicit state, context, and history through `bind`, so memory
effects are expressible without leaving the abstraction), and (iv) **offline discovery and online reasoning** (a
Causal Discovery Language for fitting models from data and a `PropagatingEffect` runtime for executing them inside
deployed software). Because the Causaloid is recursive and isomorphic, these regimes can be mixed at arbitrary
granularity inside a single model.

Real systems rarely sit in only one regime. A flight-envelope monitor mixes deterministic aerodynamics, probabilistic
sensor models, non-Markovian fatigue history, and dynamic context. A tumor-treatment model mixes deterministic
pharmacokinetics, probabilistic response, and non-Markovian patient history. DeepCausality provides a single,
high-performance, type-safe framework in which such models can be expressed, composed, intervened on, and executed in
real time—rather than forcing the practitioner to re-implement bridging code between several incompatible toolkits.

# Software design

DeepCausality is structured as a monorepo containing 20 crates, organized into five layers (Figure 1).

```
┌─────────────────────────────────────────────────┐
│  Causal Discovery   deep_causality_discovery    │
├─────────────────────────────────────────────────┤
│  Causal Framework   deep_causality_core         │
│                     deep_causality              │
│                     deep_causality_ethos        │
├─────────────────────────────────────────────────┤
│  Physics            deep_causality_physics      │
├─────────────────────────────────────────────────┤
│  Mathematics        deep_causality_tensor       │
│                     deep_causality_multivector  │
│                     deep_causality_topology     │
├─────────────────────────────────────────────────┤
│  Foundation         deep_causality_num          │
│                     deep_causality_haft         │
│                     deep_causality_metric  ─── shared sign conventions
└─────────────────────────────────────────────────┘
```
*Figure 1: Layered architecture of DeepCausality. Upper layers depend on lower layers; the foundation layer
(`deep_causality_metric`) supplies shared sign conventions used uniformly across mathematics, physics, and the causal
framework.*

**Foundational layer.** *HAFT* implements Higher-Kinded Types (HKT) in Rust via a witness pattern, providing Functor,
Applicative, and Monad traits across container types. *NUM* defines an algebraic hierarchy (from Magma to Division
Algebras) and specialized numeric types including `DoubleFloat` (double-double precision), `Complex`, `Quaternion`,
and `Octonion`. *Rand* provides random number generation and statistical distributions for stochastic simulations.

**Dynamic causality.** The main `deep_causality` crate models causality via Causaloids that compose into causal
graphs. The `deep_causality_core` crate defines the **Causal Monad** (`CausalMonad`), realized as `PropagatingEffect`
(stateless) and `PropagatingProcess` (stateful) types that thread state, context, error, and audit logs through
monadic `bind`. It also provides a `ControlFlowBuilder` for constructing correct-by-construction static execution
graphs, with optional zero-allocation, ZST-only enforcement via the `strict-zst` feature, suitable for safety-critical,
hard real-time, and `no_std` environments. *Discovery* provides a type-safe builder pipeline for the Causal Discovery
Language (CDL).

**Physics & metrics.** *Physics* is a standard library of physics kernels and causal wrappers organized by domain—
Astrophysics, Quantum Mechanics, Electromagnetism, Relativity, Thermodynamics—leveraging Geometric Algebra and Gauge
Fields [@tong2018gauge; @furey2024algebraic] and drawing on recent results in quantum geometry [@kang2024qgt;
@haruna2025logical]. *Metric* defines foundational signatures for consistent handling of geometric properties.

**Data structures.** *Topology* implements `Graph` (sparse-matrix based), `Hypergraph`, `SimplicialComplex`, and
`Manifold`, enabling TDA algorithms such as Vietoris–Rips triangulation. *Tensor* and *Sparse* provide N-dimensional
`CausalTensor` support with Einstein summation and CSR matrices. *MultiVector* implements Clifford Algebra for
relativistic geometry. *Ultragraph* provides a high-performance hypergraph backend [@liu2022nwhy].

**Research & applications.** *Algorithms* implements **SURD** (Synergistic, Unique, and Redundant decomposition)
[@martinez2025surd] and **mRMR** feature selection [@zhao2019mrmr]. *Ethos* introduces a programmable deontic logic
layer (`Teloid`) [@olson2024ddic] for verifying proposed actions against operational objectives. *Uncertain*
provides `Uncertain<T>` [@bornholt2014uncertain], representing values as probability distributions and propagating
uncertainty through computations.

## Causaloid and Causal Monad: structure and sequencing on equal footing

Most existing causal frameworks force a choice between two incompatible shapes. Either the model is a **structural
artifact**—a DAG, a structural causal model, a Bayesian network—which captures rich causal topology but offers no
native story for functional sequencing or side-effect discipline; or the model is a **sequenced pipeline**—a chain, a
stream, a monadic computation—which composes cleanly but flattens causal structure into a linear or tree-shaped flow.
Practitioners typically pick one and pay for the other in glue code.

DeepCausality refuses that compromise. The **Causaloid** is the isomorphic, recursive structural primitive: it can be
an atomic causal unit, a causal collection, or a causal graph, and because a Causaloid-of-Causaloids is itself a
Causaloid, arbitrarily complex causal relations—nested graphs, hierarchies, hypergraphs of sub-models—can be expressed
without leaving the type. The **Causal Monad** is the sequencing primitive: it enforces strict functional composition,
ordered evaluation, state and context propagation, error handling, and audit logging via monadic `bind`. Neither is
built on top of the other; they are co-equal abstractions targeting orthogonal concerns.

The two meet through `PropagatingEffect` (Figure 2).

```
        Causaloid                              Causal Monad
   (structural primitive)                  (sequencing primitive)
   ─────────────────────                   ─────────────────────
   • atomic | collection | graph           • PropagatingEffect
   • recursive: Causaloid-of-Causaloids    • PropagatingProcess
   • isomorphic nesting                    • bind / state / context
   • arbitrary causal topology             • error / audit logging
            │                                       │
            │     evaluates to / consumed by        │
            └──────────────┬────────────────────────┘
                           ▼
                   PropagatingEffect
                  (shared value type)
                           │
                           ▼
        freely interleavable: DAG inside pipeline,
        pipeline inside DAG, at any granularity
```
*Figure 2: Causaloid and Causal Monad are co-equal primitives targeting orthogonal concerns (structure vs.
sequencing). They meet through `PropagatingEffect`, the shared value type produced by Causaloid evaluation and consumed
by monadic `bind`, which makes structural and sequenced models freely interleavable.*
 A Causaloid's evaluation produces a `PropagatingEffect`, which is also the
value carried by the Causal Monad's `bind`. As a result, structure and sequencing become freely composable: an
arbitrarily complex causal graph can be lifted as a single monadic step, a monadic pipeline can sit inside a Causaloid,
and the two can be interleaved at any granularity. A user can build a pure DAG of Causaloids, a pure monadic pipeline,
or—most usefully—a mixed model in which functionally sequenced stages each contain rich causal sub-structure. This is
the property that makes the multi-physics pipelines in `examples/physics_examples` (e.g., `gauge_gr`,
`quantum_geometric_tensor`, `gravitational_wave`, `multi_physics_pipeline`) compose end-to-end: each stage is a
sequenced Causal Monad step, and each step internally evaluates a Causaloid that may itself contain a graph of further
Causaloids.

## Algebraic and type-level integration

Two additional mechanisms keep the ecosystem coherent. First, core data structures (`CausalTensor`, `CausalMultiVector`,
`Manifold`) are generic over numeric types implementing `Field` or `RealField` traits from `deep_causality_num`, which
decouples algorithms from numeric representation and lets the 106-bit `DoubleFloat` type drop into all tensor and
topological operations without code changes. Second, the witness-pattern HKT in `deep_causality_haft` makes
heterogeneous types (a `Manifold`, a `CausalTensor`, a `CausalMultiVector`) implement the same `Functor`/`Monad` traits,
so they interoperate within one monadic flow—the basis on which the physics crate is built.

# Research impact statement

DeepCausality has been hosted by the Linux Foundation for Data & AI since 2023, with public development history
spanning roughly three years from the initial commit on 2023-06-18 to the present, accumulated through sustained,
iterative work rather than a single bulk drop. The library supports work in: **causal AI safety** (the `Ethos` layer
enables formal verification of agent actions against safety protocols); **algorithm development** (open-source SURD
and mRMR implementations for decomposing causal signals in high-dimensional data); **geometric deep learning**
(Topology and MultiVector algebra as geometric priors); and **physics-informed causal modeling** (digital twins that
are both causally sound and physically valid).

The repository ships example crates covering classical causality, contextual structural models, chronometric
reasoning, avionics (`flight_envelope_monitor`, `geometric_tcas`, `magnav`), medicine (`aneurysm_risk`, `epilepsy`,
`protein_folding`, `tumor_treatment`), material science, and physics (`gauge_gr`, `quantum_geometric_tensor`,
`gravitational_wave`, `multi_physics_pipeline`).

**Adoption and downstream use.** DeepCausality is used in production by the ServiceRadar observability platform
(<https://github.com/carverauto/serviceradar>), which describes its causal engine as providing "real-time triage and
isolation via DeepCausality" over heterogeneous telemetry. It is also the
core dependency of the research program at the Center for Dynamic Causality, which has produced multiple preprints
that build on DeepCausality primitives (Causaloid, Causal Monad, `PropagatingEffect`, the physics and topology
crates). A running list of project preprints, publications, talks, and technical write-ups is maintained at
<https://www.causalcenter.com/publications/>.

## Open development practices

The project is developed openly on GitHub under the MIT license. All 20 crates are published on crates.io with
documentation on docs.rs; releases are tagged and archived on Zenodo for DOI-based citation. The public issue tracker
and pull request workflow are open to external contributors, and a Developer Certificate of Origin (DCO) governs code
contributions. Continuous integration runs the full test matrix on every push (build, test, lint via `make build`,
`make test`, `make fix`), and each crate carries its own README, CHANGELOG, and SBOM artifact. Governance is documented
in `MAINTAINERS.md`, `CODEOWNERS`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `SECURITY.md` at the repository root.

# AI usage disclosure

The DeepCausality project uses generative AI for software specification, code review, and testing and QA assistance.
Tools used include Google Gemini Pro 3.1, Anthropic Claude Opus 4.6, and Claude Opus 4.7. All code design and
architectural decisions were made by the human author, and all AI-assisted outputs were reviewed, edited, and validated
by the human author before being incorporated into the codebase, documentation, or this manuscript.

# Conflict of interest

The author declares no conflict of interest.

# Funding

This project received no external funding.

# Acknowledgements

We acknowledge the Linux Foundation for Data & AI for their stewardship of the project.

# References
