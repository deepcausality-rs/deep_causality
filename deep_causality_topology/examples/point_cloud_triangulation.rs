/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, PointCloud};
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MRI Topological Analysis: Tissue Segmentation ===\n");

    // ------------------------------------------------------------------------
    // ENGINEERING VALUE:
    // In Medical Imaging (MRI/CT), distinguishing between healthy tissue and
    // pathological structures (tumors, necrotic cores, vascular anomalies) is critical.
    //
    // Traditional methods rely on pixel intensity (density). However, some tumors
    // have similar density to healthy tissue but distinct *structures* (e.g.,
    // a necrotic core forms a "void" or "hole" inside the tissue).
    //
    // Topological Data Analysis (TDA) offers a robust way to detect these features.
    // By computing the Euler Characteristic ($\chi$) of the triangulated tissue,
    // we can mathematically count the number of connected components and holes.
    //
    // - Healthy Tissue (Dense Cluster): $\chi \approx 1$ (Contractible, no holes)
    // - Pathological Tissue (Necrotic Ring): $\chi \approx 0$ (1 connected component - 1 hole)
    // ------------------------------------------------------------------------

    // 1. Simulate MRI Data
    println!("--- 1. Simulating MRI Point Cloud Data ---");

    // Case A: Healthy Tissue
    // Simulated as a dense, solid cluster of points (Gaussian-like blob).
    // Topologically equivalent to a point (contractible).
    // Reduced N to 20 to avoid combinatorial explosion in Rips complex with large radius.
    let healthy_tissue = generate_dense_cluster(20, 0.5);
    println!("Generated 20 points for Healthy Tissue sample.");

    // Case B: Pathological Tissue (Suspected Tumor)
    // Simulated as a ring of points with a hollow center (necrotic core).
    // Topologically equivalent to a circle ($S^1$).
    let tumor_tissue = generate_necrotic_ring(20, 1.0);
    println!("Generated 20 points for Pathological Tissue sample.");

    // 2. Topological Analysis Pipeline
    println!("\n--- 2. Running Topological Analysis Pipeline ---");

    analyze_tissue("Sample A (Healthy)", &healthy_tissue)?;
    analyze_tissue("Sample B (Pathological)", &tumor_tissue)?;

    Ok(())
}

/// Analyzes a tissue sample by triangulating it and computing topological invariants.
fn analyze_tissue(label: &str, points_flat: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nAnalyzing {}:", label);

    // A. Ingest Data into PointCloud
    // Points are 3D (x, y, z), flattened.
    let num_points = points_flat.len() / 3;
    let points_tensor = CausalTensor::new(points_flat.to_vec(), vec![num_points, 3])?;
    // Dummy metadata (e.g., signal intensity)
    let metadata = CausalTensor::new(vec![1.0; num_points], vec![num_points])?;

    let pc = PointCloud::new(points_tensor, metadata, 0)?;

    // B. Triangulate (Vietoris-Rips)
    // Radius is critical: too small = discrete points, too large = giant blob.
    // Adjusted to 0.6 to ensure the ring connects into a single component.
    let radius = 0.6;
    let complex = pc.triangulate(radius)?;

    println!("  Triangulation complete (Radius={}).", radius);
    println!(
        "  - 0-Simplices (Vertices): {}",
        complex.num_elements_at_grade(0).unwrap_or(0)
    );
    println!(
        "  - 1-Simplices (Edges):    {}",
        complex.num_elements_at_grade(1).unwrap_or(0)
    );
    println!(
        "  - 2-Simplices (Faces):    {}",
        complex.num_elements_at_grade(2).unwrap_or(0)
    );

    // C. Compute Euler Characteristic ($\chi$) directly on SimplicialComplex
    // We bypass the `Manifold` wrapper because raw Rips complexes from point clouds
    // often have singularities or non-manifold junctions that fail strict validation,
    // but we can still compute topological invariants for classification.

    // $\chi = \sum_{i=0}^{d} (-1)^i \times N_i$
    // N_i = number of i-simplices
    let mut chi = 0;
    let dim = complex.dimension();

    for d in 0..=dim {
        if let Some(count) = complex.num_elements_at_grade(d) {
            let term = if d % 2 == 0 {
                count as isize
            } else {
                -(count as isize)
            };
            chi += term;
        }
    }

    println!(r"  > Euler Characteristic ($\chi$): {}", chi);

    // D. Diagnosis Logic
    // Healthy (Contractible): Chi ~ 1
    // Necrotic Ring (Hole): Chi ~ 0 (or <= 0 for multiple holes)
    // Disconnected: Chi > 1
    if chi <= 0 {
        println!("  > DIAGNOSIS: [PATHOLOGICAL] - Detected topological hole/void (Necrotic Core).");
    } else if chi == 1 {
        println!("  > DIAGNOSIS: [HEALTHY] - Tissue is topologically contractible (Dense Mass).");
    } else {
        println!("  > DIAGNOSIS: [INCONCLUSIVE] - Tissue is disconnected/sparse (Chi > 1).");
    }

    Ok(())
}

// --- Helper Functions for Data Simulation ---

/// Generates a dense cluster of points around (0,0,0).
fn generate_dense_cluster(n: usize, scale: f64) -> Vec<f64> {
    let mut points = Vec::with_capacity(n * 3);
    // Simple deterministic pseudo-random generation for reproducibility
    for i in 0..n {
        let u = (i as f64) * 0.1;
        let v = (i as f64) * 0.3;
        // Map to sphere-like volume but filled
        let r = scale * (u.sin().abs());
        let theta = v * 2.0 * PI;
        let phi = u * PI;

        let x = r * theta.cos() * phi.sin();
        let y = r * theta.sin() * phi.sin();
        let z = r * phi.cos();

        points.push(x);
        points.push(y);
        points.push(z);
    }
    points
}

/// Generates a ring of points (torus/annulus like) with a hole in the middle.
fn generate_necrotic_ring(n: usize, radius: f64) -> Vec<f64> {
    let mut points = Vec::with_capacity(n * 3);
    for i in 0..n {
        let theta = (i as f64 / n as f64) * 2.0 * PI;
        // Add some noise/thickness to the ring
        let noise = (i % 5) as f64 * 0.05;
        let r = radius + noise;

        let x = r * theta.cos();
        let y = r * theta.sin();
        let z = (i % 3) as f64 * 0.1; // Slight z-variation (flat-ish ring)

        points.push(x);
        points.push(y);
        points.push(z);
    }
    points
}
