/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Spinor transport along a Minkowski path
//!
//! Four crates and the causal monad cooperate in one program, each supplying the one thing it
//! owns:
//!
//! ```text
//! deep_causality_topology      a 1D timelike path as vertices, edges and a boundary operator
//! deep_causality_tensor        per-edge boost rapidities in a CausalTensor
//! deep_causality_multivector   the spinor and the boost rotors in Cl(3,1), signature (+,-,-,-)
//! deep_causality_core          the per-edge chain, with a step log and a stability check
//! ```
//!
//! A unit timelike vector `ψ = e₀` is parallel-transported edge by edge. At each edge the local
//! rapidity `θᵢ` builds the boost rotor `Bᵢ = cosh(θᵢ/2) − sinh(θᵢ/2)·e₀∧e₁`, and the spinor
//! updates by the sandwich `ψ → Bᵢ ψ ~Bᵢ`. Boosts along one plane compose by adding rapidities, so
//! the four steps together produce `θ_total = Σ θᵢ`, and the final check reads `cosh θ_total` and
//! `sinh θ_total` straight off the transported spinor.
//!
//! What the program exercises:
//!
//! * A simplicial complex with an explicit boundary operator.
//! * A 16-dimensional Clifford algebra with Minkowski signature.
//! * Hyperbolic boost rotors as bivector exponentials.
//! * The sandwich transformation `ψ → B ψ ~B`, a representation-theoretic action.
//! * A monadic pipeline with state threading, log accumulation and error short-circuit.
//! * A stability invariant, the timelike norm, checked after every step.
//!
//! `main` stays short because all of it sits behind four call sites: `Manifold::new`,
//! `CausalMultiVector::new`, `geometric_product` and `ProcessWitness::bind`.

use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Pure};
use deep_causality_linear::CsrMatrix;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{Float106, const_scalar_from_float, const_scalar_from_int, lift, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ManifoldWitness, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};
use mathematics_examples::effect_helpers::{Process, ProcessWitness, fail, ok, print_log};

/// The rapidity carried by each edge of the path. Their sum is the total boost.
const RAPIDITIES: [f64; 4] = [0.10, 0.15, 0.20, 0.25];
/// One edge per rapidity, and one vertex more than that.
const N_EDGES: usize = RAPIDITIES.len();
const N_VERTICES: usize = N_EDGES + 1;

/// `Cl(3,1)` has four dimensions and `2^4` coefficients.
const DIMENSION: usize = 4;
const COEFFICIENTS: usize = 16;

/// Blade indices in `Cl(3,1)`: basis vector `eᵢ` owns bit `i`, and the index is the bitmask.
const I_SCALAR: usize = 0b0000; // 1
const I_E0: usize = 0b0001; //     e0, timelike, e0² = +1
const I_E1: usize = 0b0010; //     e1, spacelike, e1² = -1
const I_E2: usize = 0b0100; //     e2
const I_E3: usize = 0b1000; //     e3
const I_E01: usize = I_E0 | I_E1; // e0∧e1, the boost generator, squares to +1

/// How far the timelike norm may move from 1 before a step is reported as unstable.
const NORM_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-9);

/// The working scalar. Switch the alias to `f32` for low precision, `f64` for standard, or
/// `Float106` for the double-double carried here.
pub type FloatType = Float106;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rapidities: Vec<FloatType> = RAPIDITIES.iter().map(|x| lift::<FloatType>(*x)).collect();
    let theta_total = rapidities.iter().fold(ZERO, |acc, v| acc + *v);

    print_header();
    print_path(&rapidities, theta_total);

    // The initial spinor: the timelike unit vector ψ = e₀.
    let mut psi_data = vec![ZERO; COEFFICIENTS];
    psi_data[I_E0] = ONE;
    let psi = CausalMultiVector::new(psi_data, Metric::Minkowski(DIMENSION))?;

    let manifold = build_path_manifold(&rapidities)?;

    // One bind per edge. Each closure captures the manifold by reference and reads its own
    // rapidity out of it; a failing step short-circuits the rest of the chain.
    let mut process: Process<CausalMultiVector<FloatType>> = ProcessWitness::pure(psi);
    for edge in 0..N_EDGES {
        process = process.bind(|p, _, _| match p.into_value() {
            Some(spinor) => transport_across_edge(spinor, &manifold, edge),
            None => fail(format!(
                "edge {edge}: the chain arrived with an empty value"
            )),
        });
        if process.error().is_some() {
            break;
        }
    }

    print_result(theta_total, &process);
    Ok(())
}

/// The discretized timelike path, with per-edge rapidities held in the data tensor. Vertex
/// entries hold zero and edge entries hold the rapidity.
fn build_path_manifold(
    rapidities: &[FloatType],
) -> Result<SimplicialManifold<FloatType, FloatType>, Box<dyn std::error::Error>> {
    let vertices: Vec<Simplex> = (0..N_VERTICES).map(|i| Simplex::new(vec![i])).collect();
    let edges: Vec<Simplex> = (0..N_EDGES).map(|i| Simplex::new(vec![i, i + 1])).collect();

    // The boundary operator ∂₁ carries incidence signs: an edge leaves one vertex and enters the
    // next, so the entries are integers and stay clear of the working scalar.
    let mut triplets: Vec<(usize, usize, i8)> = Vec::with_capacity(2 * N_EDGES);
    for e in 0..N_EDGES {
        triplets.push((e, e, -1));
        triplets.push((e + 1, e, 1));
    }
    let d1 = CsrMatrix::from_triplets(N_VERTICES, N_EDGES, &triplets)?;

    let complex = SimplicialComplex::new(
        vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)],
        vec![d1],
        vec![],
        vec![],
    );

    let mut data = vec![ZERO; N_VERTICES];
    data.extend_from_slice(rapidities);
    let tensor = CausalTensor::new(data, vec![N_VERTICES + N_EDGES])?;

    Ok(Manifold::new(complex, tensor, 0)?)
}

/// The rapidity stored on edge `e`, read by moving the comonadic cursor onto that cell and
/// calling `extract`. The manifold supplies the addressing and the tensor supplies the storage.
fn read_edge_rapidity(
    m: &SimplicialManifold<FloatType, FloatType>,
    e: usize,
) -> Result<FloatType, Box<dyn std::error::Error>> {
    let cursor = N_VERTICES + e;
    let repositioned = Manifold::new(m.complex().clone(), m.data().clone(), cursor)?;

    Ok(ManifoldWitness::extract(&repositioned))
}

/// The boost rotor for rapidity `θ` in the `e₀∧e₁` plane, together with its reverse.
///
/// `e₀∧e₁` squares to `+1` in `Cl(3,1)`, so the exponential opens into hyperbolic functions:
/// `B = cosh(θ/2) − sinh(θ/2)·e₀∧e₁`. The sign puts the sandwich `B ψ ~B` on the standard active
/// convention `(t, x) → (cosh θ·t + sinh θ·x, …)`. Reversion flips the sign of the grade-2 part.
fn boost_rotor(
    theta: FloatType,
) -> Result<(CausalMultiVector<FloatType>, CausalMultiVector<FloatType>), Box<dyn std::error::Error>>
{
    let metric = Metric::Minkowski(DIMENSION);
    let half = theta / TWO;
    let c = half.cosh();
    let s = half.sinh();

    let mut b = vec![ZERO; COEFFICIENTS];
    b[I_SCALAR] = c;
    b[I_E01] = -s;

    let mut b_rev = vec![ZERO; COEFFICIENTS];
    b_rev[I_SCALAR] = c;
    b_rev[I_E01] = s;

    Ok((
        CausalMultiVector::new(b, metric)?,
        CausalMultiVector::new(b_rev, metric)?,
    ))
}

/// One parallel-transport step across edge `e`: read the rapidity, build the rotor, apply
/// `ψ → B ψ ~B`, and check that the step left the timelike norm where it was.
fn transport_across_edge(
    psi: CausalMultiVector<FloatType>,
    manifold: &SimplicialManifold<FloatType, FloatType>,
    e: usize,
) -> Process<CausalMultiVector<FloatType>> {
    let theta = match read_edge_rapidity(manifold, e) {
        Ok(theta) => theta,
        Err(err) => return fail(format!("edge {e}: {err}")),
    };
    let (b, b_rev) = match boost_rotor(theta) {
        Ok(pair) => pair,
        Err(err) => return fail(format!("edge {e}: {err}")),
    };
    let new_psi = b.geometric_product(&psi).geometric_product(&b_rev);

    // The stability invariant: a boost preserves the timelike norm, so |ψ|² stays at +1.
    let d = new_psi.data();
    let norm_sq = d[I_E0] * d[I_E0] - d[I_E1] * d[I_E1] - d[I_E2] * d[I_E2] - d[I_E3] * d[I_E3];
    if !norm_sq.is_finite() {
        return fail(format!("edge {e}: the norm left the finite range"));
    }
    let drift = (norm_sq - ONE).abs();
    if drift > NORM_TOLERANCE {
        return fail(format!(
            "edge {e}: norm drift {} exceeds the tolerance",
            lower(drift)
        ));
    }

    let msg = format!(
        "edge {e} theta={}: psi e0={} e1={} |psi|^2={}",
        lower(theta),
        lower(d[I_E0]),
        lower(d[I_E1]),
        lower(norm_sq)
    );
    ok(new_psi, msg)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Spinor Transport in Minkowski Cl(3,1) ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_path(rapidities: &[FloatType], theta_total: FloatType) {
    let shown: Vec<f64> = rapidities.iter().map(|r| lower(*r)).collect();
    println!("Path: {N_VERTICES} vertices, {N_EDGES} edges");
    println!("Rapidities per edge: {shown:?}");
    println!("Expected total rapidity: {}\n", lower(theta_total));
}

fn print_result(theta_total: FloatType, process: &Process<CausalMultiVector<FloatType>>) {
    println!("Per-edge log:");
    print_log(process.logs());

    if let Some(err) = process.error() {
        println!("\nTransport errored: {err}");
        return;
    }

    match process.value_cloned() {
        Some(final_psi) => print_composition(theta_total, &final_psi),
        None => println!("\nThe chain finished with an empty value slot."),
    }
}

fn print_composition(theta_total: FloatType, final_psi: &CausalMultiVector<FloatType>) {
    let d = final_psi.data();
    let observed_e0 = d[I_E0];
    let observed_e1 = d[I_E1];
    let expected_e0 = theta_total.cosh();
    let expected_e1 = theta_total.sinh();
    let drift = (observed_e0 - expected_e0).abs() + (observed_e1 - expected_e1).abs();

    println!();
    println!("Final spinor components:");
    println!(
        "  observed e0 = {}, expected cosh(theta) = {}",
        lower(observed_e0),
        lower(expected_e0)
    );
    println!(
        "  observed e1 = {}, expected sinh(theta) = {}",
        lower(observed_e1),
        lower(expected_e1)
    );
    println!("  composition drift = {:.3e}", lower(drift));
    println!();
    println!("Topology supplied the path. Tensor stored the per-edge data.");
    println!("Multivector built the boost rotors. The causal monad ordered the");
    println!("steps and watched the stability invariant. One uniform composition.");
}
