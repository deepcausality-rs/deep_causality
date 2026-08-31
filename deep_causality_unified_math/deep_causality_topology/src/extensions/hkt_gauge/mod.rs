/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT3 witness and trait implementations for GaugeField.
//!
//! This module provides the `HKT3Unbound` witness for `GaugeField` and the gauge-theoretic
//! operators built on it: the Stokes adjunction, curvature, and the lattice gauge action.
//!
//! It once carried `MonoidalMerge` and `ParametricMonad` impls as well. Both were laws-free
//! stubs and were deleted in `7ec185d49`; the `MonoidalMerge` one is used as the worked
//! cautionary example in `deep_causality_haft::lax_monoidal`. Do not reintroduce either
//! without law tests.
//!
//! # Architectural Note
//!
//! GaugeField<G, A, F> has a non-uniform constraint: G must implement GaugeGroup,
//! while A and F can be any type. The standard HKT3Unbound trait expects a single
//! uniform constraint for all three type parameters.
//!
//! We work around this by:
//! 1. Implementing HKT3Unbound with NoConstraint (allowing any types)
//! 2. Providing type-safe operations through specialized methods
//! 3. Using concrete GaugeField operations that enforce G: GaugeGroup at call sites

pub mod hkt_adjunction_stokes;
pub mod hkt_curvature;
pub mod hkt_gauge_witness;
pub mod hkt_lattice_gauge;
