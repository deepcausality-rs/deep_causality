/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Diffusion on a Manifold via Stacked Comonad and Monad
//!
//! A scalar field on a 1D manifold evolves under the discrete heat equation
//! `phi_{t+1} = phi_t + alpha * Delta phi_t`. The spatial Laplacian comes
//! from `ManifoldWitness::extend` (comonad). The temporal stepping comes from
//! `CausalEffectPropagationProcessWitness::bind` (monad). Both abstractions
//! act on the same value at different layers.
//!
//! ## APIs Demonstrated
//! - `ManifoldWitness::extend`
//! - `CausalEffectPropagationProcessWitness::pure` and `bind`
//! - Monadic short-circuit on numerical instability

use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Pure};
use deep_causality_linear::CsrMatrix;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ManifoldWitness, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};
use mathematics_examples::effect_helpers::{Process, ProcessWitness, fail, ok};

const N_VERTICES: usize = 9;
const N_STEPS: usize = 6;
// 2 alpha < 1 keeps the 1D explicit scheme stable.
fn alpha() -> FloatType {
    QUARTER
}

/// `f64` is the right precision here: `alpha = 0.25` and integer initial data
/// produce exactly representable binary fractions at every step. Float106
/// yields no observable gain. Switch to `Float106` only
/// if you push the scheme near its CFL boundary over many steps.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const QUARTER: FloatType = const_scalar_from_float!(FloatType, 0.25);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const EIGHT: FloatType = const_scalar_from_int!(FloatType, 8);

fn main() {
    print_header();

    // Initial bump centered at index 4.
    let mut initial: Vec<FloatType> = vec![ZERO; N_VERTICES];
    initial[4] = EIGHT;
    let manifold = build_manifold(initial);
    print_step(0, &shown(&manifold));

    let mut process: Process<SimplicialManifold<FloatType, FloatType>> =
        ProcessWitness::pure(manifold);

    for step in 1..=N_STEPS {
        process = process.bind(|m, _, _| diffuse_one_step(m.into_value().expect("manifold")));
        if process.error().is_some() {
            break;
        }
        // Peek at the current value without consuming the chain.
        if let Some(m) = process.value() {
            print_step(step, &shown(m));
        }
    }

    let total = process
        .value_cloned()
        .map(|final_m| snapshot(&final_m).into_iter().fold(ZERO, |acc, v| acc + v));
    print_outcome(process.error().map(|e| e.to_string()), total);
}

fn build_manifold(vertex_values: Vec<FloatType>) -> SimplicialManifold<FloatType, FloatType> {
    let vertices: Vec<Simplex> = (0..N_VERTICES).map(|i| Simplex::new(vec![i])).collect();
    let skeleton_0 = Skeleton::new(0, vertices);
    let edges: Vec<Simplex> = (0..N_VERTICES - 1)
        .map(|i| Simplex::new(vec![i, i + 1]))
        .collect();
    let skeleton_1 = Skeleton::new(1, edges);

    let n_edges = N_VERTICES - 1;
    let mut triplets: Vec<(usize, usize, i8)> = Vec::with_capacity(2 * n_edges);
    for e in 0..n_edges {
        triplets.push((e, e, -1));
        triplets.push((e + 1, e, 1));
    }
    let d1 = CsrMatrix::from_triplets(N_VERTICES, n_edges, &triplets)
        .expect("two triplets per edge, all in range");

    let complex = SimplicialComplex::new(vec![skeleton_0, skeleton_1], vec![d1], vec![], vec![]);

    let mut data = vertex_values;
    data.extend(std::iter::repeat_n(ZERO, n_edges));
    let tensor = CausalTensor::new(data, vec![N_VERTICES + n_edges])
        .expect("one entry per vertex and per edge");
    Manifold::new(complex, tensor, 0).expect("manifold")
}

/// One explicit Euler step of the heat equation. Returns a new manifold whose
/// vertex values are `phi + alpha * Delta phi`.
fn diffuse_one_step(
    m: SimplicialManifold<FloatType, FloatType>,
) -> Process<SimplicialManifold<FloatType, FloatType>> {
    let two = TWO;
    let zero = ZERO;
    let a = alpha();

    let updated = ManifoldWitness::extend(&m, |w| {
        let i = w.cursor();
        let data = w.data().as_slice();
        if i >= N_VERTICES {
            return zero;
        }
        let phi_i = data[i];
        let phi_l = if i > 0 { data[i - 1] } else { phi_i };
        let phi_r = if i + 1 < N_VERTICES {
            data[i + 1]
        } else {
            phi_i
        };
        let laplacian = phi_l + phi_r - two * phi_i;
        phi_i + a * laplacian
    });

    let any_nan = updated.data().as_slice().iter().any(|v| !v.is_finite());
    if any_nan {
        return fail("non-finite value detected in diffusion step");
    }

    let max_abs = updated.data().as_slice().iter().fold(ZERO, |acc, &v| {
        let av = Real::abs(v);
        if av > acc { av } else { acc }
    });
    ok(updated, format!("step ok: max |phi| = {}", lower(max_abs)))
}

fn snapshot(m: &SimplicialManifold<FloatType, FloatType>) -> Vec<FloatType> {
    m.data().as_slice()[..N_VERTICES].to_vec()
}

/// The display boundary: `f64` appears here and nowhere else.
fn shown(m: &SimplicialManifold<FloatType, FloatType>) -> Vec<f64> {
    snapshot(m).into_iter().map(lower).collect()
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Diffusion on a Manifold: Comonad (space) x Monad (time) ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_step(step: usize, phi: &[f64]) {
    println!("t={step} phi: {phi:?}");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_outcome(error: Option<String>, total: Option<FloatType>) {
    println!();
    match error {
        Some(e) => println!("Diffusion errored: {e}"),
        None => {
            let sum = total.expect("a chain with no error carries a value");
            println!(
                "Total mass conserved (Neumann boundaries): sum phi = {}",
                lower(sum)
            );
            println!("Expected initial mass: 8.0");
        }
    }
    println!("\nSpatial step came from `extend`. Temporal step came from `bind`.");
    println!("Both abstractions act on the same value at different layers.");
}
