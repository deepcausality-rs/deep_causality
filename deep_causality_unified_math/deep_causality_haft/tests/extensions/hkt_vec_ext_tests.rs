/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{
    Applicative, BoxWitness, Collectable, Foldable, Functor, HKT, Monad, OptionWitness, Pure,
    ResultWitness, Traversable, VecWitness,
};

// --- Applicative Tests ---

#[test]
fn test_applicative_vec_pure() {
    let vec = VecWitness::pure(10);
    assert_eq!(vec, vec![10]);
}

#[test]
fn test_applicative_vec_apply_non_empty() {
    let f_funcs = vec![|x| x + 1, |x| x * 2];
    let vals = vec![10, 20];
    let result = VecWitness::apply(f_funcs, vals);
    // Expected: [(10+1), (20+1), (10*2), (20*2)] = [11, 21, 20, 40]
    assert_eq!(result, vec![11, 21, 20, 40]);
}

#[test]
fn test_applicative_vec_apply_empty_func() {
    let f_funcs: Vec<fn(i32) -> i32> = Vec::new();
    let vals = vec![10, 20];
    let result = VecWitness::apply(f_funcs, vals);
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_applicative_vec_apply_empty_val() {
    let f_funcs = vec![|x| x + 1, |x| x * 2];
    let vals: Vec<i32> = Vec::new();
    let result = VecWitness::apply(f_funcs, vals);
    assert_eq!(result, Vec::<i32>::new());
}

// --- Foldable Tests ---

#[test]
fn test_foldable_vec_non_empty() {
    let vec = vec![1, 2, 3];
    let result = VecWitness::fold(vec, 0, |acc, x| acc + x);
    assert_eq!(result, 6);
}

#[test]
fn test_foldable_vec_empty() {
    let vec: Vec<i32> = Vec::new();
    let result = VecWitness::fold(vec, 0, |acc, x| acc + x);
    assert_eq!(result, 0);
}

#[test]
fn test_foldable_vec_string_concat() {
    let vec = vec!["hello".to_string(), " ".to_string(), "world".to_string()];
    let result = VecWitness::fold(vec, String::new(), |mut acc, x| {
        acc.push_str(&x);
        acc
    });
    assert_eq!(result, "hello world");
}

// --- HKT Tests ---

#[test]
fn test_hkt_vec_witness() {
    let value: <VecWitness as HKT>::Type<i32> = vec![1, 2, 3];
    assert_eq!(value, vec![1, 2, 3]);

    let empty_value: <VecWitness as HKT>::Type<i32> = Vec::new();
    assert_eq!(empty_value, Vec::<i32>::new());
}

// --- Functor Tests ---

#[test]
fn test_functor_vec() {
    let vec_a = vec![1, 2, 3];
    let f = |x| x * 2;
    let vec_b = VecWitness::fmap(vec_a, f);
    assert_eq!(vec_b, vec![2, 4, 6]);

    let vec_empty: Vec<i32> = Vec::new();
    let f_empty = |x: i32| x * 2;
    let vec_empty_mapped = VecWitness::fmap(vec_empty, f_empty);
    assert_eq!(vec_empty_mapped, Vec::<i32>::new());
}

// --- Monad Tests ---

#[test]
fn test_monad_vec() {
    let vec_a = vec![1, 2, 3];
    let f = |x| vec![x, x * 10];
    let vec_b = VecWitness::bind(vec_a, f);
    assert_eq!(vec_b, vec![1, 10, 2, 20, 3, 30]);

    let pure_val = VecWitness::pure(100);
    assert_eq!(pure_val, vec![100]);

    let vec_empty: Vec<i32> = Vec::new();
    let f_empty = |x: i32| vec![x, x * 10];
    let vec_empty_bound = VecWitness::bind(vec_empty, f_empty);
    assert_eq!(vec_empty_bound, Vec::<i32>::new());
}

// --- Traversable Tests ---

/// The Identity applicative, required by the *accepted* identity law. The trait docstring's own
/// phrasing is vacuous, so the law is stated at this carrier instead. Mirrors the private fixture
/// in `tests/formalization_lean/traversable_tests.rs`.
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

/// A Writer-style carrier that records the order in which `apply` combines its arguments.
///
/// This is the substitute for the composition law, which cannot be tested: a `Compose<M, N>`
/// applicative is unwritable because its `apply` would need `N::Type<A>: Clone` on a method-level
/// parameter. The one defect class composition would have caught is a traversal that visits
/// elements in the wrong order while still returning the right result, and that is exactly what an
/// order-recording carrier detects.
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

/// C3 — every element succeeds, and the order of the result is the order of the input.
#[test]
fn test_traversable_vec_all_success_preserves_order() {
    let v = vec![Some(1), Some(2), Some(3)];
    assert_eq!(
        VecWitness::sequence::<i32, OptionWitness>(v),
        Some(vec![1, 2, 3])
    );
}

/// C1 — the empty container is the fold's identity element and succeeds.
#[test]
fn test_traversable_vec_empty_is_pure_empty() {
    let v: Vec<Option<i32>> = Vec::new();
    assert_eq!(
        VecWitness::sequence::<i32, OptionWitness>(v),
        Some(Vec::new())
    );
}

/// C2 — one element. Listed because it is blind to the order defects C3 and C9 catch.
#[test]
fn test_traversable_vec_single_element() {
    assert_eq!(
        VecWitness::sequence::<i32, OptionWitness>(vec![Some(7)]),
        Some(vec![7])
    );
}

/// C5 — a failure that is neither first nor last collapses the whole traversal.
#[test]
fn test_traversable_vec_interior_failure_collapses() {
    let v = vec![Some(1), None, Some(3)];
    assert_eq!(VecWitness::sequence::<i32, OptionWitness>(v), None);
}

/// C6 — a failure in the last position still fails; the off-by-one partner of C4.
#[test]
fn test_traversable_vec_last_error_still_fails() {
    let v = vec![Some(1), Some(2), None];
    assert_eq!(VecWitness::sequence::<i32, OptionWitness>(v), None);
}

/// C4, C7 — two distinct errors, and the *first* in index order is the one reported.
#[test]
fn test_traversable_vec_first_error_wins() {
    let v = vec![
        Ok::<i32, String>(1),
        Err("first".to_string()),
        Err("second".to_string()),
    ];
    assert_eq!(
        VecWitness::sequence::<i32, ResultWitness<String>>(v),
        Err("first".to_string())
    );
}

/// C8 — a shaped inner applicative. `VecWitness`'s own `apply` is cartesian, so the fold must
/// delegate to it rather than assume a failure-shaped carrier.
#[test]
fn test_traversable_vec_cartesian_inner_applicative() {
    let v: Vec<Vec<i32>> = vec![vec![1, 2], vec![10, 20]];
    assert_eq!(
        VecWitness::sequence::<i32, VecWitness>(v),
        vec![vec![1, 10], vec![1, 20], vec![2, 10], vec![2, 20]]
    );
}

/// C8 — `BoxWitness` as the inner carrier. Named in the archived measurement as one of the
/// sixteen witnesses a `Semigroupal` bound move would have lost; it stays admissible here.
#[test]
fn test_traversable_vec_box_inner_applicative() {
    let v: Vec<Box<i32>> = vec![Box::new(1), Box::new(2)];
    assert_eq!(*VecWitness::sequence::<i32, BoxWitness>(v), vec![1, 2]);
}

/// C9 — effect order. The substitute for the composition law: a right-to-left traversal would
/// return the same values and be caught only here.
#[test]
fn test_traversable_vec_effect_order_is_left_to_right() {
    let v = vec![Logged(1, vec![1]), Logged(2, vec![2]), Logged(3, vec![3])];
    let r = VecWitness::sequence::<i32, LoggedWitness>(v);
    assert_eq!(r.0, vec![1, 2, 3], "values");
    assert_eq!(r.1, vec![1, 2, 3], "effects run left to right");
}

/// L1 — the identity law at the Identity applicative, over three distinct elements so that a
/// permutation defect cannot pass.
#[test]
fn test_traversable_vec_identity_law() {
    let v = vec![Ident(1), Ident(2), Ident(3)];
    assert_eq!(
        VecWitness::sequence::<i32, IdentWitness>(v),
        Ident(vec![1, 2, 3])
    );
}

/// L2 — naturality: an applicative morphism `phi: Ident -> Option` commutes with `sequence`.
#[test]
fn test_traversable_vec_naturality_law() {
    fn phi<T>(i: Ident<T>) -> Option<T> {
        Some(i.0)
    }
    for xs in [vec![Ident(1), Ident(2), Ident(3)], Vec::<Ident<i32>>::new()] {
        let lhs: Option<Vec<i32>> = phi(VecWitness::sequence::<i32, IdentWitness>(xs.clone()));
        let rhs: Option<Vec<i32>> =
            VecWitness::sequence::<i32, OptionWitness>(VecWitness::fmap(xs, phi));
        assert_eq!(lhs, rhs);
    }
}

// --- Collectable Tests ---

#[test]
fn test_collectable_vec_preserves_order() {
    let v: Vec<i32> = VecWitness::collect([10, 20, 30, 40]);
    assert_eq!(v, vec![10, 20, 30, 40]);
}

#[test]
fn test_collectable_vec_empty_sequence() {
    let v: Vec<i32> = VecWitness::collect(Vec::new());
    assert!(v.is_empty());
    // Folding the empty structure returns the initial accumulator unchanged.
    assert_eq!(VecWitness::fold(v, 7, |acc, x| acc + x), 7);
}

/// L1 — round trip: collecting a sequence and folding the result is folding the sequence.
///
/// The fold is `acc * 2 + x`, which is not commutative, so a reversed or off-by-one order
/// cannot cancel out the way a sum would.
#[test]
fn test_collectable_vec_round_trip_law() {
    for xs in [vec![1, 2, 3, 4, 5], Vec::<i32>::new(), vec![9]] {
        let through_structure =
            VecWitness::fold(VecWitness::collect(xs.clone()), 0, |acc, x| acc * 2 + x);
        let direct = xs.into_iter().fold(0, |acc, x| acc * 2 + x);
        assert_eq!(through_structure, direct);
    }
}

#[test]
fn test_collectable_vec_from_a_lazy_sequence() {
    // `IntoIterator` rather than `Vec`: a sequence that was never a collection needs no
    // intermediate one.
    let v: Vec<i32> = VecWitness::collect((0..4).map(|i| i * i));
    assert_eq!(v, vec![0, 1, 4, 9]);
}

#[test]
fn test_collectable_vec_element_type_carries_no_bound() {
    let v: Vec<&str> = VecWitness::collect(["a", "b", "c"]);
    assert_eq!(v, vec!["a", "b", "c"]);
}

/// `pure` and `collect` agree on one value here, unlike on a carrier with a shape: a `Vec` has
/// no rank to distinguish a scalar from a run of length one.
#[test]
fn test_collectable_vec_one_value_matches_pure() {
    let collected: Vec<i32> = VecWitness::collect([42]);
    let pured: Vec<i32> = VecWitness::pure(42);
    assert_eq!(collected, pured);
}
