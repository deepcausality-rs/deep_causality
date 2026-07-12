/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lean↔Rust traceability witnesses.
//!
//! Every proved theorem under `lean/DeepCausalityFormal/Quantum/` carries a
//! `THEOREM_MAP: <id>` tag; the `theorem-map` job in
//! `.github/workflows/formalization.yml` requires each such id to appear in a
//! Rust file (the executable witness) and in `lean/THEOREM_MAP.md`. These
//! witnesses live here — one module per Lean source file — rather than being
//! scattered through the general operator/channel test suites.

#[cfg(test)]
mod choi_tests;
#[cfg(test)]
mod partial_trace_tests;
