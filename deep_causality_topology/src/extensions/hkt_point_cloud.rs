/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PointCloud;
use deep_causality_haft::{CoMonad, Functor, HKT, NoConstraint, Satisfies};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

pub struct PointCloudWitness;

impl HKT for PointCloudWitness {
    type Constraint = NoConstraint;
    type Type<T>
        = PointCloud<T>
    where
        T: Satisfies<NoConstraint>;
}

impl Functor<PointCloudWitness> for PointCloudWitness {
    fn fmap<A, B, F>(fa: PointCloud<A>, f: F) -> PointCloud<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
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

impl CoMonad<PointCloudWitness> for PointCloudWitness {
    fn extract<A>(fa: &PointCloud<A>) -> A
    where
        A: Satisfies<NoConstraint> + Clone,
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
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
    {
        let size = fa.len();
        let shape = fa.metadata.shape().to_vec();
        let mut result_vec = Vec::with_capacity(size);

        for i in 0..size {
            let mut view = fa.clone_shallow();
            view.cursor = i;

            let val = f(&view);
            result_vec.push(val);
        }

        let new_metadata = CausalTensor::from_vec(result_vec, &shape);

        PointCloud {
            points: fa.points.clone(),
            metadata: new_metadata,
            cursor: 0,
        }
    }
}
