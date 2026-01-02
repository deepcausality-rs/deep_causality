/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Core constraint system for the unified GAT-bounded HKT hierarchy.
//!
//! This module provides the foundational `Satisfies` trait that enables declarative
//! type constraints for higher-kinded types. By using this system:
//!
//! - **Unconstrained types** use `type Constraint = NoConstraint;`
//! - **Constrained types** use `type Constraint = TensorDataConstraint;` (etc.)
//!
//! # Example
//!
//! ```rust
//! use deep_causality_haft::{HKT, Satisfies, NoConstraint};
//!
//! pub struct VecWitness;
//!
//! impl HKT for VecWitness {
//!     type Constraint = NoConstraint;
//!     type Type<T> = Vec<T> where T: Satisfies<NoConstraint>;
//! }
//! ```

/// Marker trait indicating that type `T` satisfies constraint `C`.
///
/// This is the core abstraction that enables type-safe constraint checking
/// at compile time. Constraints are implemented as marker structs, and
/// blanket implementations of `Satisfies` define which types satisfy each constraint.
///
/// # Design Philosophy
///
/// The `?Sized` bound on `C` allows for flexibility in constraint definitions,
/// including trait objects if needed in the future.
///
/// # Safety
///
/// This trait is a pure marker and has no methods. Implementations should
/// not have any runtime behavior.
pub trait Satisfies<C: ?Sized> {}

/// The universal constraint — every type satisfies it.
///
/// Use this constraint for fully polymorphic HKT implementations like
/// `Vec`, `Option`, `Box`, etc., where no specific bounds are required
/// on the inner type.
///
/// # Example
///
/// ```rust
/// use deep_causality_haft::{Satisfies, NoConstraint};
///
/// // This blanket impl means String, Vec<u8>, custom types, etc. all satisfy NoConstraint
/// fn accepts_any<T: Satisfies<NoConstraint>>(_: T) {}
///
/// accepts_any("hello");
/// accepts_any(42);
/// accepts_any(vec![1, 2, 3]);
/// ```
pub struct NoConstraint;

impl<T> Satisfies<NoConstraint> for T {}
