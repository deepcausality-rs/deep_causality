/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Forward-mode automatic differentiation over [`Dual`](crate::Dual).
//!
//! These free functions are the user-facing surface of the [`Dual`](crate::Dual)
//! number: they seed the independent variable with `Dual::variable`, run a
//! closure, and read back the infinitesimal (`ε`) channel — so a caller gets exact
//! derivatives without ever touching the dual mechanics.
//!
//! Differentiation here is a **Layer-1 scalar** operation: the chain rule is a ring
//! homomorphism, so the derivative is carried inside the number. Every helper is
//! bounded on `R: Real + Div<Output = R>` — the operations a differentiand needs,
//! and exactly what makes `Dual<R>` itself a [`Real`](crate::Real) so the surface
//! nests (`Dual<Dual<R>>` gives second derivatives) and stays precision-generic
//! over `f32`, `f64`, and `Float106`.
//!
//! - Scalar: [`derivative`](derivative::derivative),
//!   [`value_and_derivative`](derivative::value_and_derivative),
//!   [`second_derivative`](derivative::second_derivative).
//! - Multi-input: [`gradient`](gradient::gradient),
//!   [`directional_derivative`](gradient::directional_derivative),
//!   [`jacobian`](jacobian::jacobian).
//!
//! The complement — integration as a Layer-2 operator over functions — lives in the
//! sibling `autointegration` surface; the two meet via the Leibniz rule.
//! Forward-mode AD is exact only for closed-form differentiands; discrete
//! fields on a mesh remain the province of the topology exterior-calculus operators.

pub mod derivative;
pub mod gradient;
pub mod jacobian;
