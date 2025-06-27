/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{window_type, SlidingWindow, VectorStorage, WindowStorage};

// Maximum number of elements held in the sliding window.
const SIZE: usize = 4;
// Multiplier to calculate capacity as size * multiple
const MULT: usize = 12;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

fn get_sliding_window() -> SlidingWindow<VectorStorage<Data>, Data> {
    window_type::new_with_vector_storage(SIZE, MULT)
}

#[test]
fn test_new() {
    let window = get_sliding_window();
    assert!(window.empty());
    assert_eq!(window.size(), SIZE);
}

#[test]
fn test_empty() {
    let d1 = Data { dats: 0 };
    let mut window = get_sliding_window();
    assert!(window.empty());

    window.push(d1);
    assert_eq!(window.size(), SIZE);
    assert!(!window.empty());
}

#[test]
fn test_push() {
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());
    assert!(window.empty());

    let d1 = Data { dats: 0 };
    window.push(d1);
    assert!(!window.filled());
    assert!(!window.empty());
}

#[test]
fn test_filled() {
    let d = Data { dats: 0 };
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);

    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    window.push(d);
    assert!(!window.filled());

    window.push(d);
    assert!(!window.filled());

    window.push(d);
    assert!(!window.filled());

    // Filled
    window.push(d);
    assert!(window.filled());

    window.push(d);
    assert!(window.filled());

    window.push(d);
    assert!(window.filled());

    window.push(d);
    assert!(window.filled());

    window.push(d);
    assert!(window.filled());

    // Rewinds b/c max capacity of 8 was reached
    window.push(d);
    assert!(window.filled());

    window.push(d);
    assert!(window.filled());
}

#[test]
fn test_first() {
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let res = window.first();
    assert!(res.is_err());

    window.push(Data { dats: 3 });
    window.push(Data { dats: 2 });
    window.push(Data { dats: 1 });
    window.push(Data { dats: 0 });
    assert!(window.filled());

    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 3);

    window.push(Data { dats: 4 });
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 2);

    window.push(Data { dats: 5 });
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 1);

    window.push(Data { dats: 6 });
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 0);

    window.push(Data { dats: 7 });
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 4);
}

#[test]
fn test_last() {
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let res = window.last();
    assert!(res.is_err());

    let d = Data { dats: 0 };
    window.push(d);
    window.push(d);
    window.push(d);
    window.push(d);
    assert!(window.filled());

    let res = window.first();
    assert!(res.is_ok());

    let data = res.unwrap();
    assert_eq!(data.dats, 0);

    let d = Data { dats: 42 };
    window.push(d);

    let res = window.last();
    assert!(res.is_ok());

    let data = res.unwrap();
    assert_eq!(data.dats, 42);
}

#[test]
fn test_slice() {
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let d = Data { dats: 0 };
    window.push(d);
    window.push(d);
    window.push(d);
    window.push(d);
    assert!(window.filled());

    let d = Data { dats: 42 };
    window.push(d);

    let slice = window.slice().unwrap();
    assert_eq!(slice.len(), SIZE);

    assert_eq!(slice[0].dats, 0);
    assert_eq!(slice[1].dats, 0);
    assert_eq!(slice[2].dats, 0);
    assert_eq!(slice[3].dats, 42);
}

#[test]
fn test_slice_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let res = window.slice();
    assert!(res.is_err());
}

#[test]
fn test_rewind_behavior() {
    let mut window = get_sliding_window();

    // Fill the window
    for i in 0..SIZE {
        window.push(Data { dats: i as i32 });
    }
    assert!(window.filled());

    // Push more elements to test rewind
    for i in SIZE..(SIZE * 2) {
        window.push(Data { dats: i as i32 });
    }

    // Verify the window contains the latest SIZE elements
    let slice = window.slice().unwrap();
    assert_eq!(slice.len(), SIZE);
    let start = SIZE * 2 - SIZE;
    for (i, item) in slice.iter().enumerate() {
        assert_eq!(item.dats, (start + i) as i32);
    }

    // Test first and last elements
    assert_eq!(window.first().unwrap().dats, start as i32);
    assert_eq!(window.last().unwrap().dats, (SIZE * 2 - 1) as i32);
}

#[test]
fn test_sequential_push() {
    let mut window = get_sliding_window();

    // Test sequential pushes and verify window state
    for i in 0..SIZE * 2 {
        window.push(Data { dats: i as i32 });

        if i < SIZE - 1 {
            assert!(!window.filled());
            assert_eq!(window.first().unwrap().dats, 0);
            assert!(window.last().is_err());
        } else {
            assert!(window.filled());
            let first = window.first().unwrap();
            assert_eq!(first.dats, (i + 1 - SIZE) as i32);
            assert_eq!(window.last().unwrap().dats, i as i32);
        }
    }
}

#[test]
fn test_edge_cases() {
    let mut window = get_sliding_window();

    // Test empty window edge cases
    assert!(window.first().is_err());
    assert!(window.last().is_err());
    assert!(window.slice().is_err());

    // Test single element
    window.push(Data { dats: 42 });
    assert_eq!(window.first().unwrap().dats, 42);
    assert!(window.last().is_err()); // Window not filled yet
    assert!(window.slice().is_err()); // Window not filled yet

    // Fill the window
    for _ in 1..SIZE {
        window.push(Data { dats: 42 });
    }
    assert!(window.filled());
    assert!(window.slice().is_ok());
    assert_eq!(window.slice().unwrap().len(), SIZE);

    // Test maximum value
    window.push(Data { dats: i32::MAX });
    assert!(window.filled());
    assert_eq!(window.last().unwrap().dats, i32::MAX);

    // Test minimum value
    window.push(Data { dats: i32::MIN });
    assert!(window.filled());
    assert_eq!(window.last().unwrap().dats, i32::MIN);
}

#[test]
fn test_rapid_pushes() {
    let mut window = get_sliding_window();

    // Perform rapid pushes
    for i in 0..SIZE * 3 {
        window.push(Data { dats: i as i32 });
    }

    // Verify final state
    assert!(window.filled());
    let slice = window.slice().unwrap();
    assert_eq!(slice.len(), SIZE);

    // Verify the contents are the last SIZE elements
    let start = (SIZE * 3) - SIZE;
    for (i, item) in slice.iter().enumerate() {
        assert_eq!(item.dats, (start + i) as i32);
    }
}

#[test]
fn test_vec() {
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let d = Data { dats: 0 };
    window.push(d);
    window.push(d);
    window.push(d);
    window.push(d);
    assert!(window.filled());

    let d = Data { dats: 42 };
    window.push(d);

    let vec = window.vec().unwrap();
    assert_eq!(vec.len(), SIZE);

    assert_eq!(vec[0].dats, 0);
    assert_eq!(vec[1].dats, 0);
    assert_eq!(vec[2].dats, 0);
    assert_eq!(vec[3].dats, 42);
}

#[test]
fn test_vec_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let res = window.vec();
    assert!(res.is_err());
}

#[test]
fn test_arr() {
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let d = Data { dats: 0 };
    window.push(d);
    window.push(d);
    window.push(d);
    window.push(d);
    assert!(window.filled());

    let d = Data { dats: 42 };
    window.push(d);

    let arr: [Data; SIZE] = window.arr().unwrap();
    assert_eq!(arr.len(), SIZE);

    assert_eq!(arr[0].dats, 0);
    assert_eq!(arr[1].dats, 0);
    assert_eq!(arr[2].dats, 0);
    assert_eq!(arr[3].dats, 42);
}

#[test]
fn test_arr_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let res: Result<[Data; SIZE], String> = window.arr();
    assert!(res.is_err());
}

#[test]
fn test_push_boundary_conditions() {
    let mut storage = VectorStorage::<i32>::new(2, 2);

    // Test pushing values and checking last
    for i in 0..2 {
        storage.push(i);
        // last() should return error until window is filled
        if i < 1 {
            assert!(storage.last().is_err());
        } else {
            assert_eq!(storage.last().unwrap(), i);
        }
    }

    // Now the window is filled, push more values
    storage.push(2);
    assert_eq!(storage.vec().unwrap(), vec![1, 2]);

    storage.push(3);
    assert_eq!(storage.vec().unwrap(), vec![2, 3]);

    storage.push(4);
    assert_eq!(storage.vec().unwrap(), vec![2, 3, 4]);

    storage.push(5);
    assert_eq!(storage.vec().unwrap(), vec![3, 4, 5]);
}

#[test]
fn test_error_conditions() {
    let mut storage = VectorStorage::<i32>::new(2, 2);

    // Test empty storage conditions
    assert!(storage.first().is_err());
    assert!(storage.last().is_err());
    assert!(storage.vec().is_err());

    // Add one element
    storage.push(1);
    assert_eq!(storage.first().unwrap(), 1); // first() works with any elements
    assert!(storage.last().is_err()); // last() requires window to be filled
    assert!(storage.vec().is_err());

    // Add second element to fill the window
    storage.push(2);
    assert_eq!(storage.first().unwrap(), 1);
    assert_eq!(storage.last().unwrap(), 2); // Now last() works as window is filled
    assert_eq!(storage.vec().unwrap(), vec![1, 2]);

    // Verify error messages
    let empty_storage = VectorStorage::<i32>::new(2, 2);
    assert_eq!(
        empty_storage.first().unwrap_err(),
        "Vector is empty. Add some elements to the array first"
    );

    // Add one element to test last() error message
    let mut partial_storage = VectorStorage::<i32>::new(2, 2);
    partial_storage.push(1);
    assert_eq!(
        partial_storage.last().unwrap_err(),
        "Vector is not yet filled. Add some elements to the array first"
    );
}

#[test]
fn test_rewind_with_different_sizes() {
    let sizes = [2, 3, 4, 5];
    let mults = [2, 3, 4];

    for &size in sizes.iter() {
        for &mult in mults.iter() {
            let mut storage = VectorStorage::<i32>::new(size, mult);

            // Fill the storage
            for i in 0..size {
                storage.push(i as i32);
            }

            // Verify the content
            let expected: Vec<i32> = (0..size).map(|x| x as i32).collect();
            assert_eq!(storage.vec().unwrap(), expected);
        }
    }
}

#[test]
fn test_performance_scaling() {
    use std::time::Instant;

    // Test with different capacity multipliers
    let sizes = [100, 1000, 10000];
    let multipliers = [2, 4, 8];

    for &size in &sizes {
        for &mult in &multipliers {
            let mut storage = VectorStorage::<i32>::new(size, mult);
            let start = Instant::now();

            // Push elements up to capacity
            for i in 0..size * mult {
                storage.push(i as i32);
            }

            let duration = start.elapsed();
            println!(
                "Size: {size}, Multiplier: {mult}, Duration: {duration:?}"
            );

            // Verify correctness
            assert_eq!(storage.last().unwrap(), ((size * mult - 1) as i32));
            assert!(storage.filled());
        }
    }
}
