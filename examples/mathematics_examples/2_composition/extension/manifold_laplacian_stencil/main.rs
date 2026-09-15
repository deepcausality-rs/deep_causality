/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Tensor x Topology: Discrete Laplacian via Comonadic Extension
//!
//! A scalar field lives on the vertices of a 1D simplicial manifold. The
//! discrete Laplacian at vertex `i` is `phi(i-1) + phi(i+1) - 2 phi(i)`.
//! `ManifoldWitness::extend` walks every simplex; the closure reads neighbor
//! values from `w.data()` and computes the stencil.
//!
//! Geometry is the context (manifold). Numbers are the payload (tensor).
//!
//! ## APIs Demonstrated
//! - `SimplicialComplex`, `Skeleton`, `Simplex` construction
//! - `Manifold::new`
//! - `ManifoldWitness::extend` (CoMonad)

use deep_causality_haft::CoMonad;
use deep_causality_linear::CsrMatrix;
use deep_causality_num::{lift, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ManifoldWitness, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};

const N_VERTICES: usize = 7;

/// `f64` is the right precision here: the Laplacian stencil on integer inputs
/// produces integer outputs, so Float106 yields no observable gain.
pub type FloatType = f64;

fn main() {
    print_header();

    // A triangular bump on the vertex field.
    let phi: Vec<FloatType> = [0.0, 1.0, 2.0, 4.0, 2.0, 1.0, 0.0]
        .iter()
        .map(|x| lift::<FloatType>(*x))
        .collect();
    let manifold = build_line_manifold(phi.clone());
    print_field(&phi);

    let two = lift::<FloatType>(2.0);
    let zero = lift::<FloatType>(0.0);

    // Comonadic extension: at each cursor position, compute the stencil.
    let laplacian = ManifoldWitness::extend(&manifold, |w| {
        let i = w.cursor();
        let data = w.data().as_slice();

        // Only vertex simplices participate; edge entries stay at zero.
        if i >= N_VERTICES {
            return zero;
        }

        let phi_i = data[i];
        let phi_left = if i > 0 { data[i - 1] } else { phi_i };
        let phi_right = if i + 1 < N_VERTICES {
            data[i + 1]
        } else {
            phi_i
        };

        phi_left + phi_right - two * phi_i
    });

    let result = laplacian.data().as_slice();
    print_laplacian(&result[..N_VERTICES], &result[N_VERTICES..]);
}

fn build_line_manifold(vertex_values: Vec<FloatType>) -> SimplicialManifold<FloatType, FloatType> {
    assert_eq!(vertex_values.len(), N_VERTICES);

    let vertices: Vec<Simplex> = (0..N_VERTICES).map(|i| Simplex::new(vec![i])).collect();
    let skeleton_0 = Skeleton::new(0, vertices);

    let edges: Vec<Simplex> = (0..N_VERTICES - 1)
        .map(|i| Simplex::new(vec![i, i + 1]))
        .collect();
    let skeleton_1 = Skeleton::new(1, edges);

    // Boundary matrix d1: rows = vertices, cols = edges. Each edge (i, i+1)
    // contributes -1 to row i and +1 to row i+1.
    let n_edges = N_VERTICES - 1;
    let mut triplets: Vec<(usize, usize, i8)> = Vec::with_capacity(2 * n_edges);
    for e in 0..n_edges {
        triplets.push((e, e, -1));
        triplets.push((e + 1, e, 1));
    }
    let d1 = CsrMatrix::from_triplets(N_VERTICES, n_edges, &triplets)
        .expect("two triplets per edge, all in range");

    let complex = SimplicialComplex::new(vec![skeleton_0, skeleton_1], vec![d1], vec![], vec![]);

    // Data layout: vertex values first, then a zero per edge.
    let mut data_vec = vertex_values;
    data_vec.extend(std::iter::repeat_n(lift::<FloatType>(0.0), n_edges));
    let data = CausalTensor::new(data_vec, vec![N_VERTICES + n_edges])
        .expect("one entry per vertex and per edge");

    Manifold::new(complex, data, 0).expect("manifold construction")
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Tensor x Topology: Discrete Laplacian on a 1D Manifold ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_field(phi: &[FloatType]) {
    println!("Vertex field phi: {:?}", shown(phi));
}

fn print_laplacian(vertices: &[FloatType], edges: &[FloatType]) {
    println!("Laplacian (Delta phi) at vertices: {:?}", shown(vertices));
    println!("Laplacian at edges (unused):       {:?}", shown(edges));
    println!("\nThe vertex with the highest value (index 3) has the most negative");
    println!("Laplacian, as expected for a discrete peak. Boundary vertices use");
    println!("Neumann reflection (phi outside = phi at boundary).");
}

/// The display boundary: `f64` appears here and nowhere else.
fn shown(xs: &[FloatType]) -> Vec<f64> {
    xs.iter().map(|&x| lower(x)).collect()
}
