/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CsrMatrix;
use deep_causality_haft::{Applicative, BoundedAdjunction, BoundedComonad, Foldable, Functor, HKT, Monad};
use deep_causality_num::Zero;
use std::ops::{Add, Mul};

/// `CsrMatrixWitness` is a zero-sized type that acts as a Higher-Kinded Type (HKT) witness
/// for the `CsrMatrix<T>` type constructor.
///
/// It enables `CsrMatrix` to participate in the unified monadic interface of DeepCausality,
/// allowing composition with Tensors, Multivectors, and Monadic Effects.
pub struct CsrMatrixWitness;

impl HKT for CsrMatrixWitness {
    /// Specifies that `CsrMatrixWitness` represents the `CsrMatrix<T>` type constructor.
    type Type<T> = CsrMatrix<T>;
}

// ----------------------------------------------------------------------------
// Functor
// ----------------------------------------------------------------------------
impl Functor<CsrMatrixWitness> for CsrMatrixWitness {
    fn fmap<A, B, Func>(m_a: CsrMatrix<A>, f: Func) -> CsrMatrix<B>
    where
        Func: FnMut(A) -> B,
    {
        // For sparse matrices, we typically only map the stored values.
        // Implicit zeros remain implicit zeros (assuming f(0) is ignored or 0->0).
        let new_values: Vec<B> = m_a.values.into_iter().map(f).collect();
        
        CsrMatrix {
            row_indices: m_a.row_indices,
            col_indices: m_a.col_indices,
            values: new_values,
            shape: m_a.shape,
        }
    }
}

// ----------------------------------------------------------------------------
// Foldable
// ----------------------------------------------------------------------------
impl Foldable<CsrMatrixWitness> for CsrMatrixWitness {
    fn fold<A, B, Func>(fa: CsrMatrix<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        // Fold over stored non-zero values
        fa.values.into_iter().fold(init, f)
    }
}

// ----------------------------------------------------------------------------
// Applicative
// ----------------------------------------------------------------------------
impl Applicative<CsrMatrixWitness> for CsrMatrixWitness {
    fn pure<T>(value: T) -> CsrMatrix<T> {
        // Creates a 1x1 matrix containing the value at (0,0)
        CsrMatrix {
            row_indices: vec![0, 1],
            col_indices: vec![0],
            values: vec![value],
            shape: (1, 1),
        }
    }

    fn apply<A, B, Func>(f_ab: CsrMatrix<Func>, f_a: CsrMatrix<A>) -> CsrMatrix<B>
    where
        Func: FnMut(A) -> B,
    {
        // Broadcast logic:
        // If f_ab is a scalar matrix (1x1), map the function over f_a.
        if f_ab.shape == (1, 1) && f_ab.values.len() == 1 {
            let func = f_ab.values.into_iter().next().unwrap();
            let new_values = f_a.values.into_iter().map(func).collect();
            CsrMatrix {
                row_indices: f_a.row_indices,
                col_indices: f_a.col_indices,
                values: new_values,
                shape: f_a.shape,
            }
        } else {
             // For general matrices, creating an empty matrix is the safest fallback
             // consistent with other HKT implementations where broadcast isn't possible.
             CsrMatrix::new()
        }
    }
}

// ----------------------------------------------------------------------------
// Monad
// ----------------------------------------------------------------------------
impl Monad<CsrMatrixWitness> for CsrMatrixWitness {
    fn bind<A, B, Func>(m_a: CsrMatrix<A>, mut f: Func) -> CsrMatrix<B>
    where
        Func: FnMut(A) -> CsrMatrix<B>,
    {
        // Monadic bind for a sparse container:
        // Map each non-zero value 'a' to a matrix 'M_b'.
        // Then flatten/concatenate these results.
        // Similar to CausalTensor, we linearize the output into a single row vector
        // or just a 1 x N matrix to preserve the data.
        
        let result_values: Vec<B> = m_a
            .values
            .into_iter()
            .flat_map(|val_a| f(val_a).values.into_iter())
            .collect();
            
        let count = result_values.len();
        
        // Construct a 1 x N matrix (Row Vector)
        CsrMatrix {
            row_indices: vec![0, count],
            col_indices: (0..count).collect(),
            values: result_values,
            shape: (1, count),
        }   
    }
}

// ----------------------------------------------------------------------------
// Comonad
// ----------------------------------------------------------------------------
impl BoundedComonad<CsrMatrixWitness> for CsrMatrixWitness {
    fn extract<A>(fa: &CsrMatrix<A>) -> A
    where
        A: Clone,
    {
        if !fa.values.is_empty() {
             fa.values[0].clone()
        } else {
             panic!("Comonad::extract cannot be called on an empty CsrMatrix");
        }
    }

    fn extend<A, B, Func>(fa: &CsrMatrix<A>, mut f: Func) -> CsrMatrix<B>
    where
        Func: FnMut(&CsrMatrix<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone,
    {
        // Simplified extension: apply f to the whole matrix for each non-zero element.
        // In a true spatial comonad, we'd shift the view. For general sparse matrices,
        // shifting is non-trivial. We map over indices 0..len(values) to simulate interaction.
        let new_values: Vec<B> = (0..fa.values.len())
            .map(|_| f(fa)) 
            .collect();
            
        CsrMatrix {
            row_indices: fa.row_indices.clone(),
            col_indices: fa.col_indices.clone(),
            values: new_values,
            shape: fa.shape,
        }
    }
}

// ----------------------------------------------------------------------------
// Adjunction
// ----------------------------------------------------------------------------
impl BoundedAdjunction<CsrMatrixWitness, CsrMatrixWitness, (usize, usize)>
    for CsrMatrixWitness
{
    fn left_adjunct<A, B, F>(ctx: &(usize, usize), a: A, f: F) -> CsrMatrix<B>
    where
        F: Fn(CsrMatrix<A>) -> B,
        A: Clone + Zero + Copy + PartialEq,
        B: Clone,
    {
        let m_m_a = Self::unit(ctx, a);
        <Self as Functor<Self>>::fmap(m_m_a, f)
    }

    fn right_adjunct<A, B, F>(ctx: &(usize, usize), la: CsrMatrix<A>, f: F) -> B
    where
        F: FnMut(A) -> CsrMatrix<B>,
        A: Clone + Zero,
        B: Clone + Zero + Add<Output = B> + Mul<Output = B>,
    {
        let mapped = <Self as Functor<Self>>::fmap(la, f);
        Self::counit(ctx, mapped)
    }

    fn unit<A>(ctx: &(usize, usize), a: A) -> CsrMatrix<CsrMatrix<A>>
    where
        A: Clone + Zero + Copy + PartialEq,
    {
        // Inner matrix uses the context shape (usually 1x1 for scalar unit)
        // Or if ctx is (1,1), creates [[a]]
        let inner = CsrMatrix::from_triplets(ctx.0, ctx.1, &[(0, 0, a)])
            .unwrap_or_else(|_| CsrMatrix::new());
            
        // Outer matrix is 1x1 wrapper
        CsrMatrix {
            row_indices: vec![0, 1],
            col_indices: vec![0],
            values: vec![inner],
            shape: (1, 1),
        }
    }

    fn counit<B>(_ctx: &(usize, usize), lrb: CsrMatrix<CsrMatrix<B>>) -> B
    where
        B: Clone + Zero + Add<Output = B> + Mul<Output = B>,
    {
        let flattened = <Self as Monad<Self>>::bind(lrb, |x| x);
        <Self as BoundedComonad<Self>>::extract(&flattened)
    }
}
