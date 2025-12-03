/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
#[cfg(all(feature = "alloc", not(feature = "strict-zst")))]
use alloc::boxed::Box;
use core::fmt::{Display, Formatter};

/// A generic wrapper for a causal function.
/// It handles the extraction from the Protocol, execution, and re-wrapping.
pub struct ExecutableNode<P> {
    pub(crate) id: usize,
    /// The adapter logic. Takes Protocol -> Protocol.
    ///
    /// If `strict-zst` is enabled:
    /// We use a plain function pointer `fn(P) -> P` for certification and performance.
    /// This requires the user logic to be stateless (ZST function items).
    ///
    /// If `strict-zst` is disabled (default):
    /// We use `Box<dyn Fn>` to allow closures to capture configuration/context.
    #[cfg(feature = "strict-zst")]
    pub(crate) func: fn(P) -> P,
    #[cfg(not(feature = "strict-zst"))]
    pub(crate) func: Box<dyn Fn(P) -> P + Send + Sync>,
}

impl<P> ExecutableNode<P> {
    #[cfg(feature = "strict-zst")]
    pub fn new(id: usize, func: fn(P) -> P) -> Self {
        Self { id, func }
    }

    #[cfg(not(feature = "strict-zst"))]
    pub fn new(id: usize, func: Box<dyn Fn(P) -> P + Send + Sync>) -> Self {
        Self { id, func }
    }
}

impl<P> Display for ExecutableNode<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ExecutableNode(id: {})", self.id)
    }
}
