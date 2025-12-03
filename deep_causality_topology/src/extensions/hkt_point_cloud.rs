/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PointCloud;
use alloc::vec::Vec;
use deep_causality_haft::{BoundedComonad, Functor, HKT};
use deep_causality_num::Zero;
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

pub struct PointCloudWitness;

impl HKT for PointCloudWitness {
    type Type<T> = PointCloud<T>;
}

impl Functor<PointCloudWitness> for PointCloudWitness {
    fn fmap<A, B, F>(fa: PointCloud<A>, f: F) -> PointCloud<B>
    where
        F: FnMut(A) -> B,
    {
        let new_metadata = CausalTensorWitness::fmap(fa.metadata, f);
        PointCloud {
            points: fa.points.clone(),
            metadata: new_metadata,
            cursor: fa.cursor,
        }
    }
}

impl BoundedComonad<PointCloudWitness> for PointCloudWitness {
    fn extract<A>(fa: &PointCloud<A>) -> A
    where
        A: Clone, // As per BoundedComonad trait
    {
        fa.metadata
            .as_slice()
            .get(fa.cursor)
            .cloned()
            .expect("Cursor OOB")
    }

    fn extend<A, B, Func>(fa: &PointCloud<A>, mut f: Func) -> PointCloud<B>
    where
        Func: FnMut(&PointCloud<A>) -> B,
        A: Zero + Copy + Clone, // As per BoundedComonad trait
        B: Zero + Copy + Clone, // As per BoundedComonad trait
    {
        let size = fa.len();
        let mut result_vec = Vec::with_capacity(size);

        for i in 0..size {
            let mut view = fa.clone_shallow();
            view.cursor = i;

            let val = f(&view);
            result_vec.push(val);
        }

        let new_metadata = CausalTensor::new(result_vec, vec![size])
            .expect("Metadata tensor creation should succeed");

        PointCloud {
            points: fa.points.clone(),
            metadata: new_metadata,
            cursor: 0,
        }
    }
}
