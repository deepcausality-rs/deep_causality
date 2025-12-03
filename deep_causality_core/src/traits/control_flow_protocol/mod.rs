/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt::Debug;

/// The fundamental contract for data flowing through the system.
/// Users implement this on their own Enum to define their domain.
pub trait ControlFlowProtocol: Clone + Debug + Send + Sync + 'static {
    /// A standard way to represent a protocol-level error.
    fn error<E: Debug>(e: E) -> Self;
}

/// Trait to unwrap specific types from the Protocol Enum.
pub trait FromProtocol<P: ControlFlowProtocol>: Sized {
    type Error: Debug + Clone + Copy;
    fn from_protocol(p: P) -> Result<Self, Self::Error>;
}

/// Trait to wrap specific types into the Protocol Enum.
pub trait ToProtocol<P> {
    fn to_protocol(self) -> P;
}
