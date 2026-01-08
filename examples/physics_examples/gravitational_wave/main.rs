/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Gravitational Wave Simulation (Regge Calculus)
//!
//! Simulates gravitational wave propagation on a discrete spacetime mesh.
//!
//! ## Key Concepts
//! - **Regge Calculus**: Discrete approach to General Relativity using simplicial meshes
//! - **Deficit Angle**: Curvature concentrated at bones (n-2 simplices)
//! - **Wave Propagation**: Curvature ↔ edge length feedback loop
//!
//! ## APIs Demonstrated
//! - `SimplicialComplexBuilder` - Construct discrete spacetime
//! - `ReggeGeometry::calculate_ricci_curvature()` - Compute deficit angles
//! - `BaseTopology` trait - Mesh element access

use std::f64::consts::PI;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, ReggeGeometry, Simplex, SimplicialComplexBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Gravitational Wave: Regge Calculus ===\n");

    // Build a 2D triangulated mesh (hexagonal lattice around a center)
    // This represents a discrete slice of spacetime
    let mut builder = SimplicialComplexBuilder::new(2);

    // Create a hexagonal pattern: center vertex 0, surrounded by 6 triangles
    // This gives us an internal vertex at 0 with 6 incident triangles
    let perimeter_vertices = [1, 2, 3, 4, 5, 6];
    for i in 0..6 {
        let v1 = perimeter_vertices[i];
        let v2 = perimeter_vertices[(i + 1) % 6];
        builder.add_simplex(Simplex::new(vec![0, v1, v2]))?;
    }
    let complex = builder.build()?;

    println!("Created 2D Simplicial Complex:");
    println!(
        "  Vertices: {}",
        complex.num_elements_at_grade(0).unwrap_or(0)
    );
    println!("  Edges: {}", complex.num_elements_at_grade(1).unwrap_or(0));
    println!(
        "  Triangles: {}",
        complex.num_elements_at_grade(2).unwrap_or(0)
    );

    // Initialize edge lengths (flat spacetime = all edges equal)
    let num_edges = complex.num_elements_at_grade(1).expect("No edges found");
    let mut edge_lengths: Vec<f64> = vec![1.0; num_edges];

    // Introduce a "gravitational wave" perturbation
    // We'll oscillate some edge lengths to simulate a passing wave
    println!("\n[INITIAL] Flat spacetime (all edges = 1.0)");

    // Simulation parameters
    let time_steps = 8;
    let wave_amplitude = 0.1;
    let damping = 0.05;

    for t in 0..time_steps {
        // Create geometry with current edge lengths
        let tensor = CausalTensor::new(edge_lengths.clone(), vec![num_edges])?;
        let geometry = ReggeGeometry::new(tensor);

        // Calculate curvature (deficit angles)
        let curvature = geometry.calculate_ricci_curvature(&complex)?;

        // Find the center vertex (0) curvature
        let center_idx = complex.skeletons()[0]
            .get_index(&Simplex::new(vec![0]))
            .unwrap_or(0);
        let center_curvature = curvature.data()[center_idx];

        // Calculate total curvature (Gaussian curvature integral)
        let total_curvature: f64 = curvature.data().iter().sum();

        println!(
            "[t={}] Center Curvature: {:>+7.4}, Total: {:>+7.4}",
            t, center_curvature, total_curvature
        );

        // Gravitational wave update rule:
        // - Curvature drives edge length changes (Einstein's equation analog)
        // - Wave oscillation modulated by time
        let wave_phase = (t as f64) * PI / 3.0; // 60 degree phase per step
        let wave_factor = wave_amplitude * wave_phase.sin();

        // Update edge lengths based on curvature feedback
        for (i, edge_len) in edge_lengths.iter_mut().enumerate() {
            // Edges connected to center experience wave perturbation
            // Apply damping to stabilize
            let curvature_correction = if i < 6 {
                // Inner edges (connected to center)
                -center_curvature * damping + wave_factor
            } else {
                // Outer edges
                wave_factor * 0.5
            };

            *edge_len = (*edge_len + curvature_correction).clamp(0.5, 1.5);
        }
    }

    println!("\n--- Final Edge Lengths ---");
    for (i, &len) in edge_lengths.iter().enumerate().take(12) {
        println!("  Edge {:>2}: {:.4}", i, len);
    }

    println!("\n[COMPLETE] Gravitational Wave Simulation Finished.");
    println!("Interpretation: Curvature oscillations represent spacetime ripples.");

    Ok(())
}
