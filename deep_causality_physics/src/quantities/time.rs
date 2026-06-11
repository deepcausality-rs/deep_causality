/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_num::{FromPrimitive, RealField};

/// Time (Seconds).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Time<R: RealField>(R);

impl<R: RealField> Default for Time<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Time<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Time cannot be negative (relative time duration assumed positive)".into(),
            ));
        }
        Ok(Self(val))
    }

    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }

    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + FromPrimitive> Time<R> {
    pub fn from_minutes(minutes: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(60.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(60) failed".into()))?;
        Self::new(minutes * k)
    }

    pub fn from_hours(hours: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(3600.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(3600) failed".into()))?;
        Self::new(hours * k)
    }

    pub fn from_days(days: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(86400.0).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(86400) failed".into())
        })?;
        Self::new(days * k)
    }

    pub fn from_years(years: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(31_557_600.0).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(31557600) failed".into())
        })?;
        Self::new(years * k)
    }

    pub fn as_minutes(&self) -> R {
        let k = R::from_f64(60.0).expect("R::from_f64(60) failed");
        self.0 / k
    }

    pub fn as_hours(&self) -> R {
        let k = R::from_f64(3600.0).expect("R::from_f64(3600) failed");
        self.0 / k
    }

    pub fn as_days(&self) -> R {
        let k = R::from_f64(86400.0).expect("R::from_f64(86400) failed");
        self.0 / k
    }

    pub fn as_years(&self) -> R {
        let k = R::from_f64(31_557_600.0).expect("R::from_f64(31557600) failed");
        self.0 / k
    }
}

impl<R: RealField + Into<f64>> From<Time<R>> for f64 {
    fn from(val: Time<R>) -> Self {
        val.0.into()
    }
}
