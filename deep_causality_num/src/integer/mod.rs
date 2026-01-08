/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Integer traits for abstracting over primitive integer types.
//!
//! This module provides three traits:
//! - [`Integer`]: Common operations for all primitive integers
//! - [`SignedInt`]: Operations specific to signed integers
//! - [`UnsignedInt`]: Operations specific to unsigned integers

mod integer;
mod integer_impl;
mod signed;
mod signed_impl;
mod unsigned;
mod unsigned_impl;

pub use integer::Integer;
pub use signed::SignedInt;
pub use unsigned::UnsignedInt;
