/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where `DenseMatrix` sits in the algebra tower.
//!
//! # An HKT witness is not enough
//!
//! A `deep_causality_haft` witness makes a container composable through that crate. It does not make
//! it composable through the tower: a function bounded on `Ring` or `Module` cannot take a container
//! that never declares them. Both surfaces have to be present, or generic code picks one and loses
//! the other.
//!
//! `CsrMatrix` is the cautionary case. It reached `AbelianGroup` and stopped, because `Distributive`
//! and `Annihilating` were absent, even though `One`, `Mul` and `Associative<Multiplicative>` were
//! all present. A matrix ring over a ring is a ring, and two marker impls were the whole distance.
//!
//! # What this type claims, and what it does not
//!
//! | trait | claimed | why |
//! |---|---|---|
//! | `Associative<Additive>` | yes | entrywise addition associates when the scalar does |
//! | `Commutative<Additive>` | yes | entrywise addition commutes when the scalar does |
//! | `Associative<Multiplicative>` | yes | matrix multiplication associates |
//! | `Commutative<Multiplicative>` | **no** | `AB ≠ BA` |
//! | `Distributive` | yes | `A(B + C) = AB + AC` entrywise |
//! | `Annihilating` | yes | every entry of `0 · A` is a sum of terms with a zero factor |
//! | `Ring` | by blanket | once the six above are present |
//! | `CommutativeRing` | **no** | follows from the missing multiplicative commutativity |
//! | `Module<R>` | by blanket | `AbelianGroup` plus scaling by a ring element |
//! | `IntegralDomain` | **no** | a matrix with a zero row is a zero divisor |
//!
//! The two deliberate absences are load-bearing. A `CommutativeRing` bound must refuse a matrix,
//! because claiming `AB = BA` would be false. And `IntegralDomain` must refuse one: `[[1,0],[0,0]]`
//! times `[[0,0],[0,1]]` is zero with neither factor zero, so cancellation fails and any algorithm
//! resting on it — Bareiss, for one — would be wrong.

/// The two absences above are checked rather than merely documented.
///
/// A false `Commutative<Multiplicative>` claim changes no runtime value — `AB` and `BA` are computed
/// the same way whether or not the marker is present — so no behavioural test can catch it. What
/// catches it is a bound that must keep failing:
///
/// ```compile_fail,E0277
/// use deep_causality_algebra::CommutativeRing;
/// use deep_causality_linear::DenseMatrix;
///
/// fn requires_commutative_ring<T: CommutativeRing>() {}
///
/// // Matrix multiplication does not commute, so this must not compile. If someone adds
/// // `Commutative<Multiplicative>` for `DenseMatrix`, this starts compiling and the doctest fails.
/// requires_commutative_ring::<DenseMatrix<f64>>();
/// ```
///
/// The same for `IntegralDomain`: `[[1,0],[0,0]] * [[0,0],[0,1]]` is zero with neither factor zero,
/// so cancellation fails and any algorithm resting on it would be wrong.
///
/// ```compile_fail,E0277
/// use deep_causality_algebra::IntegralDomain;
/// use deep_causality_linear::DenseMatrix;
///
/// fn requires_integral_domain<T: IntegralDomain>() {}
///
/// requires_integral_domain::<DenseMatrix<f64>>();
/// ```
use crate::types::dense_matrix::DenseMatrix;
use deep_causality_algebra::{
    Additive, Annihilating, Associative, Commutative, Distributive, Multiplicative,
};

impl<T> Associative<Additive> for DenseMatrix<T> where T: Associative<Additive> + Clone {}
impl<T> Commutative<Additive> for DenseMatrix<T> where T: Commutative<Additive> + Clone {}
impl<T> Associative<Multiplicative> for DenseMatrix<T> where T: Associative<Multiplicative> + Clone {}
impl<T> Distributive for DenseMatrix<T> where T: Distributive + Clone {}
impl<T> Annihilating for DenseMatrix<T> where T: Annihilating + Clone {}

// Deliberately absent: `Commutative<Multiplicative>` and `IntegralDomain`. See the module header.
