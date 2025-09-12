/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{CausalTensor, CausalTensorError};

#[test]
fn test_add_tensors_same_shape() {
    let a = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let b = CausalTensor::new(vec![4, 5, 6], vec![3]).unwrap();
    let result = (&a + &b).unwrap();
    assert_eq!(result.as_slice(), &[5, 7, 9]);
    assert_eq!(result.shape(), &[3]);
}

#[test]
fn test_add_tensors_broadcast_row() {
    // Broadcast row vector
    let a = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let b = CausalTensor::new(vec![10, 20, 30], vec![1, 3]).unwrap();
    let result = (&a + &b).unwrap();
    assert_eq!(result.as_slice(), &[11, 22, 33, 14, 25, 36]);
    assert_eq!(result.shape(), &[2, 3]);
}

#[test]
fn test_add_tensors_broadcast_col() {
    // Broadcast column vector
    let a = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let b = CausalTensor::new(vec![10, 20], vec![2, 1]).unwrap();
    let result = (&a + &b).unwrap();
    assert_eq!(result.as_slice(), &[11, 12, 13, 24, 25, 26]);
    assert_eq!(result.shape(), &[2, 3]);
}

#[test]
fn test_add_tensors_broadcast_scalar() {
    // Broadcast scalar tensor
    let a = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let b = CausalTensor::new(vec![10], vec![]).unwrap();
    let result = (&a + &b).unwrap();
    assert_eq!(result.as_slice(), &[11, 12, 13]);
}

#[test]
fn test_add_tensors_shape_mismatch() {
    let a = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result = &a + &b;
    assert_eq!(result, Err(CausalTensorError::ShapeMismatch));
}

#[test]
fn test_sub_tensors() {
    let a = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let b = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result = (&a - &b).unwrap();
    assert_eq!(result.as_slice(), &[9, 18, 27]);
}

#[test]
fn test_mul_tensors() {
    let a = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let b = CausalTensor::new(vec![4, 5, 6], vec![3]).unwrap();
    let result = (&a * &b).unwrap();
    assert_eq!(result.as_slice(), &[4, 10, 18]);
}

#[test]
fn test_div_tensors() {
    let a = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let b = CausalTensor::new(vec![2, 5, 10], vec![3]).unwrap();
    let result = (&a / &b).unwrap();
    assert_eq!(result.as_slice(), &[5, 4, 3]);
}

#[test]
fn test_owned_and_borrowed_variants_add() {
    let a = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3, 4], vec![2]).unwrap();

    // &a + &b
    let res1 = (&a + &b).unwrap();
    assert_eq!(res1.as_slice(), &[4, 6]);

    // a + &b
    let res2 = (a.clone() + &b).unwrap();
    assert_eq!(res2.as_slice(), &[4, 6]);

    // &a + b
    let res3 = (&a + b.clone()).unwrap();
    assert_eq!(res3.as_slice(), &[4, 6]);

    // a + b
    let res4 = (a + b).unwrap();
    assert_eq!(res4.as_slice(), &[4, 6]);
}

#[test]
fn test_owned_and_borrowed_variants_sub() {
    let a = CausalTensor::new(vec![10, 20], vec![2]).unwrap();
    let b = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let expected = vec![9, 18];

    // &a - &b
    let res1 = (&a - &b).unwrap();
    assert_eq!(res1.as_slice(), expected.as_slice());

    // a - &b
    let res2 = (a.clone() - &b).unwrap();
    assert_eq!(res2.as_slice(), expected.as_slice());

    // &a - b
    let res3 = (&a - b.clone()).unwrap();
    assert_eq!(res3.as_slice(), expected.as_slice());

    // a - b
    let res4 = (a - b).unwrap();
    assert_eq!(res4.as_slice(), expected.as_slice());
}

#[test]
fn test_owned_and_borrowed_variants_mul() {
    let a = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3, 4], vec![2]).unwrap();
    let expected = vec![3, 8];

    // &a * &b
    let res1 = (&a * &b).unwrap();
    assert_eq!(res1.as_slice(), expected.as_slice());

    // a * &b
    let res2 = (a.clone() * &b).unwrap();
    assert_eq!(res2.as_slice(), expected.as_slice());

    // &a * b
    let res3 = (&a * b.clone()).unwrap();
    assert_eq!(res3.as_slice(), expected.as_slice());

    // a * b
    let res4 = (a * b).unwrap();
    assert_eq!(res4.as_slice(), expected.as_slice());
}

#[test]
fn test_owned_and_borrowed_variants_div() {
    let a = CausalTensor::new(vec![10, 20], vec![2]).unwrap();
    let b = CausalTensor::new(vec![2, 5], vec![2]).unwrap();
    let expected = vec![5, 4];

    // &a / &b
    let res1 = (&a / &b).unwrap();
    assert_eq!(res1.as_slice(), expected.as_slice());

    // a / &b
    let res2 = (a.clone() / &b).unwrap();
    assert_eq!(res2.as_slice(), expected.as_slice());

    // &a / b
    let res3 = (&a / b.clone()).unwrap();
    assert_eq!(res3.as_slice(), expected.as_slice());

    // a / b
    let res4 = (a / b).unwrap();
    assert_eq!(res4.as_slice(), expected.as_slice());
}

#[test]
fn test_binary_op_same_shape() {
    let t1 = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let t2 = CausalTensor::new(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();
    let expected_data = vec![6.0, 8.0, 10.0, 12.0];
    let expected_shape = vec![2, 2];

    let result = (&t1 + &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_scalar_tensors() {
    let t1 = CausalTensor::new(vec![10.0], vec![]).unwrap();
    let t2 = CausalTensor::new(vec![2.0], vec![]).unwrap();
    let expected_data = vec![20.0];
    let expected_shape = vec![];

    let result = (&t1 * &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_scalar_with_matrix() {
    let t1 = CausalTensor::new(vec![2.0], vec![]).unwrap(); // Scalar
    let t2 = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap(); // Matrix
    let expected_data = vec![2.0, 4.0, 6.0, 8.0];
    let expected_shape = vec![2, 2];

    let result = (&t1 * &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());

    let result_rev = (&t2 * &t1).unwrap();
    assert_eq!(result_rev.as_slice(), expected_data.as_slice());
    assert_eq!(result_rev.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_vector_with_matrix_right_aligned() {
    let t1 = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap(); // Matrix 2x3
    let t2 = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap(); // Vector 3
    let expected_data = vec![11.0, 22.0, 33.0, 14.0, 25.0, 36.0];
    let expected_shape = vec![2, 3];

    let result = (&t1 + &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_dimensions_with_size_one() {
    let t1 = CausalTensor::new(vec![1.0, 2.0], vec![2, 1]).unwrap(); // Shape 2x1
    let t2 = CausalTensor::new(vec![10.0, 20.0], vec![1, 2]).unwrap(); // Shape 1x2
    let expected_data = vec![11.0, 21.0, 12.0, 22.0];
    let expected_shape = vec![2, 2];

    let result = (&t1 + &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_shape_mismatch() {
    let t1 = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let t2 = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();

    let result = &t1 + &t2;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CausalTensorError::ShapeMismatch);
}

#[test]
fn test_binary_op_division_by_zero() {
    let t1 = CausalTensor::new(vec![1.0], vec![]).unwrap();
    let t2 = CausalTensor::new(vec![0.0], vec![]).unwrap();

    let result = &t1 / &t2;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CausalTensorError::InvalidOperation);
}

#[test]
fn test_binary_op_complex_broadcasting() {
    // Example: (2, 1, 3) + (1, 5, 3)
    let t1 = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 1, 3]).unwrap();
    let t2 = CausalTensor::new(
        vec![
            10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0, 130.0,
            140.0, 150.0,
        ],
        vec![1, 5, 3],
    )
    .unwrap();

    // Expected shape: (2, 5, 3)
    // t1: [[ [1,2,3] ], [ [4,5,6] ]]
    // t2: [ [10,20,30], [40,50,60], [70,80,90], [100,110,120], [130,140,150] ]
    // Expected data (t1 + t2, broadcast t1's middle dim, t2's first dim)
    let expected_data = vec![
        11.0, 22.0, 33.0, // 1+10, 2+20, 3+30
        41.0, 52.0, 63.0, // 1+40, 2+50, 3+60
        71.0, 82.0, 93.0, // 1+70, 2+80, 3+90
        101.0, 112.0, 123.0, // 1+100, 2+110, 3+120
        131.0, 142.0, 153.0, // 1+130, 2+140, 3+150
        14.0, 25.0, 36.0, // 4+10, 5+20, 6+30
        44.0, 55.0, 66.0, // 4+40, 5+50, 6+60
        74.0, 85.0, 96.0, // 4+70, 5+80, 6+90
        104.0, 115.0, 126.0, // 4+100, 5+110, 6+120
        134.0, 145.0, 156.0, // 4+130, 5+140, 6+150
    ];
    let expected_shape = vec![2, 5, 3];

    let result = (&t1 + &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_division_with_broadcasting() {
    // p_as: p(T, S1) shape [2,2]
    let p_as = CausalTensor::new(vec![0.1, 0.2, 0.0, 0.2], vec![2, 2]).unwrap();
    // p_s: p(T) shape [2]
    let p_s = CausalTensor::new(vec![0.3, 0.7], vec![2]).unwrap();

    // Expected result shape: [2,2]
    // Expected data:
    // data[0, 0] = 0.1 / 0.3 = 0.333...
    // data[0, 1] = 0.2 / 0.7 = 0.285...
    // data[1, 0] = 0.0 / 0.3 = 0.0
    // data[1, 1] = 0.2 / 0.7 = 0.285...
    let expected_data = vec![0.1 / 0.3, 0.2 / 0.7, 0.0 / 0.3, 0.2 / 0.7];
    let expected_shape = vec![2, 2];

    let result = (&p_as / &p_s).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_scalar_division_by_vector() {
    let t1 = CausalTensor::new(vec![1.0], vec![]).unwrap(); // Scalar
    let t2 = CausalTensor::new(vec![2.0, 4.0], vec![2]).unwrap(); // Vector
    let expected_data = vec![1.0 / 2.0, 1.0 / 4.0];
    let expected_shape = vec![2];

    let result = (&t1 / &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_vector_division_by_scalar() {
    let t1 = CausalTensor::new(vec![2.0, 4.0], vec![2]).unwrap(); // Vector
    let t2 = CausalTensor::new(vec![2.0], vec![]).unwrap(); // Scalar
    let expected_data = vec![2.0 / 2.0, 4.0 / 2.0];
    let expected_shape = vec![2];

    let result = (&t1 / &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_different_dims_no_broadcasting() {
    // This should result in a ShapeMismatch error
    let t1 = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let t2 = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();

    let result = &t1 + &t2;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CausalTensorError::ShapeMismatch);
}

#[test]
fn test_binary_op_empty_tensors() {
    let t1: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let t2: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();

    let result = (&t1 + &t2).unwrap();
    assert!(result.is_empty());
    assert_eq!(result.shape(), &[0]); // Expect scalar empty tensor
}

#[test]
fn test_binary_op_one_empty_tensor() {
    let t1 = CausalTensor::new(vec![1.0], vec![]).unwrap();
    let t2 = CausalTensor::new(vec![], vec![0]).unwrap();

    let result = &t1 + &t2;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CausalTensorError::ShapeMismatch);
}

#[test]
fn test_binary_op_large_tensors_with_broadcasting() {
    // Test with larger tensors to ensure performance and correctness
    let t1 = CausalTensor::new((0..1000).map(|i| i as f64).collect(), vec![10, 10, 10]).unwrap();
    let t2 = CausalTensor::new((0..10).map(|i| i as f64).collect(), vec![10]).unwrap(); // Vector to broadcast

    let result = (&t1 + &t2).unwrap();
    assert_eq!(result.shape(), &[10, 10, 10]);
    // Spot check a few values
    assert_eq!(result.get(&[0, 0, 0]), Some(&0.0)); // 0 + 0
    assert_eq!(result.get(&[0, 0, 9]), Some(&18.0)); // 9 + 9
    assert_eq!(result.get(&[1, 0, 0]), Some(&100.0)); //  100 + 0
    assert_eq!(result.get(&[9, 9, 9]), Some(&1008.0)); // 999 + 9
}

#[test]
fn test_binary_op_subtraction() {
    let t1 = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let t2 = CausalTensor::new(vec![5.0], vec![]).unwrap();
    let expected_data = vec![5.0, 15.0];
    let expected_shape = vec![2];

    let result = (&t1 - &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_multiplication() {
    let t1 = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let t2 = CausalTensor::new(vec![2.0], vec![]).unwrap();
    let expected_data = vec![2.0, 4.0, 6.0];
    let expected_shape = vec![3];

    let result = (&t1 * &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}

#[test]
fn test_binary_op_division() {
    let t1 = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let t2 = CausalTensor::new(vec![5.0], vec![]).unwrap();
    let expected_data = vec![2.0, 4.0, 6.0];
    let expected_shape = vec![3];

    let result = (&t1 / &t2).unwrap();
    assert_eq!(result.as_slice(), expected_data.as_slice());
    assert_eq!(result.shape(), expected_shape.as_slice());
}
