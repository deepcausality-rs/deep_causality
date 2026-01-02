/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalTensor, Tensor};
use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, HKT, Monad, Satisfies};

use deep_causality_num::Complex;

/// `TensorConstraint` enforces strict algebraic validity for CausalTensor HKTs.
///
/// This marker uses an **Explicit Whitelist** to ensure only
/// mathematically valid types enter the system.
///
/// # Allowed Types
/// 1. **Data**: Types satisfying `Field` (`f32`, `f64`, `Complex`).
/// 2. **Tensors**: `CausalTensor` itself (nested).
/// 3. **Functions**: `fn` pointers and `Box<dyn Fn>` traits.
///
/// # Exclusions
/// - **Quaternions**: Not supported because `TensorData` requires `Field` (Commutative multiplication),
///   and Quaternions are non-commutative associative rings.
/// - **Octonions**: Not supported because they are non-associative (Alternative Algebra), failing both
///   Field and Ring requirements.
pub struct TensorConstraint;

// Because of Rust's Oprhan Rule,

// --- 1. Data Types (Fields) ---
impl Satisfies<TensorConstraint> for f32 {}
impl Satisfies<TensorConstraint> for f64 {}
impl Satisfies<TensorConstraint> for Complex<f32> {}
impl Satisfies<TensorConstraint> for Complex<f64> {}

// --- 2. Nested Tensors ---
impl<T> Satisfies<TensorConstraint> for CausalTensor<T> {}

// --- 3. Functions ---
impl<A, B> Satisfies<TensorConstraint> for fn(A) -> B {}
impl<A, B> Satisfies<TensorConstraint> for Box<dyn Fn(A) -> B> {}
impl<A, B> Satisfies<TensorConstraint> for Box<dyn Fn(A) -> B + Send> {}
impl<A, B> Satisfies<TensorConstraint> for Box<dyn Fn(A) -> B + Send + Sync> {}

/// `CausalTensorWitness` is a zero-sized type that acts as a Higher-Kinded Type (HKT) witness
/// for the `CausalTensor<T>` type constructor. It allows `CausalTensor` to be used with generic
/// functional programming traits like `Functor`, `Applicative`, `Foldable`, `Monad`, and `CoMonad`
/// from the `deep_causality_haft` crate.
///
/// By incorporating `TensorConstraint`, we support the full algebraic hierarchy including
/// `Monad` (nested tensors) and `Applicative` (functions), enabling composition of tensor operations.
pub struct CausalTensorWitness;

impl HKT for CausalTensorWitness {
    // TensorConstraint covers Data (Ring), Tensors (Ring), and Functions.
    type Constraint = TensorConstraint;

    /// Specifies that `CausalTensorWitness` represents the `CausalTensor<T>` type constructor.
    type Type<T> = CausalTensor<T>;
}

// Implementation of Applicative for CausalTensorWitness
impl Applicative<CausalTensorWitness> for CausalTensorWitness {
    /// Lifts a pure value into a scalar `CausalTensor`.
    fn pure<T>(value: T) -> CausalTensor<T>
    where
        T: Satisfies<TensorConstraint>,
    {
        CausalTensor::from_vec(vec![value], &[])
    }

    /// Applies a function wrapped in a `CausalTensor` (`f_ab`) to a value wrapped in a `CausalTensor` (`f_a`).
    ///
    /// This implementation uses a Zip strategy (element-wise application) to avoid cloning arguments.
    /// Broadcast behavior is supported for Scalar function tensors (len 1).
    /// If lengths differ and function tensor is not scalar, returns an empty tensor.
    fn apply<A, B, Func>(f_ab: CausalTensor<Func>, f_a: CausalTensor<A>) -> CausalTensor<B>
    where
        A: Satisfies<TensorConstraint> + Clone,
        B: Satisfies<TensorConstraint>,
        Func: FnMut(A) -> B,
    {
        // Zip strategy: match elements. Broadcast is not supported without Clone.
        // Assuming shapes match or broadcasting logic handled externally.
        let shape_a = f_a.shape().to_vec();

        let mut funcs = f_ab.into_vec();
        let args = f_a.into_vec();

        let result_data = if funcs.len() == 1 {
            // Scalar broadcast: apply single function to all arguments
            let f = funcs.pop().unwrap();
            args.into_iter().map(f).collect()
        } else if funcs.len() != args.len() {
            // Mismatch: returns empty tensor per test expectation
            return CausalTensor::from_vec(vec![], &[0]);
        } else {
            // Zip strategy
            let mut data = Vec::with_capacity(args.len());
            for (mut f, a) in funcs.into_iter().zip(args.into_iter()) {
                data.push(f(a));
            }
            data
        };

        CausalTensor::from_vec(result_data, &shape_a)
    }
}

// Implementation of Foldable for CausalTensorWitness
impl Foldable<CausalTensorWitness> for CausalTensorWitness {
    /// Folds (reduces) a `CausalTensor` into a single value.
    fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
    where
        A: Satisfies<TensorConstraint>,
        Func: FnMut(B, A) -> B,
    {
        fa.into_vec().into_iter().fold(init, f)
    }
}

// Implementation of Functor for CausalTensorWitness
impl Functor<CausalTensorWitness> for CausalTensorWitness {
    /// Implements the `fmap` operation for `CausalTensor<T>`.
    fn fmap<A, B, Func>(m_a: CausalTensor<A>, f: Func) -> CausalTensor<B>
    where
        A: Satisfies<TensorConstraint>,
        B: Satisfies<TensorConstraint>,
        Func: FnMut(A) -> B,
    {
        let shape = m_a.shape().to_vec(); // Clone shape before moving data
        let new_data = m_a.into_vec().into_iter().map(f).collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

// Implementation of Monad for CausalTensorWitness
impl Monad<CausalTensorWitness> for CausalTensorWitness {
    /// Implements the `bind` (or `flat_map`) operation for `CausalTensor<T>`.
    fn bind<A, B, Func>(m_a: CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        A: Satisfies<TensorConstraint>,
        B: Satisfies<TensorConstraint>,
        Func: FnMut(A) -> CausalTensor<B>,
    {
        let mut result_data = Vec::new();
        // Bind consumes elements and flattens results.
        // This effectively flattens the structure.

        for a in m_a.into_vec() {
            let mb = f(a);
            result_data.extend(mb.into_vec());
        }

        let len = result_data.len();
        CausalTensor::from_vec(result_data, &[len])
    }
}

// Implementation of CoMonad for CausalTensorWitness
impl CoMonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Satisfies<TensorConstraint> + Clone,
    {
        if fa.ndim() == 0 && !fa.is_empty() {
            let v = fa.to_vec();
            v.into_iter().next().unwrap()
        } else if fa.is_empty() {
            panic!("CoMonad::extract cannot be called on an empty CausalTensor.");
        } else {
            let v = fa.to_vec();
            v.into_iter().next().unwrap()
        }
    }

    fn extend<A, B, Func>(fa: &CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(&CausalTensor<A>) -> B,
        A: Satisfies<TensorConstraint> + Clone,
        B: Satisfies<TensorConstraint>,
    {
        let len = fa.len();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let focused_view = fa.shifted_view(i);
                f(&focused_view)
            })
            .collect();

        CausalTensor::from_vec(new_data, fa.shape())
    }
}
