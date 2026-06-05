// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! # Cubical Heat Diffusion
//!
//! Demonstrates Stage C of the cubical-complex change set: a
//! `Manifold<CubicalComplex<2, FloatType>, FloatType>` carrying a scalar heat field,
//! evolved 10 explicit-Euler steps with a Moore-neighborhood stencil. Prints an ASCII
//! heatmap after each step.
//!
//! The stencil for cell `c` is:
//!
//!     u'[c] = u[c] + α * (Σ_{n ∈ Moore(c)} u[n] − |Moore(c)| · u[c])
//!
//! This is the discrete Laplacian on the Moore neighborhood. Boundary cells trim the
//! neighborhood naturally because the grid is open (non-periodic) and the `Moore`
//! strategy omits out-of-bounds candidates.
//!
//! ## Precision as a parameter
//!
//! Precision is declared once at the top of `main` via the `FloatType` alias and
//! threaded through every numerical site below. Swap `f64` for `f32`, `Float106`, or
//! any other `RealField` implementor to re-run at a different precision without
//! touching the algorithm.

use deep_causality_calculus::Euler;
use deep_causality_haft::Arrow;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{CubicalComplex, Manifold, Moore};
use std::ops::{Add, Mul};

/// `f64` is the right precision here: the explicit-Euler loop is short and the
/// neighborhood stencil is local. Higher precision yields no observable gain.
pub type FloatType = f64;

const N: usize = 16; // grid side (top cubes are (N-1)×(N-1))
const STEPS: usize = 10;
const ALPHA: FloatType = 0.15;

fn main() {
    let complex = CubicalComplex::<2, FloatType>::open([N, N]);
    let top_n = N - 1; // open: top-cube positions are 0..N-1 per axis
    let cell_count = top_n * top_n;

    // Initial condition: 1.0 at the center cell, 0.0 elsewhere.
    let center = (top_n / 2) + (top_n / 2) * top_n;
    let mut data = vec![0.0 as FloatType; cell_count];
    data[center] = 1.0;

    let manifold: Manifold<CubicalComplex<2, FloatType>, FloatType> =
        Manifold::from_cubical(complex, CausalTensor::from_vec(data, &[cell_count]), 0);

    println!("== Step 0 ==");
    print_heatmap(manifold.data().as_slice(), top_n);

    // The Moore neighborhood depends only on the complex, not the field, so precompute it once and
    // the rate field becomes a pure `Fn(&Field) -> Field` — the discrete Laplacian (spatial
    // operator), unchanged.
    let neighbors: Vec<Vec<usize>> = (0..cell_count)
        .map(|c| manifold.neighbors(Moore, c).collect())
        .collect();

    // The explicit-Euler *time* update `u' = u + α·Δu` becomes an `Euler` endo-arrow with dt = α.
    // Only the time integration is encapsulated here; the stencil is untouched. Swapping `Euler`
    // for `Rk4` raises the time order with no change to the rate field.
    let step = Euler::new(ALPHA, move |f: &Field| {
        Field(
            (0..cell_count)
                .map(|c| {
                    // Discrete Laplacian: Σ u[n] − k · u[c], where k = |Moore(c)|.
                    let acc: FloatType = neighbors[c].iter().map(|&nb| f.0[nb]).sum();
                    acc - (neighbors[c].len() as FloatType) * f.0[c]
                })
                .collect(),
        )
    });

    let mut field = Field(manifold.data().as_slice().to_vec());
    for s in 1..=STEPS {
        field = step.run(field);
        println!("== Step {s} ==");
        print_heatmap(&field.0, top_n);
    }

    println!("\nDone. {STEPS} Euler steps on {top_n}×{top_n} top cubes with α = {ALPHA}.");
}

/// The scalar heat field — the integrator state. `Euler`/`Rk4` need a module-valued state
/// (`Add` + scalar `Mul`), which `Vec<FloatType>` lacks, so the field rides in this newtype.
#[derive(Clone)]
struct Field(Vec<FloatType>);

impl Add for Field {
    type Output = Field;
    fn add(self, rhs: Field) -> Field {
        Field(self.0.iter().zip(rhs.0).map(|(a, b)| a + b).collect())
    }
}

impl Mul<FloatType> for Field {
    type Output = Field;
    fn mul(self, s: FloatType) -> Field {
        Field(self.0.iter().map(|x| x * s).collect())
    }
}

fn print_heatmap(values: &[FloatType], side: usize) {
    // Map values to a small ASCII gradient.
    let max = values
        .iter()
        .cloned()
        .fold(0.0 as FloatType, FloatType::max)
        .max(1e-12);
    let ramp = [' ', '.', ':', '-', '+', '*', '#', '@'];
    for row in 0..side {
        let mut line = String::with_capacity(side * 2);
        for col in 0..side {
            let v = values[col + row * side];
            let bucket = ((v / max) * (ramp.len() as FloatType - 1.0)).round() as usize;
            let bucket = bucket.min(ramp.len() - 1);
            line.push(ramp[bucket]);
            line.push(' ');
        }
        println!("{line}");
    }
}
