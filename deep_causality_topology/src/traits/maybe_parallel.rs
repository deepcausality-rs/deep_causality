/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Feature-conditional thread-safety alias for the parallel DEC operators.
//!
//! The per-cell loops of the lattice DEC operators (wedge, interior
//! product, de Rham, sharp) fan out over Rayon when the crate's
//! `parallel` feature is enabled, which requires the scalar to cross
//! threads. This alias trait expresses that requirement without forcing
//! `Send + Sync` on serial builds:
//!
//! - **`parallel` on**: `MaybeParallel ≡ Send + Sync` (blanket-implemented
//!   for every such type — all workspace scalars qualify).
//! - **`parallel` off**: `MaybeParallel` is empty and blanket-implemented
//!   for every type, so the bound is vacuous and the public API is
//!   unchanged.
//!
//! Bound additions of the form `R: … + MaybeParallel` on operator impl
//! blocks are therefore invisible to serial consumers and exactly the
//! Rayon requirement for parallel ones.

/// Thread-safety requirement of the parallel DEC operator loops; vacuous
/// without the `parallel` feature. See the module doc.
#[cfg(feature = "parallel")]
pub trait MaybeParallel: Send + Sync {}

#[cfg(feature = "parallel")]
impl<T: Send + Sync + ?Sized> MaybeParallel for T {}

/// Thread-safety requirement of the parallel DEC operator loops; vacuous
/// without the `parallel` feature. See the module doc.
#[cfg(not(feature = "parallel"))]
pub trait MaybeParallel {}

#[cfg(not(feature = "parallel"))]
impl<T: ?Sized> MaybeParallel for T {}
