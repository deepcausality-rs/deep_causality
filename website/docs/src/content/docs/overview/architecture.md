---
title: Architecture
description: 'A map of the DeepCausality workspace: 29 library crates, the path a reasoning run takes through them, and where the boundaries fall.'
sidebar:
  order: 5
---

DeepCausality ships as one Cargo workspace of 45 members: 29 library crates and 16 example packages. This page maps that workspace.

**[Open the interactive architecture map](/architecture.html)**. It pans, zooms, traces any single relationship, and steps through four guided views. Every node on it carries a source reference that was checked against the repository before the page was written. The rest of this page reads the same map in prose.

## The primary path

A reasoning run descends five crates.

The model author builds a `Causaloid`, and, when the rule needs an environment, a `Context`. A `Causaloid` is isomorphic across three shapes: Singleton, Collection, and Graph. `CausaloidGraph::freeze()` locks the graph structure. `evaluate_subgraph_from_cause` then walks it as a Kahn-ordered ready set with a 1024-round relay budget, short-circuiting on the first error.

Every node hands back the same carrier: `CausalEffectPropagationProcess<Value, State, Context, Error, Log>`. It has one outcome channel holding a value or an error, never both, plus state, context, and an audit log. `PropagatingEffect<T>` is the stateless alias of that carrier; `PropagatingProcess<T, S, C>` is the alias that carries real state and context. Neither is an enum.

Underneath, the descent is mechanical. `deep_causality` calls `deep_causality_core` for the carrier. The carrier is built on `deep_causality_haft`, which supplies `bind` and the free monad. `haft` is bounded by `deep_causality_algebra`, and the algebraic tower bottoms out in `deep_causality_num`.

Two crates hang off `deep_causality` itself. `ultragraph` stores both the `CausaloidGraph` and the `Context` as a dual-state directed graph that freezes from adjacency lists into a CSR layout. `deep_causality_data_structures` supplies the sliding windows and grid arrays a context feeds on.

When a `CausalState` evaluates active, its `CausalAction` fires. The Effect Ethos decides whether that action is permissible under an immutable ethos, but nothing invokes it for you: `deep_causality_ethos` depends on `deep_causality`, not the reverse, so the deontic verdict is a call the operator wires in alongside the CSM.

## Crate boundaries

Three directories hold the 29 library crates. Package names stay flat, so every `use` statement, every `cargo -p <name>`, and every crates.io entry reads the same as before the crates moved.

**Repository root, ten crates.** `deep_causality`, `deep_causality_core`, `deep_causality_ethos`, `deep_causality_algorithms`, `deep_causality_cfd`, `deep_causality_data_structures`, `deep_causality_discovery`, `deep_causality_physics`, `deep_causality_quantum`, and `ultragraph`.

**`deep_causality_unified_math/`, sixteen crates.** `algebra`, `calculus`, `fft`, `haft`, `homology`, `linear`, `metric`, `multivector`, `num`, `num_complex`, `num_dual`, `num_rational`, `rand`, `tensor`, `topology`, and `uncertain`, each prefixed `deep_causality_`. They form a strict seven-tier DAG of their own. Only two dependencies leave the folder: `deep_causality_ast` for `tensor` and `uncertain`, and `deep_causality_par` for `fft` and `topology`.

**`deep_causality_utils/`, three crates.** `deep_causality_ast` holds one type, the persistent copy-on-write `ConstTree`. `deep_causality_file` expresses every loader as a lazy `IoAction`, including the RINEX SP3 and CLK readers. `deep_causality_par` exports the `MaybeParallel` marker and `scoped_map`.

The nesting changed Cargo paths and Bazel labels, nothing else. A path dependency into the mathematics crates reads `../deep_causality_unified_math/deep_causality_x`; the matching Bazel label is `//deep_causality_unified_math/deep_causality_x`.

Four nodes on the map stand for more than one crate. Compute kernels covers `linear`, `tensor`, `fft`, and `calculus`. Numbers and uncertainty covers `num_complex`, `num_dual`, `num_rational`, `rand`, `metric`, and `uncertain`. Geometry and topology covers `topology`, `multivector`, and `homology`. Causal discovery covers `discovery` and `algorithms`.

## Internal dependencies

The workspace sorts into nine tiers, strictly acyclic. A crate depends only on crates in lower tiers.

| Tier | Crates |
| --- | --- |
| 0 | `ast`, `data_structures`, `metric`, `num`, `par`, `ultragraph` |
| 1 | `algebra` |
| 2 | `haft`, `num_rational`, `rand` |
| 3 | `core`, `file`, `linear`, `num_complex`, `num_dual`, `uncertain` |
| 4 | `deep_causality`, `calculus`, `fft`, `homology`, `tensor` |
| 5 | `ethos`, `multivector` |
| 6 | `quantum`, `topology` |
| 7 | `algorithms`, `physics` |
| 8 | `cfd`, `discovery` |

`scripts/check_tiers.py` re-derives this table from `cargo metadata` and diffs it against the copies in `AGENTS.md` and `deep_causality_unified_math/README.md`. It fails in both directions, so a new crate or a new edge surfaces as a failed check rather than as documentation drift.

The map draws the edges that carry architectural meaning and leaves the rest to this table. Off the canvas: `deep_causality` also depends on `algebra`, `ast`, and `uncertain`; `cfd` on `core`, `calculus`, `fft`, `file`, `tensor`, and `topology`; `physics` and `quantum` both on `core`.

## External dependencies

Seven of the 29 library crates reach crates.io at runtime. The other 22 reach none at all.

| Crate | Dependency | Status |
| --- | --- | --- |
| `deep_causality_discovery` | `csv`, `parquet` | required |
| `deep_causality_file` | `chrono` | required |
| `deep_causality_num` | `libm` | optional, `libm_math` / `no-std` |
| `deep_causality_rand` | `getrandom` | optional, `os-random` |
| `deep_causality_algorithms` | `rayon` | optional, `parallel` |
| `deep_causality_fft` | `rayon` | optional, `parallel` |
| `deep_causality_topology` | `rayon` | optional, `parallel` |

Three required crates in the whole library surface: `csv`, `parquet`, and `chrono`. Everything else is feature-gated and off by default.

Test and benchmark code adds `criterion` to nine crates, `tempfile` to three, and `rusty-fork` to `deep_causality_uncertain`. The example packages add `candle-core` and `tokio`.

`unsafe_code = "forbid"` is a workspace lint, and all 45 members opt in.

## Regenerating the map

The diagram is generated from `scripts/workspace-architecture.json`, a typed specification of nodes, boundaries, relationships, and cards. Each node names the source files it stands for, and the renderer verifies every one of those paths against the repository before it writes the page. Edit the specification, re-render, and replace `public/architecture.html`.
