/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::design::instrument_time::InstrumentTime;

/// How `design` covers a pair: one experiment alone at its own shots, the chosen experiments'
/// bits added up, or one experiment alone with its shots sized in time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoverMode<R> {
    /// One experiment covers a pair alone, at its own shots and cost.
    Fixed,
    /// A pair is covered by the bits the chosen experiments add up to.
    Combining,
    /// Each experiment takes the fewest shots that reach the floor, at this instrument time's
    /// shot time and within its white-noise range, and is priced in seconds.
    Timed(InstrumentTime<R>),
}
