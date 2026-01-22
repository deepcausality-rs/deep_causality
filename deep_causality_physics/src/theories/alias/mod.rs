/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{GaugeField, SO3_1, SU2, SU2_U1, SU3, SU3_SU2_U1, U1};

/// Gauge-Theoretic Electromagnetism (U(1) gauge field).
///
/// This type represents the electromagnetic field using U(1) gauge theory
/// formalism. It computes classical observables from the relativistic
/// field strength tensor F_μν.
///
/// Standard Convention: West Coast (+---)
///
/// FloatType determines the numerical precision level
/// *  7 digits precision: f32
/// * 16 digits precision: f64
/// * 31 digits precision: DoubleFloat:
///
pub type EM<FloatType> = GaugeField<U1, FloatType, FloatType>;

/// Weak force field.
///
/// Standard Convention: West Coast (+---)
pub type WeakField<FloatType> = GaugeField<SU2, FloatType, FloatType>;

/// Electroweak field (unified EM + Weak).
///
/// Standard Convention: West Coast (+---)
pub type ElectroweakField<FloatType> = GaugeField<SU2_U1, FloatType, FloatType>;

/// Quantum Chromodynamics field (strong force).
///
/// Standard Convention: West Coast (+---)
pub type QCD<FloatType> = GaugeField<SU3, FloatType, FloatType>;

/// Standard Model field (all forces except gravity).
///
/// Standard Convention: West Coast (+---)
pub type SMField<FloatType> = GaugeField<SU3_SU2_U1, FloatType, FloatType>;

/// General Relativity field (gravity).
///
/// Standard Convention: East Coast (-+++)
pub type GR<FloatType> = GaugeField<SO3_1, FloatType, FloatType>;

// ============================================================================
// Alternate Names
// ============================================================================

pub type ElectromagneticField<FloatType> = EM<FloatType>;
pub type GravitationalField<FloatType> = GR<FloatType>;
pub type ColorField<FloatType> = QCD<FloatType>;
