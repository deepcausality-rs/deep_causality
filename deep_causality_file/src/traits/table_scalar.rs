/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The [`TableScalar`] trait: one symmetric text codec that both the table reader and the
//! table writer bound on, so `parse_cell(write_cell(x)) == x` holds by construction at every
//! supported precision. Implemented for `f64`, `f32`, and `Float106`.
//!
//! - `f64` / `f32` use Rust's shortest round-trip `Display`, which is exact-parse-recoverable.
//! - `Float106` writes a two-component `hi|lo` pair cell (each an exact `f64`), reconstructed
//!   with [`Float106::from_raw`] so no bits are lost. Every parser also accepts a plain decimal
//!   literal (lifted through exact `f64`), so a hand-authored specification table loads at any
//!   precision.

use core::fmt::Write as _;
use deep_causality_num::Float106;

/// The separator between the `hi` and `lo` components of a `Float106` pair cell. Chosen so it
/// never appears inside an `f64` decimal literal, keeping a plain decimal unambiguously a
/// single-component cell.
const PAIR_SEP: char = '|';

/// A scalar that round-trips through a table cell exactly. Both the typed table reader and the
/// result-table writer bound on this, so a written table reads back with identical bits at the
/// written precision. `Copy` because the supported cell scalars (`f64`, `f32`, `Float106`) are
/// all small `Copy` values.
pub trait TableScalar: Copy {
    /// Append this value's exact text encoding to `out`.
    fn write_cell(&self, out: &mut String);

    /// Parse a cell written by [`write_cell`](Self::write_cell), or a plain decimal literal
    /// (lifted through exact `f64`). Returns `None` on a malformed cell.
    fn parse_cell(cell: &str) -> Option<Self>;
}

impl TableScalar for f64 {
    fn write_cell(&self, out: &mut String) {
        // Rust's `f64` Display is shortest round-trip: parsing this text recovers the exact bits.
        let _ = write!(out, "{self}");
    }

    fn parse_cell(cell: &str) -> Option<Self> {
        cell.trim().parse::<f64>().ok()
    }
}

impl TableScalar for f32 {
    fn write_cell(&self, out: &mut String) {
        // Rust's `f32` Display is likewise shortest round-trip at `f32` precision.
        let _ = write!(out, "{self}");
    }

    fn parse_cell(cell: &str) -> Option<Self> {
        cell.trim().parse::<f32>().ok()
    }
}

impl TableScalar for Float106 {
    fn write_cell(&self, out: &mut String) {
        // The exact double-double representation is the pair (hi, lo); each is written shortest
        // round-trip, so `from_raw(parse(hi), parse(lo))` reconstructs the original bit-for-bit.
        let _ = write!(out, "{}{PAIR_SEP}{}", self.hi(), self.lo());
    }

    fn parse_cell(cell: &str) -> Option<Self> {
        let cell = cell.trim();
        match cell.split_once(PAIR_SEP) {
            // A pair cell written by `write_cell`: reconstruct exactly from its components.
            Some((hi, lo)) => {
                let hi = hi.trim().parse::<f64>().ok()?;
                let lo = lo.trim().parse::<f64>().ok()?;
                Some(Float106::from_raw(hi, lo))
            }
            // A plain decimal literal (a hand-authored spec table): lift through exact `f64`.
            None => cell.parse::<f64>().ok().map(Float106::from),
        }
    }
}
