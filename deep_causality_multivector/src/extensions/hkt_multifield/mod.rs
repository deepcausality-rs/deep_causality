/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Higher-Kinded Type (HKT) operations for CausalMultiField.
//!
//! This module provides functional programming operations for `CausalMultiField<B, T>`
//! that mirror the haft traits (Functor, Applicative, Monad, Comonad).
//!
//! # Design Note
//!
//! `CausalMultiField<B, T>` requires `T: TensorData` at the type level, which is
//! incompatible with the haft HKT trait's unconditional GAT. Therefore, we provide
//! these operations as inherent methods on the witness type rather than trait impls.
//!
//! This is similar to how `CausalMultiVector<T>` implements HKT traits, but adapted
//! for the additional backend type parameter and TensorData constraints.
//!
//! # Provided Operations
//!
//! - **`fmap`**: Map a function over all coefficients (Functor)
//! - **`pure`**: Lift a value into a minimal field (Applicative)
//! - **`apply`**: Apply a field of functions to a field of values (Applicative)
//! - **`bind`**: Flatmap operation (Monad)
//! - **`extract`**: Get the focal value (Comonad)
//! - **`extend`**: Apply context-dependent function at each position (Comonad)
//!
use crate::{CausalMultiField, CausalMultiVector, Metric};
use deep_causality_num::Zero;
use deep_causality_tensor::{CpuBackend, LinearAlgebraBackend, TensorData};
use std::marker::PhantomData;

/// Witness type for `CausalMultiField<B, T>` with backend `B` fixed.
///
/// This provides functional programming operations for fields by fixing
/// the backend at the type level and varying the scalar type.
///
/// # Example
///
/// ```rust,ignore
/// use deep_causality_multivector::extensions::hkt_multifield::CpuMultiFieldWitness;
///
/// // Map a function over all field coefficients
/// let doubled = CpuMultiFieldWitness::fmap(field, |x| x * 2.0);
///
/// // Extract the focal value
/// let focus = CpuMultiFieldWitness::extract(&field);
/// ```
pub struct CausalMultiFieldWitness<B: LinearAlgebraBackend> {
    _phantom: PhantomData<B>,
}

// =============================================================================
// Functor: fmap
// =============================================================================

impl<B: LinearAlgebraBackend> CausalMultiFieldWitness<B> {
    /// Maps a function over all coefficients in the field.
    ///
    /// This is the Functor `fmap` operation: `F<A> -> (A -> B) -> F<B>`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let doubled = CpuMultiFieldWitness::fmap(field, |x| x * 2.0);
    /// ```
    pub fn fmap<A, NewT, F>(fa: CausalMultiField<B, A>, mut f: F) -> CausalMultiField<B, NewT>
    where
        F: FnMut(A) -> NewT,
        A: TensorData
            + Clone
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = A>
            + std::ops::Div<Output = A>,
        NewT: TensorData
            + Clone
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = NewT>
            + std::ops::Div<Output = NewT>,
        B: crate::types::multifield::gamma::GammaProvider<A>
            + crate::types::multifield::gamma::GammaProvider<NewT>,
    {
        let mvs = fa.to_coefficients();
        let shape = *fa.shape();
        let dx_a = fa.dx();

        let new_mvs: Vec<_> = mvs
            .into_iter()
            .map(|mv| {
                let new_data: Vec<NewT> = mv.data.into_iter().map(&mut f).collect();
                CausalMultiVector {
                    data: new_data,
                    metric: mv.metric,
                }
            })
            .collect();

        let dx_new = [f(dx_a[0]), f(dx_a[1]), f(dx_a[2])];
        CausalMultiField::from_coefficients(&new_mvs, shape, dx_new)
    }
}

// =============================================================================
// Applicative: pure, apply
// =============================================================================

impl<B: LinearAlgebraBackend> CausalMultiFieldWitness<B> {
    /// Lifts a value into a minimal 1x1x1 field with scalar metric.
    ///
    /// This is the Applicative `pure` operation: `A -> F<A>`
    pub fn pure<T>(value: T) -> CausalMultiField<B, T>
    where
        T: TensorData
            + Clone
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = T>
            + std::ops::Div<Output = T>,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        let mv = CausalMultiVector {
            data: vec![value],
            metric: Metric::Euclidean(0),
        };
        CausalMultiField::from_coefficients(&[mv], [1, 1, 1], [value, value, value])
    }

    /// Applies a field of functions to a field of values (element-wise).
    ///
    /// This is the Applicative `apply` operation: `F<A -> B> -> F<A> -> F<B>`
    ///
    /// Note: Requires `Func: TensorData` which is rarely satisfied by closures.
    /// Consider using `fmap` for most transformations.
    pub fn apply<A, NewT, Func>(
        f_ab: CausalMultiField<B, Func>,
        f_a: CausalMultiField<B, A>,
    ) -> CausalMultiField<B, NewT>
    where
        Func: FnMut(A) -> NewT
            + TensorData
            + Clone
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = Func>
            + std::ops::Div<Output = Func>,
        A: Clone
            + TensorData
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = A>
            + std::ops::Div<Output = A>,
        NewT: TensorData
            + Clone
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = NewT>
            + std::ops::Div<Output = NewT>,
        B: crate::types::multifield::gamma::GammaProvider<Func>
            + crate::types::multifield::gamma::GammaProvider<A>
            + crate::types::multifield::gamma::GammaProvider<NewT>,
    {
        assert_eq!(f_ab.shape(), f_a.shape(), "Shape mismatch in apply");

        let funcs = f_ab.to_coefficients();
        let values = f_a.to_coefficients();
        let shape = *f_a.shape();

        let new_mvs: Vec<_> = funcs
            .into_iter()
            .zip(values)
            .map(|(func_mv, val_mv)| {
                let new_data: Vec<NewT> = func_mv
                    .data
                    .into_iter()
                    .zip(val_mv.data)
                    .map(|(mut func, val)| func(val))
                    .collect();
                CausalMultiVector {
                    data: new_data,
                    metric: val_mv.metric,
                }
            })
            .collect();

        let dx_new = [NewT::default(); 3];
        CausalMultiField::from_coefficients(&new_mvs, shape, dx_new)
    }
}

// =============================================================================
// Monad: bind
// =============================================================================

impl<B: LinearAlgebraBackend> CausalMultiFieldWitness<B> {
    /// Binds a function that returns a field to each scalar, flattening the result.
    ///
    /// This is the Monad `bind` operation: `F<A> -> (A -> F<B>) -> F<B>`
    ///
    /// The resulting field has shape = outer_shape * inner_shape.
    pub fn bind<A, NewT, Func>(
        m_a: CausalMultiField<B, A>,
        mut f: Func,
    ) -> CausalMultiField<B, NewT>
    where
        Func: FnMut(A) -> CausalMultiField<B, NewT>,
        A: TensorData
            + Clone
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = A>
            + std::ops::Div<Output = A>,
        NewT: TensorData
            + Clone
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = NewT>
            + std::ops::Div<Output = NewT>,
        B: crate::types::multifield::gamma::GammaProvider<A>
            + crate::types::multifield::gamma::GammaProvider<NewT>,
    {
        let mvs = m_a.to_coefficients();
        let mut all_inner_mvs = Vec::new();
        let mut inner_shape = [0usize; 3];
        let mut inner_dx = [NewT::default(); 3];
        let mut first = true;

        for mv in mvs {
            for a in mv.data {
                let inner_field = f(a);
                if first {
                    inner_shape = *inner_field.shape();
                    inner_dx = *inner_field.dx();
                    first = false;
                }
                all_inner_mvs.extend(inner_field.to_coefficients());
            }
        }

        let outer_shape = m_a.shape();
        let new_shape = [
            outer_shape[0] * inner_shape[0],
            outer_shape[1] * inner_shape[1],
            outer_shape[2] * inner_shape[2],
        ];

        CausalMultiField::from_coefficients(&all_inner_mvs, new_shape, inner_dx)
    }
}

// =============================================================================
// Comonad: extract, extend
// =============================================================================

impl<B: LinearAlgebraBackend> CausalMultiFieldWitness<B> {
    /// Extracts the scalar (grade-0) component of the first cell.
    ///
    /// This is the Comonad `extract` operation: `F<A> -> A`
    ///
    /// The "focus" is the scalar component of cell [0,0,0].
    pub fn extract<A>(fa: &CausalMultiField<B, A>) -> A
    where
        A: Clone
            + TensorData
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = A>
            + std::ops::Div<Output = A>,
        B: crate::types::multifield::gamma::GammaProvider<A>,
    {
        let mvs = fa.to_coefficients();
        mvs[0].data[0]
    }

    /// Applies a context-dependent function at each grid position.
    ///
    /// This is the Comonad `extend` operation: `(F<A> -> B) -> F<A> -> F<B>`
    ///
    /// For each cell, `f` receives the entire field and produces a value.
    /// A full implementation would provide shifted views per cell.
    pub fn extend<A, NewT, Func>(
        fa: &CausalMultiField<B, A>,
        mut f: Func,
    ) -> CausalMultiField<B, NewT>
    where
        Func: FnMut(&CausalMultiField<B, A>) -> NewT,
        A: Clone
            + Copy
            + Zero
            + TensorData
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = A>
            + std::ops::Div<Output = A>,
        NewT: Clone
            + Copy
            + Zero
            + TensorData
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = NewT>
            + std::ops::Div<Output = NewT>,
        B: crate::types::multifield::gamma::GammaProvider<A>
            + crate::types::multifield::gamma::GammaProvider<NewT>,
    {
        let shape = *fa.shape();
        let num_cells = fa.num_cells();

        let mut result_data = Vec::with_capacity(num_cells);
        for _i in 0..num_cells {
            result_data.push(f(fa));
        }

        let new_mvs: Vec<_> = result_data
            .into_iter()
            .map(|val| CausalMultiVector {
                data: vec![val],
                metric: Metric::Euclidean(0),
            })
            .collect();

        let dx_new = [NewT::default(); 3];
        CausalMultiField::from_coefficients(&new_mvs, shape, dx_new)
    }
}

// =============================================================================
// Type aliases for common backends
// =============================================================================

/// HKT witness for CausalMultiField with CPU backend.
pub type CpuMultiFieldWitness = CausalMultiFieldWitness<CpuBackend>;

/// HKT witness for CausalMultiField with MLX backend (Apple Silicon only).
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type MlxMultiFieldWitness = CausalMultiFieldWitness<deep_causality_tensor::MlxBackend>;
