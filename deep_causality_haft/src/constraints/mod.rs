/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Standard constraint markers for the unified GAT-bounded HKT hierarchy.
//!
//! These constraints mirror `deep_causality_num`'s algebraic structure and provide
//! mathematically principled type bounds for HKT implementations.
//!
//! # Algebraic Hierarchy
//!
//! ```text
//!                     NoConstraint (all types)
//!                           │
//!              ┌────────────┼────────────┐
//!              ▼            ▼            ▼
//!       CloneConstraint  ThreadSafe  AbelianGroupConstraint
//!                                        │
//!                               RingConstraint
//!                                        │
//!                    ┌───────────────────┼───────────────────┐
//!                    ▼                   ▼                   ▼
//!       AssociativeRingConstraint  CommutativeRingConstraint  Distributive
//!                    │                   │
//!                    └─────────┬─────────┘
//!                              ▼
//!                       FieldConstraint
//!                              │
//!                       RealFieldConstraint
//! ```
//!
//! # Usage Guide
//!
//! | Your Type           | Algebraic Properties                 | Use Constraint              |
//! |---------------------|--------------------------------------|------------------------------|
//! | `f32`, `f64`        | Real field                           | `RealFieldConstraint`       |
//! | `Complex<f64>`      | Field (commutative)                  | `FieldConstraint`           |
//! | `Quaternion<f64>`   | Associative ring (non-commutative)   | `AssociativeRingConstraint` |
//! | `Octonion<f64>`     | Abelian group only (non-associative) | `AbelianGroupConstraint`    |
//! | `String`, `Vec<u8>` | No algebraic structure               | `NoConstraint`              |

use crate::Satisfies;
use deep_causality_num::{AbelianGroup, AssociativeRing, CommutativeRing, Field, RealField, Ring};

// ============================================================================
// TIER 0: Universal Constraints
// ============================================================================

/// Clonable types.
///
/// Use for HKTs that need to clone inner values but have no other
/// algebraic requirements.
pub struct CloneConstraint;
impl<T: Clone> Satisfies<CloneConstraint> for T {}

/// Thread-safe types.
///
/// Use for HKTs that require their contents to be sendable across
/// threads and shareable between threads.
pub struct ThreadSafeConstraint;
impl<T: Send + Sync> Satisfies<ThreadSafeConstraint> for T {}

// ============================================================================
// TIER 1: Additive Algebra (Linear Combinations)
// Use Case: Octonion buffers, accumulators, superposition states
// ============================================================================

/// Abelian Group: Add + Sub + Zero, commutative addition.
///
/// Minimal requirement for linear combinations. Works with Octonions
/// (which lack associative multiplication).
///
/// # Mathematical Definition
///
/// An Abelian group satisfies:
/// - Closure under addition
/// - Associativity: `(a + b) + c = a + (b + c)`
/// - Identity: `a + 0 = a`
/// - Inverse: `a + (-a) = 0`
/// - Commutativity: `a + b = b + a`
pub struct AbelianGroupConstraint;
impl<T: AbelianGroup + Copy> Satisfies<AbelianGroupConstraint> for T {}

// ============================================================================
// TIER 2: Multiplicative Algebra (Ring Operations)
// Use Case: Matrix algebra, polynomial evaluation
// ============================================================================

/// Ring: AbelianGroup + Mul + One + Distributive.
///
/// No commutativity or associativity guarantees on multiplication.
pub struct RingConstraint;
impl<T: Ring + Copy> Satisfies<RingConstraint> for T {}

/// Associative Ring: Ring where `(ab)c = a(bc)`.
///
/// Allows Quaternions, Matrices, but NOT Octonions.
///
/// # Use Cases
///
/// - Matrix multiplication
/// - Quaternion rotation composition
/// - Any algorithm requiring `(a * b) * c = a * (b * c)`
pub struct AssociativeRingConstraint;
impl<T: AssociativeRing + Copy> Satisfies<AssociativeRingConstraint> for T {}

/// Commutative Ring: Ring where `ab = ba`.
///
/// Allows Integers, Polynomials, but NOT Matrices.
pub struct CommutativeRingConstraint;
impl<T: CommutativeRing + Copy> Satisfies<CommutativeRingConstraint> for T {}

// ============================================================================
// TIER 3: Division Algebra (Field Operations)
// Use Case: Standard numerical computing, Clifford algebra
// ============================================================================

/// Field: CommutativeRing + Division.
///
/// Standard numerical type constraint for most algorithms.
/// Includes f32, f64, Complex numbers.
///
/// # Mathematical Definition
///
/// A field is a commutative ring where every non-zero element has
/// a multiplicative inverse.
pub struct FieldConstraint;
impl<T: Field + Copy> Satisfies<FieldConstraint> for T {}

/// Real Field: Field + Ordering + Transcendentals.
///
/// Required for algorithms using sqrt, sin, cos, comparisons.
/// Only f32 and f64 satisfy this constraint.
pub struct RealFieldConstraint;
impl<T: RealField + Copy> Satisfies<RealFieldConstraint> for T {}

// ============================================================================
// TIER 4: Composite Constraints
// ============================================================================

/// Field + Thread Safety.
///
/// Common constraint for parallel numerical computing.
pub struct FieldThreadSafe;
impl<T: Field + Copy + Send + Sync> Satisfies<FieldThreadSafe> for T {}

/// Real Field + Thread Safety.
///
/// Most restrictive standard constraint - only f32, f64.
pub struct RealFieldThreadSafe;
impl<T: RealField + Copy + Send + Sync> Satisfies<RealFieldThreadSafe> for T {}

// ============================================================================
// TIER 5: Legacy/Compatibility Constraints
// ============================================================================

/// Numeric types with zero element and copy semantics.
///
/// This constraint provides backward compatibility with the old
/// `BoundedComonad` pattern that used `Zero + Copy` bounds.
pub struct NumericConstraint;
impl<T: deep_causality_num::Zero + Copy + Clone> Satisfies<NumericConstraint> for T {}
