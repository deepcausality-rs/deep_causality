/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{PointIndex, Storage};

#[test]
fn test_3d_array_indexing_bug() {
    // Distinct dimensions to reveal the bug
    const WIDTH: usize = 3; // X
    const HEIGHT: usize = 4; // Y
    const DEPTH: usize = 2; // Z

    let mut storage: [[[i32; WIDTH]; HEIGHT]; DEPTH] = [[[0; WIDTH]; HEIGHT]; DEPTH];

    // Raw array: [depth][height][width]
    // Set value at depth=1, height=2, width=1
    storage[1][2][1] = 99;

    let point = PointIndex::new3d(
        1, // x = width
        2, // y = height
        1, // z = depth
    );

    // This should retrieve storage[1][2][1]
    // Buggy impl does storage[y][x][z] -> storage[2][1][1] -> out of bounds or wrong value
    let retrieved = *<[[[i32; WIDTH]; HEIGHT]; DEPTH] as Storage<i32>>::get(&storage, point);

    assert_eq!(retrieved, 99, "3D Indexing is incorrect");
}

#[test]
fn test_4d_array_indexing_bug() {
    const WIDTH: usize = 2; // X
    const HEIGHT: usize = 3; // Y
    const DEPTH: usize = 2; // Z
    const TIME: usize = 2; // T

    let mut storage: [[[[i32; WIDTH]; HEIGHT]; DEPTH]; TIME] =
        [[[[0; WIDTH]; HEIGHT]; DEPTH]; TIME];

    // Raw array: [time][depth][height][width]
    // Set at time=1, depth=1, height=2, width=1
    storage[1][1][2][1] = 88;

    let point = PointIndex::new4d(
        1, // x
        2, // y
        1, // z
        1, // t
    );

    // This should retrieve storage[1][1][2][1]
    let retrieved =
        *<[[[[i32; WIDTH]; HEIGHT]; DEPTH]; TIME] as Storage<i32>>::get(&storage, point);

    assert_eq!(retrieved, 88, "4D Indexing is incorrect");
}
