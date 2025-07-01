/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use dcl_data_structures::prelude::{UnsafeArrayStorage, WindowStorage};
#[cfg(feature = "unsafe")]
use dcl_data_structures::window_type;
use dcl_data_structures::window_type::SlidingWindow;

const SIZE: usize = 4;
const CAPACITY: usize = 1200;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    dats: i32,
}

#[cfg(feature = "unsafe")]
fn get_sliding_window() -> SlidingWindow<UnsafeArrayStorage<Data, SIZE, CAPACITY>, Data> {
    window_type::new_with_unsafe_array_storage()
}

#[cfg(feature = "unsafe")]
#[test]
fn test_empty() {
    let d1 = Data { dats: 0 };
    let mut window = get_sliding_window();
    assert!(window.empty());

    window.push(d1);
    assert_eq!(window.size(), SIZE);
    assert!(!window.empty());
}

#[cfg(feature = "unsafe")]
#[test]
fn test_partial_fill() {
    let mut window = get_sliding_window();

    // Fill half of the window
    for i in 0..SIZE / 2 {
        window.push(Data { dats: i as i32 });
    }

    assert!(window.slice().is_err());
    assert!(!window.filled());
}

#[cfg(feature = "unsafe")]
#[test]
fn test_array_storage_default() {
    const SIZE: usize = 3;
    const CAPACITY: usize = 6;
    let storage = UnsafeArrayStorage::<i32, SIZE, CAPACITY>::default();
    assert_eq!(storage.size(), SIZE);
    assert_eq!(storage.tail(), 0);
    assert!(storage.get_slice().is_empty());
}

#[cfg(feature = "unsafe")]
#[test]
#[should_panic(expected = "CAPACITY must be greater than SIZE")]
fn test_array_storage_invalid_capacity() {
    const SIZE: usize = 4;
    const CAPACITY: usize = 3; // Invalid: CAPACITY < SIZE
    let _storage = UnsafeArrayStorage::<i32, SIZE, CAPACITY>::new();
}

#[cfg(feature = "unsafe")]
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

#[cfg(feature = "unsafe")]
#[test]
fn test_drop_old_values() {
    const SIZE: usize = 3;
    const CAPACITY: usize = 6;
    let mut window: SlidingWindow<UnsafeArrayStorage<i32, SIZE, CAPACITY>, i32> =
        window_type::new_with_unsafe_array_storage();

    // Fill the window
    window.push(1);
    window.push(2);
    window.push(3);
    assert!(window.filled());
    assert_eq!(window.vec().unwrap(), vec![1, 2, 3]);

    // Push more values, older values should be dropped
    window.push(4);
    assert_eq!(window.vec().unwrap(), vec![2, 3, 4]);

    window.push(5);
    assert_eq!(window.vec().unwrap(), vec![3, 4, 5]);

    window.push(6);
    assert_eq!(window.vec().unwrap(), vec![4, 5, 6]);

    // Verify first and last values
    assert_eq!(window.first().unwrap(), 4);
    assert_eq!(window.last().unwrap(), 6);

    // Push enough values to trigger a rewind
    window.push(7);
    window.push(8);
    window.push(9);
    assert_eq!(window.vec().unwrap(), vec![7, 8, 9]);
}

#[cfg(feature = "unsafe")]
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

#[cfg(feature = "unsafe")]
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

#[cfg(feature = "unsafe")]
#[test]
fn test_multiple_rewinds() {
    const SIZE: usize = 4;
    const CAPACITY: usize = 8;
    let mut window = get_sliding_window();

    // Perform multiple rewinds
    for i in 0..CAPACITY * 3 {
        let d = Data { dats: i as i32 };

        window.push(d);
        if i >= SIZE {
            let slice = window.slice().unwrap();
            assert_eq!(slice.len(), SIZE);
        }
    }
}

#[cfg(feature = "unsafe")]
#[test]
fn test_big_size() {
    const SIZE: usize = 128;
    const CAPACITY: usize = 8 * SIZE;

    fn get_sliding_window() -> SlidingWindow<UnsafeArrayStorage<Data, SIZE, CAPACITY>, Data> {
        window_type::new_with_unsafe_array_storage()
    }

    let mut window = get_sliding_window();

    // Perform multiple rewinds
    for i in 0..CAPACITY * 3 {
        let d = Data { dats: i as i32 };

        window.push(d);
        if i >= SIZE {
            let slice = window.slice().unwrap();
            assert_eq!(slice.len(), SIZE);
        }
    }
}

#[cfg(feature = "unsafe")]
#[test]
fn test_small_size() {
    const SIZE: usize = 2;
    const CAPACITY: usize = 4 * SIZE;

    #[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
    pub struct SmallData {
        dats: bool,
    }

    fn get_sliding_window()
    -> SlidingWindow<UnsafeArrayStorage<SmallData, SIZE, CAPACITY>, SmallData> {
        window_type::new_with_unsafe_array_storage()
    }

    let mut window = get_sliding_window();

    // Perform multiple rewinds
    for i in 0..CAPACITY * 3 {
        let d = SmallData { dats: false };

        window.push(d);
        if i >= SIZE {
            let slice = window.slice().unwrap();
            assert_eq!(slice.len(), SIZE);
        }
    }
}

#[cfg(feature = "unsafe")]
#[test]
fn test_rewind_at_capacity_boundary() {
    const SIZE: usize = 4;
    const CAPACITY: usize = 8;
    let mut window = get_sliding_window();

    // Fill exactly to capacity
    for i in 0..CAPACITY {
        let d = Data { dats: i as i32 };

        window.push(d);
    }
    let slice_before_rewind = window.vec().unwrap();
    assert_eq!(slice_before_rewind.len(), SIZE);
    assert_eq!(slice_before_rewind[0].dats, 4);
    assert_eq!(slice_before_rewind[1].dats, 5);
    assert_eq!(slice_before_rewind[2].dats, 6);
    assert_eq!(slice_before_rewind[3].dats, 7);

    // Trigger rewind
    let d = Data { dats: 42 };
    window.push(d);

    let slice_after_rewind = window.slice().unwrap();
    assert_eq!(slice_after_rewind.len(), SIZE);
    assert_eq!(slice_after_rewind[0].dats, 5);
    assert_eq!(slice_after_rewind[1].dats, 6);
    assert_eq!(slice_after_rewind[2].dats, 7);
    assert_eq!(slice_after_rewind[3].dats, 42);
}

#[cfg(feature = "unsafe")]
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

#[cfg(feature = "unsafe")]
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

#[cfg(feature = "unsafe")]
#[test]
fn test_first_element_sliding() {
    let mut window = get_sliding_window();

    window.push(Data { dats: 1 });
    window.push(Data { dats: 2 });
    window.push(Data { dats: 3 });
    window.push(Data { dats: 4 });

    assert_eq!(window.first().unwrap().dats, 1);

    window.push(Data { dats: 5 });
    assert_eq!(window.first().unwrap().dats, 2);

    window.push(Data { dats: 6 });
    assert_eq!(window.first().unwrap().dats, 3);
}

#[cfg(feature = "unsafe")]
#[test]
fn test_slice_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let s: Result<&[Data], String> = window.slice();
    assert!(s.is_err());
}

#[cfg(feature = "unsafe")]
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

#[cfg(feature = "unsafe")]
#[test]
fn test_vec_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let v: Result<Vec<Data>, String> = window.vec();
    assert!(v.is_err());
}

#[cfg(feature = "unsafe")]
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

#[cfg(feature = "unsafe")]
#[test]
fn test_arr_err() {
    let window = get_sliding_window();
    assert_eq!(window.size(), SIZE);
    assert!(!window.filled());

    let arr: Result<[Data; SIZE], String> = window.arr();
    assert!(arr.is_err());
}

#[cfg(feature = "unsafe")]
#[test]
fn test_push_when_full() {
    let mut window = get_sliding_window();

    // Fill the window completely
    for i in 0..CAPACITY + SIZE {
        window.push(Data { dats: i as i32 });
    }
    assert!(window.filled());

    // Add one more element, which should trigger rewind
    window.push(Data { dats: 42 });

    // Verify the window still contains SIZE elements
    let slice: [Data; SIZE] = window.arr().unwrap();
    assert_eq!(slice.len(), SIZE);

    // The last element should be what we just pushed (42)
    assert_eq!(slice[SIZE - 1].dats, 42);
}
