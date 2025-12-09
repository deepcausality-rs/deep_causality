/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// Time (Seconds).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Time(f64);

impl Time {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
             return Err(PhysicsError::new(
                crate::error::PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Time cannot be negative (relative time duration assumed positive)".into(),
                ),
            ));
        }
        Ok(Self(val))
    }

    pub fn from_minutes(minutes: f64) -> Result<Self, PhysicsError> {
        Self::new(minutes * 60.0)
    }

    pub fn from_hours(hours: f64) -> Result<Self, PhysicsError> {
        Self::new(hours * 3600.0)
    }

    pub fn from_days(days: f64) -> Result<Self, PhysicsError> {
        Self::new(days * 86400.0)
    }

    pub fn from_years(years: f64) -> Result<Self, PhysicsError> {
        // Julian year = 365.25 days
        Self::new(years * 31_557_600.0)
    }

    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn as_minutes(&self) -> f64 {
        self.0 / 60.0
    }

    pub fn as_hours(&self) -> f64 {
        self.0 / 3600.0
    }

    pub fn as_days(&self) -> f64 {
        self.0 / 86400.0
    }

    pub fn as_years(&self) -> f64 {
        self.0 / 31_557_600.0
    }
}

impl From<Time> for f64 {
    fn from(val: Time) -> Self {
        val.0
    }
}
