/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Duality: Stokes' theorem as an adjunction
//!
//! Two functors face each other across a simplicial complex:
//!
//! ```text
//! ExteriorDerivativeWitness   Type<T> = DifferentialForm<T>     the left functor,  d
//! BoundaryWitness<R>          Type<G> = Chain<R, G>             the right functor, ∂
//! ```
//!
//! `StokesAdjunction` relates them as `d ⊣ ∂`, which is Stokes' theorem read as a pairing:
//!
//! ```text
//! ⟨dω, C⟩ = ⟨ω, ∂C⟩
//! ```
//!
//! Integrating the derivative of a form over a region equals integrating the form over that
//! region's boundary. The adjunction gives four operations, and which two can fail is the part
//! worth reading:
//!
//! ```text
//! unit(ctx, a)                 A -> Chain<DifferentialForm<A>>          total: it builds
//! left_adjunct(ctx, a, f)      (A, Form<A> -> B) -> Chain<B>            total: it builds
//! counit(ctx, lrb)             Form<Chain<B>> -> Result<B>              partial: it extracts
//! right_adjunct(ctx, rla, f)   (Form<A>, A -> Chain<B>) -> Result<B>    partial: it extracts
//! ```
//!
//! A container can be empty, and a `Chain` whose weights are a sparse matrix storing no explicit
//! entry is exactly that: CSR drops explicit zeros, so an all-zero chain has nothing to extract.
//! The two extracting operations therefore return `Result` rather than assuming a value is there.

use deep_causality_haft::{Adjunction, Pure};
use deep_causality_linear::{CsrMatrix, CsrMatrixWitness};
use deep_causality_num::const_scalar_from_int;
use deep_causality_topology::{
    BoundaryWitness, Chain, DifferentialForm, ExteriorDerivativeWitness, Simplex,
    SimplicialComplex, Skeleton, StokesAdjunction, StokesContext,
};

/// The adjunction, named once so the call sites read as mathematics.
type Stokes = StokesAdjunction;

/// Vertices of the interval complex below.
const N_VERTICES: usize = 4;

/// The working scalar. The complex's incidence weights carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const TWENTY: FloatType = const_scalar_from_int!(FloatType, 20);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // A 1D interval complex: four vertices joined by three edges.
    let ctx = StokesContext::new(build_interval_complex());
    print_context(ctx.dim(), ctx.num_simplices(0), ctx.num_simplices(1));

    // ---------------------------------------------------------------------
    // 1. `unit`: A -> R(L(A)). Total, because it builds.
    // ---------------------------------------------------------------------
    // A bare coefficient becomes a chain of 0-forms: the constant field with that value.
    let temperature = TWENTY;
    let embedded: Chain<FloatType, DifferentialForm<FloatType>> = <Stokes as Adjunction<
        ExteriorDerivativeWitness,
        BoundaryWitness<FloatType>,
        StokesContext<FloatType>,
    >>::unit(&ctx, temperature);
    print_unit(embedded.grade());

    // ---------------------------------------------------------------------
    // 2. `left_adjunct`: (A, Form<A> -> B) -> Chain<B>. Total, because it builds.
    // ---------------------------------------------------------------------
    // The transposing operation. A function defined on forms becomes a chain, without the
    // caller ever assembling the form itself.
    let as_chain: Chain<FloatType, usize> =
        <Stokes as Adjunction<
            ExteriorDerivativeWitness,
            BoundaryWitness<FloatType>,
            StokesContext<FloatType>,
        >>::left_adjunct(&ctx, temperature, |form: DifferentialForm<FloatType>| {
            // Anything computed from the form; here its coefficient count.
            form.coefficients().as_slice().len()
        });
    print_left_adjunct(as_chain.grade());

    // ---------------------------------------------------------------------
    // 3. `right_adjunct`: the partial direction, and what it does when empty.
    // ---------------------------------------------------------------------
    // Extraction can find nothing, so the result is a `Result`. Asking for a value out of a
    // chain built to be empty is the reachable case, not a corner case.
    let form = DifferentialForm::from_coefficients(0, ctx.dim(), vec![temperature]);
    let extracted = <Stokes as Adjunction<
        ExteriorDerivativeWitness,
        BoundaryWitness<FloatType>,
        StokesContext<FloatType>,
    >>::right_adjunct(&ctx, form, |a: FloatType| {
        // A chain carrying one explicit weight, so there is something to extract.
        Chain::new(
            ctx.complex_arc(),
            0,
            <CsrMatrixWitness as Pure<CsrMatrixWitness>>::pure(a),
        )
    });
    print_right_adjunct(&extracted.map(|_| ()).map_err(|e| e.to_string()));

    print_footer();
    Ok(())
}

/// A 1D interval: four vertices, three edges, with the boundary operator `d1`.
///
/// Each edge `(i, i+1)` contributes `-1` to row `i` and `+1` to row `i + 1`, which is the
/// discrete `∂` the right functor is named for.
fn build_interval_complex() -> SimplicialComplex<FloatType> {
    let vertices: Vec<Simplex> = (0..N_VERTICES).map(|i| Simplex::new(vec![i])).collect();
    let edges: Vec<Simplex> = (0..N_VERTICES - 1)
        .map(|i| Simplex::new(vec![i, i + 1]))
        .collect();
    let n_edges = edges.len();

    let mut triplets: Vec<(usize, usize, i8)> = Vec::with_capacity(2 * n_edges);
    for e in 0..n_edges {
        triplets.push((e, e, -1));
        triplets.push((e + 1, e, 1));
    }
    let d1 = CsrMatrix::from_triplets(N_VERTICES, n_edges, &triplets)
        .expect("two triplets per edge, all in range");

    SimplicialComplex::new(
        vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)],
        vec![d1],
        vec![],
        vec![],
    )
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Duality: Stokes' theorem as an adjunction (d ⊣ ∂) ===\n");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
    println!("  ⟨dω, C⟩ = ⟨ω, ∂C⟩");
    println!("  the derivative of a form over a region, or the form over its boundary\n");
}

fn print_context(dim: usize, vertices: usize, edges: usize) {
    println!("--- The complex both functors sit over ---");
    println!("  dimension : {dim}");
    println!("  vertices  : {vertices}");
    println!("  edges     : {edges}");
}

fn print_unit(grade: usize) {
    println!("\n--- 1. unit: A -> Chain<DifferentialForm<A>>  (total) ---");
    println!("  a bare coefficient becomes a chain of 0-forms");
    println!("  resulting chain grade: {grade}");
}

fn print_left_adjunct(grade: usize) {
    println!("\n--- 2. left_adjunct: (A, Form<A> -> B) -> Chain<B>  (total) ---");
    println!("  a function on forms becomes a chain, with no form assembled by the caller");
    println!("  resulting chain grade: {grade}");
}

fn print_right_adjunct(outcome: &Result<(), String>) {
    println!("\n--- 3. right_adjunct: (Form<A>, A -> Chain<B>) -> Result<B>  (partial) ---");
    match outcome {
        Ok(()) => println!("  extracted a value: the chain carried a weight"),
        Err(e) => println!("  nothing to extract: {e}"),
    }
}

fn print_footer() {
    println!("\n--- Why two of the four return Result ---");
    println!("  unit and left_adjunct build a structure, so there is always one to return.");
    println!("  counit and right_adjunct extract a bare value, and a container can be empty:");
    println!("  a Chain stores its weights as CSR, which drops explicit zeros, so an all-zero");
    println!("  chain holds nothing. That is reachable input, so it is a Result, not a panic.");
}
