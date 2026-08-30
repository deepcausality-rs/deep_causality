/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the free-monad laws.
//!
//! Mirrors `lean/DeepCausalityFormal/Haft/FreeMonad.lean`. The Lean proof is over a representative
//! single-hole functor; these witnesses instantiate `Free<F, A>` over TWO real functors:
//! `OptionWitness` (0-or-1 hole) and `VecWitness` (multi-hole), the latter exercising the
//! `Fn + Clone` continuation-threading through many holes. One `#[test]` per THEOREM_MAP id.
//!
//! `Free<F,A>` has no `PartialEq` (a bound on the GAT field cycles the trait solver), so two
//! programs are compared by folding each to a canonical `String` that records both structure and
//! leaves — an injective serialization, so equal canonical forms ⟺ equal programs.

use deep_causality_haft::{Free, OptionWitness, VecWitness};

// ---- canonical serializers (the `fold` interpreter used as an equality oracle) -----------------

fn canon_opt(m: Free<OptionWitness, i32>) -> String {
    m.fold(
        &|a: i32| format!("P{a}"),
        &|node: Option<String>| match node {
            Some(s) => format!("S({s})"),
            None => "S()".to_string(),
        },
    )
}

fn canon_vec(m: Free<VecWitness, i32>) -> String {
    m.fold(&|a: i32| format!("P{a}"), &|children: Vec<String>| {
        format!("S[{}]", children.join(","))
    })
}

// ---- program builders (return a fresh program each call — `Free` is not `Clone`) ---------------

fn opt_op(inner: Free<OptionWitness, i32>) -> Free<OptionWitness, i32> {
    Free::Suspend(Some(Box::new(inner)))
}
fn vec_op(children: Vec<Free<VecWitness, i32>>) -> Free<VecWitness, i32> {
    Free::Suspend(children.into_iter().map(Box::new).collect())
}

// ---- haft.free_monad.left_id : bind (pure a) k = k a --------------------------------------------

/// THEOREM_MAP: haft.free_monad.left_id
#[test]
fn test_free_monad_left_identity() {
    let k = |x: i32| opt_op(Free::pure(x + 1));
    let lhs = Free::<OptionWitness, i32>::pure(5).bind(k);
    let rhs = k(5);
    assert_eq!(canon_opt(lhs), canon_opt(rhs));
}

// ---- haft.free_monad.right_id : bind m pure = m -------------------------------------------------

/// THEOREM_MAP: haft.free_monad.right_id
#[test]
fn test_free_monad_right_identity_single_hole() {
    let build = || opt_op(opt_op(Free::pure(7)));
    let bound = build().bind(Free::<OptionWitness, i32>::pure);
    assert_eq!(canon_opt(bound), canon_opt(build()));
}

#[test]
fn test_free_monad_right_identity_multi_hole() {
    // Branching program: the continuation must thread through every hole (Clone).
    let build = || {
        vec_op(vec![
            Free::pure(1),
            vec_op(vec![Free::pure(2), Free::pure(3)]),
        ])
    };
    let bound = build().bind(Free::<VecWitness, i32>::pure);
    assert_eq!(canon_vec(bound), canon_vec(build()));
}

// ---- haft.free_monad.assoc : bind (bind m f) g = bind m (|x| bind (f x) g) ----------------------

/// THEOREM_MAP: haft.free_monad.assoc
#[test]
fn test_free_monad_associativity_single_hole() {
    let f = |x: i32| opt_op(Free::pure(x * 2));
    let g = |x: i32| opt_op(Free::pure(x + 100));
    let build = || opt_op(Free::pure(4));

    let lhs = build().bind(f).bind(g);
    let rhs = build().bind(move |x| f(x).bind(g));
    assert_eq!(canon_opt(lhs), canon_opt(rhs));
}

#[test]
fn test_free_monad_associativity_multi_hole() {
    let f = |x: i32| vec_op(vec![Free::pure(x * 2), Free::pure(x)]);
    let g = |x: i32| vec_op(vec![Free::pure(x + 100)]);
    let build = || vec_op(vec![Free::pure(4), Free::pure(5)]);

    let lhs = build().bind(f).bind(g);
    let rhs = build().bind(move |x| f(x).bind(g));
    assert_eq!(canon_vec(lhs), canon_vec(rhs));
}

// ---- haft.free_monad.lift_bind : bind (lift fa) k = Suspend node holding k(a) -------------------

/// THEOREM_MAP: haft.free_monad.lift_bind
#[test]
fn test_free_monad_lift_bind() {
    let lifted = Free::<OptionWitness, i32>::lift(Some(9));
    let bound = lifted.bind(|x: i32| opt_op(Free::pure(x + 1)));
    // one outer node (from lift) whose child is k(9) = a node holding Pure(10).
    let expected = opt_op(opt_op(Free::pure(10)));
    assert_eq!(canon_opt(bound), canon_opt(expected));
}

// ---- haft.free_monad.map_id : map id = id -------------------------------------------------------

/// THEOREM_MAP: haft.free_monad.map_id
#[test]
fn test_free_monad_map_id() {
    let build = || vec_op(vec![Free::pure(1), Free::pure(2)]);
    let mapped = build().map(|a| a);
    assert_eq!(canon_vec(mapped), canon_vec(build()));
}

// ---- fold / interpreter (the handler that gives operations meaning) -----------------------------

#[test]
fn test_free_monad_fold_interprets_program() {
    // Three nested Option operations terminating in Pure(0); handler counts operation nodes.
    let program: Free<OptionWitness, i32> = opt_op(opt_op(opt_op(Free::pure(0))));
    let depth = program.fold(&|a: i32| a, &|node: Option<i32>| node.map_or(0, |x| x + 1));
    assert_eq!(depth, 3, "three operation nodes fold to depth 3");
}

#[test]
fn test_free_monad_fold_sums_multi_hole() {
    // Sum all pure leaves of a branching program via the Vec algebra.
    let program: Free<VecWitness, i32> = vec_op(vec![
        Free::pure(10),
        vec_op(vec![Free::pure(20), Free::pure(30)]),
    ]);
    let total = program.fold(&|a: i32| a, &|children: Vec<i32>| {
        children.into_iter().sum()
    });
    assert_eq!(total, 60);
}
