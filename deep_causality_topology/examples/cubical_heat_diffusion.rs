// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! # Cubical Heat Diffusion
//!
//! Demonstrates Stage C of the cubical-complex change set: a `Manifold<CubicalComplex<2>, f64>`
//! carrying a scalar heat field, evolved 10 explicit-Euler steps with a Moore-neighborhood
//! stencil. Prints an ASCII heatmap after each step.
//!
//! The stencil for cell `c` is:
//!
//!     u'[c] = u[c] + α * (Σ_{n ∈ Moore(c)} u[n] − |Moore(c)| · u[c])
//!
//! This is the discrete Laplacian on the Moore neighborhood. Boundary cells trim the
//! neighborhood naturally because the grid is open (non-periodic) and the `Moore`
//! strategy omits out-of-bounds candidates.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{CubicalComplex, Manifold, Moore};

const N: usize = 16; // grid side (top cubes are (N-1)×(N-1))
const STEPS: usize = 10;
const ALPHA: f64 = 0.15;

fn main() {
    let complex = CubicalComplex::<2>::open([N, N]);
    let top_n = N - 1; // open: top-cube positions are 0..N-1 per axis
    let cell_count = top_n * top_n;

    // Initial condition: 1.0 at the center cell, 0.0 elsewhere.
    let center = (top_n / 2) + (top_n / 2) * top_n;
    let mut data = vec![0.0f64; cell_count];
    data[center] = 1.0;

    let mut manifold: Manifold<CubicalComplex<2>, f64> = Manifold::from_cubical(
        complex,
        CausalTensor::from_vec(data.clone(), &[cell_count]),
        0,
    );

    println!("== Step 0 ==");
    print_heatmap(manifold.data().as_slice(), top_n);

    for step in 1..=STEPS {
        let current: &[f64] = manifold.data().as_slice();
        let mut next: Vec<f64> = current.to_vec();
        for c in 0..cell_count {
            let mut acc = 0.0;
            let mut count = 0usize;
            for n in manifold.neighbors(Moore, c) {
                acc += current[n];
                count += 1;
            }
            // Discrete Laplacian: Σ u[n] − k · u[c], where k = |Moore(c)|.
            let laplacian = acc - (count as f64) * current[c];
            next[c] = current[c] + ALPHA * laplacian;
        }
        manifold = Manifold::from_cubical(
            manifold.complex().clone(),
            CausalTensor::from_vec(next, &[cell_count]),
            0,
        );

        println!("== Step {step} ==");
        print_heatmap(manifold.data().as_slice(), top_n);
    }

    println!("\nDone. {STEPS} explicit-Euler steps on {top_n}×{top_n} top cubes with α = {ALPHA}.");
}

fn print_heatmap(values: &[f64], side: usize) {
    // Map values to a small ASCII gradient.
    let max = values.iter().cloned().fold(0.0f64, f64::max).max(1e-12);
    let ramp = [' ', '.', ':', '-', '+', '*', '#', '@'];
    for row in 0..side {
        let mut line = String::with_capacity(side * 2);
        for col in 0..side {
            let v = values[col + row * side];
            let bucket = ((v / max) * (ramp.len() as f64 - 1.0)).round() as usize;
            let bucket = bucket.min(ramp.len() - 1);
            line.push(ramp[bucket]);
            line.push(' ');
        }
        println!("{line}");
    }
}
