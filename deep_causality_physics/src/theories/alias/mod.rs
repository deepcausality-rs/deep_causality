/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{Electroweak, GaugeField, Lorentz, SU2, SU3, StandardModel, U1};

// ============================================================================
// Fundamental Force Type Aliases
// ============================================================================

/// Quantum Electrodynamics field (electromagnetism).
///
/// Standard Convention: West Coast (+---)
pub type QED = GaugeField<U1, f64, f64>;

/// Weak force field.
///
/// Standard Convention: West Coast (+---)
pub type WeakField = GaugeField<SU2, f64, f64>;

/// Electroweak field (unified EM + Weak).
///
/// Standard Convention: West Coast (+---)
pub type ElectroweakField = GaugeField<Electroweak, f64, f64>;

/// Quantum Chromodynamics field (strong force).
///
/// Standard Convention: West Coast (+---)
pub type QCD = GaugeField<SU3, f64, f64>;

/// Standard Model field (all forces except gravity).
///
/// Standard Convention: West Coast (+---)
pub type SMField = GaugeField<StandardModel, f64, f64>;

/// General Relativity field (gravity).
///
/// Standard Convention: East Coast (-+++)
pub type GR = GaugeField<Lorentz, f64, f64>;

// ============================================================================
// Alternate Names
// ============================================================================

pub type ElectromagneticField = QED;
pub type GravitationalField = GR;
pub type ColorField = QCD;
