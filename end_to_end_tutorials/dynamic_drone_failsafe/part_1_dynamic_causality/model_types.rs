/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What the fail-safe controller's causal process carries from one second to the next.

use deep_causality::PropagatingProcess;
use dynamic_drone_failsafe::Command;

/// The controller's causal process. It has no context: in part 1 the controller knows only what
/// the drone reports about itself.
pub type FailsafeProcess<V> = PropagatingProcess<V, FailsafeState, ()>;

/// The faults the controller has confirmed this second.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Faults {
    pub gnss_degraded: bool,
    pub gnss_lost: bool,
    pub link_lost: bool,
    pub battery_critical: bool,
}

/// What the controller remembers: how long the fix and the link have been gone, the faults it has
/// confirmed, and the fail-safe it has latched.
#[derive(Debug, Clone, Default)]
pub struct FailsafeState {
    pub no_fix_for_s: usize,
    pub link_down_for_s: usize,
    pub faults: Faults,
    pub failsafe: Command,
}
