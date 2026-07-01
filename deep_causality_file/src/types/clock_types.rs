/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::satelite_types::SatId;
use chrono::NaiveDateTime;
use deep_causality_num::RealField;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ClockData<R>
where
    R: RealField,
{
    timestamp: NaiveDateTime,
    sat_id: SatId,
    bias_s: R, // Seconds
}

impl<R> ClockData<R>
where
    R: RealField,
{
    pub fn new(timestamp: NaiveDateTime, sat_id: SatId, bias_s: R) -> Self {
        Self {
            timestamp,
            sat_id,
            bias_s,
        }
    }
}

impl<R> ClockData<R>
where
    R: RealField + Clone,
{
    pub fn timestamp(&self) -> NaiveDateTime {
        self.timestamp
    }

    pub fn sat_id(&self) -> SatId {
        self.sat_id
    }

    pub fn bias_s(&self) -> R {
        self.bias_s
    }
}

impl<R> Display for ClockData<R>
where
    R: RealField + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClockData: timestamp: {}, Satellite ID: {}, Bias (Sec.): {}",
            self.timestamp, self.sat_id, self.bias_s
        )
    }
}
