/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Propulsion quantity types for the retropulsion kernel family: the mass-flow
//! newtype, the nozzle-branch selector, and the composite exit-state and
//! plume-geometry results. Scalar quantities from other domains (`Force`,
//! `Acceleration`, `Pressure`, `Temperature`, `Density`, `Speed`, `Length`,
//! `Mass`, `Area`) are reused — not duplicated — by the propulsion kernels.

use crate::{Density, Length, PhysicsError, Pressure, Speed, Temperature};

/// Propellant mass-flow rate $\dot m$. Unit: $kg \cdot s^{-1}$.
/// Constraint: finite, $\geq 0$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MassFlowRate<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for MassFlowRate<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> MassFlowRate<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Mass flow rate must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Mass flow rate cannot be negative".into(),
            ));
        }
        Ok(Self(val))
    }
    /// Creates a new `MassFlowRate` without validation.
    /// Use only if the value is guaranteed finite and non-negative.
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<MassFlowRate<R>> for f64 {
    fn from(val: MassFlowRate<R>) -> Self {
        val.0.into()
    }
}

/// The isentropic branch of the area–Mach relation: for every area ratio
/// $A/A^* > 1$ the relation has one subsonic and one supersonic root, and the
/// caller must say which flow regime it is asking about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowBranch {
    Subsonic,
    Supersonic,
}

/// The isentropically expanded nozzle exit state: exit Mach number and the
/// static exit quantities, as composed by `nozzle_exit_state_kernel`. Each
/// component is a validated quantity; the struct only groups them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NozzleExitState<R: deep_causality_algebra::RealField> {
    mach: R,
    pressure: Pressure<R>,
    temperature: Temperature<R>,
    density: Density<R>,
    velocity: Speed<R>,
}

impl<R: deep_causality_algebra::RealField> Default for NozzleExitState<R> {
    fn default() -> Self {
        Self {
            mach: R::zero(),
            pressure: Pressure::default(),
            temperature: Temperature::default(),
            density: Density::default(),
            velocity: Speed::default(),
        }
    }
}

impl<R: deep_causality_algebra::RealField> NozzleExitState<R> {
    pub fn new(
        mach: R,
        pressure: Pressure<R>,
        temperature: Temperature<R>,
        density: Density<R>,
        velocity: Speed<R>,
    ) -> Self {
        Self {
            mach,
            pressure,
            temperature,
            density,
            velocity,
        }
    }
    /// Exit Mach number $M_e$ (dimensionless).
    pub fn mach(&self) -> R {
        self.mach
    }
    /// Static exit pressure $p_e$ (Pa).
    pub fn pressure(&self) -> Pressure<R> {
        self.pressure
    }
    /// Static exit temperature $T_e$ (K).
    pub fn temperature(&self) -> Temperature<R> {
        self.temperature
    }
    /// Static exit density $\rho_e$ (kg·m⁻³).
    pub fn density(&self) -> Density<R> {
        self.density
    }
    /// Exit velocity $u_e$ (m·s⁻¹).
    pub fn velocity(&self) -> Speed<R> {
        self.velocity
    }
}

/// The analytic plume-as-effective-obstruction geometry the SRP plume kernel
/// returns: the maximum plume radius, the upstream penetration length of the
/// plume from the nozzle exit, and the terminal-shock (Mach-disk) standoff.
/// All lengths; shaping any discrete forcing region from them is the CFD
/// stage's job — kernels do not discretize space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlumeGeometry<R: deep_causality_algebra::RealField> {
    max_radius: Length<R>,
    penetration_length: Length<R>,
    terminal_shock_standoff: Length<R>,
}

impl<R: deep_causality_algebra::RealField> Default for PlumeGeometry<R> {
    fn default() -> Self {
        Self {
            max_radius: Length::default(),
            penetration_length: Length::default(),
            terminal_shock_standoff: Length::default(),
        }
    }
}

impl<R: deep_causality_algebra::RealField> PlumeGeometry<R> {
    pub fn new(
        max_radius: Length<R>,
        penetration_length: Length<R>,
        terminal_shock_standoff: Length<R>,
    ) -> Self {
        Self {
            max_radius,
            penetration_length,
            terminal_shock_standoff,
        }
    }
    /// Maximum radius of the plume boundary (m).
    pub fn max_radius(&self) -> Length<R> {
        self.max_radius
    }
    /// Upstream penetration length of the plume from the nozzle exit (m).
    pub fn penetration_length(&self) -> Length<R> {
        self.penetration_length
    }
    /// Terminal-shock (Mach-disk) standoff distance from the nozzle exit (m).
    pub fn terminal_shock_standoff(&self) -> Length<R> {
        self.terminal_shock_standoff
    }
}
