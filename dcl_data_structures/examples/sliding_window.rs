// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use dcl_data_structures::prelude::{ArrayStorage, SlidingWindow,sliding_window};

// Size refers to the maximum number of elements the sliding window can store.
const SIZE: usize = 4;
// Capacity refers to the maximum number of elements before a rewind occurs.
// Note, CAPACITY > SIZE and capacity should be a multiple of size.
// For example, size 4 should be stored 300 times before rewind;
// 4 * 300 = 1200
const CAPACITY: usize = 1200;

// SlidingWindow requires PartialEq + Copy + Default
#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

// Util function that helps with type inference.
fn get_sliding_window() -> SlidingWindow<ArrayStorage<Data, SIZE, CAPACITY>, Data> {
    sliding_window::new_with_array_storage()
}

pub fn main(){
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);

    // Filled means, the window holds 4 elements.
    assert!(!window.filled());

    // If you try to access an element before the window id filled, you get an error.
    let res = window.first();
    assert!(res.is_err());

    let res = window.last();
    assert!(res.is_err());

    // Add some data
    window.push(Data { dats: 3 });
    window.push(Data { dats: 2 });
    window.push(Data { dats: 1 });
    window.push(Data { dats: 0 });
    assert!(window.filled());

    // Now we can access elements of the window
    // Last element added was 0
    let res = window.last();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 0);

    // First (oldest) element added was 3
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 3);

    // When we add more data after the window filled,
    // the "last" element refers to the last added
    // and the oldest element will be dropped.
    // This shifts the order of the elements in the window.

    let d = Data { dats: 42 };
    window.push(d);

    let res = window.last();
    assert!(res.is_ok());

    let data = res.unwrap();
    assert_eq!(data.dats, 42);

    // Because 42 was added at the front,
    // 3 was dropped at the end therefore
    // the oldest element is now 2
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 2);

}

