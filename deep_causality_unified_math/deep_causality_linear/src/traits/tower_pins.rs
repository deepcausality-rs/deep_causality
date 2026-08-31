/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Compile-time pins on the scalar bands and the container tower memberships.
//!
//! # Why these are not tests
//!
//! A `#[test]` that calls `admits::<u64>()` where `admits<T: CommutativeSemiring>` passes by
//! **compiling**. If `u64` were not a `CommutativeSemiring` the crate would not build, and the test
//! would never run to report it. Running it checks nothing the build has not already checked, so
//! writing it as a test overstates what the suite verifies.
//!
//! What it does do is *pin* the membership: without something that names the bound, a container
//! could silently lose an impl and nothing would notice. So the pins live here, in `src`, next to
//! what they pin, as ordinary items rather than as tests.
//!
//! # The half that is a real check
//!
//! "This must **not** compile" is not established by the build succeeding, so it cannot be a pin.
//! Those are `compile_fail` doctests, and they live on the item whose bound they guard:
//!
//! - [`DenseMatrix`](crate::DenseMatrix)'s algebra module — that a matrix is refused by
//!   `CommutativeRing` and by `IntegralDomain`
//! - [`determinant_exact`](crate::determinant_exact) and [`rank_exact`](crate::rank_exact) — that
//!   `f64` is refused by `EuclideanDomain`
//! - `deep_causality_algebra::DivisibleByIntegers` — that halving is refused over 𝔽₂

use crate::{
    CsrMatrix, CsrMatrixWitness, DenseMatrix, DenseMatrixWitness, DenseVector, DenseVectorWitness,
    PackedGf2,
};
use deep_causality_algebra::{
    AbelianGroup, CommutativeRing, CommutativeSemiring, ConjugateScalar, DivisibleByIntegers,
    EuclideanDomain, Field, IntegralDomain, Module, NormedScalar, RealField, Ring,
};
use deep_causality_haft::HKT;
use deep_causality_num::{Float106, Gf2};

const fn pin_semiring<T: CommutativeSemiring>() {}
const fn pin_ring<T: CommutativeRing>() {}
const fn pin_integral<T: IntegralDomain>() {}
const fn pin_euclidean<T: EuclideanDomain>() {}
const fn pin_field<T: Field>() {}
const fn pin_divisible<T: DivisibleByIntegers>() {}
const fn pin_normed<T: NormedScalar>() {}
const fn pin_real<T: RealField>() {}
const fn pin_conjugate<T: ConjugateScalar>() {}
const fn pin_container_ring<T: Ring>() {}
const fn pin_abelian<T: AbelianGroup>() {}
const fn pin_module<M: Module<R>, R: Ring>() {}
const fn pin_projection<W, T, C>()
where
    W: HKT<Type<T> = C>,
{
}
#[cfg(feature = "std")]
const fn pin_std_error<E: std::error::Error>() {}

/// The scalar bands, as the band table in `linear-scalar-contract` states them.
const _: () = {
    // semiring: ℕ
    pin_semiring::<u8>();
    pin_semiring::<u64>();
    pin_semiring::<usize>();
    // ring: ℤ and above, 𝔽₂ included
    pin_ring::<i8>();
    pin_ring::<i64>();
    pin_ring::<f64>();
    pin_ring::<Float106>();
    pin_ring::<Gf2>();
    // integral domain: where cancellation holds
    pin_integral::<i64>();
    pin_integral::<f64>();
    pin_integral::<Gf2>();
    // Euclidean domain: the integers, and nothing else in this tower
    pin_euclidean::<i8>();
    pin_euclidean::<i64>();
    pin_euclidean::<isize>();
    // field
    pin_field::<f32>();
    pin_field::<f64>();
    pin_field::<Float106>();
    pin_field::<Gf2>();
    // integer-divisible: characteristic zero, so 𝔽₂ is absent
    pin_divisible::<f32>();
    pin_divisible::<f64>();
    pin_divisible::<Float106>();
    // normed and real
    pin_normed::<f64>();
    pin_normed::<Float106>();
    pin_real::<f32>();
    pin_real::<f64>();
    pin_real::<Float106>();
    pin_conjugate::<f64>();
};

/// The container tower memberships.
///
/// Every matrix reaches `Ring` and `Module<R>`; the vector reaches `AbelianGroup` and `Module<R>`
/// and no multiplicative rung, having no `Mul`.
const _: () = {
    pin_container_ring::<DenseMatrix<f64>>();
    pin_container_ring::<DenseMatrix<i64>>();
    pin_container_ring::<CsrMatrix<f64>>();
    pin_container_ring::<CsrMatrix<i64>>();
    pin_container_ring::<PackedGf2<u8>>();
    pin_container_ring::<PackedGf2<u64>>();

    pin_module::<DenseMatrix<f64>, f64>();
    pin_module::<DenseMatrix<i64>, i64>();
    pin_module::<CsrMatrix<f64>, f64>();
    pin_module::<PackedGf2<u64>, Gf2>();

    pin_abelian::<DenseVector<f64>>();
    pin_abelian::<DenseVector<i64>>();
    pin_module::<DenseVector<f64>, f64>();
    pin_module::<DenseVector<i64>, i64>();
};

/// The HKT witness projections.
///
/// A witness stands for a type constructor, and what makes it the right witness is that
/// `<W as HKT>::Type<T>` *is* the container it names. `pin_projection` takes the projection as an
/// equality bound, so naming the container as its third argument is the assertion — and it is one
/// the build settles, which is why it is here and not in the suite.
const _: () = {
    pin_projection::<DenseMatrixWitness, f64, DenseMatrix<f64>>();
    pin_projection::<DenseVectorWitness, f64, DenseVector<f64>>();
    pin_projection::<CsrMatrixWitness, f64, CsrMatrix<f64>>();

    // `PackedGf2` has no witness of its own: it is generic in its *word* and fixed to `Gf2` in its
    // element, so there is no `PackedGf2<T>` for `Type<T>` to name. The absence cannot be pinned —
    // this MSRV has no negative impls — so what is pinned is the documented route out, which is the
    // conversion to `DenseMatrix<Gf2>`. That the conversion carries the entries is a separate
    // question, and a runtime one; `conversions_tests.rs` answers it.
    pin_projection::<DenseMatrixWitness, Gf2, DenseMatrix<Gf2>>();
};

/// [`LinearError`](crate::LinearError) is a `std::error::Error` under the `std` feature.
///
/// Callers put it behind `Box<dyn Error>` and `?`, both of which need the impl. Losing it is a
/// breaking change that the build catches here.
#[cfg(feature = "std")]
const _: () = {
    pin_std_error::<crate::LinearError>();
};
