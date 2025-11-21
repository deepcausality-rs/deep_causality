/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalMultiVector, Metric, MultiVector};
use deep_causality_haft::{Applicative, BoundedComonad, Functor, HKT, Monad};
use deep_causality_num::Zero;

pub struct CausalMultiVectorWitness;

impl HKT for CausalMultiVectorWitness {
    type Type<A> = CausalMultiVector<A>;
}

impl Functor<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn fmap<A, B, F>(fa: CausalMultiVector<A>, f: F) -> CausalMultiVector<B>
    where
        F: FnMut(A) -> B,
    {
        let new_data = fa.data.into_iter().map(f).collect();
        CausalMultiVector {
            data: new_data,
            metric: fa.metric,
        }
    }
}

impl Applicative<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn pure<T>(value: T) -> CausalMultiVector<T> {
        // Dimension 0 = 2^0 = 1 element.
        CausalMultiVector {
            data: vec![value],
            metric: Metric::Euclidean(0),
        }
    }

    fn apply<A, B, Func>(
        f_ab: CausalMultiVector<Func>,
        f_a: CausalMultiVector<A>,
    ) -> CausalMultiVector<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
    {
        // Case 1: Broadcast (Scalar Function applied to Vector)
        if f_ab.data.len() == 1 {
            let func = f_ab.data.into_iter().next().unwrap();
            let new_data = f_a.data.into_iter().map(func).collect();
            return CausalMultiVector {
                data: new_data,
                metric: f_a.metric,
            };
        }

        // Case 2: Element-wise (Zip)
        if f_ab.data.len() != f_a.data.len() {
            panic!(
                "Applicative::apply shape mismatch: {:?} vs {:?}",
                f_ab.metric, f_a.metric
            );
        }

        let new_data = f_ab
            .data
            .into_iter()
            .zip(f_a.data)
            .map(|(mut f, a)| f(a))
            .collect();

        CausalMultiVector {
            data: new_data,
            metric: f_a.metric,
        }
    }
}

impl Monad<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn bind<A, B, Func>(m_a: CausalMultiVector<A>, mut f: Func) -> CausalMultiVector<B>
    where
        Func: FnMut(A) -> CausalMultiVector<B>,
    {
        let mut result_data = Vec::new();
        let mut resulting_metric = Metric::Euclidean(0);
        let mut first_run = true;

        for a in m_a.data {
            let inner_mv = f(a);

            if first_run {
                resulting_metric = m_a.metric.tensor_product(&inner_mv.metric);
                result_data.reserve(inner_mv.data.len() * 10); // Heuristic
                first_run = false;
            }

            result_data.extend(inner_mv.data);
        }

        CausalMultiVector {
            data: result_data,
            metric: resulting_metric,
        }
    }
}

// Implementation of CoMonad for CausalMultiVectorWitness
impl BoundedComonad<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn extract<A>(fa: &CausalMultiVector<A>) -> A
    where
        A: Clone,
    {
        // The scalar part (coefficient of the 1 basis blade, data[0]) is the natural focus.
        // CausalMultiVector guarantees data is never empty.
        fa.data[0].clone()
    }

    fn extend<A, B, Func>(fa: &CausalMultiVector<A>, mut f: Func) -> CausalMultiVector<B>
    where
        Func: FnMut(&CausalMultiVector<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone,
    {
        {
            // Functional transformation:
            // 1. Iterate 0..size
            // 2. Map index -> Shifted View -> Result
            // 3. Collect into Vec
            let new_data: Vec<B> = (0..fa.data.len())
                .map(|i| {
                    // Create the permuted view (Context Shift)
                    let permuted_view = fa.basis_shift(i);
                    // Since map is sequential, it is safe to call mutable f
                    f(&permuted_view)
                })
                .collect();

            // 3. Return MultiVector of same metric as the original MultiVector
            CausalMultiVector::new(new_data, fa.metric).expect("Metric mismatch")
        }
    }
}
