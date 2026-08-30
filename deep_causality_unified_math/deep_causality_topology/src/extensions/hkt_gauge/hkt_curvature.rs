/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `RiemannMap` witness for `CurvatureTensor`.
//!
//! # Why this is not an arity-4 HKT witness
//!
//! The Riemann curvature operator is a multilinear map `R: V ⊗ V ⊗ V → V` over **one** vector
//! space. `CurvatureTensor` previously carried four phantom type parameters so it could be viewed
//! through `HKT4Unbound`, and `RiemannMap::curvature` was generic in all four bounded only by
//! `Satisfies<NoConstraint>`, which admits every type. The implementation then reinterpreted its
//! arguments as `TensorVector<T>` through raw pointers, which made a safe function undefined
//! behaviour for inputs its own signature accepted.
//!
//! The vector space is now an associated type on the witness. The implementation receives the type
//! it needs, no cast is required, and a caller passing anything else is a compile error. See
//! `openspec/notes/hkt_gat/hkt_gat_topology_rewrite.md`.
//!
//! # The calls this now rejects
//!
//! Each of these compiled before the rewrite and reinterpreted its arguments as `TensorVector<T>`.
//!
//! A vector of the right scalar type but the wrong container:
//!
//! ```compile_fail
//! use deep_causality_haft::RiemannMap;
//! use deep_causality_topology::{CurvatureTensor, CurvatureTensorWitness};
//!
//! let ct = CurvatureTensor::<f64>::flat(4);
//! let u = vec![1.0f64, 0.0, 0.0, 0.0];
//! let _ = CurvatureTensorWitness::<f64>::curvature(&ct, &u, &u, &u);
//! ```
//!
//! A type sharing nothing with the vector space:
//!
//! ```compile_fail
//! use deep_causality_haft::RiemannMap;
//! use deep_causality_topology::{CurvatureTensor, CurvatureTensorWitness};
//!
//! struct Zst;
//! let ct = CurvatureTensor::<f64>::flat(4);
//! let _ = CurvatureTensorWitness::<f64>::curvature(&ct, &Zst, &Zst, &Zst);
//! ```
//!
//! The right container carrying the wrong scalar:
//!
//! ```compile_fail
//! use deep_causality_haft::RiemannMap;
//! use deep_causality_topology::{CurvatureTensor, CurvatureTensorWitness, TensorVector};
//!
//! let ct = CurvatureTensor::<f64>::flat(4);
//! let u = TensorVector::<f32>::new(&[1.0, 0.0, 0.0, 0.0]);
//! let _ = CurvatureTensorWitness::<f64>::curvature(&ct, &u, &u, &u);
//! ```
//!
//! And the call that is meant to work:
//!
//! ```
//! use deep_causality_haft::RiemannMap;
//! use deep_causality_topology::{CurvatureTensor, CurvatureTensorWitness, TensorVector};
//!
//! let ct = CurvatureTensor::<f64>::flat(4);
//! let u = TensorVector::<f64>::basis(4, 0);
//! let v = TensorVector::<f64>::basis(4, 1);
//! let w = TensorVector::<f64>::basis(4, 2);
//! let out = CurvatureTensorWitness::<f64>::curvature(&ct, &u, &v, &w);
//! assert!(out.as_slice().iter().all(|x| *x == 0.0));
//! ```

use crate::CurvatureTensor;
use deep_causality_algebra::Field;
use deep_causality_haft::RiemannMap;
use deep_causality_num::Float;
// use deep_causality_tensor::CausalTensor; // Removed unused
use std::marker::PhantomData;

// ============================================================================
// HKT4 Witness
// ============================================================================

/// Witness for the curvature operations on [`CurvatureTensor`].
#[derive(Debug, Clone, Copy, Default)]
pub struct CurvatureTensorWitness<T>(PhantomData<T>);

// ============================================================================
// TensorVector - Concrete Vector Type
// ============================================================================

/// A concrete vector type for curvature and scattering operations.
#[derive(Debug, Clone, PartialEq)]
pub struct TensorVector<T> {
    /// Vector components.
    pub data: Vec<T>,
}

impl<T> TensorVector<T>
where
    T: Field + Copy,
{
    /// Creates a new tensor vector from a slice.
    #[inline]
    pub fn new(data: &[T]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Creates a zero vector of given dimension.
    #[inline]
    pub fn zeros(dim: usize) -> Self {
        Self {
            data: vec![T::zero(); dim],
        }
    }

    /// Creates a basis vector e_i.
    #[inline]
    pub fn basis(dim: usize, i: usize) -> Self {
        let mut data = vec![T::zero(); dim];
        if i < dim {
            data[i] = T::one();
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
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
}

impl<T> From<Vec<T>> for TensorVector<T> {
    fn from(data: Vec<T>) -> Self {
        Self { data }
    }
}

impl<T> From<TensorVector<T>> for Vec<T> {
    fn from(v: TensorVector<T>) -> Self {
        v.data
    }
}

// ============================================================================
// RiemannMap
// ============================================================================

impl<T> RiemannMap for CurvatureTensorWitness<T>
where
    T: Field + Float + Clone + From<f64> + Into<f64> + Copy + PartialOrd,
{
    type Tensor = CurvatureTensor<T>;
    type Vector = TensorVector<T>;

    /// Computes the curvature contraction `R(u,v)w`.
    fn curvature(
        tensor: &CurvatureTensor<T>,
        u: &TensorVector<T>,
        v: &TensorVector<T>,
        w: &TensorVector<T>,
    ) -> TensorVector<T> {
        TensorVector::from(tensor.contract(u.as_slice(), v.as_slice(), w.as_slice()))
    }

    /// Computes S-matrix scattering: two in-states produce two out-states.
    fn scatter(
        interaction: &CurvatureTensor<T>,
        in_1: &TensorVector<T>,
        in_2: &TensorVector<T>,
    ) -> (TensorVector<T>, TensorVector<T>) {
        let dim = interaction.dim();
        let mut out_1 = vec![T::zero(); dim];
        let mut out_2 = vec![T::zero(); dim];
        let point_five: T = <T as From<f64>>::from(0.5);

        for (c, out1_val) in out_1.iter_mut().enumerate() {
            for (d, out2_val) in out_2.iter_mut().enumerate() {
                let mut amplitude = T::zero();
                for a in 0..dim {
                    for b in 0..dim {
                        let val = interaction.get(c, a, b, d);
                        amplitude += val * in_1.data[a] * in_2.data[b];
                    }
                }
                *out1_val += amplitude * point_five;
                *out2_val += amplitude * point_five;
            }
        }

        (TensorVector::from(out_1), TensorVector::from(out_2))
    }
}
