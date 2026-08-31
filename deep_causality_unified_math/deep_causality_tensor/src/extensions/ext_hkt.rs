/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;

use crate::CausalTensor;
use crate::traits::tensor::Tensor;
use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, HKT, Monad, Pure};

// ============================================================================
// HKT Witness Implementation
// ============================================================================

/// HKT witness for [`CausalTensor`].
///
/// # Why the element type carries no bound
///
/// `CausalTensor<T>` declares no bound on `T`, and the categorical operations implemented here
/// do no arithmetic: `fmap`, `fold`, `pure`, `bind`, `extend` and `apply` move elements without
/// computing on them. `fmap` maps `CausalTensor<A>` to `CausalTensor<B>` for unrelated `A` and
/// `B`, so a tensor of labels maps as readily as a tensor of `f64`.
///
/// The tensor operations that *do* compute carry their bounds on the impls that need them:
/// `ConjugateScalar` for complex-aware algebra, `RealField + Zero + One + Sum + FromPrimitive`
/// for statistics and reductions, `Clone` for reshaping. Those name real traits, so the compiler
/// enforces them and no downstream crate can satisfy them by declaration.
///
/// See `openspec/notes/archive/hkt_gat/hkt_CausalTensor.md` for the measurement behind this.
///
/// # The elementwise applicative lives elsewhere
///
/// This witness carries `Monad`, so its `apply` is pinned to the cartesian product by the
/// applicative/monad coherence law. [`ZipTensorWitness`](crate::ZipTensorWitness) stands for the
/// same container and supplies the elementwise reading instead.
pub struct CausalTensorWitness;

impl HKT for CausalTensorWitness {
    type Type<T> = CausalTensor<T>;
}

// ============================================================================
// Algebraic Implementations
// ============================================================================

impl Functor<CausalTensorWitness> for CausalTensorWitness {
    fn fmap<A, B, Func>(m_a: CausalTensor<A>, f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> B,
    {
        let shape = m_a.shape().to_vec();
        let new_data: Vec<B> = m_a.into_vec().into_iter().map(f).collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

impl Foldable<CausalTensorWitness> for CausalTensorWitness {
    fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.into_vec().into_iter().fold(init, f)
    }
}

impl Pure<CausalTensorWitness> for CausalTensorWitness {
    fn pure<T>(value: T) -> CausalTensor<T> {
        CausalTensor::from_vec(vec![value], &[])
    }
}

impl Monad<CausalTensorWitness> for CausalTensorWitness {
    /// # One element, two shapes
    ///
    /// A container holding a single element can carry any shape whose extents multiply to one:
    /// `[]`, `[1]`, `[1, 1]`. On such an input the two identity laws want different answers.
    /// `bind(m, pure) == m` wants the *input's* shape, and `bind(pure(a), f) == f(a)` wants the
    /// shape of what `f` returned. When those differ, no rule satisfies both.
    ///
    /// Right identity wins here, because `bind(m, pure) == m` is the law a caller is most likely
    /// to rely on. The consequence is narrow and worth naming: for a one-function `ff` of shape
    /// `[1]` against a one-element `fa` of shape `[1, 1]`, `apply(ff, fa)` reports `[1, 1]` while
    /// `bind(ff, |f| fmap(fa, f))` reports `[1]`. The data agrees; only the shape does not, and
    /// only when both sides hold exactly one element under different shapes.
    fn bind<A, B, Func>(m_a: CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> <Self as HKT>::Type<B>,
    {
        let shape = m_a.shape().to_vec();
        let count = m_a.len();
        let mut result_data = Vec::with_capacity(count);
        let mut every_step_gave_one = true;
        let mut only_step_shape: Option<Vec<usize>> = None;
        for a in m_a.into_vec() {
            let mb = f(a);
            if mb.len() != 1 {
                every_step_gave_one = false;
            }
            if count == 1 {
                only_step_shape = Some(mb.shape().to_vec());
            }
            result_data.extend(mb.into_vec());
        }
        let len = result_data.len();

        // A one-element input is exactly the left identity case, `bind(pure(a), f) == f(a)`, so
        // the answer is `f(a)` including its shape. This is also what keeps `bind` coherent with
        // `apply`: `apply` broadcasts a single function across `fa` and reports `fa`'s shape, and
        // `bind(ff, |f| fmap(fa, f))` over a one-function `ff` has to agree with it.
        if count == 1 && len != 1 {
            let s = only_step_shape.expect("count is one, so the loop ran once");
            return CausalTensor::from_vec(result_data, &s);
        }

        // Otherwise the shape survives only when the map was genuinely one-in-one-out. Testing
        // `len == count` instead is too weak: `f` returning two elements for one input and none
        // for another sums to the same total while being a concat-map, and would stamp the
        // input's shape onto a result that no longer has it.
        if every_step_gave_one {
            CausalTensor::from_vec(result_data, &shape)
        } else {
            CausalTensor::from_vec(result_data, &[len])
        }
    }
}

impl CoMonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Clone,
    {
        fa.as_slice()
            .first()
            .cloned()
            .expect("CoMonad::extract cannot be called on an empty CausalTensor.")
    }

    fn extend<A, B, Func>(fa: &CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(&CausalTensor<A>) -> B,
        A: Clone,
    {
        let len = fa.len();
        let shape = fa.shape().to_vec();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let view = fa.shifted_view(i);
                f(&view)
            })
            .collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

impl Applicative<CausalTensorWitness> for CausalTensorWitness {
    /// The cartesian applicative: every function against every argument, function-major.
    ///
    /// This witness also carries [`Monad`], which owes the coherence law
    /// `apply(ff, fa) == bind(ff, |f| fmap(fa, f))`. `bind` runs the continuation once per
    /// function, so coherence admits only the cartesian product. The elementwise reading is a
    /// genuinely different applicative and lives on [`ZipTensorWitness`](crate::ZipTensorWitness),
    /// which carries no `Monad` and therefore owes no coherence.
    fn apply<A, B, Func>(f_ab: CausalTensor<Func>, f_a: CausalTensor<A>) -> CausalTensor<B>
    where
        A: Clone,
        Func: FnMut(A) -> B,
    {
        let shape = f_a.shape().to_vec();
        let funcs = f_ab.into_vec();
        let args = f_a.into_vec();
        let n_funcs = funcs.len();

        let mut data: Vec<B> = Vec::with_capacity(n_funcs.saturating_mul(args.len()));
        for mut f in funcs {
            for a in args.iter() {
                data.push(f(a.clone()));
            }
        }

        // One function leaves the argument count unchanged, so the argument's shape survives and
        // `apply(pure(id), t) == t` holds. Beyond that the cartesian product has no shape to
        // inherit, and the flat `[len]` is both honest and what `bind` yields for the same inputs.
        if n_funcs == 1 {
            CausalTensor::from_vec(data, &shape)
        } else {
            let len = data.len();
            CausalTensor::from_vec(data, &[len])
        }
    }
}
