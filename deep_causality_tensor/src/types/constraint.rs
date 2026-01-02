/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::Satisfies;
use deep_causality_num::Complex;

/// Constraint marker for types that implement `TensorData`.
///
/// This marker is used by `CausalTensorWitness` to enforce that the inner type `T`
/// satisfies `TensorData` (i.e., is a valid numeric type for tensor operations).
pub struct TensorDataConstraint;

// Iimplement TensorDataConstraint for allowed types.
// We explicitly exclude integers because they are not Fields.
impl Satisfies<TensorDataConstraint> for f32 {}
impl Satisfies<TensorDataConstraint> for f64 {}
impl Satisfies<TensorDataConstraint> for Complex<f32> {}
impl Satisfies<TensorDataConstraint> for Complex<f64> {}
