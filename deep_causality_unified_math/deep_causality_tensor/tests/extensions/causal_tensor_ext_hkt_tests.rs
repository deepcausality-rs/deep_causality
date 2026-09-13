/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{
    Applicative, BoxWitness, CoMonad, Foldable, Functor, HKT, Monad, OptionWitness, Pure,
    ResultWitness, Traversable,
};
use deep_causality_haft::{MonoidalApplicative, Semigroupal};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness, ZipTensorWitness};

// --- HKT Tests ---

#[test]
fn test_hkt_causal_tensor_witness() {
    let value: <CausalTensorWitness as HKT>::Type<i32> =
        CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    assert_eq!(value.as_slice(), &[1, 2, 3]);

    let empty_value: <CausalTensorWitness as HKT>::Type<f64> =
        CausalTensor::new(vec![], vec![0]).unwrap();
    assert!(empty_value.is_empty());
}

// --- Applicative Tests ---

#[test]
fn test_applicative_causal_tensor_pure() {
    let tensor = CausalTensorWitness::pure(42);
    assert_eq!(tensor.as_slice(), &[42]);
    assert_eq!(tensor.shape(), &[] as &[usize]); // Scalar tensor
}

#[test]
fn test_applicative_causal_tensor_apply_scalar_func() {
    let f_tensor = CausalTensor::new(vec![|x: i32| x * 2], vec![]).unwrap(); // Scalar function, added type annotation
    let a_tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result_tensor = CausalTensorWitness::apply(f_tensor, a_tensor);
    assert_eq!(result_tensor.as_slice(), &[2, 4, 6]);
    assert_eq!(result_tensor.shape(), &[3]);
}

#[test]
fn test_applicative_causal_tensor_apply_equal_length() {
    // Equal lengths, cartesian: three functions against three arguments is nine results,
    // function-major. The elementwise reading of the same inputs is a different applicative and
    // is tested on `ZipTensorWitness` below.
    let f_tensor = CausalTensor::new(
        vec![|x: i32| x + 1, |x: i32| x * 10, |x: i32| x - 3],
        vec![3],
    )
    .unwrap();
    let a_tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result_tensor = CausalTensorWitness::apply(f_tensor, a_tensor);
    assert_eq!(result_tensor.as_slice(), &[2, 3, 4, 10, 20, 30, -2, -1, 0]);
    assert_eq!(result_tensor.shape(), &[9]);
}

#[test]
fn test_applicative_causal_tensor_apply_non_scalar_func() {
    // Different lengths, cartesian: two functions against three arguments is six results. The
    // assertions that used to sit here read `is_empty()` and `shape() == &[0]` under the comment
    // "as per the updated implementation", pinning the answer rather than deriving it.
    let f_tensor = CausalTensor::new(vec![|x: i32| x * 2, |x: i32| x * 3], vec![2]).unwrap();
    let a_tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result_tensor = CausalTensorWitness::apply(f_tensor, a_tensor);
    assert_eq!(result_tensor.as_slice(), &[2, 4, 6, 3, 6, 9]);
    assert_eq!(result_tensor.shape(), &[6]);
}

#[test]
fn test_applicative_causal_tensor_coherence_with_bind_rank_two() {
    // The coherence law with a *single* function against a rank-2 argument. This is the case the
    // 2-against-3 test above cannot reach: `apply` broadcasts the one function and reports the
    // argument's shape, so `bind` over the one-element `ff` has to report it too rather than
    // flattening to `[6]`.
    let u: CausalTensor<fn(i32) -> i32> =
        CausalTensor::from_vec(vec![(|x: i32| x * 2) as fn(i32) -> i32], &[1]);
    let fa = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let fa2 = fa.clone();

    let via_apply = CausalTensorWitness::apply(u.clone(), fa);
    let via_bind = CausalTensorWitness::bind(u, move |f: fn(i32) -> i32| {
        CausalTensorWitness::fmap(fa2.clone(), f)
    });
    assert_eq!(via_apply, via_bind);
    assert_eq!(via_apply.shape(), &[2, 3], "the argument's rank survives");
    assert_eq!(via_apply.as_slice(), &[2, 4, 6, 8, 10, 12]);
}

#[test]
fn test_monad_causal_tensor_bind_reports_a_concat_map_flatly() {
    // A concat-map whose parts happen to total the input count is still a concat-map. Two
    // elements in, one contributing two and the other none, sums back to two while the shape is
    // gone; the result must say `[2]` rather than borrow the input's `[1, 2]`.
    let m = CausalTensor::new(vec![1i32, 2], vec![1, 2]).unwrap();
    let out = CausalTensorWitness::bind(m, |x: i32| {
        if x == 1 {
            CausalTensor::new(vec![x, x * 10], vec![2]).unwrap()
        } else {
            CausalTensor::new(Vec::<i32>::new(), vec![0]).unwrap()
        }
    });
    assert_eq!(out.as_slice(), &[1, 10]);
    assert_eq!(
        out.shape(),
        &[2],
        "a concat-map does not inherit the input's shape"
    );
}

// --- ZipTensorWitness: the elementwise applicative ---

#[test]
fn test_zip_witness_applies_elementwise() {
    // The reading `CausalTensorWitness` cannot give without breaking coherence with its `bind`.
    let f_tensor = CausalTensor::new(
        vec![|x: i32| x + 1, |x: i32| x * 10, |x: i32| x - 3],
        vec![3],
    )
    .unwrap();
    let a_tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let out = ZipTensorWitness::apply(f_tensor, a_tensor);
    assert_eq!(out.as_slice(), &[2, 20, 0]);
    assert_eq!(out.shape(), &[3]);
}

#[test]
fn test_zip_witness_zip_with_keeps_a_shared_shape() {
    let a = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let b = CausalTensor::new(vec![10, 20, 30, 40, 50, 60], vec![2, 3]).unwrap();
    let out = ZipTensorWitness::zip_with(a, b, |x, y| x + y);
    assert_eq!(out.as_slice(), &[11, 22, 33, 44, 55, 66]);
    assert_eq!(out.shape(), &[2, 3], "a shared shape survives the pairing");
}

#[test]
fn test_zip_witness_semigroupal_naturality() {
    // zip(fmap(fa, f), fmap(fb, g)) == fmap(zip(fa, fb), |(a, b)| (f(a), g(b)))
    let fa = CausalTensor::new(vec![1i32, 2, 3], vec![3]).unwrap();
    let fb = CausalTensor::new(vec![10i32, 20, 30], vec![3]).unwrap();
    let f = |x: i32| x + 1;
    let g = |y: i32| y * 2;

    let lhs = ZipTensorWitness::zip(
        ZipTensorWitness::fmap(fa.clone(), f),
        ZipTensorWitness::fmap(fb.clone(), g),
    );
    let rhs = ZipTensorWitness::fmap(ZipTensorWitness::zip(fa, fb), move |(a, b)| (f(a), g(b)));
    assert_eq!(lhs, rhs);
}

#[test]
fn test_zip_witness_semigroupal_associativity() {
    // zip(zip(a, b), c) is zip(a, zip(b, c)) up to reassociating the tuple.
    let a = CausalTensor::new(vec![1i32, 2], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3i32, 4], vec![2]).unwrap();
    let c = CausalTensor::new(vec![5i32, 6], vec![2]).unwrap();

    let left = ZipTensorWitness::zip(ZipTensorWitness::zip(a.clone(), b.clone()), c.clone());
    let right = ZipTensorWitness::zip(a, ZipTensorWitness::zip(b, c));
    let reassociated = ZipTensorWitness::fmap(left, |((x, y), z)| (x, (y, z)));
    assert_eq!(reassociated, right);
}

#[test]
fn test_applicative_causal_tensor_interchange() {
    // apply(u, pure(y)) == apply(pure(|f| f(y)), u)
    let u: CausalTensor<fn(i32) -> i32> = CausalTensor::new(
        vec![
            (|x: i32| x * 2) as fn(i32) -> i32,
            (|x: i32| x + 100) as fn(i32) -> i32,
        ],
        vec![2],
    )
    .unwrap();
    let lhs = CausalTensorWitness::apply(u.clone(), CausalTensorWitness::pure(5));
    type IntFn = fn(i32) -> i32;
    type ApplyAt5 = fn(IntFn) -> i32;
    let g: CausalTensor<ApplyAt5> =
        CausalTensor::from_vec(vec![(|f: IntFn| f(5)) as ApplyAt5], &[]);
    let rhs = CausalTensorWitness::apply(g, u);
    assert_eq!(lhs, rhs);
}

#[test]
fn test_applicative_causal_tensor_coherence_with_bind() {
    // apply(f_ab, f_a) == bind(f_ab, |f| fmap(f_a, f))
    let u: CausalTensor<fn(i32) -> i32> = CausalTensor::new(
        vec![
            (|x: i32| x * 2) as fn(i32) -> i32,
            (|x: i32| x + 100) as fn(i32) -> i32,
        ],
        vec![2],
    )
    .unwrap();
    let fa = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let fa2 = fa.clone();
    assert_eq!(
        CausalTensorWitness::apply(u.clone(), fa),
        CausalTensorWitness::bind(u, move |f: fn(i32) -> i32| {
            CausalTensorWitness::fmap(fa2.clone(), f)
        })
    );
}

// --- Functor Tests ---

#[test]
fn test_functor_causal_tensor() {
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let f = |x| x * 2;
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert_eq!(mapped_tensor.as_slice(), &[2, 4, 6]);
    assert_eq!(mapped_tensor.shape(), &[3]);
}

#[test]
fn test_functor_causal_tensor_empty() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let f = |x| x * 2;
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert!(mapped_tensor.is_empty());
    assert_eq!(mapped_tensor.shape(), &[0]);
}

#[test]
fn test_functor_causal_tensor_type_change() {
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let f = |x: i32| x.to_string(); // Added type annotation for x
    let mapped_tensor = CausalTensorWitness::fmap(tensor, f);
    assert_eq!(
        mapped_tensor.as_slice(),
        &["1".to_string(), "2".to_string(), "3".to_string()]
    );
    assert_eq!(mapped_tensor.shape(), &[3]);
}

// --- Foldable Tests ---

#[test]
fn test_foldable_causal_tensor_sum() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5], vec![5]).unwrap();
    let sum = CausalTensorWitness::fold(tensor, 0, |acc, x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
fn test_foldable_causal_tensor_empty() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let sum = CausalTensorWitness::fold(tensor, 0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}

#[test]
fn test_foldable_causal_tensor_string_concat() {
    let tensor =
        CausalTensor::new(vec!["hello".to_string(), "world".to_string()], vec![2]).unwrap();
    let concatenated = CausalTensorWitness::fold(tensor, String::new(), |mut acc, x| {
        if !acc.is_empty() {
            acc.push(' ');
        }
        acc.push_str(&x);
        acc
    });
    assert_eq!(concatenated, "hello world");
}

// --- Monad Tests ---

#[test]
fn test_monad_causal_tensor_bind() {
    // Pointwise expansion. The shape assertion that used to sit here read `&[4]` with the
    // comment "Flattened to 1D", recording the defect below as though it were the contract.
    let tensor = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let f = |x: i32| CausalTensor::new(vec![x, x * 10], vec![2]).unwrap();
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert_eq!(bound_tensor.as_slice(), &[1, 10, 2, 20]);
}

#[test]
fn test_monad_causal_tensor_right_identity() {
    // bind(m, pure) == m, on shapes where the reshape is not a no-op. A rank-1 fixture cannot
    // see this: `bind` rebuilds as `shape: [len]`, already the shape of a rank-1.
    for shape in [vec![2usize, 3], vec![2, 2, 2], vec![6]] {
        let n: usize = shape.iter().product();
        let m = CausalTensor::new((1..=n as i32).collect::<Vec<_>>(), shape.clone()).unwrap();
        assert_eq!(
            CausalTensorWitness::bind(m.clone(), CausalTensorWitness::pure),
            m,
            "right identity for shape {shape:?}"
        );
    }
}

#[test]
fn test_monad_causal_tensor_left_identity() {
    // bind(pure(a), f) == f(a).
    assert_eq!(
        CausalTensorWitness::bind(CausalTensorWitness::pure(7), CausalTensorWitness::pure),
        CausalTensorWitness::pure(7)
    );
}

#[test]
fn test_monad_causal_tensor_associativity() {
    // bind(bind(m, f), g) == bind(m, |x| bind(f(x), g)). Holds today; pinned so that a
    // shape-respecting fix cannot silently break it.
    let ma = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let f = |x: i32| CausalTensor::new(vec![x, x + 1], vec![2]).unwrap();
    let g = |x: i32| CausalTensor::new(vec![x * 10, x * 100], vec![2]).unwrap();
    assert_eq!(
        CausalTensorWitness::bind(CausalTensorWitness::bind(ma.clone(), f), g),
        CausalTensorWitness::bind(ma, move |x| CausalTensorWitness::bind(f(x), g))
    );
}

#[test]
fn test_monad_causal_tensor_bind_empty() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let f = |x: i32| CausalTensor::new(vec![x, x * 10], vec![2]).unwrap(); // Added type annotation for x
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert!(bound_tensor.is_empty());
    assert_eq!(bound_tensor.shape(), &[0]);
}

#[test]
fn test_monad_causal_tensor_bind_to_empty() {
    let tensor = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let f = |_x: i32| CausalTensor::<i32>::new(vec![], vec![0]).unwrap(); // Explicitly specify <i32>
    let bound_tensor = CausalTensorWitness::bind(tensor, f);
    assert!(bound_tensor.is_empty());
    assert_eq!(bound_tensor.shape(), &[0]);
}

// ---CoMonad Tests ---

#[test]
fn test_comonad_causal_tensor_extract_scalar() {
    let scalar_tensor = CausalTensor::new(vec![10.0], vec![]).unwrap();
    let extracted = CausalTensorWitness::extract(&scalar_tensor);
    assert_eq!(extracted, 10.0);
}

#[test]
#[should_panic(expected = "CoMonad::extract cannot be called on an empty CausalTensor.")]
fn test_comonad_causal_tensor_extract_empty_panics() {
    let empty_tensor: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    CausalTensorWitness::extract(&empty_tensor);
}

#[test]
fn test_comonad_causal_tensor_extract_non_scalar_first_element() {
    let vector_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let extracted = CausalTensorWitness::extract(&vector_tensor);
    // Arbitrary choice, should extract the first element
    assert_eq!(extracted, 1.0);
}

#[test]
fn test_comonad_causal_tensor_extend_scalar() {
    let scalar_tensor = CausalTensor::new(vec![5.0], vec![]).unwrap();
    // Function that observes the context (the scalar tensor) and returns a new value
    let f = |ct: &CausalTensor<f64>| ct.data()[0] * 2.0;
    let extended = CausalTensorWitness::extend(&scalar_tensor, f);
    assert_eq!(extended, CausalTensor::new(vec![10.0], vec![]).unwrap());
}

#[test]
fn test_comonad_causal_tensor_extend_non_scalar() {
    let vector_tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    // Function that observes the context (the vector tensor) and returns a summary value
    // Requires T: Add for sum()
    let f = |ct: &CausalTensor<f64>| ct.data().iter().cloned().sum::<f64>(); // Added .cloned() for sum
    let extended = CausalTensorWitness::extend(&vector_tensor, f);
    // The result should be a scalar tensor containing the sum of the vector elements
    assert_eq!(
        extended,
        CausalTensor::new(vec![6.0, 6.0, 6.0], vec![3]).unwrap()
    );
}

#[test]
fn test_comonad_causal_tensor_extend_topology_check() {
    let vector_tensor = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();

    // The Law: "My value plus the value of the element to my right"
    // This relies on the Shifted View putting 'Me' at 0 and 'Neighbor' at 1.
    let f = |ct: &CausalTensor<f64>| {
        let me = ct.data()[0];
        // If strictly 1D, neighbor is at 1.
        // Ideally verify size > 1 to avoid panic, but for test we know input is len 3.
        let neighbor = ct.data()[1];
        me + neighbor
    };

    let extended = CausalTensorWitness::extend(&vector_tensor, f);

    // Expected: [10+20, 20+30, 30+10] -> [30, 50, 40]
    let expected = CausalTensor::new(vec![30.0, 50.0, 40.0], vec![3]).unwrap();

    assert_eq!(
        extended, expected,
        "Topology check failed: Shift/Wrap-around logic is incorrect"
    );
}


// --- Traversable for CausalTensorWitness ---
//
// Corner rows are enumerated in
// `openspec/changes/add-hkt-traversable-cochain/corner-cases.md`; the ids (C1..C9, T1..T4, L1..L2)
// in the test names below refer to that table.

/// The Identity applicative, required by the *accepted* identity law; the trait docstring's own
/// phrasing is vacuous.
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

/// A Writer-style carrier recording the order in which `apply` combines its arguments — the
/// substitute for the untestable composition law.
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

/// C3, T4 — every element succeeds, order is kept, and the rank-2 shape survives. The only row
/// where a flattening defect is visible.
#[test]
fn test_traversable_tensor_rank2_shape_survives() {
    let t = CausalTensor::from_vec((1..=6).map(Some).collect::<Vec<Option<i32>>>(), &[2, 3]);
    let r = CausalTensorWitness::sequence::<i32, OptionWitness>(t).expect("all present");
    assert_eq!(r.shape(), &[2, 3], "shape");
    assert_eq!(r.as_slice(), &[1, 2, 3, 4, 5, 6], "order");
}

/// T2 — a zero-extent shape has no elements but is not `[0]`; distinct from C1.
#[test]
fn test_traversable_tensor_zero_extent_shape_survives() {
    let t: CausalTensor<Option<i32>> = CausalTensor::from_vec(Vec::new(), &[0, 3]);
    let r = CausalTensorWitness::sequence::<i32, OptionWitness>(t).expect("empty succeeds");
    assert_eq!(r.shape(), &[0, 3]);
}

/// T1 — the rank-0 shape. Shape and length disagree: a `[len]` defect returns `[1]`.
#[test]
fn test_traversable_tensor_rank0_shape_survives() {
    let t = CausalTensor::from_vec(vec![Some(7)], &[]);
    let r = CausalTensorWitness::sequence::<i32, OptionWitness>(t).expect("present");
    assert_eq!(r.shape(), &[] as &[usize]);
    assert_eq!(r.as_slice(), &[7]);
}

/// T3, C2 — one element *and* a non-empty shape. Separates T1 from the bare single-element case:
/// both hold one element and differ exactly on the shape a defect would fabricate.
#[test]
fn test_traversable_tensor_single_element_shape_survives() {
    let t = CausalTensor::from_vec(vec![Some(9)], &[1]);
    let r = CausalTensorWitness::sequence::<i32, OptionWitness>(t).expect("present");
    assert_eq!(r.shape(), &[1]);
    assert_eq!(r.as_slice(), &[9]);
}

/// C1 — an empty tensor succeeds, carrying nothing.
#[test]
fn test_traversable_tensor_empty_is_pure_empty() {
    let t: CausalTensor<Option<i32>> = CausalTensor::from_vec(Vec::new(), &[0]);
    let r = CausalTensorWitness::sequence::<i32, OptionWitness>(t).expect("empty succeeds");
    assert!(r.as_slice().is_empty());
}

/// C5 — a failure neither first nor last collapses the whole tensor.
#[test]
fn test_traversable_tensor_interior_failure_collapses() {
    let mut v: Vec<Option<i32>> = (1..=6).map(Some).collect();
    v[3] = None;
    let t = CausalTensor::from_vec(v, &[2, 3]);
    assert!(CausalTensorWitness::sequence::<i32, OptionWitness>(t).is_none());
}

/// C6 — a failure in the last position still fails.
#[test]
fn test_traversable_tensor_last_error_still_fails() {
    let mut v: Vec<Option<i32>> = (1..=4).map(Some).collect();
    v[3] = None;
    let t = CausalTensor::from_vec(v, &[2, 2]);
    assert!(CausalTensorWitness::sequence::<i32, OptionWitness>(t).is_none());
}

/// C4, C7 — two distinct errors; the first in index order is reported.
#[test]
fn test_traversable_tensor_first_error_wins() {
    let t = CausalTensor::from_vec(
        vec![
            Ok::<i32, String>(1),
            Err("first".to_string()),
            Err("second".to_string()),
            Ok(4),
        ],
        &[2, 2],
    );
    assert_eq!(
        CausalTensorWitness::sequence::<i32, ResultWitness<String>>(t).err(),
        Some("first".to_string())
    );
}

/// C8 — a shaped inner applicative. `CausalTensorWitness`'s `apply` is cartesian, pinned by its
/// `Monad`, so the fold must delegate to it.
#[test]
fn test_traversable_tensor_cartesian_inner_applicative() {
    let t = CausalTensor::from_vec(
        vec![
            CausalTensor::from_vec(vec![1, 2], &[2]),
            CausalTensor::from_vec(vec![10, 20], &[2]),
        ],
        &[2],
    );
    let r = CausalTensorWitness::sequence::<i32, CausalTensorWitness>(t);
    let got: Vec<Vec<i32>> = r.as_slice().iter().map(|d| d.as_slice().to_vec()).collect();
    assert_eq!(got, vec![vec![1, 10], vec![1, 20], vec![2, 10], vec![2, 20]]);
}

/// C8 — `BoxWitness` as the inner carrier.
#[test]
fn test_traversable_tensor_box_inner_applicative() {
    let t = CausalTensor::from_vec(vec![Box::new(1), Box::new(2)], &[2]);
    let r = CausalTensorWitness::sequence::<i32, BoxWitness>(t);
    assert_eq!(r.as_slice(), &[1, 2]);
    assert_eq!(r.shape(), &[2]);
}

/// C9 — effect order. A right-to-left traversal returns the same values and the same shape, and
/// is caught only here.
#[test]
fn test_traversable_tensor_effect_order_is_left_to_right() {
    let t = CausalTensor::from_vec(
        vec![
            Logged(1, vec![1]),
            Logged(2, vec![2]),
            Logged(3, vec![3]),
            Logged(4, vec![4]),
        ],
        &[2, 2],
    );
    let r = CausalTensorWitness::sequence::<i32, LoggedWitness>(t);
    assert_eq!(r.0.as_slice(), &[1, 2, 3, 4], "values");
    assert_eq!(r.0.shape(), &[2, 2], "shape");
    assert_eq!(r.1, vec![1, 2, 3, 4], "effects run left to right");
}

/// L1 — identity at the Identity applicative, over four distinct elements in a rank-2 shape.
#[test]
fn test_traversable_tensor_identity_law() {
    let t = CausalTensor::from_vec(vec![Ident(1), Ident(2), Ident(3), Ident(4)], &[2, 2]);
    let r = CausalTensorWitness::sequence::<i32, IdentWitness>(t);
    assert_eq!(r.0.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(r.0.shape(), &[2, 2]);
}

/// L2 — naturality across the applicative morphism `phi: Ident -> Option`.
#[test]
fn test_traversable_tensor_naturality_law() {
    fn phi<T>(i: Ident<T>) -> Option<T> {
        Some(i.0)
    }
    for xs in [
        CausalTensor::from_vec(vec![Ident(1), Ident(2), Ident(3), Ident(4)], &[2, 2]),
        CausalTensor::from_vec(Vec::<Ident<i32>>::new(), &[0]),
    ] {
        let lhs: Option<CausalTensor<i32>> =
            phi(CausalTensorWitness::sequence::<i32, IdentWitness>(xs.clone()));
        let rhs: Option<CausalTensor<i32>> = CausalTensorWitness::sequence::<i32, OptionWitness>(
            CausalTensorWitness::fmap(xs, phi),
        );
        assert_eq!(lhs, rhs);
    }
}
