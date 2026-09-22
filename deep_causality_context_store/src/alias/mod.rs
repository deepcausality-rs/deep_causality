/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The identifier width the records carry.

/// The primitive every record identifier is.
///
/// Declared on the contract because a record stores this width and a backend reads it. The
/// context crate declares its own alias of the same width, and the projection writes one into the
/// other, so a widening on either side alone fails the context crate's build.
pub type IdentificationValue = u64;

/// The identifier of a stored context.
pub type ContextId = IdentificationValue;

/// The identifier of a stored contextoid.
pub type ContextoidId = IdentificationValue;
