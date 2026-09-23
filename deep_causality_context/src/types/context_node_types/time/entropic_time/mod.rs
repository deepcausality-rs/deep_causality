/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
mod adjustable;
mod display;
mod identifiable;
mod recordable;
mod scalar_projector;
mod temporable;

use crate::TimeKind;
use deep_causality_algebra::RealField;

/// A time model based on **entropy-driven progression**, suitable for
/// emergent systems, self-organization, and irreversible state evolution.
///
/// Unlike clock-based time models (e.g., `LorentzianTime`), `EntropicTime`
/// defines time by the **monotonic increase in system entropy**, or more generally,
/// by the irreversible advancement of causal structure.
///
/// This makes it ideal for:
/// - Causal emergence
/// - Thermodynamic processes
/// - Symbolic systems with irreversible updates
/// - Planning systems where time is induced from transition irreversibility
///
/// # Fields
/// - `id`: Unique identifier
/// - `entropy_tick`: Monotonically increasing entropy count (not physical time)
///
/// # Examples
/// ```rust
/// use deep_causality_context::{EntropicTime, Temporal, TimeScale};
///
/// let t1 = EntropicTime::new(1, 0); // system start
/// let t2 = EntropicTime::new(2, 1); // one entropy event later
///
/// assert!(t1.time_unit() < t2.time_unit());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntropicTime {
    /// Unique ID for this time instance
    id: ContextoidId,

    /// Irreversible "tick" counter driven by entropy or state progression
    entropy_tick: u64,
}

impl EntropicTime {
    pub fn new(id: ContextoidId, entropy_tick: u64) -> Self {
        Self { id, entropy_tick }
    }
}

impl<R: RealField> From<EntropicTime> for TimeKind<R> {
    fn from(t: EntropicTime) -> Self {
        TimeKind::Entropic(t)
    }
}
