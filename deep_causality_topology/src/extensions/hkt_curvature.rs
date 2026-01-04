/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT4 witness and RiemannMap implementation for CurvatureTensor.
//!
//! This module provides the `CurvatureTensorWitness` type that enables `CurvatureTensor`
//! to participate in HKT4 abstractions, along with the `RiemannMap` trait implementation.
//!
//! # Implementation Note
//!
//! The `RiemannMap` trait is generic over its input types `A, B, C, D`. However,
//! the logic requires concrete `TensorVector` types.
//!
//! Since we cannot constrain the HKT trait's generic parameters to strict types
//! without changing the core abstraction, and runtime checks via `Any` require
//! static lifetimes that are overly restrictive, this implementation uses
//! **unsafe dispatch**.
//!
//! **SAFETY:** The caller MUST ensure that `A`, `B`, `C`, and `D` are `TensorVector`.
//! Passing any other type will result in Undefined Behavior.
use crate::CurvatureTensor;
use deep_causality_haft::{HKT4Unbound, NoConstraint, RiemannMap, Satisfies};

// ============================================================================
// HKT4 Witness
// ============================================================================

/// HKT4 witness for `CurvatureTensor<A, B, C, D>`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CurvatureTensorWitness;

impl HKT4Unbound for CurvatureTensorWitness {
    // We use NoConstraint because we are using unsafe dispatch and don't rely on Any.
    type Constraint = NoConstraint;
    type Type<A, B, C, D>
        = CurvatureTensor<A, B, C, D>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>;
}

// ============================================================================
// TensorVector - Concrete Vector Type
// ============================================================================

/// A concrete vector type for curvature and scattering operations.
#[derive(Debug, Clone, PartialEq)]
pub struct TensorVector {
    /// Vector components.
    pub data: Vec<f64>,
}

impl TensorVector {
    /// Creates a new tensor vector from a slice.
    #[inline]
    pub fn new(data: &[f64]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Creates a zero vector of given dimension.
    #[inline]
    pub fn zeros(dim: usize) -> Self {
        Self {
            data: vec![0.0; dim],
        }
    }

    /// Creates a basis vector e_i.
    #[inline]
    pub fn basis(dim: usize, i: usize) -> Self {
        let mut data = vec![0.0; dim];
        if i < dim {
            data[i] = 1.0;
        }
        Self { data }
    }

    /// Returns the dimension.
    #[inline]
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Returns a slice of the data.
    #[inline]
    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }
}

impl From<Vec<f64>> for TensorVector {
    fn from(data: Vec<f64>) -> Self {
        Self { data }
    }
}

impl From<TensorVector> for Vec<f64> {
    fn from(v: TensorVector) -> Self {
        v.data
    }
}

// ============================================================================
// RiemannMap Trait Implementation (Unsafe Dispatch)
// ============================================================================

impl RiemannMap<CurvatureTensorWitness> for CurvatureTensorWitness {
    /// Computes curvature contraction R(u,v)w.
    ///
    /// # Safety
    ///
    /// This generic method **unsafely casts** inputs `u`, `v`, `w` to `TensorVector`.
    /// The caller **MUST** ensure A, B, C are `TensorVector`.
    fn curvature<A, B, C, D>(tensor: CurvatureTensor<A, B, C, D>, u: A, v: B, w: C) -> D
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>,
    {
        // SAFETY: We assume the caller respects the implicit contract that A, B, C are TensorVector.
        // We cast the references to &TensorVector to invoke the underlying logic.
        // Since we take arguments by value, we must be careful.
        // TensorVector is a wrapper around Vec<f64>. If A is TensorVector, memory layout is identical.
        //
        // NOTE: This avoids the Any/downcast overhead and static lifetime requirement.
        unsafe {
            let u_ptr = &u as *const A as *const TensorVector;
            let v_ptr = &v as *const B as *const TensorVector;
            let w_ptr = &w as *const C as *const TensorVector;

            // Dispatch to safe implementation
            let result = Self::geodesic_deviation_impl(&tensor, &*u_ptr, &*v_ptr, &*w_ptr);

            // Transmute result to D
            let result_ptr = &result as *const TensorVector as *const D;
            // We read the D out of the result. Since result is a local variable,
            // we must ensure it isn't dropped twice. reading moves it out.
            let ret = std::ptr::read(result_ptr);
            std::mem::forget(result);
            ret
        }
    }

    /// Computes S-matrix scattering: (A, B) → (C, D).
    ///
    /// # Safety
    ///
    /// This method **unsafely casts** inputs to `TensorVector`.
    fn scatter<A, B, C, D>(interaction: CurvatureTensor<A, B, C, D>, in_1: A, in_2: B) -> (C, D)
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>,
    {
        unsafe {
            let in1_ptr = &in_1 as *const A as *const TensorVector;
            let in2_ptr = &in_2 as *const B as *const TensorVector;

            let (out1, out2) = Self::scatter_impl(&interaction, &*in1_ptr, &*in2_ptr);

            // Transmute tuple (TensorVector, TensorVector) to (C, D).
            // Layout of (C, D) might differ from (TensorVector, TensorVector) if C!=D?
            let out1_ptr = &out1 as *const TensorVector as *const C;
            let out2_ptr = &out2 as *const TensorVector as *const D;

            let c = std::ptr::read(out1_ptr);
            let d = std::ptr::read(out2_ptr);

            // Ensure out1, out2 are not dropped since we moved their contents
            std::mem::forget(out1);
            std::mem::forget(out2);

            (c, d)
        }
    }
}

// ============================================================================
// Private saafe implementations.
// ============================================================================

impl CurvatureTensorWitness {
    /// Internal implementation of geodesic deviation.
    fn geodesic_deviation_impl<A, B, C, D>(
        tensor: &CurvatureTensor<A, B, C, D>,
        u: &TensorVector,
        v: &TensorVector,
        w: &TensorVector,
    ) -> TensorVector {
        let result = tensor.contract(u.as_slice(), v.as_slice(), w.as_slice());
        TensorVector::from(result)
    }

    /// Internal implementation of scattering.
    fn scatter_impl<A, B, C, D>(
        tensor: &CurvatureTensor<A, B, C, D>,
        in_1: &TensorVector,
        in_2: &TensorVector,
    ) -> (TensorVector, TensorVector) {
        let dim = tensor.dim();
        let mut out_1 = vec![0.0; dim];
        let mut out_2 = vec![0.0; dim];

        for (c, out1_val) in out_1.iter_mut().enumerate() {
            for (d, out2_val) in out_2.iter_mut().enumerate() {
                let mut amplitude = 0.0;
                for a in 0..dim {
                    for b in 0..dim {
                        amplitude += tensor.get(c, a, b, d) * in_1.data[a] * in_2.data[b];
                    }
                }
                *out1_val += amplitude * 0.5;
                *out2_val += amplitude * 0.5;
            }
        }

        (TensorVector::from(out_1), TensorVector::from(out_2))
    }
}
