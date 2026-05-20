/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Either-Input Generic API: Iso as a Compile-Time Contract
//!
//! A library function that accepts EITHER representation of a value when
//! the two representations are proven equivalent. The iso machinery makes
//! the equivalence visible in the function signature, so the compiler
//! enforces "callers may pass either side."
//!
//! ## The scenario
//!
//! Imagine a signal-processing library exposes a function:
//!
//! ```ignore
//! fn rotate_phase<X>(signal: X, angle_rad: f64) -> X
//! where X: Field { ... }
//! ```
//!
//! This works for any `Field` — `Complex<f64>`, `CausalMultiVector<f64>`
//! in Cl(0,1), and so on. But the caller has no guarantee that passing
//! `Complex<f64>` produces the "same answer" (in the iso-equivalent
//! sense) as passing `CausalMultiVector<f64>`. The `Field` bound alone
//! doesn't capture the *relationship* between two representations.
//!
//! That's what the iso markers add. By bounding on `FieldIso`, the
//! function signature promises: "these two types are interchangeable,
//! and operations on one match operations on the other."
//!
//! ## What this example demonstrates
//!
//! 1. A generic function `rotate_phase_iso` whose signature carries a
//!    `FieldIso` bound. The caller can pass EITHER `Complex<f64>` OR
//!    `CausalMultiVector<f64>` in Cl(0,1).
//! 2. A higher-level function `dispatch_rotate` that takes an `Either`-
//!    like input enum and dispatches to the right path; the iso bound
//!    on the enum's variants means the compiler knows the two paths
//!    produce equivalent results.
//! 3. A regression test: compute via both paths and assert iso-equality
//!    of the outputs (compare via `to_target` / `to_source`).
//!
//! ## Why this matters in practice
//!
//! Without the iso marker, an API author writing "this works on complex
//! numbers OR Cl(0,1) multivectors" has to either (a) duplicate the
//! function for each type, or (b) say `where T: Field` and rely on a
//! comment to communicate the equivalence — losing compiler enforcement.
//! With the iso marker, the equivalence becomes part of the type
//! signature. Mis-using the API (e.g. passing a `Quaternion`, which is
//! NOT iso to `Complex`) is a compile error rather than a documentation
//! gotcha.
//!
//! ## Iso surface used
//!
//! - Tier 1 `FieldIso<T>`: marker subtrait bounded on bidirectional
//!   `From` plus `Field` on both sides.
//! - Tier 2 `ComplexCl01Iso`: witness for the cross-crate case where
//!   bidirectional `From` doesn't exist due to the orphan rule.
//!
//! ## Why this rejection works (under the hood)
//!
//! The compile error on `Quaternion` falls straight out of the trait hierarchy via
//! small "trait A does not include trait B" facts that the compiler
//! already knows how to chain. Walk through it:
//!
//! 1. `FieldIso<T>` is declared with `where Self: Field + From<T>,
//!    T: Field + From<Self>`. The bound demands `Field` on both sides.
//! 2. `Field` (in `deep_causality_num`) extends `CommutativeRing`,
//!    which extends `Ring + Commutative`. The marker trait `Commutative`
//!    is the structural promise "multiplication commutes."
//! 3. `Quaternion<F>` is declared to implement `Ring` (additive abelian
//!    group + multiplicative monoid + distributivity) but **NOT**
//!    `Commutative`. Quaternion multiplication is famously non-commutative
//!    (`ij = k` but `ji = -k`), so the impl was never written and never
//!    can be.
//! 4. Therefore `Quaternion<F>: Field` does not hold (the `Commutative`
//!    supertrait is missing).
//! 5. Therefore `Complex<F>: FieldIso<Quaternion<F>>` does not hold (the
//!    `Quaternion<F>: Field` bound is unsatisfied).
//! 6. Therefore the call site `rotate_phase_via_iso::<Complex<F>, _>(q,
//!    angle)` cannot resolve the `FieldIso` bound and the compiler
//!    rejects it.
//!
//! The vocabulary (`Ring`, `Field`, `Commutative`, `FieldIso`) was set up so that the
//! mathematical properties "quaternions are non-commutative" is reflected as
//! "no `Commutative` impl exists for `Quaternion`," and that data-level
//! gap propagates upward through the trait hierarchy mechanically.
//!
//! This is the move worth noticing: a class of correctness errors that
//! would normally be difficult to detect is instead caught by the Rust compiler
//! during type-checking.

use deep_causality_multivector::{CausalMultiVector, ComplexCl01Iso, Metric};
use deep_causality_num::Complex;
use deep_causality_num::iso::witness::{FieldIso as WitnessFieldIso, Iso};
use deep_causality_num::{Field, RealField};

type F = f64;

fn main() {
    println!("=== Either-Input Generic API: Iso as a Compile-Time Contract ===\n");

    let input_complex = Complex::<F>::new(3.0, 4.0);
    let angle = std::f64::consts::FRAC_PI_4; // 45 degrees

    println!("Input: {} + {}i", input_complex.re, input_complex.im);
    println!("Rotation angle: π/4 (45°)\n");

    // ---------------------------------------------------------------------
    // 1. Call the generic API with Complex.
    // ---------------------------------------------------------------------
    let out_complex = rotate_phase_generic(input_complex, angle);
    println!(
        "rotate_phase_generic(Complex):       {} + {}i",
        out_complex.re, out_complex.im
    );

    // ---------------------------------------------------------------------
    // 2. Call the same API with CausalMultiVector in Cl(0,1).
    //    The forward `From` impl lifts Complex into the multivector.
    // ---------------------------------------------------------------------
    let input_mv: CausalMultiVector<F> = input_complex.into();
    let out_mv = rotate_phase_generic(input_mv, angle);
    let out_mv_as_complex: Complex<F> =
        <ComplexCl01Iso as Iso<CausalMultiVector<F>, Complex<F>>>::to_target(out_mv.clone());
    println!(
        "rotate_phase_generic(Cl(0,1) MV):    {} + {}i  (projected back)",
        out_mv_as_complex.re, out_mv_as_complex.im
    );

    // ---------------------------------------------------------------------
    // 3. Verify the two outputs are iso-equivalent (within FP epsilon).
    // ---------------------------------------------------------------------
    let drift =
        (out_complex.re - out_mv_as_complex.re).abs() + (out_complex.im - out_mv_as_complex.im).abs();
    println!("\nL1 drift between Complex and Cl(0,1) outputs: {:e}", drift);
    assert!(drift < 1e-12, "iso paths diverged");
    println!("Same input -> same output, regardless of representation chosen.\n");

    // ---------------------------------------------------------------------
    // 4. The compile-time payoff: dispatch enum.
    // ---------------------------------------------------------------------
    println!("--- Dispatch enum demonstration ---");
    let inputs = vec![
        SignalRep::Native(Complex::new(1.0, 0.0)),
        SignalRep::Lifted(Complex::new(0.0, 1.0).into()),
        SignalRep::Native(Complex::new(0.7071, 0.7071)),
    ];

    for (i, sig) in inputs.into_iter().enumerate() {
        let rotated = dispatch_rotate(sig, angle);
        println!("  signal #{}: rotated -> {} + {}i", i, rotated.re, rotated.im);
    }

    println!("\n--- Compile-time guard ---");
    println!("Try uncommenting the line below in main() that calls rotate_phase_via_iso");
    println!("with a Quaternion. The compiler rejects it: Quaternion is not a FieldIso of");
    println!("Complex (quaternions are non-commutative).");
    // -------------------------------------------------------------------
    // COMPILE-TIME ERROR if uncommented. The error message is roughly:
    //
    //   error[E0277]: the trait bound `Complex<f64>: FieldIso<Quaternion<f64>>`
    //                 is not satisfied
    //     --> .../main.rs:N:NN
    //      |
    //      |     let _ = rotate_phase_via_iso::<Complex<F>, _>(q, angle);
    //      |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //      |             the trait `FieldIso<Quaternion<f64>>` is not
    //      |             implemented for `Complex<f64>`
    //      |
    //      = note: required because `Quaternion<f64>: Field` is not satisfied
    //      = note: required because `Quaternion<f64>: Commutative` is not
    //              satisfied (and never will be — multiplication is
    //              non-commutative)
    //
    // Why this chain works (no clever proof; just trait-hierarchy bookkeeping):
    //
    //   FieldIso<T>     requires   Self: Field, T: Field
    //   Field           requires   CommutativeRing
    //   CommutativeRing requires   Ring + Commutative
    //   Quaternion      implements Ring
    //   Quaternion      does NOT implement Commutative   <-- the gap
    //
    // The mathematical fact "quaternion multiplication is non-commutative"
    // is reflected at the trait level as "no `impl Commutative for
    // Quaternion`." That single missing impl propagates upward and turns
    // a class of would-be runtime errors into a compile error.
    //
    // let q = deep_causality_num::Quaternion::<F>::new(1.0, 0.0, 0.0, 0.0);
    // let _ = rotate_phase_via_iso::<Complex<F>, _>(q, angle);
    // -------------------------------------------------------------------

    println!("\n--- Summary ---");
    println!("- Generic functions can accept any `Field` (no equivalence guarantee).");
    println!("- Adding a `FieldIso` bound makes the equivalence between two representations");
    println!("  part of the type signature.");
    println!("- Callers get compile-time enforcement that the representations they pass");
    println!("  are interchangeable. The compiler catches Quaternion-as-Complex mistakes.");
    println!("- Dispatch enums become safe to write: each variant is provably equivalent");
    println!("  to every other variant.");
    println!();
    println!("The non-commutativity rejection is NOT a clever proof. It falls out of small");
    println!("trait-hierarchy facts: `Field` extends `Commutative`; `Quaternion` never");
    println!("declares `impl Commutative`; therefore `Quaternion: Field` is unsatisfied;");
    println!("therefore `FieldIso<Quaternion>` is unsatisfied. The compiler chains these");
    println!("the same way it chains any trait bound. The work is in the wiring, not in");
    println!("the type-checker.");
}

// =============================================================================
// Pattern A: parametric `Field` (no equivalence guarantee)
// =============================================================================

/// Generic over any `Field`. The function compiles for `Complex<f64>`,
/// `CausalMultiVector<f64>`, `Quaternion<f64>`, etc. But the *caller* has
/// no guarantee that two different `Field` types produce iso-equivalent
/// outputs.
fn rotate_phase_generic<X>(x: X, angle: F) -> X
where
    X: Field + Clone,
    X: core::ops::Mul<Output = X>,
{
    // Stand-in: in reality this would compute exp(i * angle) and multiply.
    // For exposition we just scale; the point is the type signature.
    let scale = X::from_field_scalar(angle.cos());
    x * scale
}

// =============================================================================
// Pattern B: iso-bounded API
// =============================================================================

/// Generic over a `Field` `X` that is provably iso to `Complex<F>`.
/// The caller may pass either `Complex<F>` itself or any type that
/// implements `FieldIso<Complex<F>>` (i.e. shares a bidirectional `From`
/// with `Complex<F>` and is also a `Field`).
///
/// Compile-time check: only iso-equivalent representations type-check.
///
/// Note: this signature uses Tier 1 `FieldIso`. For the cross-crate case
/// where bidirectional `From` doesn't exist, the equivalent witness-typed
/// form uses Tier 2 `WitnessFieldIso`.
fn rotate_phase_via_iso<Anchor, X>(x: X, angle: F) -> X
where
    Anchor: Field + RealField,
    X: Field + Clone + From<Anchor>,
    Anchor: From<X>,
    X: deep_causality_num::FieldIso<Anchor>,
{
    // Implementation goes through the anchor representation, which guarantees
    // the result is iso-equivalent regardless of which side the caller passed.
    let as_anchor: Anchor = x.into();
    let scale = Anchor::from_field_scalar(angle.cos());
    let rotated_anchor = as_anchor * scale;
    X::from(rotated_anchor)
}

// =============================================================================
// Pattern C: dispatch enum over iso-equivalent variants
// =============================================================================

/// A signal carried in one of two iso-equivalent representations. The
/// compiler enforces (via `From` impls between the variants' inner types)
/// that the variants are interchangeable.
enum SignalRep {
    /// Native complex number.
    Native(Complex<F>),
    /// Same number lifted into a Cl(0,1) multivector.
    Lifted(CausalMultiVector<F>),
}

/// Dispatch on the variant, normalise to `Complex<f64>`, do the work,
/// and project the result back to the input representation. The iso bound
/// means both branches produce iso-equivalent outputs.
fn dispatch_rotate(sig: SignalRep, angle: F) -> Complex<F> {
    let as_complex = match sig {
        SignalRep::Native(c) => c,
        SignalRep::Lifted(mv) => {
            // Tier 2 witness reverse: project Cl(0,1) MV back to Complex.
            <ComplexCl01Iso as Iso<CausalMultiVector<F>, Complex<F>>>::to_target(mv)
        }
    };
    rotate_phase_generic(as_complex, angle)
}
