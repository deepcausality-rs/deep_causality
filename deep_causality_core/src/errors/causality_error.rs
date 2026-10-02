/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use alloc::string::String;
use core::fmt::{Display, Formatter};

/// The standard error type for DeepCausality operations.
///
/// This wrapper struct ensures that all errors within the system share a common type,
/// facilitating uniform error propagation and handling within the monadic structures.
///
/// Every variant of [`CausalityErrorEnum`] has a constructor of the same name, so an error reads
/// `CausalityError::GraphNotFrozen()` or `CausalityError::Custom("…")` without naming the enum.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct CausalityError(pub CausalityErrorEnum);

impl CausalityError {
    /// Creates a new `CausalityError` from the specific error variant.
    pub const fn new(error_enum: CausalityErrorEnum) -> Self {
        Self(error_enum)
    }
}

#[allow(non_snake_case)]
impl CausalityError {
    // Generic

    /// See [`CausalityErrorEnum::Unspecified`].
    pub const fn Unspecified() -> Self {
        Self(CausalityErrorEnum::Unspecified)
    }

    /// See [`CausalityErrorEnum::InternalLogicError`].
    pub fn InternalLogicError(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::InternalLogicError(msg.into()))
    }

    /// See [`CausalityErrorEnum::TypeConversionError`].
    pub fn TypeConversionError(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::TypeConversionError(msg.into()))
    }

    /// See [`CausalityErrorEnum::ValueNotAvailable`].
    pub const fn ValueNotAvailable() -> Self {
        Self(CausalityErrorEnum::ValueNotAvailable)
    }

    /// See [`CausalityErrorEnum::MissingParameter`].
    pub fn MissingParameter(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::MissingParameter(msg.into()))
    }

    /// See [`CausalityErrorEnum::UnsupportedOperation`].
    pub fn UnsupportedOperation(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::UnsupportedOperation(msg.into()))
    }

    // Execution

    /// See [`CausalityErrorEnum::MaxStepsExceeded`].
    pub const fn MaxStepsExceeded() -> Self {
        Self(CausalityErrorEnum::MaxStepsExceeded)
    }

    /// See [`CausalityErrorEnum::UnexpectedCommand`].
    pub fn UnexpectedCommand(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::UnexpectedCommand(msg.into()))
    }

    // Causal structure

    /// See [`CausalityErrorEnum::GraphNotFrozen`].
    pub const fn GraphNotFrozen() -> Self {
        Self(CausalityErrorEnum::GraphNotFrozen)
    }

    /// See [`CausalityErrorEnum::GraphContainsCycle`].
    pub const fn GraphContainsCycle() -> Self {
        Self(CausalityErrorEnum::GraphContainsCycle)
    }

    /// See [`CausalityErrorEnum::CausaloidNotFound`].
    pub const fn CausaloidNotFound(index: usize) -> Self {
        Self(CausalityErrorEnum::CausaloidNotFound(index))
    }

    /// See [`CausalityErrorEnum::GraphError`].
    pub fn GraphError(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::GraphError(msg.into()))
    }

    /// See [`CausalityErrorEnum::EmptyCollection`].
    pub const fn EmptyCollection() -> Self {
        Self(CausalityErrorEnum::EmptyCollection)
    }

    // Context and uncertainty

    /// See [`CausalityErrorEnum::MissingContext`].
    pub const fn MissingContext() -> Self {
        Self(CausalityErrorEnum::MissingContext)
    }

    /// See [`CausalityErrorEnum::UncertainError`].
    pub fn UncertainError(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::UncertainError(msg.into()))
    }

    // IO

    /// See [`CausalityErrorEnum::IoError`].
    pub fn IoError(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::IoError(msg.into()))
    }

    // User-defined and domain

    /// See [`CausalityErrorEnum::Custom`].
    pub fn Custom(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::Custom(msg.into()))
    }

    /// See [`CausalityErrorEnum::ActionError`].
    pub fn ActionError(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::ActionError(msg.into()))
    }

    /// See [`CausalityErrorEnum::DeonticError`].
    pub fn DeonticError(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::DeonticError(msg.into()))
    }

    /// See [`CausalityErrorEnum::ModelError`].
    pub fn ModelError(msg: impl Into<String>) -> Self {
        Self(CausalityErrorEnum::ModelError(msg.into()))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CausalityError {}

impl Display for CausalityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Delegate to the debug representation of the inner enum.
        write!(f, "{:?}", self.0)
    }
}

/// Detailed variants of potential errors in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum CausalityErrorEnum {
    // Generic Errors
    /// An error occurred that doesn't fit into other categories.
    #[default]
    Unspecified,
    /// An internal invariant was violated; indicates a bug in the library. Carries the invariant.
    InternalLogicError(String),
    /// A value had a different type than the operation expected. Carries the expected and found
    /// types.
    TypeConversionError(String),
    /// A required value was expected but `None` was found.
    ValueNotAvailable,
    /// An operation needs a parameter the caller did not supply. Carries the parameter.
    MissingParameter(String),
    /// The operation is not supported on this input or form. Carries what was attempted.
    UnsupportedOperation(String),

    // Execution Errors
    /// The execution exceeded the maximum allowed steps (infinite loop protection).
    MaxStepsExceeded,
    /// A command, such as `RelayTo`, arrived where no command can be handled. Carries where.
    UnexpectedCommand(String),

    // Causal Structure Errors
    /// A causal graph was used for reasoning before it was frozen.
    GraphNotFrozen,
    /// A causal graph contains a directed cycle; reasoning requires an acyclic graph.
    GraphContainsCycle,
    /// No causaloid exists at this graph index.
    CausaloidNotFound(usize),
    /// The graph backend refused an operation. Carries its message.
    GraphError(String),
    /// A collection or reduction received no elements.
    EmptyCollection,

    // Context and Uncertainty Errors
    /// A step that reads a context received none.
    MissingContext,
    /// Evaluating an uncertain value failed. Carries the underlying message.
    UncertainError(String),

    // IO Errors
    /// A file input/output action failed (e.g. an unreadable or unwritable path). Carries the
    /// underlying error message. Produced by the `Io` file actions in `crate::types::io`.
    IoError(String),

    // User-defined and Domain Errors
    /// A user-defined custom error message.
    Custom(String),
    /// Error related to executing an action.
    ActionError(String),
    /// Error related to deontic logic constraints.
    DeonticError(String),
    /// Error related to the internal model state.
    ModelError(String),
}
