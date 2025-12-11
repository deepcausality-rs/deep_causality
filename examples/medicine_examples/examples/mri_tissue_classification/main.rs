/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # MRI Tissue Classification via Topological Data Analysis
//!
//! Demonstrates using TDA (Topological Data Analysis) to classify tissue samples
//! as healthy or pathological based on topological features (holes/voids).
//!
//! ## Key Concepts
//! - **Euler Characteristic (χ)**: Topological invariant = V - E + F
//! - **Vietoris-Rips Complex**: Triangulation from point cloud
//! - **Necrotic Core Detection**: Holes in tissue indicate pathology
//!
//! ## APIs Demonstrated
//! - `PropagatingEffect` - Monadic effect pipeline
//! - `PointCloud::triangulate()` - Build simplicial complex
//! - `BaseTopology` trait - Compute topological invariants

use deep_causality_core::PropagatingEffect;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, PointCloud};
use std::f64::consts::PI;

/// Analysis result from the TDA pipeline
#[derive(Debug, Clone, Default)]
struct TissueAnalysis {
    label: String,
    num_vertices: usize,
    num_edges: usize,
    num_faces: usize,
    euler_characteristic: isize,
    diagnosis: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MRI Topological Analysis: Causal Pipeline ===\n");

    // Generate test samples
    let healthy_points = generate_dense_cluster(20, 0.5);
    let tumor_points = generate_necrotic_ring(20, 1.0);

    println!("Generated 2 tissue samples for analysis.\n");

    // Analyze both samples using the causal monad pipeline
    analyze_with_monad("Sample A (Healthy)", healthy_points)?;
    println!();
    analyze_with_monad("Sample B (Pathological)", tumor_points)?;

    Ok(())
}

/// Runs the full TDA pipeline as a monadic chain
fn analyze_with_monad(label: &str, points: Vec<f64>) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Analyzing {} ---", label);

    let label_owned = label.to_string();

    // Step 1: Ingest data into PointCloud
    let step1: PropagatingEffect<(String, Vec<f64>)> =
        PropagatingEffect::pure((label_owned, points));

    // Step 2: Create PointCloud and triangulate
    let step2 = step1.bind(|effect_value, _, _| {
        let (label, points) = effect_value.into_value().unwrap_or_default();

        println!("[Step 1] Ingesting {} points...", points.len() / 3);

        let num_points = points.len() / 3;
        let points_tensor = CausalTensor::new(points, vec![num_points, 3]).expect("Tensor failed");
        let metadata = CausalTensor::new(vec![1.0; num_points], vec![num_points]).unwrap();
        let pc = PointCloud::new(points_tensor, metadata, 0).expect("PointCloud failed");

        println!("[Step 2] Triangulating with radius=0.6...");
        let complex = pc.triangulate(0.6).expect("Triangulation failed");

        let v = complex.num_elements_at_grade(0).unwrap_or(0);
        let e = complex.num_elements_at_grade(1).unwrap_or(0);
        let f = complex.num_elements_at_grade(2).unwrap_or(0);

        println!("         Vertices: {}, Edges: {}, Faces: {}", v, e, f);

        // Compute Euler Characteristic: χ = V - E + F
        println!("[Step 3] Computing Euler Characteristic...");

        let mut chi: isize = 0;
        let dim = complex.dimension();
        for d in 0..=dim {
            if let Some(count) = complex.num_elements_at_grade(d) {
                chi += if d % 2 == 0 {
                    count as isize
                } else {
                    -(count as isize)
                };
            }
        }
        println!("         χ = {}", chi);

        PropagatingEffect::pure((label, v, e, f, chi))
    });

    // Step 3: Diagnose based on topology
    let result = step2.bind(|effect_value, _, _| {
        let (label, v, e, f, chi): (String, usize, usize, usize, isize) =
            effect_value.into_value().unwrap_or_default();

        println!("[Step 4] Generating diagnosis...");

        let diagnosis = if chi <= 0 {
            "PATHOLOGICAL - Detected topological hole/void (Necrotic Core)".to_string()
        } else if chi == 1 {
            "HEALTHY - Tissue is topologically contractible (Dense Mass)".to_string()
        } else {
            "INCONCLUSIVE - Tissue is disconnected/sparse".to_string()
        };

        let analysis = TissueAnalysis {
            label,
            num_vertices: v,
            num_edges: e,
            num_faces: f,
            euler_characteristic: chi,
            diagnosis,
        };

        PropagatingEffect::pure(analysis)
    });

    // Extract and display the final result
    if let Some(analysis) = result.value.into_value() {
        println!("\n=== {} Results ===", analysis.label);
        println!(
            "  Simplices: {} vertices, {} edges, {} faces",
            analysis.num_vertices, analysis.num_edges, analysis.num_faces
        );
        println!("  Euler Characteristic: {}", analysis.euler_characteristic);
        println!("  DIAGNOSIS: {}", analysis.diagnosis);
    } else {
        eprintln!("Analysis failed");
    }

    Ok(())
}

// --- Helper Functions for Data Simulation ---

/// Generates a dense cluster of points (healthy tissue)
fn generate_dense_cluster(n: usize, scale: f64) -> Vec<f64> {
    let mut points = Vec::with_capacity(n * 3);
    for i in 0..n {
        let u = (i as f64) * 0.1;
        let v = (i as f64) * 0.3;
        let r = scale * (u.sin().abs());
        let theta = v * 2.0 * PI;
        let phi = u * PI;

        points.push(r * theta.cos() * phi.sin());
        points.push(r * theta.sin() * phi.sin());
        points.push(r * phi.cos());
    }
    points
}

/// Generates a ring of points with a hole (pathological tissue)
fn generate_necrotic_ring(n: usize, radius: f64) -> Vec<f64> {
    let mut points = Vec::with_capacity(n * 3);
    for i in 0..n {
        let theta = (i as f64 / n as f64) * 2.0 * PI;
        let noise = (i % 5) as f64 * 0.05;
        let r = radius + noise;

        points.push(r * theta.cos());
        points.push(r * theta.sin());
        points.push((i % 3) as f64 * 0.1);
    }
    points
}
