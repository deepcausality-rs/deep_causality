/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt::Debug;

/// The fundamental contract for data flowing through the system.
///
/// Users implement this on their own Enum to define their domain-specific data types.
/// This trait ensures that any data passed through the causal graph satisfies basic
/// requirements like threading support (`Send + Sync`), debugging, and cloning.
///
/// Use this to define the "Protocol" (the vocabulary) of your causal system.
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
