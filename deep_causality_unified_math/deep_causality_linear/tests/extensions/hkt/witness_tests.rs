/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What the witnesses are at runtime.
//!
//! That each witness *projects* to its container is settled by the build, so it is pinned in
//! `src/traits/tower_pins.rs` rather than asserted here — a test of it would pass by compiling and
//! report nothing the build had not already refused.

use deep_causality_haft::{
    Applicative, BoxWitness, Functor, HKT, OptionWitness, Pure, ResultWitness, Traversable,
};
use deep_causality_linear::{
    CsrMatrixWitness, DenseMatrixWitness, DenseVector, DenseVectorWitness,
};

#[test]
fn test_the_witnesses_are_zero_sized() {
    // A witness is a stand-in for a type constructor, so it carries no data.
    assert_eq!(core::mem::size_of::<DenseMatrixWitness>(), 0);
    assert_eq!(core::mem::size_of::<DenseVectorWitness>(), 0);
    assert_eq!(core::mem::size_of::<CsrMatrixWitness>(), 0);
}

#[test]
fn test_the_witnesses_are_defaultable() {
    // `Default` has to be *invoked* to be covered. The earlier form of this test wrote
    // `let _ = DenseMatrixWitness;`, a unit-struct literal that needs no `Default` impl at all,
    // so deleting `Default` from the derives left it green.
    // Routed through a `Default`-bounded generic. A bare `Witness::default()` would catch a
    // missing impl just as well, since it resolves through the trait; the generic form is here
    // because clippy's `default_constructed_unit_structs` fires on the direct call and suggests
    // the bare literal, which would *not* catch it.
    fn defaulted<T: Default>() -> T {
        T::default()
    }
    assert_eq!(defaulted::<DenseMatrixWitness>(), DenseMatrixWitness);
    assert_eq!(defaulted::<DenseVectorWitness>(), DenseVectorWitness);
    assert_eq!(defaulted::<CsrMatrixWitness>(), CsrMatrixWitness);
}

// --- Traversable for DenseVectorWitness ---
//
// Corner rows are enumerated in
// `openspec/changes/add-hkt-traversable-cochain/corner-cases.md`; the ids (C1..C9, L1..L2) in the
// test names below refer to that table.

/// The Identity applicative, required by the *accepted* identity law. The trait docstring's own
/// phrasing is vacuous, so the law is stated at this carrier instead.
#[derive(Debug, PartialEq, Clone)]
struct Ident<T>(T);
struct IdentWitness;
impl HKT for IdentWitness {
    type Type<T> = Ident<T>;
}
impl Functor<IdentWitness> for IdentWitness {
    fn fmap<A, B, Func>(m_a: Ident<A>, mut f: Func) -> Ident<B>
    where
        Func: FnMut(A) -> B,
    {
        Ident(f(m_a.0))
    }
}
impl Pure<IdentWitness> for IdentWitness {
    fn pure<T>(value: T) -> Ident<T> {
        Ident(value)
    }
}
impl Applicative<IdentWitness> for IdentWitness {
    fn apply<A, B, Func>(f_ab: Ident<Func>, f_a: Ident<A>) -> Ident<B>
    where
        A: Clone,
        Func: FnMut(A) -> B,
    {
        let mut f = f_ab.0;
        Ident(f(f_a.0))
    }
}

/// A Writer-style carrier recording the order in which `apply` combines its arguments.
///
/// The substitute for the composition law, which cannot be tested: a `Compose<M, N>` applicative
/// is unwritable because its `apply` needs `N::Type<A>: Clone` on a method-level parameter. The
/// defect it would have caught — a traversal visiting elements in the wrong order while still
/// returning the right result — is caught here instead.
#[derive(Debug, PartialEq, Clone)]
struct Logged<T>(T, Vec<i32>);
struct LoggedWitness;
impl HKT for LoggedWitness {
    type Type<T> = Logged<T>;
}
impl Functor<LoggedWitness> for LoggedWitness {
    fn fmap<A, B, Func>(m_a: Logged<A>, mut f: Func) -> Logged<B>
    where
        Func: FnMut(A) -> B,
    {
        Logged(f(m_a.0), m_a.1)
    }
}
impl Pure<LoggedWitness> for LoggedWitness {
    fn pure<T>(value: T) -> Logged<T> {
        Logged(value, Vec::new())
    }
}
impl Applicative<LoggedWitness> for LoggedWitness {
    fn apply<A, B, Func>(f_ab: Logged<Func>, f_a: Logged<A>) -> Logged<B>
    where
        A: Clone,
        Func: FnMut(A) -> B,
    {
        let mut log = f_ab.1;
        log.extend(f_a.1);
        let mut f = f_ab.0;
        Logged(f(f_a.0), log)
    }
}

/// C3 — every element succeeds and the result keeps the input order.
#[test]
fn test_traversable_dense_vector_all_success_preserves_order() {
    let v = DenseVector::from_vec(vec![Some(1), Some(2), Some(3)]);
    assert_eq!(
        DenseVectorWitness::sequence::<i32, OptionWitness>(v),
        Some(DenseVector::from_vec(vec![1, 2, 3]))
    );
}

/// C1 — the empty container is the fold's identity element and succeeds.
#[test]
fn test_traversable_dense_vector_empty_is_pure_empty() {
    let v: DenseVector<Option<i32>> = DenseVector::from_vec(Vec::new());
    assert_eq!(
        DenseVectorWitness::sequence::<i32, OptionWitness>(v),
        Some(DenseVector::from_vec(Vec::new()))
    );
}

/// C2 — one element. Blind to the order defects C3 and C9 catch, which is why it is not alone.
#[test]
fn test_traversable_dense_vector_single_element() {
    let v = DenseVector::from_vec(vec![Some(7)]);
    assert_eq!(
        DenseVectorWitness::sequence::<i32, OptionWitness>(v),
        Some(DenseVector::from_vec(vec![7]))
    );
}

/// C5 — a failure neither first nor last collapses the whole traversal.
#[test]
fn test_traversable_dense_vector_interior_failure_collapses() {
    let v = DenseVector::from_vec(vec![Some(1), None, Some(3)]);
    assert_eq!(DenseVectorWitness::sequence::<i32, OptionWitness>(v), None);
}

/// C6 — a failure in the last position still fails; the off-by-one partner of C4.
#[test]
fn test_traversable_dense_vector_last_error_still_fails() {
    let v = DenseVector::from_vec(vec![Some(1), Some(2), None]);
    assert_eq!(DenseVectorWitness::sequence::<i32, OptionWitness>(v), None);
}

/// C4, C7 — two distinct errors; the first in index order is reported.
#[test]
fn test_traversable_dense_vector_first_error_wins() {
    let v = DenseVector::from_vec(vec![
        Ok::<i32, String>(1),
        Err("first".to_string()),
        Err("second".to_string()),
    ]);
    assert_eq!(
        DenseVectorWitness::sequence::<i32, ResultWitness<String>>(v),
        Err("first".to_string())
    );
}

/// C8 — a shaped inner applicative. `DenseVectorWitness`'s own `apply` is cartesian, pinned by its
/// `Monad`, so the fold must delegate to it rather than assume a failure-shaped carrier.
#[test]
fn test_traversable_dense_vector_cartesian_inner_applicative() {
    let v = DenseVector::from_vec(vec![
        DenseVector::from_vec(vec![1, 2]),
        DenseVector::from_vec(vec![10, 20]),
    ]);
    let r = DenseVectorWitness::sequence::<i32, DenseVectorWitness>(v);
    let got: Vec<Vec<i32>> = r.as_slice().iter().map(|d| d.as_slice().to_vec()).collect();
    assert_eq!(
        got,
        vec![vec![1, 10], vec![1, 20], vec![2, 10], vec![2, 20]]
    );
}

/// C8 — `BoxWitness` as the inner carrier, one of the sixteen a `Semigroupal` bound move
/// would have lost.
#[test]
fn test_traversable_dense_vector_box_inner_applicative() {
    let v = DenseVector::from_vec(vec![Box::new(1), Box::new(2)]);
    assert_eq!(
        *DenseVectorWitness::sequence::<i32, BoxWitness>(v),
        DenseVector::from_vec(vec![1, 2])
    );
}

/// C9 — effect order. A right-to-left traversal returns the same values and is caught only here.
#[test]
fn test_traversable_dense_vector_effect_order_is_left_to_right() {
    let v = DenseVector::from_vec(vec![
        Logged(1, vec![1]),
        Logged(2, vec![2]),
        Logged(3, vec![3]),
    ]);
    let r = DenseVectorWitness::sequence::<i32, LoggedWitness>(v);
    assert_eq!(r.0, DenseVector::from_vec(vec![1, 2, 3]), "values");
    assert_eq!(r.1, vec![1, 2, 3], "effects run left to right");
}

/// L1 — identity at the Identity applicative, over three distinct elements.
#[test]
fn test_traversable_dense_vector_identity_law() {
    let v = DenseVector::from_vec(vec![Ident(1), Ident(2), Ident(3)]);
    assert_eq!(
        DenseVectorWitness::sequence::<i32, IdentWitness>(v),
        Ident(DenseVector::from_vec(vec![1, 2, 3]))
    );
}

/// L2 — naturality across the applicative morphism `phi: Ident -> Option`.
#[test]
fn test_traversable_dense_vector_naturality_law() {
    fn phi<T>(i: Ident<T>) -> Option<T> {
        Some(i.0)
    }
    for xs in [
        DenseVector::from_vec(vec![Ident(1), Ident(2), Ident(3)]),
        DenseVector::from_vec(Vec::<Ident<i32>>::new()),
    ] {
        let lhs: Option<DenseVector<i32>> = phi(DenseVectorWitness::sequence::<i32, IdentWitness>(
            xs.clone(),
        ));
        let rhs: Option<DenseVector<i32>> =
            DenseVectorWitness::sequence::<i32, OptionWitness>(DenseVectorWitness::fmap(xs, phi));
        assert_eq!(lhs, rhs);
    }
}
