/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::sync::{Arc, RwLock};

/// A type alias that combines [`Arc`] and [`RwLock`] to provide thread-safe shared mutable state.
///
/// This type provides:
/// - Thread-safe shared ownership through `Arc` (Atomic Reference Counting)
/// - Interior mutability through `RwLock` (Read-Write Lock)
/// - Multiple readers or single writer access pattern
///
/// # Example
/// ```
/// use std::sync::{Arc, RwLock};
/// use deep_causality::prelude::ArcRWLock;
///
/// let data: ArcRWLock<i32> = Arc::new(RwLock::new(0));
/// ```
pub type ArcRWLock<T> = Arc<RwLock<T>>;
