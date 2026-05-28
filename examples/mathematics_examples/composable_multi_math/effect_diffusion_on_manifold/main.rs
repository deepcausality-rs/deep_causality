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

use deep_causality_haft::{CoMonad, Pure};
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ManifoldWitness, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};
use mathematics_examples::effect_helpers::{Process, ProcessWitness, expect_value, fail, ok};

/// `f64` is the right precision here: `alpha = 0.25` and integer initial data
/// produce exactly representable binary fractions at every step. Float106
/// yields no observable gain. Switch to `Float106` only
/// if you push the scheme near its CFL boundary over many steps.
pub type FloatType = f64;

const N_VERTICES: usize = 9;
const N_STEPS: usize = 6;
// 2 alpha < 1 keeps the 1D explicit scheme stable.
fn alpha() -> FloatType {
    FloatType::from(0.25)
}

fn main() {
    println!("=== Diffusion on a Manifold: Comonad (space) x Monad (time) ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());

    // Initial bump centered at index 4.
    let mut initial: Vec<FloatType> = vec![FloatType::from(0.0); N_VERTICES];
    initial[4] = FloatType::from(8.0);
    let manifold = build_manifold(initial);
    println!("t=0 phi: {:?}", snapshot(&manifold));

    let mut process: Process<SimplicialManifold<FloatType, FloatType>> =
        ProcessWitness::pure(manifold);

    for step in 1..=N_STEPS {
        process = process.bind(|m, _, _| diffuse_one_step(m.into_value().expect("manifold")));
        if process.error.is_some() {
            break;
        }
        // Peek at the current value without consuming the chain.
        if let deep_causality_core::EffectValue::Value(ref m) = process.value {
            println!("t={} phi: {:?}", step, snapshot(m));
        }
    }

    println!();
    match process.error {
        Some(e) => println!("Diffusion errored: {}", e),
        None => {
            let final_m = expect_value(&process.value);
            let total: FloatType = snapshot(&final_m)
                .into_iter()
                .fold(FloatType::from(0.0), |acc, v| acc + v);
            println!(
                "Total mass conserved (Neumann boundaries): sum phi = {}",
                total
            );
            println!("Expected initial mass: 8.0");
        }
    }
    println!("\nSpatial step came from `extend`. Temporal step came from `bind`.");
    println!("Both abstractions act on the same value at different layers.");
}

fn build_manifold(vertex_values: Vec<FloatType>) -> SimplicialManifold<f64, FloatType> {
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
    let d1 = CsrMatrix::from_triplets(N_VERTICES, n_edges, &triplets).unwrap();

    let complex = SimplicialComplex::new(vec![skeleton_0, skeleton_1], vec![d1], vec![], vec![]);

    let mut data = vertex_values;
    data.extend(std::iter::repeat_n(FloatType::from(0.0), n_edges));
    let tensor = CausalTensor::new(data, vec![N_VERTICES + n_edges]).unwrap();
    Manifold::new(complex, tensor, 0).expect("manifold")
}

/// One explicit Euler step of the heat equation. Returns a new manifold whose
/// vertex values are `phi + alpha * Delta phi`.
fn diffuse_one_step(
    m: SimplicialManifold<f64, FloatType>,
) -> Process<SimplicialManifold<f64, FloatType>> {
    let two = FloatType::from(2.0);
    let zero = FloatType::from(0.0);
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

    let max_abs = updated
        .data()
        .as_slice()
        .iter()
        .fold(FloatType::from(0.0), |acc, &v| {
            let av = v.abs();
            if av > acc { av } else { acc }
        });
    ok(updated, format!("step ok: max |phi| = {}", max_abs))
}

fn snapshot(m: &SimplicialManifold<f64, FloatType>) -> Vec<FloatType> {
    m.data().as_slice()[..N_VERTICES].to_vec()
}
