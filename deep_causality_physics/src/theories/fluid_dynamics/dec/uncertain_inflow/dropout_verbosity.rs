/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! How densely the uncertain-inflow march records sensor dropouts in the `EffectLog`.

/// Verbosity policy for the BC-fallback record an [`UncertainInflowZone`] writes when a sensor
/// sample fails its presence gate (CFD Stage-4 design D6, open question 3).
///
/// [`UncertainInflowZone`]: super::uncertain_inflow_zone::UncertainInflowZone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DropoutVerbosity {
    /// Record every dropped step and its fallback substitution — the default; per-step records
    /// are cheap and give a complete audit trail.
    #[default]
    EachDropout,
    /// Record only the transitions — the first dropout after a present run (onset) and the
    /// first present sample after a dropout run (recovery) — to throttle a long march's log.
    Transitions,
}
