/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the QCM freeze-check example.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!`, so the
//! compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};
use deep_causality_num_complex::Complex;

/// The complex scalar carried by every Choi–Jamiołkowski factor.
pub type C = Complex<FloatType>;

// =============================================================================
// The small numbers the operators are written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

/// The two diagonal entries of the commuting partner, `diag(3, −1)`.
///
/// Any two diagonal operators commute, so the values carry no weight beyond being distinct: a
/// multiple of the identity would commute with everything and prove nothing about diagonality.
pub const DIAGONAL_FIRST: FloatType = const_scalar_from_int!(FloatType, 3);
pub const DIAGONAL_SECOND: FloatType = const_scalar_from_int!(FloatType, -1);

/// The observation level each node reports against.
///
/// It plays no part in the commutativity check; a node has to compute something, and this is what
/// these compute.
pub const DETECTION_THRESHOLD: FloatType = const_scalar_from_float!(FloatType, 0.55);

/// The Hilbert leg both factors are declared on.
///
/// Sharing a leg is what brings the pair within scope of the check. Factors on disjoint supports
/// commute for trivial reasons and the freeze never compares them.
pub const SHARED_LEG: usize = 0;

/// The two nodes of the graph, and the edge between them.
pub const SOURCE_NODE: usize = 0;

/// The same two nodes as the graph identifiers the causaloid constructor takes.
pub const SOURCE_ID: u64 = 0;
pub const TARGET_ID: u64 = 1;
pub const TARGET_NODE: usize = 1;
