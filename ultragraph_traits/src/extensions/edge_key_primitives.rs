/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::EdgeKey;

// A unit, Boolean or numeric payload is a weight or nothing, not an identity, so every one of
// these returns the key `()`: a graph carrying such a payload holds at most one edge per ordered
// pair.

impl EdgeKey for () {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for bool {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for u8 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for u16 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for u32 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for u64 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for u128 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for usize {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for i8 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for i16 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for i32 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for i64 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for i128 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for isize {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for f32 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

impl EdgeKey for f64 {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}
