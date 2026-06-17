/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared parallelism primitives for the DeepCausality workspace.
//!
//! Today this crate carries exactly one item: the [`MaybeParallel`]
//! feature-conditional thread-safety marker that the `parallel` features
//! of `deep_causality_topology`, `deep_causality_fft`, and their
//! consumers share. Hosting it in one Tier-0 crate guarantees a single
//! definition — Cargo feature unification on `deep_causality_par/parallel`
//! keeps every crate in a build agreeing on whether the bound means
//! `Send + Sync` or nothing.
//!
//! Forward-looking: this is also the designated home for a minimal
//! in-house replacement of the Rayon surface the workspace actually uses
//! (scoped fork-join over slices), should the external dependency ever
//! need to go.

pub mod traits;

pub use crate::traits::maybe_parallel::MaybeParallel;
