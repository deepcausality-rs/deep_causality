/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Structure-preserving maps between algebraic structures, with named domain and codomain.
//!
//! The [`iso`](crate::iso) module models *isomorphisms* as a relation on a pair of types. That
//! shape cannot express a map that is not invertible, which is every canonical embedding of the
//! number tower: ℕ ↪ ℤ ↪ ℚ ↪ ℝ ↪ ℂ are injective and none is surjective.
//!
//! This module models the maps themselves.

pub mod compose;
#[allow(clippy::module_inception)]
pub mod hom;
pub mod iso_bridge;
pub mod properties;
pub mod ring_hom;
