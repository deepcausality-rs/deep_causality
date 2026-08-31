/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalMultiField;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use deep_causality_algebra::Field;
use deep_causality_haft::{CoMonad, Functor, HKT, Pure};
use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;

/// HKT witness for `CausalMultiField<T>`.
///
/// The type parameter `T` specifies the concrete coefficient type (e.g., `f64`, `f32`).
/// All HKT operations assume the generic type parameters match `T`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CausalMultiFieldWitness<T>(PhantomData<T>);

impl<T> CausalMultiFieldWitness<T> {
    /// Creates a new witness.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> HKT for CausalMultiFieldWitness<T> {
    type Type<A> = CausalMultiField<A, T>;
}

// ============================================================================
// Functor Implementation
// ============================================================================

impl<T> Functor<CausalMultiFieldWitness<T>> for CausalMultiFieldWitness<T>
where
    T: Field + Copy + Default + PartialOrd,
{
    /// Maps a function over the field coefficients.
    fn fmap<A, C, Func>(fa: CausalMultiField<A, T>, f: Func) -> CausalMultiField<C, T>
    where
        Func: FnMut(A) -> C,
    {
        let CausalMultiField {
            data,
            metric,
            dx,
            shape,
        } = fa;
        let tensor_shape = data.shape().to_vec();
        let mapped: Vec<C> = data.into_vec().into_iter().map(f).collect();
        CausalMultiField {
            data: CausalTensor::from_vec(mapped, &tensor_shape),
            metric,
            dx,
            shape,
        }
    }
}

// ============================================================================
// Pure Implementation
// ============================================================================

impl<T> Pure<CausalMultiFieldWitness<T>> for CausalMultiFieldWitness<T>
where
    T: Field + Copy + Default + PartialOrd,
{
    /// Lifts one coefficient into the smallest field that can hold it.
    ///
    /// `pure` is handed exactly one `A`, and `A` carries no `Clone`, so the context it builds
    /// must have exactly one cell or it cannot be built at all. Two choices make that so: the
    /// algebra is `Cl(0,0,0)`, whose matrix dimension is `1 << 0.div_ceil(2) == 1`, and the
    /// grid is `[1, 1, 1]`. The coefficient tensor is then `[1,1,1,1,1]`, a single cell, and
    /// `value` moves into it.
    ///
    /// Spacing is `T::one()` on each axis, reachable without a `Clone` on `A` because `dx` is
    /// typed by the witness parameter `T` rather than by the coefficient type.
    fn pure<A>(value: A) -> CausalMultiField<A, T> {
        let metric = Metric::from_signature(0, 0, 0);
        debug_assert_eq!(1usize << metric.dimension().div_ceil(2), 1);
        CausalMultiField {
            data: CausalTensor::from_vec(vec![value], &[1, 1, 1, 1, 1]),
            metric,
            dx: [T::one(); 3],
            shape: [1, 1, 1],
        }
    }
}

// `Applicative` and `Monad` remain absent. Both are writable in safe Rust on top of this `pure`,
// but `pure` builds a one-cell field while a general `fa` has many, so `apply(pure(id), fa) == fa`
// and `bind(m, pure) == m` hold only once shape reconciliation is settled. That is issue H1 in
// `openspec/notes/archive/unified_math/unified_math_gaps.md`; both are withheld until it is settled and
// until law tests cover them.

// ============================================================================
// CoMonad Implementation
// ============================================================================

impl<T> CoMonad<CausalMultiFieldWitness<T>> for CausalMultiFieldWitness<T>
where
    T: Field + Copy + Default + PartialOrd,
{
    /// Extracts the "focus" value from the field.
    ///
    /// Returns the first coefficient (scalar part at origin).
    fn extract<A>(fa: &CausalMultiField<A, T>) -> A
    where
        A: Clone,
    {
        // Partial, like indexing a slice: a field with no cells has no focus to extract.
        // `dx` can no longer stand in as a fallback, because it is spacing, not coefficients.
        fa.data()
            .as_slice()
            .first()
            .expect("CoMonad::extract on a CausalMultiField with no coefficients")
            .clone()
    }

    /// Extends a local computation over every focus of the field.
    ///
    /// `f` is applied once per cell, not once for the whole field, so every `C` is produced by
    /// `f` and none is duplicated. That is what keeps this safe without a `Clone` on `C`.
    ///
    /// The focus at `i` is the coefficient tensor rotated left by `i`, which puts cell `i` at
    /// position 0 where [`Self::extract`] reads.
    fn extend<A, C, Func>(fa: &CausalMultiField<A, T>, mut f: Func) -> CausalMultiField<C, T>
    where
        A: Clone,
        Func: FnMut(&CausalMultiField<A, T>) -> C,
    {
        let tensor_shape = fa.data().shape().to_vec();
        let cells = fa.data().as_slice();
        let n = cells.len();

        let data: Vec<C> = (0..n)
            .map(|i| {
                let rotated: Vec<A> = (0..n).map(|k| cells[(k + i) % n].clone()).collect();
                let focus = CausalMultiField {
                    data: CausalTensor::from_vec(rotated, &tensor_shape),
                    metric: fa.metric(),
                    dx: *fa.dx(),
                    shape: *fa.shape(),
                };
                f(&focus)
            })
            .collect();

        CausalMultiField {
            data: CausalTensor::from_vec(data, &tensor_shape),
            metric: fa.metric(),
            dx: *fa.dx(),
            shape: *fa.shape(),
        }
    }
}
