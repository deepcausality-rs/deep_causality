/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Names one of a type's binary operations, so a law can be stated about a specific one.
///
/// # Why the laws need this
///
/// In the mathematics a law is a property of a **pair** — a set together with an operation. ℍ is
/// the case that forces the distinction: quaternion addition commutes and quaternion multiplication
/// does not, so "ℍ is commutative" is not a well-formed claim. Only "(ℍ, +) commutes" and
/// "(ℍ, ×) does not" are.
///
/// A marker on the type alone cannot say which operation it means, and a type may implement a
/// non-generic trait only once — so a flat `Commutative` has exactly one slot where ℍ needs two
/// opposite answers. Parameterising by an operator gives each law its operation.
///
/// # Scope
///
/// These are tags, not operations. [`Additive`] names whatever `Add` a type implements; it does not
/// carry the function. Reifying the operation would make the laws generically testable at the cost
/// of every bound in the tower naming an operator explicitly, and is deliberately not done.
///
/// The trait is open, so a downstream crate may add an operator for a structure with further
/// operations.
pub trait Operator {}

/// The additive operation — whatever a type's `Add` implementation does.
///
/// Written `Associative<Additive>` or `Commutative<Additive>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Additive;

/// The multiplicative operation — whatever a type's `Mul` implementation does.
///
/// This is the **default** operator for the law markers, so `Associative` and
/// `Associative<Multiplicative>` are the same bound and the same impl. The default matches what the
/// flat markers already meant: every law impl, and six of the eight law bounds, stated the
/// multiplicative case.
///
/// The two that did not are why the operators exist — [`AddSemigroup`](crate::AddSemigroup) meant
/// addition, and [`CommutativeMonoid`](crate::CommutativeMonoid) meant [`Combining`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Multiplicative;

/// The abstract operation of [`Monoid`](crate::Monoid) — whatever `combine` does.
///
/// Neither addition nor multiplication, and not reducible to either: `Prob::combine` multiplies,
/// `Count::combine` adds, and `Conjunction` / `Disjunction` combine with `∧` and `∨`. The operation
/// is `combine`, whatever it happens to wrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Combining;

impl Operator for Additive {}
impl Operator for Multiplicative {}
impl Operator for Combining {}
