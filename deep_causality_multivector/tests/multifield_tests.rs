/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector};
use deep_causality_tensor::CpuBackend;

#[test]
fn test_roundtrip_conversion_cpu() {
    // 1. Create random coefficients
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [2, 2, 2];
    let num_cells = 8;
    let num_blades = 1 << 3;

    let mut mvs = Vec::with_capacity(num_cells);
    for i in 0..num_cells {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32; // Scalar
        data[1] = 1.0; // Vector x
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let dx = [1.0, 1.0, 1.0];

    // 2. Convert to Field
    let field = CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, shape, dx);

    // 3. Convert back
    let mvs_back = field.to_coefficients();

    // 4. Verify
    assert_eq!(mvs.len(), mvs_back.len());
    for (orig, back) in mvs.iter().zip(mvs_back.iter()) {
        for (a, b) in orig.data().iter().zip(back.data().iter()) {
            assert!((*a - *b).abs() < 1e-5_f32);
        }
    }
}

#[test]
fn test_grade_projection_cpu() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [1, 1, 1];
    let dx = [1.0, 1.0, 1.0];

    // Create pure grade 1 field
    let mut data = vec![0.0f32; 8];
    data[1] = 5.0; // e_1 (bit 0)
    data[2] = 3.0; // e_2 (bit 1)
    let mv = CausalMultiVector::unchecked(data, metric);

    let field = CausalMultiField::<CpuBackend, f32>::from_coefficients(&[mv], shape, dx);

    // Project to grade 1
    let vec_part = field.vector_part();
    let vec_coeffs = vec_part.to_coefficients();
    assert!((vec_coeffs[0].data()[1] - 5.0).abs() < 1e-5);

    // Project to grade 0 (should be zero)
    let scalar_part = field.scalar_part();
    let scalar_coeffs = scalar_part.to_coefficients();
    assert!(scalar_coeffs[0].data()[0].abs() < 1e-5);
}

#[test]
fn test_gradient_identity_cpu() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [4, 4, 4]; // Minimum 3 for central difference
    let dx = [1.0, 1.0, 1.0];

    // Create scalar field F(x,y,z) = x
    // ∇F = e_1

    let mut mvs = Vec::with_capacity(64);
    for _z in 0..4 {
        for _y in 0..4 {
            for x in 0..4 {
                let mut data = vec![0.0f32; 8];
                data[0] = x as f32; // Scalar part
                mvs.push(CausalMultiVector::unchecked(data, metric));
            }
        }
    }

    let field = CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, shape, dx);
    let grad = field.gradient(); // Returns vector field

    // Check center point (x=1, y=1, z=1)
    // Index mapping (z,y,x): z*16 + y*4 + x
    // 1*16 + 1*4 + 1 = 21
    let grad_coeffs = grad.to_coefficients();
    let center_mv = &grad_coeffs[21];

    // Central diff: (2 - 0) / 2 = 1.0
    // Expected: 1.0 * e_1
    // e_1 corresponds to bit 0 -> Index 1

    assert!(
        (center_mv.data()[1] - 1.0).abs() < 1e-4,
        "Expected e1=1.0 at center, got {:?}",
        center_mv.data()
    );
    assert!(center_mv.data()[2].abs() < 1e-4, "Expected e2=0");
    assert!(center_mv.data()[4].abs() < 1e-4, "Expected e3=0"); // e3 is bit 2 -> index 4
}

#[cfg(feature = "mlx")]
#[test]
fn test_roundtrip_conversion_mlx() {
    use deep_causality_tensor::MlxBackend;

    let metric = Metric::from_signature(3, 0, 0);
    let shape = [2, 2, 2];
    let num_cells = 8;
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(num_cells);
    for i in 0..num_cells {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let dx = [1.0, 1.0, 1.0];
    let field = CausalMultiField::<MlxBackend, f32>::from_coefficients(&mvs, shape, dx);
    let mvs_back = field.to_coefficients();

    for (orig, back) in mvs.iter().zip(mvs_back.iter()) {
        for (a, b) in orig.data().iter().zip(back.data().iter()) {
            assert!((*a - *b).abs() < 1e-5_f32);
        }
    }
}
