/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalTensor;
use deep_causality_haft::{Foldable, Functor, HKT, Satisfies};
use deep_causality_num::Complex;

/// `TensorConstraint` enforces strict algebraic bounds on the types allowed within a CausalTensor HKT.
///
/// This corresponds to **Tier 4: TensorDataConstraint** in the spec.
/// It limits usage to types that are mathematically valid for tensor physics (Fields, Rings).
///
/// **Note:** Because this constraint excludes closures (`Fn`), this witness CANNOT implement
/// `Applicative` or `Monad`. It is restricted to `Functor`, `Foldable`, and `CoMonad`.
pub struct TensorConstraint;

// ============================================================================
// Allowed Physics Types (Whitelist)
// ============================================================================

impl Satisfies<TensorConstraint> for f32 {}
impl Satisfies<TensorConstraint> for f64 {}
impl Satisfies<TensorConstraint> for Complex<f32> {}
impl Satisfies<TensorConstraint> for Complex<f64> {}

impl Satisfies<TensorConstraint> for i8 {}
impl Satisfies<TensorConstraint> for i16 {}
impl Satisfies<TensorConstraint> for i32 {}
impl Satisfies<TensorConstraint> for i64 {}
impl Satisfies<TensorConstraint> for i128 {}

impl Satisfies<TensorConstraint> for u8 {}
impl Satisfies<TensorConstraint> for u16 {}
impl Satisfies<TensorConstraint> for u32 {}
impl Satisfies<TensorConstraint> for u64 {}
impl Satisfies<TensorConstraint> for u128 {}

impl Satisfies<TensorConstraint> for usize {}
impl Satisfies<TensorConstraint> for isize {}

// Allow nested tensors (Recursive structures)
impl<T> Satisfies<TensorConstraint> for CausalTensor<T> {}

// ============================================================================
// Strict HKT Witness
// ============================================================================

pub struct StrictCausalTensorWitness;

impl HKT for StrictCausalTensorWitness {
    type Constraint = TensorConstraint;
    type Type<T>
        = CausalTensor<T>
    where
        T: Satisfies<TensorConstraint>;
}

// ============================================================================
// Algebraic Implementations
// ============================================================================

impl Functor<StrictCausalTensorWitness> for StrictCausalTensorWitness {
    fn fmap<A, B, Func>(m_a: CausalTensor<A>, f: Func) -> CausalTensor<B>
    where
        A: Satisfies<TensorConstraint>,
        B: Satisfies<TensorConstraint>,
        Func: FnMut(A) -> B,
    {
        let shape = m_a.shape().to_vec();
        let new_data: Vec<B> = m_a.into_vec().into_iter().map(f).collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

impl Foldable<StrictCausalTensorWitness> for StrictCausalTensorWitness {
    fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
    where
        A: Satisfies<TensorConstraint>,
        Func: FnMut(B, A) -> B,
    {
        fa.into_vec().into_iter().fold(init, f)
    }
}

// CoMonad implementation is omitted for Strict Mode due to E0276/E0277 errors
