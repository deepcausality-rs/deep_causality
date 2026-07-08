/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! SI base and derived scalar quantities shared across multiple physics domains.
//! Any type that belongs to the International System of Units and is used by
//! more than one domain kernel lives here rather than in a domain-specific file.

use crate::PhysicsError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Length (m).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length<R: RealField>(R);

impl<R: RealField> Default for Length<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Length<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Length must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Length cannot be negative".into(),
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

impl<R: RealField + Into<f64>> From<Length<R>> for f64 {
    fn from(val: Length<R>) -> Self {
        val.0.into()
    }
}

/// Area (m²).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Area<R: RealField>(R);

impl<R: RealField> Default for Area<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Area<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Area must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Area cannot be negative".into(),
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

impl<R: RealField + Into<f64>> From<Area<R>> for f64 {
    fn from(val: Area<R>) -> Self {
        val.0.into()
    }
}

/// Volume (m³).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Volume<R: RealField>(R);

impl<R: RealField> Default for Volume<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Volume<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Volume must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Volume cannot be negative".into(),
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

impl<R: RealField + Into<f64>> From<Volume<R>> for f64 {
    fn from(val: Volume<R>) -> Self {
        val.0.into()
    }
}

/// Mass (kg).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass<R: RealField>(R);

impl<R: RealField> Default for Mass<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Mass<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Mass must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Mass cannot be negative".into(),
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

impl<R: RealField + Into<f64>> From<Mass<R>> for f64 {
    fn from(val: Mass<R>) -> Self {
        val.0.into()
    }
}

/// Speed — scalar magnitude of velocity (m/s).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Speed<R: RealField>(R);

impl<R: RealField> Default for Speed<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Speed<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Speed must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Speed cannot be negative".into(),
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

impl<R: RealField + Into<f64>> From<Speed<R>> for f64 {
    fn from(val: Speed<R>) -> Self {
        val.0.into()
    }
}

/// Frequency (Hz).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Frequency<R: RealField>(R);

impl<R: RealField> Default for Frequency<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Frequency<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Frequency must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Frequency cannot be negative".into(),
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

impl<R: RealField + Into<f64>> From<Frequency<R>> for f64 {
    fn from(val: Frequency<R>) -> Self {
        val.0.into()
    }
}

/// Absolute Temperature (Kelvin) — SI base unit.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature<R: RealField>(R);

const ZERO_CELSIUS_IN_KELVIN: f64 = 273.15;

impl<R: RealField> Default for Temperature<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Temperature<R> {
    /// Creates a new `Temperature` from Kelvin. Negative values are rejected.
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::ZeroKelvinViolation());
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

impl<R: RealField + FromPrimitive> Temperature<R> {
    pub fn from_celsius(celsius: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(ZERO_CELSIUS_IN_KELVIN).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(ZERO_CELSIUS_IN_KELVIN) failed".into())
        })?;
        Self::new(celsius + k)
    }

    pub fn from_fahrenheit(fahrenheit: R) -> Result<Self, PhysicsError> {
        let thirty_two = R::from_f64(32.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(32) failed".into()))?;
        let five = R::from_f64(5.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(5) failed".into()))?;
        let nine = R::from_f64(9.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(9) failed".into()))?;
        Self::from_celsius((fahrenheit - thirty_two) * (five / nine))
    }

    pub fn as_celsius(&self) -> R {
        let k = R::from_f64(ZERO_CELSIUS_IN_KELVIN)
            .expect("R::from_f64(ZERO_CELSIUS_IN_KELVIN) failed");
        self.0 - k
    }

    pub fn as_fahrenheit(&self) -> R {
        let nine = R::from_f64(9.0).expect("R::from_f64(9) failed");
        let five = R::from_f64(5.0).expect("R::from_f64(5) failed");
        let thirty_two = R::from_f64(32.0).expect("R::from_f64(32) failed");
        (self.as_celsius() * (nine / five)) + thirty_two
    }
}

impl<R: RealField + Into<f64>> From<Temperature<R>> for f64 {
    fn from(val: Temperature<R>) -> Self {
        val.0.into()
    }
}

/// Time (seconds) — SI base unit.
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

/// Energy (Joules) — SI-derived unit. Can be negative (potential wells).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Energy<R: RealField>(R);

const JOULES_PER_EV: f64 = 1.602_176_634e-19;
const JOULES_PER_CALORIE: f64 = 4.184;
const JOULES_PER_KWH: f64 = 3.6e6;

impl<R: RealField> Default for Energy<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Energy<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }

    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }

    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + FromPrimitive> Energy<R> {
    pub fn from_electron_volts(ev: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(JOULES_PER_EV).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(JOULES_PER_EV) failed".into())
        })?;
        Self::new(ev * k)
    }

    pub fn from_calories(cal: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(JOULES_PER_CALORIE).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(JOULES_PER_CALORIE) failed".into())
        })?;
        Self::new(cal * k)
    }

    pub fn from_kilowatt_hours(kwh: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(JOULES_PER_KWH).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(JOULES_PER_KWH) failed".into())
        })?;
        Self::new(kwh * k)
    }

    pub fn as_electron_volts(&self) -> R {
        let k = R::from_f64(JOULES_PER_EV).expect("R::from_f64(JOULES_PER_EV) failed");
        self.0 / k
    }

    pub fn as_calories(&self) -> R {
        let k = R::from_f64(JOULES_PER_CALORIE).expect("R::from_f64(JOULES_PER_CALORIE) failed");
        self.0 / k
    }

    pub fn as_kilowatt_hours(&self) -> R {
        let k = R::from_f64(JOULES_PER_KWH).expect("R::from_f64(JOULES_PER_KWH) failed");
        self.0 / k
    }
}

impl<R: RealField + Into<f64>> From<Energy<R>> for f64 {
    fn from(val: Energy<R>) -> Self {
        val.0.into()
    }
}
