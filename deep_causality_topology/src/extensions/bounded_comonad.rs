use alloc::vec;
use alloc::vec::Vec;
use deep_causality_haft::{BoundedComonad, Functor, HKT};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

use crate::types::topology::Topology;

pub struct CausalTopologyWitness;

impl HKT for CausalTopologyWitness {
    type Type<T> = Topology<T>;
}

use deep_causality_tensor::CausalTensorWitness;

impl Functor<CausalTopologyWitness> for CausalTopologyWitness {
    fn fmap<A, B, F>(fa: Topology<A>, f: F) -> Topology<B>
    where
        F: FnMut(A) -> B,
    {
        let new_data = CausalTensorWitness::fmap(fa.data, f);
        Topology {
            complex: fa.complex,
            grade: fa.grade,
            data: new_data,
            cursor: fa.cursor,
        }
    }
}

// Assuming CausalTopologyWitness is the HKT witness for CausalTopology
impl BoundedComonad<CausalTopologyWitness> for CausalTopologyWitness {
    fn extract<A>(fa: &Topology<A>) -> A
    where
        A: Clone,
    {
        // Use as_slice() instead of get_flat()
        fa.data
            .as_slice()
            .get(fa.cursor)
            .cloned()
            .expect("Cursor OOB")
    }

    fn extend<A, B, Func>(fa: &Topology<A>, mut f: Func) -> Topology<B>
    where
        Func: FnMut(&Topology<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone,
    {
        // Use len() instead of size()
        let size = fa.data.len();
        let mut result_vec = Vec::with_capacity(size);

        // OPTIMIZATION:
        // Instead of allocating a new View struct every iteration,
        // we keep the topology constant and only move the cursor integer.
        // The closure `f` receives a lightweight view.

        for i in 0..size {
            // 1. Create View centered at i
            // We can clone 'fa' cheaply because 'complex' is Arc
            // and 'data' is ref-counted or cloned (depending on tensor impl).
            let mut view = fa.clone_shallow();
            view.cursor = i;

            // 2. Apply Physics
            // The user's function 'f' will likely call view.laplacian() or view.neighbors()
            let val = f(&view);
            result_vec.push(val);
        }

        Topology {
            complex: fa.complex.clone(),
            grade: fa.grade,
            // CausalTensor::new takes data and shape
            data: CausalTensor::new(result_vec, vec![size]).unwrap(),
            cursor: 0,
        }
    }
}
