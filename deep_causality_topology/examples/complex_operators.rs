/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::PointCloud;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create simple 4-vertex grid
    let points = CausalTensor::new(
        vec![
            0.0, 1.0, // v0
            1.0, 1.0, // v1
            0.0, 0.0, // v2
            1.0, 0.0, // v3
        ],
        vec![4, 2],
    )?;
    let point_cloud = PointCloud::new(points, CausalTensor::new(vec![0.0; 4], vec![4])?, 0)?;

    let complex = point_cloud.triangulate(1.5)?;

    println!("Simplicial complex:");
    for (i, skel) in complex.skeletons().iter().enumerate() {
        println!("  Skeleton {}: {} simplices", i, skel.simplices().len());
        for (j, simplex) in skel.simplices().iter().take(5).enumerate() {
            println!("    Simplex {}: {:?}", j, simplex.vertices());
        }
    }

    println!("\nBoundary operators:");
    for (i, op) in complex.boundary_operators().iter().enumerate() {
        println!(
            "  ∂_{}: {} x {}, nnz={}",
            i,
            op.shape().0,
            op.shape().1,
            op.values().len()
        );
        if op.values().len() <= 20 {
            println!("    values: {:?}", op.values());
        }
    }

    println!("\nCoboundary operators:");
    for (i, op) in complex.coboundary_operators().iter().enumerate() {
        println!(
            "  ∂*_{}: {} x {}, nnz={}",
            i,
            op.shape().0,
            op.shape().1,
            op.values().len()
        );
        if op.values().len() <= 20 {
            println!("    values: {:?}", op.values());
        }
    }

    Ok(())
}
