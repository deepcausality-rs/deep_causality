---
title: "DeepCausality v0.11: Zero External Dependencies"
description: "This post summarizes the new zero external dependency architecture of DeepCausality v0.11."
date: 2025-09-21
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

The DeepCausality project is proud to announce the release of DeepCausality 0.11, an update that removes all external dependencies from the default build. This release enhances security, portability, and compilation speed, delivering a self-contained and robust library for computational causality.


## 🚀 Highlights in 0.11

*   **Zero External Dependencies:** The core library has no external dependencies by default.
*   **100% Safe Rust:** The entire codebase contains zero `unsafe` code blocks.
*   **Zero Macros:** No procedural macros are used in the core library (except for testing).
*   **New Internal Crates:** `deep_causality_rand`, `deep_causality_num`, and `deep_causality_tensor`.
*   **Blazing-Fast Compilation:** Full mono-repo builds complete in seconds.

## 💡 Zero Dependencies by Default

The entire DeepCausality codebase, as it is now, compiles by default without any external dependencies.

The only exception is an optional feature flag, `os-random`, in the `deep_causality_rand` crate. When enabled, this flag introduces a dependency on a thin Rust wrapper for `libc` to access the operating system's secure random number generator. This is clearly documented and is strictly opt-in for users who require cryptographically secure random numbers.

Standard dev-dependencies like Criterion for benchmarks and Forky for test isolation are still used for development, but they are not part of the production build. The main takeaway is that the core library is completely self-contained.

## ✨ New Internal Crates

To achieve this, we replaced previous external dependencies with three new, lightweight internal crates:

*   **`deep_causality_rand`**: Adds functionality for random number generation.
*   **`deep_causality_num`**: Provides essential numerical utilities for the library.
*   **`deep_causality_tensor`**: Implements a custom, lightweight tensor type used throughout the `deep_causality_algorithms` crate.

These internal crates are tailored specifically for the needs of DeepCausality, ensuring they are lean, efficient, and perfectly integrated.

## ⚡ The Benefits of Zero Dependencies

This architectural shift provides several powerful advantages:

*   **Greatly Improved Supply Chain Security:** In an era where supply chain attacks are a major concern (as highlighted by past `npm`  supply chain security breaches), having zero dependencies drastically reduces the attack surface. You can trust that the code you're compiling is only the code we've written and audited.
*   **Effortless Cross-Compilation & Static Linking:** With no external libraries to manage, cross-compiling and even static linking DeepCausality for different targets becomes trivial.
*   **Blazing-Fast Compilation:** The impact on build times is dramatic. A clean build of the entire mono-repo now completes in roughly **4-5 seconds** on a modern machine. Single-crate builds are even faster. This accelerates development cycles and CI/CD pipelines significantly.

## Conclusion

DeepCausality 0.11 represents a major step towards a truly independent, secure, and portable computational causality library. By achieving zero external dependencies, we provide a foundation that is robust, reliable, secure, and fast to build.

Get Started with DeepCausality 0.11. The Future is Now!

*   Explore the [code and examples on GitHub](https://github.com/deepcausality-rs/deep_causality).
*   Join the [community](https://www.deepcausality.com/community/).
*   Join the [Discord Server](https://discord.gg/Bxj9P7JXSj)

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware causal reasoning in Rust. Please give us
a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of
the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).