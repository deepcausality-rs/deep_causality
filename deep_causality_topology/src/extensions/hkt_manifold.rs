/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Manifold;
use alloc::vec::Vec;
use deep_causality_haft::{BoundedComonad, Functor, HKT};
use deep_causality_num::Zero;
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

pub struct ManifoldWitness;

impl HKT for ManifoldWitness {
    type Type<T> = Manifold<T>;
}

impl Functor<ManifoldWitness> for ManifoldWitness {
    fn fmap<A, B, F>(fa: Manifold<A>, f: F) -> Manifold<B>
    where
        F: FnMut(A) -> B,
    {
        let new_data = CausalTensorWitness::fmap(fa.data, f);
        Manifold {
            complex: fa.complex,
            data: new_data,
            metric: None,
            cursor: fa.cursor,
        }
    }
}

impl BoundedComonad<ManifoldWitness> for ManifoldWitness {
    fn extract<A>(fa: &Manifold<A>) -> A
    where
        A: Clone,
    {
        fa.data
            .as_slice()
            .get(fa.cursor)
            .cloned()
            .expect("Cursor OOB")
    }

    fn extend<A, B, Func>(fa: &Manifold<A>, mut f: Func) -> Manifold<B>
    where
        Func: FnMut(&Manifold<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone,
    {
        let size = fa.data.len();
        let mut result_vec = Vec::with_capacity(size);

        for i in 0..size {
            let mut view = fa.clone_shallow();
            view.cursor = i;

            let val = f(&view);
            result_vec.push(val);
        }

        let new_data =
            CausalTensor::new(result_vec, vec![size]).expect("Data tensor creation should succeed");

        Manifold {
            complex: fa.complex.clone(),
            data: new_data,
            metric: None,
            cursor: 0,
        }
    }
}
