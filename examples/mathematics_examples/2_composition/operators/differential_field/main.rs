/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::ops::{Add, Mul};
use deep_causality_algebra::Real;
use deep_causality_calculus::{EndoArrow, Euler};
use deep_causality_num::{lift, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry};

/// `f64` is the right precision for this diffusion demo: short stepping, small triangle. Switch
/// to `f32` or `Float106`; the Laplacian, the Regge metric and the Euler step all follow.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ------------------------------------------------------------------------
    // ENGINEERING VALUE:
    // Many physical processes (diffusion, fluid flow, electromagnetism) are described
    // by partial differential equations (PDEs). When data lives on a complex,
    // non-grid-like structure (like a sensor network or a 3D model), we need
    // tools from differential geometry to solve these equations.
    //
    // This example demonstrates how to solve the Heat Equation (∂u/∂t = -Δu) on a
    // discrete manifold. The Laplacian operator (Δ) measures the local curvature
    // of a field and drives the diffusion process.
    //
    // This allows us to simulate how a quantity (like heat or information) spreads
    // across a complex topology.
    // ------------------------------------------------------------------------

    // 1. Setup (Triangle)
    let zero = lift::<FloatType>(0.0);
    let one = lift::<FloatType>(1.0);
    let points = CausalTensor::new(
        vec![
            zero,
            zero, // v0
            one,
            zero, // v1
            lift::<FloatType>(0.5),
            lift::<FloatType>(0.866), // v2 (Equilateral)
        ],
        vec![3, 2],
    )?;
    let point_cloud = PointCloud::new(points, CausalTensor::new(vec![zero; 3], vec![3])?, 0)?;
    let complex = point_cloud.triangulate(lift::<FloatType>(1.1))?;

    // 2. Initial State (Hot Vertex 0)
    let num_simplices = complex.total_simplices();
    let mut initial_data = vec![zero; num_simplices];
    initial_data[0] = lift::<FloatType>(100.0); // Heat at v0

    let num_edges = complex.skeletons()[1].simplices().len();
    let n_verts = complex.skeletons()[0].simplices().len();

    // 3. Simulation via the Euler integration operator.
    // Heat equation du/dt = −L u. The rate field rebuilds the manifold from the current 0-form,
    // takes its Laplacian (with a unit-edge Regge metric — the impl reads the Hodge ⋆ from the
    // complex's cache and ignores the metric data), and returns −L u on the vertices. The
    // hand-rolled `next[v] -= dt·Δ[v]` loop is now one `Euler` endo-arrow stepped with `iterate_n`.
    let dt = lift::<FloatType>(0.005);
    let steps = 50;

    let rate = move |f: &Field| -> Field {
        let metric = ReggeGeometry::new(
            CausalTensor::new(vec![one; num_edges], vec![num_edges]).expect("one entry per edge"),
        );
        let manifold = Manifold::with_metric(
            complex.clone(),
            CausalTensor::new(f.0.clone(), vec![num_simplices]).expect("one entry per simplex"),
            Some(metric),
            0,
        )
        .expect("the complex and its data agree in length");
        let delta = manifold.laplacian(0);
        let delta = delta.as_slice();
        let mut out = vec![zero; num_simplices];
        for (v, slot) in out.iter_mut().enumerate().take(n_verts) {
            *slot = -delta[v];
        }
        Field(out)
    };
    let stepper = Euler::new(dt, rate);

    print_start();

    let mut field = Field(initial_data);
    for i in 0..=steps {
        field = stepper.iterate_n(field, 1);
        if i % 10 == 0 {
            print_step(i, &field.0[0..3]);
        }
    }

    // Verification: Total Energy Conservation?
    // In a closed system, heat spreads but sum(u_i * Mass_i) should be constant.
    // Or simply, temperature equilibrates.
    let final_v: &[FloatType] = &field.0[0..3];
    let equilibrated =
        Real::abs(final_v[0] - final_v[1]) < one && Real::abs(final_v[0] - final_v[2]) < one;
    print_outcome(final_v, equilibrated);

    Ok(())
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Differential Field Example: Heat Equation ===\n");
}

fn print_start() {
    println!("Starting Diffusion...");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_step(step: usize, v: &[FloatType]) {
    println!(
        "Step {:2}: [{:.2}, {:.2}, {:.2}]",
        step,
        lower(v[0]),
        lower(v[1]),
        lower(v[2])
    );
}

fn print_outcome(v: &[FloatType], equilibrated: bool) {
    println!(
        "Final:   [{:.2}, {:.2}, {:.2}]",
        lower(v[0]),
        lower(v[1]),
        lower(v[2])
    );
    if equilibrated {
        println!(">> SUCCESS: Heat diffused to equilibrium.");
    } else {
        println!(">> WARNING: Non-equilibrium state.");
    }
}

/// The discrete field (one value per simplex), wrapped so it can be the state of an `Euler`
/// endo-arrow: vector addition and scaling by the time step are all the integrator needs.
#[derive(Clone)]
struct Field(Vec<FloatType>);

impl Add for Field {
    type Output = Field;
    fn add(self, o: Field) -> Field {
        Field(self.0.iter().zip(&o.0).map(|(a, b)| a + b).collect())
    }
}

impl Mul<FloatType> for Field {
    type Output = Field;
    fn mul(self, s: FloatType) -> Field {
        Field(self.0.iter().map(|a| a * s).collect())
    }
}
