// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use dcl_data_structures::prelude::{SlidingWindow, sliding_window, VectorStorage};

const SIZE: usize = 4;
const MULT: usize = 2;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

fn get_sliding_window() -> SlidingWindow<VectorStorage<Data>, Data> {
    sliding_window::new_with_vector_storage(SIZE, MULT)
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
    let d = Data { dats: 0 };
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let res = window.first();
    assert!(res.is_err());

    window.push(d);
    assert!(!window.filled());

    let res = window.first();
    assert!(res.is_ok());

    let data = res.unwrap();
    assert_eq!(data.dats, 0);
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

    let slice = window.slice().expect("Failed to get slice");
    assert_eq!(slice.len(), SIZE);

    assert_eq!(slice[0].dats, 0);
    assert_eq!(slice[1].dats, 0);
    assert_eq!(slice[2].dats, 0);
    assert_eq!(slice[3].dats, 42);
}


#[test]
fn test_vec() {
    let d1 = Data { dats: 0 };
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);

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

    let v = window.vec().unwrap();
    assert_eq!(v.len(), SIZE);

    let e1 = window.first().unwrap();
    let v1 = v.get(0).unwrap();
    assert_eq!(e1.dats, v1.dats);

    let e2 = window.last().unwrap();
    let v2 = v.get(SIZE - 1).unwrap();
    assert_eq!(e2.dats, v2.dats);
}