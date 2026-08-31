/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT witnesses for `Manifold`.
//!
//! Under the Option 2C design (`CellularComplex::Metric` is a plain associated type;
//! `Manifold<K, F>` has no struct-level bound on `F`), the witness types implement the
//! full `deep_causality_haft` trait surface on stable Rust: `HKT`, `Functor`, `Foldable`,
//! `Pure`, `Monad`, `CoMonad`, and (for the simplicial witness) `Applicative`. All impls
//! use `T: Satisfies<NoConstraint>` only — no `RealField` bounds at the witness layer.
//!
//! Cross-algebra composition is supported by design: `F` may be a scalar (`f64`, `f32`,
//! `Float106`), a multivector from `deep_causality_multivector`, a tensor from
//! `deep_causality_tensor`, a dual number for automatic differentiation, or any other
//! algebraic value type that flows through `CausalTensor<F>`.

use crate::traits::cellular_complex::CellularComplex;
use crate::traits::chain_complex::ChainComplex;
use crate::{Manifold, SimplicialComplex};
use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, HKT, Monad, Pure};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use std::marker::PhantomData;

// ============================================================================
// PART 1: Simplicial witness — `ManifoldWitness<C>` / `SimplicialManifoldWitness<C>`
// ============================================================================

/// # Why `NoConstraint`
///
/// `Manifold<K, F>`, which is unbounded in its field type `F` carries no element bound, and the categorical operations here move elements without
/// computing on them: `fmap` maps `A` to an unrelated `B`. Constraining the element type would
/// forbid mappings that are legitimate and work today, so `NoConstraint` is the accurate statement
/// rather than a placeholder. Operations that compute carry real trait bounds on the concrete
/// types. See `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §4.
pub struct ManifoldWitness<C>(PhantomData<C>);

/// Textbook alias for the simplicial case.
pub type SimplicialManifoldWitness<C> = ManifoldWitness<C>;

impl<C> HKT for ManifoldWitness<C>
where
    SimplicialComplex<C>: ChainComplex,
    C: deep_causality_algebra::RealField + deep_causality_num::FromPrimitive,
{
    type Type<T> = Manifold<SimplicialComplex<C>, T>;
}

impl<C> Functor<ManifoldWitness<C>> for ManifoldWitness<C>
where
    SimplicialComplex<C>: ChainComplex + Clone,
    <SimplicialComplex<C> as CellularComplex>::Metric: Clone,
    C: Clone + deep_causality_algebra::RealField + deep_causality_num::FromPrimitive,
{
    fn fmap<A, B, Func>(
        m_a: Manifold<SimplicialComplex<C>, A>,
        f: Func,
    ) -> Manifold<SimplicialComplex<C>, B>
    where
        Func: FnMut(A) -> B,
    {
        // Metric is preserved across fmap: under Option 2C, `K::Metric` is a single
        // concrete type independent of the data type, so the metric clones through.
        let new_data_tensor = CausalTensorWitness::fmap(m_a.data, f);
        Manifold {
            complex: m_a.complex.clone(),
            data: new_data_tensor,
            metric: m_a.metric.clone(),
            cursor: m_a.cursor,
        }
    }
}

impl<C> Foldable<ManifoldWitness<C>> for ManifoldWitness<C>
where
    SimplicialComplex<C>: ChainComplex,
    C: deep_causality_algebra::RealField + deep_causality_num::FromPrimitive,
{
    fn fold<A, B, Func>(fa: Manifold<SimplicialComplex<C>, A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.data.into_vec().into_iter().fold(init, f)
    }
}

impl<C> Pure<ManifoldWitness<C>> for ManifoldWitness<C>
where
    SimplicialComplex<C>: ChainComplex + Default,
    C: Default + deep_causality_algebra::RealField + deep_causality_num::FromPrimitive,
{
    fn pure<T>(value: T) -> Manifold<SimplicialComplex<C>, T> {
        let tensor = CausalTensor::from_vec(vec![value], &[1]);
        Manifold {
            complex: Default::default(),
            data: tensor,
            metric: None,
            cursor: 0,
        }
    }
}

impl<C> Monad<ManifoldWitness<C>> for ManifoldWitness<C>
where
    SimplicialComplex<C>: ChainComplex + Clone + Default,
    <SimplicialComplex<C> as CellularComplex>::Metric: Clone,
    C: Clone + Default + deep_causality_algebra::RealField + deep_causality_num::FromPrimitive,
{
    fn bind<A, B, Func>(
        m_a: Manifold<SimplicialComplex<C>, A>,
        mut f: Func,
    ) -> Manifold<SimplicialComplex<C>, B>
    where
        Func: FnMut(A) -> <Self as HKT>::Type<B>,
    {
        let mut result_data = Vec::with_capacity(m_a.data.len());
        for a in m_a.data.into_vec() {
            let mb = f(a);
            result_data.extend(mb.data.into_vec());
        }
        let new_len = result_data.len();
        let new_tensor = CausalTensor::from_vec(result_data, &[new_len]);
        // Preserve the focus. `bind(m, pure)` must return `m`, and a focus reset to `0` breaks the
        // monad right identity law for every non-zero focus. This is the same reasoning the
        // `extend` implementations carry: the context of the input is the context of the result.
        //
        // The focus indexes `m_a.data`, and `f` decides how many elements each input contributes,
        // so the result may be shorter than the input. `Manifold::new` rejects a cursor at or past
        // `data.len()`, and `extract`/`extend` read at the cursor, so the index is clamped to keep
        // that invariant rather than handing on an index the data no longer has.
        let cursor = m_a.cursor.min(new_len.saturating_sub(1));
        Manifold {
            complex: m_a.complex.clone(),
            data: new_tensor,
            metric: m_a.metric.clone(),
            cursor,
        }
    }
}

impl<C> Applicative<ManifoldWitness<C>> for ManifoldWitness<C>
where
    SimplicialComplex<C>: ChainComplex + Clone + Default,
    <SimplicialComplex<C> as CellularComplex>::Metric: Clone,
    C: Clone + Default + deep_causality_algebra::RealField + deep_causality_num::FromPrimitive,
{
    fn apply<A, B, Func>(
        f_ab: Manifold<SimplicialComplex<C>, Func>,
        f_a: Manifold<SimplicialComplex<C>, A>,
    ) -> Manifold<SimplicialComplex<C>, B>
    where
        A: Clone,
        Func: FnMut(A) -> B,
    {
        let shape = f_a.data.shape().to_vec();
        let funcs = f_ab.data.into_vec();
        let args = f_a.data.into_vec();

        let new_data: Vec<B> = if funcs.len() == 1 {
            let f = funcs.into_iter().next().unwrap();
            args.into_iter().map(f).collect()
        } else {
            funcs.into_iter().zip(args).map(|(mut f, a)| f(a)).collect()
        };

        let new_tensor = CausalTensor::from_vec(new_data, &shape);
        Manifold {
            complex: f_a.complex.clone(),
            data: new_tensor,
            metric: f_a.metric.clone(),
            cursor: 0,
        }
    }
}

impl<C> CoMonad<ManifoldWitness<C>> for ManifoldWitness<C>
where
    SimplicialComplex<C>: ChainComplex + Clone,
    <SimplicialComplex<C> as CellularComplex>::Metric: Clone,
    C: Clone + deep_causality_algebra::RealField + deep_causality_num::FromPrimitive,
{
    fn extract<A>(fa: &Manifold<SimplicialComplex<C>, A>) -> A
    where
        A: Clone,
    {
        if fa.data.is_empty() {
            panic!("Cannot extract from empty Manifold");
        }
        fa.data
            .as_slice()
            .get(fa.cursor)
            .cloned()
            .expect("Cursor out of bounds")
    }

    fn extend<A, B, Func>(
        fa: &Manifold<SimplicialComplex<C>, A>,
        mut f: Func,
    ) -> Manifold<SimplicialComplex<C>, B>
    where
        Func: FnMut(&Manifold<SimplicialComplex<C>, A>) -> B,
        A: Clone,
    {
        let len = fa.data.len();
        let shape = fa.data.shape().to_vec();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let mut view = fa.clone_shallow();
                view.cursor = i;
                f(&view)
            })
            .collect();
        let new_tensor = CausalTensor::from_vec(new_data, &shape);
        Manifold {
            complex: fa.complex.clone(),
            data: new_tensor,
            metric: fa.metric.clone(),
            cursor: fa.cursor,
        }
    }
}

// ============================================================================
// PART 2: Generic witness — `GenericManifoldWitness<K>` over any `CellularComplex`
// ============================================================================

/// # Why `NoConstraint`
///
/// `Manifold<K, F>`, which is unbounded in its field type `F` carries no element bound, and the categorical operations here move elements without
/// computing on them: `fmap` maps `A` to an unrelated `B`. Constraining the element type would
/// forbid mappings that are legitimate and work today, so `NoConstraint` is the accurate statement
/// rather than a placeholder. Operations that compute carry real trait bounds on the concrete
/// types. See `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §4.
pub struct GenericManifoldWitness<K>(PhantomData<K>);

impl<K> HKT for GenericManifoldWitness<K>
where
    K: CellularComplex,
{
    type Type<T> = Manifold<K, T>;
}

impl<K> Functor<GenericManifoldWitness<K>> for GenericManifoldWitness<K>
where
    K: CellularComplex + Clone,
    K::Metric: Clone,
{
    fn fmap<A, B, Func>(m_a: Manifold<K, A>, f: Func) -> Manifold<K, B>
    where
        Func: FnMut(A) -> B,
    {
        let new_data_tensor = CausalTensorWitness::fmap(m_a.data, f);
        Manifold {
            complex: m_a.complex.clone(),
            data: new_data_tensor,
            metric: m_a.metric.clone(),
            cursor: m_a.cursor,
        }
    }
}

// `Pure`, `Monad`, `Applicative`, `CoMonad` impls for `GenericManifoldWitness<K>` remain
// deferred to a follow-up: `Pure` needs `K: Default` and the others need additional
// bounds that don't fall out generically. The simplicial fast path covers the common
// case via `SimplicialManifoldWitness<C>`.
