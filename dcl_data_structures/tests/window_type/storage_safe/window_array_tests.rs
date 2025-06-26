/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{window_type, ArrayStorage, SlidingWindow};

const SIZE: usize = 4;
const CAPACITY: usize = 1200;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

fn get_sliding_window() -> SlidingWindow<ArrayStorage<Data, SIZE, CAPACITY>, Data> {
    window_type::new_with_array_storage()
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
fn test_default() {
    let window: SlidingWindow<ArrayStorage<Data, SIZE, CAPACITY>, Data> =
        window_type::default_array_storage();
    assert_eq!(window.size(), SIZE);
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
fn test_filled() {
    let d = Data { dats: 0 };
    let mut window = get_sliding_window();

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
fn test_slice() {
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

    let slice = window.slice().expect("Failed to get slice");
    assert_eq!(slice.len(), SIZE);
    assert_eq!(slice[0].dats, 0);
    assert_eq!(slice[1].dats, 0);
    assert_eq!(slice[2].dats, 0);
    assert_eq!(slice[3].dats, 42);

    let d = Data { dats: 0 };
    window.push(d);
    assert!(window.filled());

    let slice = window.slice().expect("Failed to get slice");
    assert_eq!(slice.len(), SIZE);
    assert_eq!(slice[0].dats, 0);
    assert_eq!(slice[1].dats, 0);
    assert_eq!(slice[2].dats, 42);
    assert_eq!(slice[3].dats, 0);
}

#[test]
fn test_slice_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let s: Result<&[Data], String> = window.slice();
    assert!(s.is_err());
}

#[test]
fn test_vec() {
    let d1 = Data { dats: 0 };
    let mut window = get_sliding_window();

    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    window.push(d1);
    window.push(d1);
    window.push(d1);
    window.push(d1);
    assert!(window.filled());

    let d2 = Data { dats: 42 };
    window.push(d2);

    let e1 = window.first().unwrap();
    assert_eq!(e1.dats, d1.dats);

    let e2 = window.last().unwrap();
    assert_eq!(e2.dats, d2.dats);

    let v = window.vec().expect("Failed to get vec");
    assert_eq!(v.len(), SIZE);

    let e1 = window.first().unwrap();
    let v1 = v.first().unwrap();
    assert_eq!(e1.dats, v1.dats);

    let e2 = window.last().unwrap();
    let v2 = v.get(SIZE - 1).unwrap();
    assert_eq!(e2.dats, v2.dats);
}

#[test]
fn test_vec_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let v: Result<Vec<Data>, String> = window.vec();
    assert!(v.is_err());
}

#[test]
fn test_arr() {
    let d1 = Data { dats: 0 };
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    window.push(d1);
    window.push(d1);
    window.push(d1);
    window.push(d1);
    assert!(window.filled());

    let slice = window.slice().expect("Failed to get slice");
    assert_eq!(slice.len(), SIZE);

    // Filled
    let d2 = Data { dats: 42 };
    window.push(d2);
    assert!(window.filled());

    let e1 = window.first().unwrap();
    assert_eq!(e1.dats, d1.dats);

    let e2 = window.last().unwrap();
    assert_eq!(e2.dats, d2.dats);

    let slice = window.slice().expect("Failed to get slice");
    assert_eq!(slice.len(), SIZE);

    let arr: [Data; SIZE] = window.arr().unwrap();
    assert_eq!(arr.len(), SIZE);

    let e1 = window.first().unwrap();
    let a1 = arr.first().unwrap();
    assert_eq!(e1.dats, a1.dats);

    let e2 = window.last().unwrap();
    let a2 = arr.get(SIZE - 1).unwrap();
    assert_eq!(e2.dats, a2.dats);

    assert_eq!(arr[0].dats, 0);
    assert_eq!(arr[1].dats, 0);
    assert_eq!(arr[2].dats, 0);
    assert_eq!(arr[3].dats, 42);

    let d = Data { dats: 0 };
    window.push(d);

    let arr: [Data; SIZE] = window.arr().unwrap();
    assert_eq!(arr.len(), SIZE);

    assert_eq!(arr[0].dats, 0);
    assert_eq!(arr[1].dats, 0);
    assert_eq!(arr[2].dats, 42);
    assert_eq!(arr[3].dats, 0);
}

#[test]
fn test_arr_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let arr: Result<[Data; SIZE], String> = window.arr();
    assert!(arr.is_err());
}

#[test]
fn test_drop_old_values() {
    const SIZE: usize = 3;
    const CAPACITY: usize = 6;
    let mut window: SlidingWindow<ArrayStorage<i32, SIZE, CAPACITY>, i32> =
        window_type::new_with_array_storage();

    // Fill the window
    window.push(1);
    window.push(2);
    window.push(3);
    assert!(window.filled());
    assert_eq!(window.vec().unwrap(), vec![1, 2, 3]);

    // Push more values, older values should be dropped
    window.push(4);
    let vec = window.vec().unwrap();
    assert_eq!(vec, vec![2, 3, 4]);

    window.push(5);
    let vec = window.vec().unwrap();
    assert_eq!(vec, vec![3, 4, 5]);

    window.push(6);
    let vec = window.vec().unwrap();
    assert_eq!(vec, vec![4, 5, 6]);

    // Verify first and last values
    assert_eq!(window.first().unwrap(), 4);
    assert_eq!(window.last().unwrap(), 6);

    // Push enough values to trigger a rewind
    window.push(7);
    window.push(8);
    window.push(9);
    let vec = window.vec().unwrap();
    assert_eq!(vec, vec![7, 8, 9]);
}
