/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalMultiField, Metric};
use deep_causality_haft::{
    Applicative, CoMonad, Functor, HKT, Monad, NoConstraint, Pure, Satisfies,
};
use deep_causality_tensor::LinearAlgebraBackend;
use std::marker::PhantomData;

pub struct CausalMultiFieldWitness<B: LinearAlgebraBackend>(PhantomData<B>);

impl<B: LinearAlgebraBackend> HKT for CausalMultiFieldWitness<B> {
    type Constraint = NoConstraint;
    type Type<T> = CausalMultiField<B, T>;
}

// ----------------------------------------------------------------------------
// Functor
// ----------------------------------------------------------------------------
impl<B: LinearAlgebraBackend> Functor<CausalMultiFieldWitness<B>> for CausalMultiFieldWitness<B> {
    fn fmap<A, C, Func>(fa: CausalMultiField<B, A>, mut f: Func) -> CausalMultiField<B, C>
    where
        A: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        Func: FnMut(A) -> C,
    {
        // Copy generic parameters
        let metric = fa.metric;
        let shape = fa.shape;

        // Map grid spacing (dx)
        let dx_iter = fa.dx.into_iter();
        let mut new_dx_vec: Vec<C> = dx_iter.map(&mut f).collect();
        let dx: [C; 3] = [
            new_dx_vec.remove(0),
            new_dx_vec.remove(0),
            new_dx_vec.remove(0),
        ];

        // Map tensor data
        let data_shape = B::shape(&fa.data);
        let vec_a = B::into_vec(fa.data);
        let vec_c: Vec<C> = vec_a.into_iter().map(&mut f).collect();

        // Reconstruct tensor
        let data = B::create_from_vec(vec_c, &data_shape);

        CausalMultiField {
            data,
            metric,
            dx,
            shape,
        }
    }
}

// ----------------------------------------------------------------------------
// Pure
// ----------------------------------------------------------------------------
impl<B: LinearAlgebraBackend> Pure<CausalMultiFieldWitness<B>> for CausalMultiFieldWitness<B> {
    fn pure<T>(_value: T) -> CausalMultiField<B, T>
    where
        T: Satisfies<NoConstraint>,
    {
        // Pure for Fields is context-dependent.
        panic!(
            "Pure::pure for CausalMultiField requires context (Metric/Shape) and cannot be implemented as a pure function. Use a factory method instead."
        );
    }
}

// ----------------------------------------------------------------------------
// Applicative
// ----------------------------------------------------------------------------
impl<B: LinearAlgebraBackend> Applicative<CausalMultiFieldWitness<B>>
    for CausalMultiFieldWitness<B>
{
    fn apply<A, C, Func>(
        ff: CausalMultiField<B, Func>,
        fa: CausalMultiField<B, A>,
    ) -> CausalMultiField<B, C>
    where
        A: Satisfies<NoConstraint> + Clone,
        C: Satisfies<NoConstraint>,
        Func: Satisfies<NoConstraint> + FnMut(A) -> C,
    {
        // Intersection of fields.
        if ff.shape != fa.shape {
            panic!(
                "Applicative::apply: Shape mismatch {:?} vs {:?}",
                ff.shape, fa.shape
            );
        }
        if ff.metric != fa.metric {
            panic!("Applicative::apply: Metric mismatch");
        }

        let fa_shape = fa.shape;
        let fa_metric = fa.metric;
        let tensor_shape = B::shape(&fa.data);

        // Calculate new dx by applying function dx to argument dx.
        let ff_dx = ff.dx.into_iter();
        let fa_dx = fa.dx.into_iter();

        let mut new_dx_vec: Vec<C> = ff_dx.zip(fa_dx).map(|(mut f, a)| f(a)).collect();
        let dx: [C; 3] = [
            new_dx_vec.remove(0),
            new_dx_vec.remove(0),
            new_dx_vec.remove(0),
        ];

        // Apply on tensor data
        // Capture shape before consuming

        let vec_f = B::into_vec(ff.data);
        let vec_a = B::into_vec(fa.data);

        if vec_f.len() != vec_a.len() {
            panic!("Applicative::apply: Data length mismatch implies internal inconsistency.");
        }

        let vec_c: Vec<C> = vec_f
            .into_iter()
            .zip(vec_a)
            .map(|(mut f, a)| f(a))
            .collect();
        let data = B::create_from_vec(vec_c, &tensor_shape);

        CausalMultiField {
            data,
            metric: fa_metric,
            dx,
            shape: fa_shape,
        }
    }
}

// ----------------------------------------------------------------------------
// Monad
// ----------------------------------------------------------------------------
impl<B: LinearAlgebraBackend> Monad<CausalMultiFieldWitness<B>> for CausalMultiFieldWitness<B> {
    fn bind<A, C, Func>(ma: CausalMultiField<B, A>, mut f: Func) -> CausalMultiField<B, C>
    where
        A: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        Func: FnMut(A) -> CausalMultiField<B, C>,
    {
        // Bind flattens the result.
        let vec_a = B::into_vec(ma.data);
        if vec_a.is_empty() {
            panic!("Cannot bind empty field");
        }

        let mut result_data = Vec::new();
        // We use a safe default via Option.
        let mut captured_dx: Option<[C; 3]> = None;
        let mut captured_metric: Option<Metric> = None;

        for a in vec_a {
            let mc = f(a);
            if captured_dx.is_none() {
                let CausalMultiField {
                    data,
                    metric,
                    dx,
                    shape: _,
                } = mc;
                captured_metric = Some(metric);
                captured_dx = Some(dx);
                result_data.extend(B::into_vec(data));
            } else {
                // For subsequent items, we ignore metadata inconsistencies
                // and just take data.
                result_data.extend(B::into_vec(mc.data));
            }
        }

        let count = result_data.len();
        let new_shape = [count, 1, 1];

        let dx = captured_dx.expect("Bind resulted in no data");
        let metric = captured_metric.unwrap();

        let final_tensor = B::create_from_vec(result_data, &[count]);

        CausalMultiField {
            data: final_tensor,
            metric,
            dx,
            shape: new_shape,
        }
    }
}

// ----------------------------------------------------------------------------
// CoMonad
// ----------------------------------------------------------------------------
impl<B: LinearAlgebraBackend> CoMonad<CausalMultiFieldWitness<B>> for CausalMultiFieldWitness<B> {
    fn extract<A>(fa: &CausalMultiField<B, A>) -> A
    where
        A: Satisfies<NoConstraint> + Clone,
    {
        // Extract value at origin (0,0,0) index [0...]
        let shape = B::shape(&fa.data);
        let zeros = vec![0; shape.len()];
        B::get(&fa.data, &zeros).expect("Empty field in extract")
    }

    fn extend<A, C, Func>(fa: &CausalMultiField<B, A>, mut f: Func) -> CausalMultiField<B, C>
    where
        A: Satisfies<NoConstraint> + Clone,
        C: Satisfies<NoConstraint>,
        Func: FnMut(&CausalMultiField<B, A>) -> C,
    {
        // Extend: Spatial convolution.
        // Helper to lift a scalar 'a' to a field view.
        let mut map_scalar = |val: A| -> C {
            let scalar_shape = vec![1; B::shape(&fa.data).len()];
            let t_data = B::create_from_vec(vec![val.clone()], &scalar_shape);

            // Temp dx: uniform [val, val, val]
            let temp_dx = [val.clone(), val.clone(), val.clone()];
            let field = CausalMultiField {
                data: t_data,
                metric: fa.metric,
                dx: temp_dx,
                shape: [1, 1, 1],
            };
            f(&field)
        };

        let mut d_iter = fa.dx.iter();
        let d0 = map_scalar(d_iter.next().unwrap().clone());
        let d1 = map_scalar(d_iter.next().unwrap().clone());
        let d2 = map_scalar(d_iter.next().unwrap().clone());
        let final_dx = [d0, d1, d2];

        // 2. Convolve Data
        let tensor_shape = B::shape(&fa.data);
        let num_elements: usize = tensor_shape.iter().product();
        let mut result_vec = Vec::with_capacity(num_elements);

        for i in 0..num_elements {
            let shifted_data = B::shifted_view(&fa.data, i);
            let view = CausalMultiField {
                data: shifted_data,
                metric: fa.metric,
                dx: fa.dx.clone(),
                shape: fa.shape,
            };
            result_vec.push(f(&view));
        }

        let new_data = B::create_from_vec(result_vec, &tensor_shape);

        CausalMultiField {
            data: new_data,
            metric: fa.metric,
            dx: final_dx,
            shape: fa.shape,
        }
    }
}
