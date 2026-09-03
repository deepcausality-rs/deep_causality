/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The [`BitCodec`] trait: bit-exact encoding of a working scalar into a snapshot section
//! payload. The concrete implementations for `f32`, `f64`, and `Float106` live beside the
//! snapshot payload types in [`crate::types::snapshot_types`].

use crate::types::snapshot_types::ScalarTypeTag;

/// Bit-exact encoding of a working scalar into a section payload. Values are raw IEEE bit
/// patterns, little-endian, so encoding and decoding change no bits at any precision.
pub trait BitCodec: Sized {
    /// The tag a package of this scalar carries in its header.
    const SCALAR_TAG: ScalarTypeTag;

    /// Append this value's bit pattern to `out`.
    fn write_bits(&self, out: &mut Vec<u8>);

    /// Read one value's bit pattern at `*offset`, advancing it. `None` when `bytes` is short.
    fn read_bits(bytes: &[u8], offset: &mut usize) -> Option<Self>;
}
