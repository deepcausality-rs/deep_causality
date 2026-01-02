/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::*;

// ----------------------------------------------------
// 1. Bifunctor Test (Result)
// ----------------------------------------------------

struct ResultWitness;
impl HKT2Unbound for ResultWitness {
    type Constraint = NoConstraint;
    type Type<A, B> = Result<A, B>;
}

impl Bifunctor<ResultWitness> for ResultWitness {
    fn bimap<A, B, C, D, F1, F2>(fab: Result<A, B>, mut f1: F1, mut f2: F2) -> Result<C, D>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>,
        F1: FnMut(A) -> C,
        F2: FnMut(B) -> D,
    {
        match fab {
            Ok(a) => Ok(f1(a)),
            Err(b) => Err(f2(b)),
        }
    }
}

#[test]
fn test_bifunctor_result() {
    let ok_val: Result<i32, &str> = Ok(10);
    let err_val: Result<i32, &str> = Err("error");

    let mapped_ok = ResultWitness::bimap(ok_val, |x| x * 2, |e| e.len());
    assert_eq!(mapped_ok, Ok(20));

    let mapped_err = ResultWitness::bimap(err_val, |x| x * 2, |e| e.len());
    assert_eq!(mapped_err, Err(5));
}

// ----------------------------------------------------
// 2. Profunctor Test (Function Wrapper)
// ----------------------------------------------------

// A simple wrapper around a function A -> B
struct FuncWrapper<A, B>(Box<dyn Fn(A) -> B>);

struct FuncWitness;
impl HKT2Unbound for FuncWitness {
    type Constraint = NoConstraint;
    type Type<A, B> = FuncWrapper<A, B>;
}

impl Profunctor<FuncWitness> for FuncWitness {
    fn dimap<A, B, C, D, F1, F2>(pab: FuncWrapper<A, B>, f_pre: F1, f_post: F2) -> FuncWrapper<C, D>
    where
        A: 'static + Satisfies<NoConstraint>,
        B: 'static + Satisfies<NoConstraint>,
        C: 'static + Satisfies<NoConstraint>,
        D: 'static + Satisfies<NoConstraint>,
        F1: FnMut(C) -> A + 'static,
        F2: FnMut(B) -> D + 'static,
    {
        // We use RefCell to allow mutating the captured closures (f_pre, f_post)
        // inside the returned Fn closure, which is required because FuncWrapper
        // stores a Box<dyn Fn> (immutable), but the trait requires FnMut.
        let inner_fn = pab.0;
        let f_pre = std::cell::RefCell::new(f_pre);
        let f_post = std::cell::RefCell::new(f_post);

        FuncWrapper(Box::new(move |c| {
            let a = (f_pre.borrow_mut())(c);
            let b = inner_fn(a);
            (f_post.borrow_mut())(b)
        }))
    }
}

#[test]
fn test_profunctor_func() {
    // Core logic: Int -> String
    let core = FuncWrapper(Box::new(|x: i32| x.to_string()));

    // Adapter:
    // Pre: Bool -> Int (true -> 1, false -> 0)
    // Post: String -> usize (length)
    let adapter = FuncWitness::dimap(core, |b: bool| if b { 1 } else { 0 }, |s: String| s.len());

    assert_eq!((adapter.0)(true), 1); // "1".len()
    assert_eq!((adapter.0)(false), 1); // "0".len()
}

// ----------------------------------------------------
// 3. Promonad Test (Tuple Merge)
// ----------------------------------------------------

struct TripleWitness;
impl HKT3Unbound for TripleWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C> = (A, B, C);
}

impl Promonad<TripleWitness> for TripleWitness {
    fn merge<A, B, C, F>(pa: (A, A, A), pb: (B, B, B), mut f: F) -> (C, C, C)
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        F: FnMut(A, B) -> C,
    {
        (f(pa.0, pb.0), f(pa.1, pb.1), f(pa.2, pb.2))
    }

    fn fuse<A, B, C>(input_a: A, input_b: B) -> (A, B, C)
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
    {
        // This implementation is a dummy for Tuple, as we can't invent C.
        // But for Promonad generally, it might construct an interaction context.
        // Here we just panic or use unsafe, but for test we won't call fuse directly
        // in a way that requires C to be meaningful without a merge function.
        // A better example would be an Interaction struct.
        // For the sake of compilation, we just panic.
        panic!(
            "Tuple fusion requires a merge function, inputs: {:?}, {:?}",
            std::mem::size_of_val(&input_a),
            std::mem::size_of_val(&input_b)
        )
    }
}

#[test]
fn test_promonad_tuple() {
    let t1 = (1, 2, 3);
    let t2 = (10, 20, 30);

    let merged = TripleWitness::merge(t1, t2, |a, b| a + b);
    assert_eq!(merged, (11, 22, 33));
}

// ----------------------------------------------------
// 4. Parametric Monad Test (State Transition)
// ----------------------------------------------------

// Represents a transition from State S1 to S2 yielding Value A
// We need to make this struct compatible with the HKT3Unbound trait which expects Type<A, B, C>.
// HKT3Unbound::Type<A, B, C> maps to Transition<A, B, C>.
// So A=S1, B=S2, C=Value.
struct Transition<S1, S2, A> {
    val: A,
    _phantom: std::marker::PhantomData<(S1, S2)>,
}

impl<S1, S2, A> Transition<S1, S2, A> {
    fn new(val: A) -> Self {
        Self {
            val,
            _phantom: std::marker::PhantomData,
        }
    }
}

struct TransitionWitness;
impl HKT3Unbound for TransitionWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C> = Transition<A, B, C>;
}

impl ParametricMonad<TransitionWitness> for TransitionWitness {
    fn pure<S, A>(value: A) -> Transition<S, S, A>
    where
        S: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
    {
        Transition::new(value)
    }

    fn ibind<S1, S2, S3, A, B, F>(m: Transition<S1, S2, A>, mut f: F) -> Transition<S1, S3, B>
    where
        S1: Satisfies<NoConstraint>,
        S2: Satisfies<NoConstraint>,
        S3: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnMut(A) -> Transition<S2, S3, B>,
    {
        // Since our dummy Transition struct just holds a value and PhantomData,
        // we can just apply 'f' to the value.
        // The resulting Transition<S2, S3, B> has the correct value 'B'.
        // But its type says it starts at S2.
        // We need to "cast" the start state from S2 to S1 because the composed computation
        // *starts* at S1 (conceptually).
        // Wait, ibind signature: m: M<S1, S2, A>, f: A -> M<S2, S3, B> -> M<S1, S3, B>
        // Yes, the result must be S1 -> S3.

        let next = f(m.val); // This is Transition<S2, S3, B>

        // We need to return Transition<S1, S3, B>.
        // Since Transition only holds 'val: B' and PhantomData, we can just construct a new one.
        Transition::new(next.val)
    }
}

struct StateA;
struct StateB;
struct StateC;

#[test]
fn test_parametric_monad() {
    let start: Transition<StateA, StateB, i32> = Transition::new(10);

    let end: Transition<StateA, StateC, String> = TransitionWitness::ibind(start, |x| {
        // Transition from StateB -> StateC
        Transition::new(x.to_string())
    });

    assert_eq!(end.val, "10");
}
