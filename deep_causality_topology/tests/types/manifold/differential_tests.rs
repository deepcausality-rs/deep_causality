/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

// Setup function to create a manifold from a point cloud
fn setup_triangle_manifold() -> Manifold<f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();

    let complex = point_cloud.triangulate(1.2).unwrap();
    // Complex has 3 vertices, 3 edges, 1 face. Total 7 simplices.
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0, 1.0, 2.0, 3.0, 100.0], vec![7]).unwrap();

    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn test_exterior_derivative_d0() {
    let manifold = setup_triangle_manifold();
    // data on vertices: [10.0, 20.0, 30.0]
    let d0_form = manifold.exterior_derivative(0);
    assert_eq!(d0_form.shape(), &[3]); // 3 edges
    // d(f) on edge (v0,v1) is f(v1)-f(v0)
    // Edges are (0,1), (0,2), (1,2)
    // d(f)(e01) = f(v1) - f(v0) = 20-10=10
    // d(f)(e02) = f(v2) - f(v0) = 30-10=20
    // d(f)(e12) = f(v2) - f(v1) = 30-20=10
    let expected = vec![10.0, 10.0, 20.0]; // Order depends on complex construction
    let mut actual = d0_form.as_slice().to_vec();
    actual.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(actual, expected);
}

#[test]
fn test_exterior_derivative_d1() {
    let manifold = setup_triangle_manifold();
    let d1_form = manifold.exterior_derivative(1);
    assert_eq!(d1_form.shape(), &[1]); // 1 face
    assert_eq!(d1_form.len(), 1);
}

#[test]
fn test_exterior_derivative_nilpotency() {
    let manifold = setup_triangle_manifold();
    let d0_form = manifold.exterior_derivative(0);
    // Now, apply derivative again. We need to put d0_form back into a manifold
    let mut new_data = vec![0.0; 7];
    new_data[3..6].copy_from_slice(d0_form.as_slice());
    let complex = manifold.complex().clone();
    let manifold2 =
        Manifold::new(complex, CausalTensor::new(new_data, vec![7]).unwrap(), 0).unwrap();
    let d1_of_d0 = manifold2.exterior_derivative(1);
    // d(d(f)) should be zero
    assert_eq!(d1_of_d0.len(), 1);
    assert!((d1_of_d0.as_slice()[0]).abs() < 1e-9);
}

#[test]
fn test_exterior_derivative_out_of_bounds() {
    let manifold = setup_triangle_manifold();
    let d3_form = manifold.exterior_derivative(3);
    assert_eq!(d3_form.len(), 0);
}

#[test]
fn test_hodge_star_k0() {
    let manifold = setup_triangle_manifold();
    let star0 = manifold.hodge_star(0); // 0-form -> 2-form
    assert_eq!(star0.shape(), &[3]); // 1 face
    assert!(star0.as_slice()[0] != 0.0);
}

#[test]
fn test_hodge_star_k1() {
    let manifold = setup_triangle_manifold();
    let star1 = manifold.hodge_star(1); // 1-form -> 1-form
    assert_eq!(star1.shape(), &[3]); // 1-form
}

#[test]
fn test_hodge_star_k2() {
    let manifold = setup_triangle_manifold();
    let star2 = manifold.hodge_star(2); // 2-form -> 0-form
    assert_eq!(star2.shape(), &[1]);
}

#[test]
fn test_hodge_star_out_of_bounds() {
    let manifold = setup_triangle_manifold();
    let star3 = manifold.hodge_star(3);
    assert_eq!(star3.len(), 0);
}

#[test]
fn test_laplacian_scalar_field_geometric() {
    let manifold = setup_triangle_manifold();
    let laplacian = manifold.laplacian(0);

    assert_eq!(laplacian.shape(), &[3]);

    // 1.  **Geometry:**
    //     *   Triangle Base: 1.0, Height: 1.0. **Area = 0.5**.
    //     *   Edge $v_0 \to v_1$: Length $1.0$.
    //     *   Edge $v_0 \to v_2$: Length $\sqrt{0.5^2 + 1^2} \approx 1.118$.
    // 2.  **Mass Matrices (Weights):**
    //     *   **Vertex Mass ($M_0$):** Lumped area. $0.5 / 3 \approx 0.1666$.
    //     *   **Edge Mass ($M_1$):** Edge Lengths. $1.0$ and $1.118$.
    // 3.  **Flux at $v_0$:**
    //     *   Flow from $v_1$: $(10 - 20) \times 1.0 = -10$.
    //     *   Flow from $v_2$: $(10 - 30) \times 1.118 = -22.36$.
    //     *   Total Flux: $-32.36$.
    // 4.  **Laplacian at $v_0$ (Flux density):**
    //     *   $\Delta = \frac{\text{Flux}}{\text{Mass}} = \frac{-32.36}{0.1666} \approx \mathbf{-194.16}$.
    //
    let mut result = laplacian.as_slice().to_vec();
    result.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Geometric values calculated via DEC (Mass Lumping):
    // v0: -194.16 (Source)
    // v1: -7.08   (Sink/Source)
    // v2: 201.24  (Sink)
    let expected = [-194.16407865, -7.08203932, 201.24611797];

    for (a, b) in result.iter().zip(expected.iter()) {
        assert!((a - b).abs() < 1e-4, "Mismatch: Got {}, Expected {}", a, b);
    }
}
