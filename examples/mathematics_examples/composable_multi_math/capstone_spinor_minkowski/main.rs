/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Capstone: Spinor Transport Along a Minkowski Path
//!
//! All three crates plus the causal monad cooperate in one program:
//!
//! - `deep_causality_topology::Manifold` discretizes a 1D timelike path with
//!   vertices and edges.
//! - `deep_causality_tensor::CausalTensor` stores per-edge boost rapidities.
//! - `deep_causality_multivector::CausalMultiVector` (signature `Cl(3,1)`,
//!   convention `(+,-,-,-)`) holds the spinor and builds the boost rotors.
//! - `deep_causality_core::CausalEffectPropagationProcess` chains the per-edge
//!   transports with a step log and a numerical-stability check.
//!
//! A unit timelike vector `psi = e0` is parallel-transported edge by edge.
//! At each edge the local rapidity `theta_i` builds the boost rotor
//! `B_i = cosh(theta_i / 2) + sinh(theta_i / 2) * e0^e1`, and the spinor
//! updates as `psi -> B_i psi B_i~`. The accumulated boost equals
//! `theta_total = sum theta_i`, which the final check verifies.
//!
//! ## APIs Demonstrated
//! - All HKT-based crates threaded through one monadic chain
//! - Bit-string basis indexing in `CausalMultiVector` (`Cl(3,1)`)
//! - Composition of rotors via repeated `bind`

use deep_causality_haft::{CoMonad, Pure};
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{Float106, RealField};
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ManifoldWitness, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};
use mathematics_examples::effect_helpers::{
    Process, ProcessWitness, expect_value, fail, ok, print_log,
};

const N_VERTICES: usize = 5;
const N_EDGES: usize = N_VERTICES - 1;

// Cl(3,1) bit-string basis indices used in this example.
// Each basis vector ei corresponds to bit i; the index is the bitmask.
const I_SCALAR: usize = 0b0000; //  0  -> 1
const I_E0: usize = 0b0001; //     1  -> e0   (timelike, e0^2 = +1)
const I_E1: usize = 0b0010; //     2  -> e1   (spacelike, e1^2 = -1)
const I_E2: usize = 0b0100; //     4  -> e2
const I_E3: usize = 0b1000; //     8  -> e3
const I_E01: usize = 0b0011; //    3  -> e0^e1  (boost generator, squares to +1)

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = Float106;

// The applied mathematical  structures:
//
// * A simplicial complex with explicit boundary operator (vertices, edges, d1).
// * A 16-dimensional Clifford algebra with Minkowski signature (+, -, -, -).
// * Hyperbolic boost rotors as bivector exponentials.
// * The sandwich transformation psi -> B psi B~ (a representation-theoretic action).
// * A monadic effect pipeline with state threading, log accumulation, and error short-circuit.
// * A stability invariant (timelike norm preservation) checked after every step.
// * Composition of four transport steps that must reproduce the hyperbolic angle-addition identity.
//
// The main is short because all of that lives behind four call sites:
// * Manifold::new,
// * CausalMultiVector::new,
// * geometric_product,
// * ProcessWitness::bind.
fn main() {
    println!("=== Capstone: Spinor Transport in Minkowski Cl(3,1) ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());

    // Four small rapidities; the total boost is their sum.
    let rapidities: Vec<FloatType> = [0.10, 0.15, 0.20, 0.25]
        .iter()
        .map(|x| FloatType::from(*x))
        .collect();

    let theta_total: FloatType = rapidities
        .iter()
        .fold(FloatType::from(0.0), |acc, v| acc + *v);

    println!("Path: {} vertices, {} edges", N_VERTICES, N_EDGES);
    println!("Rapidities per edge: {:?}", rapidities);
    println!("Expected total rapidity: {}\n", theta_total);

    // Initial spinor: pure timelike unit vector psi = e0.
    let mut psi0 = vec![FloatType::from(0.0); 16];
    psi0[I_E0] = FloatType::from(1.0);
    let psi = CausalMultiVector::new(psi0, Metric::Minkowski(4)).unwrap();

    // Topoligical manifold
    let manifold = build_path_manifold(&rapidities);

    // Chain one bind per edge. The manifold is captured by reference inside
    // each closure; the rapidity is read out by `read_edge_rapidity`.
    let mut process: Process<CausalMultiVector<FloatType>> = ProcessWitness::pure(psi);
    for e in 0..N_EDGES {
        process = process.bind(|p, _, _| {
            transport_across_edge(p.into_value().expect("spinor"), &manifold, e)
        });
        if process.error.is_some() {
            break;
        }
    }

    print_result(theta_total, &process);
}

/// Build the discretized timelike path manifold with per-edge rapidities stored in the
/// data tensor. Vertex entries hold zero; edge entries hold the rapidity.
fn build_path_manifold(rapidities: &[FloatType]) -> SimplicialManifold<f64, FloatType> {
    assert_eq!(rapidities.len(), N_EDGES);

    let vertices: Vec<Simplex> = (0..N_VERTICES).map(|i| Simplex::new(vec![i])).collect();
    let edges: Vec<Simplex> = (0..N_EDGES).map(|i| Simplex::new(vec![i, i + 1])).collect();

    let mut triplets: Vec<(usize, usize, i8)> = Vec::with_capacity(2 * N_EDGES);
    for e in 0..N_EDGES {
        triplets.push((e, e, -1));
        triplets.push((e + 1, e, 1));
    }
    let d1 = CsrMatrix::from_triplets(N_VERTICES, N_EDGES, &triplets).unwrap();

    let complex = SimplicialComplex::new(
        vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)],
        vec![d1],
        vec![],
        vec![],
    );

    let mut data = vec![FloatType::from(0.0); N_VERTICES];
    data.extend_from_slice(rapidities);
    let tensor = CausalTensor::new(data, vec![N_VERTICES + N_EDGES]).unwrap();
    Manifold::new(complex, tensor, 0).expect("manifold")
}

/// Read the rapidity stored on edge `e` of the manifold by re-positioning the
/// comonadic cursor and using `extract`. This demonstrates the topology and
/// tensor cooperating: the manifold provides the addressing, the tensor
/// provides the storage.
fn read_edge_rapidity(m: &SimplicialManifold<f64, FloatType>, e: usize) -> FloatType {
    let cursor = N_VERTICES + e;
    let repositioned =
        Manifold::new(m.complex().clone(), m.data().clone(), cursor).expect("reposition");
    ManifoldWitness::extract(&repositioned)
}

/// Build the boost rotor for rapidity `theta` along the `e0^e1` plane.
fn boost_rotor(theta: FloatType) -> (CausalMultiVector<FloatType>, CausalMultiVector<FloatType>) {
    let metric = Metric::Minkowski(4);
    let half = theta / FloatType::from(2.0);
    let c = half.cosh();
    let s = half.sinh();

    // Sign chosen so the sandwich `B psi B~` matches the standard active
    // boost convention `(t, x) -> (cosh(theta) t + sinh(theta) x, ...)`.
    // With bivector `e0^e1` (which squares to +1 in Cl(3,1)), the rotor
    // is `B = cosh(theta/2) - sinh(theta/2) * e0^e1`.
    let mut b = vec![FloatType::from(0.0); 16];
    b[I_SCALAR] = c;
    b[I_E01] = -s;
    let rotor = CausalMultiVector::new(b, metric).unwrap();

    let mut b_rev = vec![FloatType::from(0.0); 16];
    b_rev[I_SCALAR] = c;
    b_rev[I_E01] = s; // reverse flips sign of grade-2
    let rotor_rev = CausalMultiVector::new(b_rev, metric).unwrap();

    (rotor, rotor_rev)
}

/// One parallel-transport step across edge `e`. Reads the rapidity from the
/// manifold, builds the rotor, and applies `psi -> B psi B~`.
fn transport_across_edge(
    psi: CausalMultiVector<FloatType>,
    manifold: &SimplicialManifold<f64, FloatType>,
    e: usize,
) -> Process<CausalMultiVector<FloatType>> {
    let theta = read_edge_rapidity(manifold, e);
    let (b, b_rev) = boost_rotor(theta);
    let new_psi = b.geometric_product(&psi).geometric_product(&b_rev);

    // Stability check: the timelike norm |psi|^2 should stay near +1.
    let d = new_psi.data();
    let norm_sq = d[I_E0] * d[I_E0] - d[I_E1] * d[I_E1] - d[I_E2] * d[I_E2] - d[I_E3] * d[I_E3];
    if !norm_sq.is_finite() {
        return fail(format!("edge {}: non-finite norm", e));
    }
    let one = FloatType::from(1.0);
    let tol = FloatType::from(1e-9);
    if (norm_sq - one).abs() > tol {
        return fail(format!(
            "edge {}: norm drift {} exceeds tolerance",
            e,
            (norm_sq - one).abs()
        ));
    }

    let msg = format!(
        "edge {} theta={}: psi e0={} e1={} |psi|^2={}",
        e, theta, d[I_E0], d[I_E1], norm_sq
    );
    ok(new_psi, msg)
}

fn print_result(theta_total: FloatType, process: &Process<CausalMultiVector<FloatType>>) {
    println!("Per-edge log:");
    print_log(&process.logs);

    match &process.error {
        Some(err) => println!("\nTransport errored: {}", err),
        None => {
            let final_psi = expect_value(&process.value);
            let d = final_psi.data();
            let observed_e0 = d[I_E0];
            let observed_e1 = d[I_E1];
            let expected_e0 = theta_total.cosh();
            let expected_e1 = theta_total.sinh();

            println!();
            println!("Final spinor components:");
            println!(
                "  observed e0 = {}, expected cosh(theta) = {}",
                observed_e0, expected_e0
            );
            println!(
                "  observed e1 = {}, expected sinh(theta) = {}",
                observed_e1, expected_e1
            );
            let drift = (observed_e0 - expected_e0).abs() + (observed_e1 - expected_e1).abs();
            println!("  composition drift = {}", drift);
            println!();
            println!("Topology supplied the path. Tensor stored the per-edge data.");
            println!("Multivector built the boost rotors. The causal monad ordered the");
            println!("steps and watched the stability invariant. One uniform composition.");
        }
    }
}
