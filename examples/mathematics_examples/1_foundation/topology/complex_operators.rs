/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `PointCloud` triangulation and the boundary operators it produces
//!
//! A point cloud becomes a simplicial complex by joining every pair closer than a radius.
//! The complex then carries, per dimension, a boundary operator `∂_k` and its adjoint
//! `∂*_k`, both stored sparsely. Those matrices are what the Laplacian, homology and the
//! discrete exterior calculus are all built from.
//!
//! The operators hold `i8`, not the working scalar: their entries are the incidence signs
//! `-1`, `0` and `+1`, which say how a simplex is oriented against its faces. That is
//! combinatorics rather than measurement, so it carries no precision and gains nothing from
//! a wider float.

use deep_causality_linear::CsrMatrix;
use deep_causality_num::lift;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{PointCloud, SimplicialComplex};

/// The working scalar. Point coordinates and the connection radius carry it.
pub type FloatType = f64;

/// Simplices listed per skeleton.
const SAMPLES: usize = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A unit square: four vertices at its corners.
    let coords: Vec<FloatType> = [
        0.0, 1.0, // v0
        1.0, 1.0, // v1
        0.0, 0.0, // v2
        1.0, 0.0, // v3
    ]
    .iter()
    .map(|&v| lift(v))
    .collect();
    let points = CausalTensor::new(coords, vec![4, 2])?;
    let payload = CausalTensor::new(vec![lift::<FloatType>(0.0); 4], vec![4])?;
    let point_cloud = PointCloud::new(points, payload, 0)?;

    // A radius of 1.5 reaches the sides (length 1) and the diagonals (sqrt(2) = 1.414), so
    // every pair connects and the square fills in.
    let complex = point_cloud.triangulate(lift::<FloatType>(1.5))?;

    print_skeletons(&complex);
    print_operators("Boundary operators", "∂", complex.boundary_operators());
    print_operators("Coboundary operators", "∂*", complex.coboundary_operators());

    Ok(())
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_skeletons(complex: &SimplicialComplex<FloatType>) {
    println!("Simplicial complex:");
    for (i, skel) in complex.skeletons().iter().enumerate() {
        println!("  Skeleton {}: {} simplices", i, skel.simplices().len());
        for (j, simplex) in skel.simplices().iter().take(SAMPLES).enumerate() {
            println!("    Simplex {}: {:?}", j, simplex.vertices());
        }
    }
}

/// The incidence entries are `i8`, so nothing crosses the precision boundary here.
fn print_operators(title: &str, symbol: &str, ops: &[CsrMatrix<i8>]) {
    println!("\n{title}:");
    for (i, op) in ops.iter().enumerate() {
        let (rows, cols) = op.shape();
        println!("  {symbol}_{i}: {rows} x {cols}, nnz={}", op.values().len());
        if op.values().len() <= 20 {
            println!("    values: {:?}", op.values());
        }
    }
}
