/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_slice_operations() {
    // 1. Setup a 3D tensor with predictable data
    let data: Vec<i32> = (0..24).collect();
    let tensor = CausalTensor::new(data, vec![2, 3, 4]).unwrap();
    // Tensor shape: [2, 3, 4]
    // Data layout:
    // Plane 0 (index 0 on axis 0):
    // [[ 0,  1,  2,  3],
    //  [ 4,  5,  6,  7],
    //  [ 8,  9, 10, 11]]
    // Plane 1 (index 1 on axis 0):
    // [[12, 13, 14, 15],
    //  [16, 17, 18, 19],
    //  [20, 21, 22, 23]]

    // 2. Test valid slicing

    // Slice along axis 0, getting the second plane (index 1)
    let slice_axis0 = tensor.slice(0, 1).unwrap();
    assert_eq!(slice_axis0.shape(), &[3, 4]);
    let expected_data_axis0: Vec<i32> = (12..24).collect();
    assert_eq!(slice_axis0.as_slice(), &expected_data_axis0);

    // Slice along axis 1, getting the third row (index 2) from each plane
    let slice_axis1 = tensor.slice(1, 2).unwrap();
    assert_eq!(slice_axis1.shape(), &[2, 4]);
    let expected_data_axis1: Vec<i32> = vec![8, 9, 10, 11, 20, 21, 22, 23];
    assert_eq!(slice_axis1.as_slice(), &expected_data_axis1);

    // Slice along axis 2, getting the fourth column (index 3) from each plane
    let slice_axis2 = tensor.slice(2, 3).unwrap();
    assert_eq!(slice_axis2.shape(), &[2, 3]);
    let expected_data_axis2: Vec<i32> = vec![3, 7, 11, 15, 19, 23];
    assert_eq!(slice_axis2.as_slice(), &expected_data_axis2);

    // 3. Test error cases

    // Test invalid axis (axis >= num_dim)
    let invalid_axis_result = tensor.slice(3, 0);
    assert!(matches!(
        invalid_axis_result,
        Err(CausalTensorError::AxisOutOfBounds)
    ));

    // Test invalid index (index >= shape[axis])
    let index_out_of_bounds_result = tensor.slice(0, 2); // Axis 0 has size 2, so index 2 is invalid
    assert!(matches!(
        index_out_of_bounds_result,
        Err(CausalTensorError::AxisOutOfBounds)
    ));
}
