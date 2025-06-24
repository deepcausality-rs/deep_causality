// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

mod display;
mod identifiable;
mod temporable;
mod scalar_projector;

use deep_causality_macros::Constructor;

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
/// use deep_causality::prelude::{EntropicTime, Temporal, TimeScale};
///
/// let t1 = EntropicTime::new(1, 0); // system start
/// let t2 = EntropicTime::new(2, 1); // one entropy event later
///
/// assert!(t1.time_unit() < t2.time_unit());
/// ```
#[derive(Constructor, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntropicTime {
    /// Unique ID for this time instance
    id: u64,

    /// Irreversible "tick" counter driven by entropy or state progression
    entropy_tick: u64,
}
