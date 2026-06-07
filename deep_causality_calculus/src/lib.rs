/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]

//! Arrow-native differentiation and integration operators for the deep_causality project.
//!
//! These are the analytic operators of the Causal Arrow, sitting above the two foundations
//! they combine: the value-level `Arrow` / `Endomorphism` machinery in `deep_causality_haft`
//! and the `Dual` number in `deep_causality_num`. Neither foundation depends on the other;
//! this crate is where they meet, so both stay self-contained.
//!
//! - **Differentiation is the tangent functor.** Its object map is `Dual`; its morphism map
//!   is running an arrow over duals. Because a concrete `Arrow<In = f64>` cannot be lifted
//!   over `Dual`, the scalar-polymorphism lives in [`DifferentiableArrow`], whose `run` is
//!   generic over the scalar. [`Diff`] is the derivative-arrow view (a concrete `Arrow` over
//!   `Dual`); the [`DifferentiateExt`] / [`DifferentiateFieldExt`] methods
//!   (`model.derivative(x)`, `field.gradient(&x)`) are its fluent surface.
//! - **Integration is endomorphism iteration.** [`Euler`] and [`Rk4`] build value-level
//!   endo-arrows (`Arrow<In = S, Out = S>`); the [`EndoArrow`] extension adds the value-level
//!   `iterate_n` / `iterate_to_fixpoint` / `iterate_until` — the three modes being fixed
//!   horizon, steady state, and integrate-until-event.
//! - **Quadrature** ([`quadrature`]) is a fold over a closed-form integrand; run over `Dual`
//!   it realizes the Leibniz rule (the naturality of the tangent functor through the fold).
//!
//! Every operator is generic over [`Scalar`] = `Real + Div + FromPrimitive`, so precision is
//! a free parameter (`f32` / `f64` / `Float106`) and duals nest for higher derivatives. A
//! user writes a model once over `Scalar` and applies operators; `Dual`, `ε`, seeding,
//! stepper coefficients, and loops are never visible.

mod extensions;
mod ops;
mod traits;
mod types;

// Extensions
pub use crate::extensions::differentiate_ext::{DifferentiateExt, DifferentiateFieldExt};

// `EndoArrow` lives in the Arrow algebra in `deep_causality_haft`; re-exported here so the
// integration operators (`Euler` / `Rk4`) and existing imports keep resolving `crate::EndoArrow`.
pub use deep_causality_haft::EndoArrow;

// The one free fold-operator.
pub use ops::quadrature::quadrature;

// Traits the user implements.
pub use crate::traits::differentiable_arrow::{DifferentiableArrow, DifferentiableField};

// `Scalar` is a numeric trait (Real + Div + FromPrimitive) and lives in the algebra tower in
// `deep_causality_num`; re-exported here so the operators and `crate::Scalar` imports keep resolving.
pub use deep_causality_num::Scalar;

// Constructed arrows.
pub use crate::types::diff::Diff;
pub use crate::types::euler::Euler;
pub use crate::types::rk4::Rk4;
