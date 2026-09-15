/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gauge fields: the typed operators, and the one genuine higher-kinded structure among them.
//!
//! [`gauge_field_ops::GaugeFieldOps`] and [`lattice_gauge_field_ops::LatticeGaugeFieldOps`] are
//! zero-sized namespaces carrying inherent methods. Neither implements a `deep_causality_haft`
//! trait, and neither is a witness; both were named `*Witness` until the suffix was found to
//! promise structure that is not there. The reason it is not there is below.
//!
//! The higher-kinded content of this module is in [`hkt_adjunction_stokes`], where
//! `ExteriorDerivativeWitness` and `BoundaryWitness` are genuine `HKT` witnesses and the
//! structure lives in the adjunction `d ⊣ ∂` between them rather than in either one.
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
//! 1. Declaring the operations type with no element bound, so any types are allowed
//! 2. Providing type-safe operations through specialized methods
//! 3. Using concrete GaugeField operations that enforce G: GaugeGroup at call sites

pub mod gauge_field_ops;
pub mod hkt_adjunction_stokes;
pub mod hkt_curvature;
pub mod lattice_gauge_field_ops;
