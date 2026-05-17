---
title: Lessons Learned from Rewriting UltraGraph
description: This post summarizes the lessons learned from rewriting UltraGraph v.0.8
date: 2025-08-15
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

When the decision was made to rewrite UltraGraph from scratch for the DeepCausality project, the goal was clear: create
a high-performance, memory-efficient, and adaptable graph data structure that is tailor-made for DeepCausality. One of
the key requirements was the support for both fast mutation as required by the graph-based context and fast graph
traversal as required by the causal reasoning engine. Seasoned engineers will spot immediately the conundrum to solve
because data structures that are fast to mutate are fundamentally different from data structures that are fast to
analyze. This subsequent quest resulted in deep insights into the trade-offs between flexibility and performance. Here
are the key lessons learned along the way.

## Lesson 0: There is excellent research on graph structures

As much as people love to joke about an AI slob, one useful application of LLMs is literature recommendation. Google
Gemini suggested the [NWHy paper](https://ieeexplore.ieee.org/document/9835472) as a starting point and that has proven
to be an invaluable foundation for the new UltraGraph. In contrast to many publications, the NWHy paper was written by
high-performance experts at the Pacific Northwest National Laboratory (PNNL) and it's immediately obvious that deep
experience was applied. However, while there is
a [reference implementation in C++](https://github.com/pnnl/NWHypergraph), a common choice for high-performance data
structures, a Rust implementation was nowhere to be found. That said, a good paper with reference code is always a
better starting point than a paper without code.

**Takeaway:** When there is excellent precedent, use it.

## Lesson 1: The "Two-State" Architecture shifts the mutability trade-off

The NWHy architecture is nothing short of an engineering masterpiece, but as stated previously, there is a trade-off
between designing for mutability and for analytics. Intuitively, this makes sense because without mutability, designing
lock-free analytics becomes a lot easier. Conversely, without analytics, locking for mutability becomes easier since
there is no consideration for performance any longer. Then the central question became: Can you have both without the
trade-off?

The most [fundamental design decision in UltraGraph 0.8 ](https://www.deepcausality.com/blog/announcement-ultragraph-0-8)was to reject a one-size-fits-all approach. Instead, we embraced
a two-state architecture, recognizing that graph construction and graph analysis are two fundamentally different
problems with different performance requirements that lead to two different internal data structures:

1. The `DynamicGraph` (The "Evolve" State): Optimized for flexibility. Built on a standard adjacency list `Vec<
   Vec<...>>`, it provides constant time O(1) additions of nodes and edges. This is the state where the graph is built
   and modified.

2. The `CsmGraph` (The "Analyze" State): Optimized for raw speed. This is an immutable, "frozen" representation that
   uses a custom Compressed Sparse Row (CSR) format. This layout is incredibly CPU-cache-friendly, making traversals and
   algorithms orders of magnitude faster. Because of the immutability, zero locking is needed for any graph algorithm.

The magic happens with the `.freeze()` and `.unfreeze()` methods, which transition the graph between these two states.
This "stop-the-world" approach allows us to have the best of both worlds: a flexible graph when we need it and a
blazing-fast one for analysis. Memory, though, is the bigger trade-off because during
the state conversion, the memory usage temporarily nearly doubles. That said, memory has become substantially cheaper
over the last decade meaning by investing in cheap memory, one can replace one or two compute instances due to faster graph
analytics. For graphs that change rarely, UltraGraph is a near perfect analytics structure.

**Takeaway:** The overhead of a controlled state transition between two specialized data structures can be far
outweighed by
the performance gains and, as a byproduct, lower TCO.

## Lesson 2: Performance by Design

Designing for performance comes down to memory layout and CPU caches:

* Struct of Arrays (SoA) over Array of Structs (AoS): Instead of storing edges as `Vec<(target, weight)>`, the custom
  CSR format uses two parallel vectors: `Vec<target>` and `Vec<weight>`. For algorithms that only care about the graph's
  topology (like cycle detection), the CPU only loads the targets vector, thus halving the memory bandwidth and reducing
  CPU cache pollution.

* Adaptive Algorithms: The contains_edge method is a prime example. It checks the node's degree at runtime. For
  low-degree nodes, it uses a cache-friendly linear scan. For high-degree "super-nodes," it switches to a faster binary
  search. This adaptive strategy ensures optimal performance regardless of the graph's degree of connectedness.

* Zero-Cost Abstractions: Rust's impl Trait feature was critical. Our traversal methods (`outbound_edges`,
  `inbound_edges`) return an `impl Iterator<Item = usize>` instead of a `Vec<usize>`. This allows us to return a direct,
  read-only " window" into the CSR memory without heap allocations. For algorithms that traverse millions or billions 
  of edges, this eliminates countless unnecessary allocations, providing a massive speed boost.

  **Takeaway:** Mechanical sympathy remains paramount for true performance. Think about data locality, cache lines,
  memory bandwidth, and avoid allocations in hot loops when possible.

## Lesson 3: A Detailed Specification is Essential

Before writing a single line of code, start with a proper specification and edit it until it feels right. The UltraGraph
0.8 specification,
in [docs/specfile.md](https://github.com/deepcausality-rs/deep_causality/blob/main/ultragraph/docs/specfile.md), served
as a comprehensive Software Requirements Specification (SRS). This document became the architectural blueprint that
detailed:

* The exact trait definitions for the public API (GraphView, GraphMut, GraphAlgorithms).
* The internal data structures for both DynamicGraph and CsmGraph.
* The precise algorithms for core operations, including the optimized, single-pass freeze algorithm.
* The design of a lightweight, Copy-able GraphError enum for panic-free, zero-allocation error handling.

The SRS forced us to think through the hard problems first, ensuring that the final implementation was consistent,
robust, and aligned with our performance goals. It is well known that changes become increasingly more complex and
expensive to fix as a project progresses, with the understanding that editing an SRS document is among the cheapest and fastest 
change one can do. However, many projects don’t write SRS documents because, conventionally, writing a correct and detailed
SRS could become a sub-project in itself and there is usually no time for that. However, enlisting AI tools has cut down 
the time substantially, helped to check the SRS for consistency, and was of great value in the design stage. The current UltraGraph API already transitioned to a modular trait system that was not documented in the initial SRS, 
but it doesn’t matter because the SRS served its purpose to prevent serious design flaws and, as a result,
reduced implementation time, increased code quality, and accelerated time to value.

**Takeaway:** For complex projects or data structures, a detailed design document is invaluable. It clarifies thinking,
aligns the team, and serves as a reference point throughout the development lifecycle. Using AI to draft, edit, check
and verify cuts down valuable time substantially so that a detailed SRS can be written in moderate time.

## Conclusion

Rewriting UltraGraph from the ground up for the 0.8 release using a structured, AI-augmented engineering was a journey
deep into the heart of Rust. The lessons learned about architecture, memory layout, API design, and the importance of a
good SRS have profoundly influenced the best practices of the DeepCausality project. For example, designing for
performance from the ground up remains an important consideration for all performance-critical parts of the project.
Then, lessons learned from using AI for drafting SRS were carried over to use AI for drafting, editing, and checking
feature requests and document pull requests. Next, using AI to find an excellent publication in the gigantic haystack
of academic publishing has been adopted for the more research-driven parts of the project. On the topic of AI usage, the
DeepCausality project like so many other open source projects, is notoriously short on maintainers, contributors, and
volunteers, thus effective AI usage has become a necessity to ensure the continuation of the project. One
positive side effect of AI usage is that more brain power is used to write better code and that is a good thing.

## Get Started

Get Started with DeepCausality. The Future is Now!

* Explore the [code examples on GitHub](https://github.com/deepcausality-rs/deep_causality/tree/main/examples).
* Join the [community](https://www.deepcausality.com/community/).
* Join the [Discord Server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware causal reasoning in Rust. Please give us
a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of
the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
