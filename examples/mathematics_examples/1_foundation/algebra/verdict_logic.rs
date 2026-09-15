/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # One aggregation
//!
//! A familiar shape: a set of checks, and a rule that says whether the system passes. Every
//! check is `true` or `false`, `All` means `&&`, `Any` means `||`, `None` means "not any".
//!
//! Then a check becomes a confidence rather than a boolean, and the whole thing is rewritten.
//! `0.8` is not a `bool`, `&&` does not apply, and "not any" is now `1 - max(...)`. Two code
//! paths for one rule, kept in step by hand.
//!
//! `Verdict` is the interface that removes the second path. It names a bounded lattice with a
//! complement:
//!
//! ```text
//! bottom / top     the bounds          false / true      0.0 / 1.0
//! meet             greatest lower      &&                min
//! join             least upper         ||                max
//! complement       the involution      !                 1 - p
//! ```
//!
//! For `bool` that is a Boolean algebra; for a probability it is an MV-algebra. Different
//! algebras, one interface, so `all` / `any` / `none` are written once.
//!
//! The law that makes this safe is the involution, `complement(complement(x)) == x`, which is
//! what lets `none` be defined as `any` post-composed with `complement` rather than as a third
//! hand-written reduction. It holds in both algebras — though in the graded one it holds in the
//! *algebra* and not in exact float equality, which section 3 shows and explains.

use deep_causality_algebra::{Prob, Verdict};

/// The working scalar of the graded carrier. `Prob` wraps one of these.
pub type FloatType = f64;

fn main() {
    print_header();

    // ---------------------------------------------------------------------
    // 1. Crisp checks: every reading is a pass or a fail.
    // ---------------------------------------------------------------------
    let crisp = [true, true, false];
    print_crisp(&crisp, all(&crisp), any(&crisp), none(&crisp));
    assert!(!all(&crisp) && any(&crisp) && !none(&crisp));

    // ---------------------------------------------------------------------
    // 2. The same three reducers over confidences.
    // ---------------------------------------------------------------------
    // Not one line of `all`, `any` or `none` changed. Only the element type did.
    let graded = [Prob(0.9), Prob(0.8), Prob(0.3)];
    print_graded(&graded, all(&graded), any(&graded), none(&graded));
    assert_eq!(all(&graded), Prob(0.3)); // the weakest link
    assert_eq!(any(&graded), Prob(0.9)); // the strongest evidence

    // ---------------------------------------------------------------------
    // 3. The involution law, which is what lets `none` be derived.
    // ---------------------------------------------------------------------
    // `none = complement . any` is only sound because double complement is the identity.
    // It holds exactly in the Boolean algebra. In the MV-algebra it holds in the *algebra*
    // and not in binary floating point: `1 - (1 - 0.9)` is `0.9000000000000001`, because
    // neither 0.9 nor 0.1 is representable. The law is exact; the carrier is not.
    let tol = 8.0 * FloatType::EPSILON;
    let graded_exact = graded.iter().all(|&p| p.complement().complement().0 == p.0);
    let graded_within = graded
        .iter()
        .all(|&p| (p.complement().complement().0 - p.0).abs() < tol);
    print_involution(
        crisp.iter().all(|&b| b.complement().complement() == b),
        graded_exact,
        graded_within,
    );
    assert!(graded_within);

    // ---------------------------------------------------------------------
    // 4. Where the two algebras differ, and why that is fine.
    // ---------------------------------------------------------------------
    // Boolean logic is idempotent and has excluded middle: `x join !x == top`. The MV-algebra
    // keeps the lattice and the involution but gives up excluded middle -- `max(p, 1-p)` is
    // not 1 unless p is 0 or 1. `Verdict` promises only what both can keep.
    let p = Prob(0.3);
    print_excluded_middle(true.join(true.complement()), p.join(p.complement()).0);

    print_footer();
}

/// Every check must pass. `meet` folded from the top bound.
fn all<V: Verdict + Copy>(checks: &[V]) -> V {
    checks.iter().fold(V::top(), |acc, &c| acc.meet(c))
}

/// At least one check must pass. `join` folded from the bottom bound.
fn any<V: Verdict + Copy>(checks: &[V]) -> V {
    checks.iter().fold(V::bottom(), |acc, &c| acc.join(c))
}

/// No check passes: `any`, complemented. Sound because complement is an involution.
fn none<V: Verdict + Copy>(checks: &[V]) -> V {
    any(checks).complement()
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== One aggregation, crisp or graded ===\n");
    println!("  `all`, `any` and `none` are written once against `Verdict`.");
    println!("  `bool` gives a Boolean algebra; `Prob` gives an MV-algebra.\n");
}

fn print_crisp(checks: &[bool], all: bool, any: bool, none: bool) {
    println!("--- 1. bool: a Boolean algebra ---");
    println!("  checks   {checks:?}");
    println!("  all      {all}      (meet = &&)");
    println!("  any      {any}       (join = ||)");
    println!("  none     {none}      (complement of any)");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_graded(checks: &[Prob], all: Prob, any: Prob, none: Prob) {
    let shown: Vec<FloatType> = checks.iter().map(|p| p.0).collect();
    println!("\n--- 2. Prob: an MV-algebra, same three functions ---");
    println!("  checks   {shown:?}");
    println!("  all      {:.4}     (meet = min, the weakest link)", all.0);
    println!(
        "  any      {:.4}     (join = max, the strongest evidence)",
        any.0
    );
    println!("  none     {:.4}     (complement = 1 - p)", none.0);
}

fn print_involution(crisp_holds: bool, graded_exact: bool, graded_within: bool) {
    println!("\n--- 3. complement is an involution in both ---");
    println!("  bool:  complement(complement(x)) == x        {crisp_holds}");
    println!("  Prob:  complement(complement(p)) == p        {graded_exact}   <- exact equality");
    println!("  Prob:  complement(complement(p)) ~= p        {graded_within}   <- within 8 eps");
    println!();
    println!("  The law holds in the MV-algebra. It does not survive exact float equality:");
    println!("  1 - (1 - 0.9) is 0.9000000000000001, because neither 0.9 nor 0.1 is a binary");
    println!("  fraction. The algebra is exact and the carrier is not, which is worth knowing");
    println!("  before a test asserts `==` on a probability.");
    println!();
    println!("  That law is what lets `none` be `any` composed with `complement`,");
    println!("  rather than a third reduction written and maintained by hand.");
}

fn print_excluded_middle(boolean: bool, graded: FloatType) {
    println!("\n--- 4. What the two algebras do not share ---");
    println!("  bool:  x join complement(x) = {boolean}      (excluded middle holds)");
    println!("  Prob:  p join complement(p) = {graded}      (it does not: max(0.3, 0.7))");
    println!("  `Verdict` promises only the bounded lattice and the involution, which is");
    println!("  exactly the part both algebras keep. Promising more would exclude one of them.");
}

fn print_footer() {
    println!("\n--- The engineering point ---");
    println!("  The usual version of this has two implementations of one rule: a boolean path");
    println!("  for pass/fail and a numeric path for confidence, drifting apart at every edit.");
    println!("  Naming the shared structure leaves one implementation, and the type at the");
    println!("  call site decides which algebra it runs in.");
}
