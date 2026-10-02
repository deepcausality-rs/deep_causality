/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::FaultTimeline;

impl FaultTimeline {
    pub fn gnss_degraded_at_s(&self) -> usize {
        self.gnss_degraded_at_s
    }

    pub fn gnss_lost_at_s(&self) -> usize {
        self.gnss_lost_at_s
    }

    pub fn gnss_restored_at_s(&self) -> Option<usize> {
        self.gnss_restored_at_s
    }

    pub fn link_lost_at_s(&self) -> usize {
        self.link_lost_at_s
    }

    pub fn link_restored_at_s(&self) -> Option<usize> {
        self.link_restored_at_s
    }

    pub fn battery_fault_at_s(&self) -> Option<usize> {
        self.battery_fault_at_s
    }

    /// Whether the fix is lost at second `t`.
    pub fn gnss_lost(&self, t: usize) -> bool {
        between(t, self.gnss_lost_at_s, self.gnss_restored_at_s)
    }

    /// Whether the fix is degraded but not lost at second `t`.
    pub fn gnss_degraded(&self, t: usize) -> bool {
        between(t, self.gnss_degraded_at_s, self.gnss_restored_at_s) && !self.gnss_lost(t)
    }

    /// Whether the link is lost at second `t`.
    pub fn link_lost(&self, t: usize) -> bool {
        between(t, self.link_lost_at_s, self.link_restored_at_s)
    }

    /// Whether a cell has failed by second `t`.
    pub fn battery_faulted(&self, t: usize) -> bool {
        self.battery_fault_at_s.is_some_and(|at| t >= at)
    }
}

/// Whether `t` falls at or after `from` and before `until`, if there is an `until`.
fn between(t: usize, from: usize, until: Option<usize>) -> bool {
    t >= from && until.is_none_or(|end| t < end)
}
