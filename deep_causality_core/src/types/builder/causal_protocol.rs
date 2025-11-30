/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt::{Debug, Display};

/// The fundamental contract for data flowing through the system.
/// Users implement this on their own Enum to define their domain.
pub trait CausalProtocol: Clone + Debug + Send + Sync + 'static {
    /// A standard error representation for runtime faults.
    fn error<E: Display>(msg: &E) -> Self;
}

/// Trait to unwrap specific types from the Protocol Enum.
pub trait FromProtocol<P>: Sized {
    type Error: Display;
    fn from_protocol(p: P) -> Result<Self, Self::Error>;
}

/// Trait to wrap specific types into the Protocol Enum.
pub trait ToProtocol<P> {
    fn to_protocol(self) -> P;
}
