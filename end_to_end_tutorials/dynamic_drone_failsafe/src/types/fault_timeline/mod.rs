/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod fault_timeline_getters;

/// When each fault strikes and clears, in seconds after launch: satellite positioning degrades
/// and then drops out, the command link drops, and a cell may fail. A lost fix or link may come
/// back; a failed cell does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaultTimeline {
    gnss_degraded_at_s: usize,
    gnss_lost_at_s: usize,
    gnss_restored_at_s: Option<usize>,
    link_lost_at_s: usize,
    link_restored_at_s: Option<usize>,
    battery_fault_at_s: Option<usize>,
}

impl FaultTimeline {
    /// A timeline whose fix and link never come back.
    pub fn new(
        gnss_degraded_at_s: usize,
        gnss_lost_at_s: usize,
        link_lost_at_s: usize,
        battery_fault_at_s: Option<usize>,
    ) -> Self {
        Self {
            gnss_degraded_at_s,
            gnss_lost_at_s,
            gnss_restored_at_s: None,
            link_lost_at_s,
            link_restored_at_s: None,
            battery_fault_at_s,
        }
    }

    /// This timeline with the fix and the link coming back at the given times, if at all.
    pub fn restoring(self, gnss_at_s: Option<usize>, link_at_s: Option<usize>) -> Self {
        Self {
            gnss_restored_at_s: gnss_at_s,
            link_restored_at_s: link_at_s,
            ..self
        }
    }
}
