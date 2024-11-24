use dcl_data_structures::prelude::{PointIndex, Storage};

#[test]
fn test_get_set() {
    let mut array: [[i32; 3]; 4] = [[0; 3]; 4];
    let point = PointIndex::new2d(1, 2);

    // Test set
    <[[i32; 3]; 4] as Storage<i32>>::set(&mut array, point, 42);
    assert_eq!(array[2][1], 42);

    // Test get
    assert_eq!(*<[[i32; 3]; 4] as Storage<i32>>::get(&array, point), 42);
}

#[test]
fn test_dimensions() {
    let array: [[i32; 3]; 4] = [[0; 3]; 4];

    assert_eq!(<[[i32; 3]; 4] as Storage<i32>>::width(&array), Some(&3));
    assert_eq!(<[[i32; 3]; 4] as Storage<i32>>::height(&array), Some(&4));
}

#[test]
fn test_multiple_updates() {
    let mut array: [[i32; 3]; 4] = [[0; 3]; 4];

    // Test multiple points
    let points = [
        (PointIndex::new2d(0, 0), 1),
        (PointIndex::new2d(1, 1), 2),
        (PointIndex::new2d(2, 2), 3),
        (PointIndex::new2d(1, 3), 4),
    ];

    // Set values
    for (point, value) in points.iter() {
        <[[i32; 3]; 4] as Storage<i32>>::set(&mut array, *point, *value);
    }

    // Verify values
    for (point, expected_value) in points.iter() {
        assert_eq!(
            *<[[i32; 3]; 4] as Storage<i32>>::get(&array, *point),
            *expected_value
        );
    }
}

#[test]
fn test_array_bounds() {
    let array: [[i32; 2]; 2] = [[1, 2], [3, 4]];

    // Test all valid positions
    let points = [
        (PointIndex::new2d(0, 0), 1),
        (PointIndex::new2d(1, 0), 2),
        (PointIndex::new2d(0, 1), 3),
        (PointIndex::new2d(1, 1), 4),
    ];

    for (point, expected_value) in points.iter() {
        assert_eq!(
            *<[[i32; 2]; 2] as Storage<i32>>::get(&array, *point),
            *expected_value
        );
    }
}

#[test]
#[should_panic]
fn test_out_of_bounds_x() {
    let array: [[i32; 2]; 2] = [[1, 2], [3, 4]];
    let point = PointIndex::new2d(2, 0); // x is out of bounds
    let _: &i32 = <[[i32; 2]; 2] as Storage<i32>>::get(&array, point);
}

#[test]
#[should_panic]
fn test_out_of_bounds_y() {
    let array: [[i32; 2]; 2] = [[1, 2], [3, 4]];
    let point = PointIndex::new2d(0, 2); // y is out of bounds
    let _: &i32 = <[[i32; 2]; 2] as Storage<i32>>::get(&array, point);
}

#[test]
fn test_different_types() {
    // Test with f64
    let mut float_array: [[f64; 2]; 2] = [[0.0; 2]; 2];
    let point = PointIndex::new2d(1, 1);
    <[[f64; 2]; 2] as Storage<f64>>::set(&mut float_array, point, 3.14);
    assert_eq!(
        *<[[f64; 2]; 2] as Storage<f64>>::get(&float_array, point),
        3.14
    );

    // Test with bool
    let mut bool_array: [[bool; 2]; 2] = [[false; 2]; 2];
    <[[bool; 2]; 2] as Storage<bool>>::set(&mut bool_array, point, true);
    assert_eq!(
        *<[[bool; 2]; 2] as Storage<bool>>::get(&bool_array, point),
        true
    );

    // Test with char
    let mut char_array: [[char; 2]; 2] = [['a'; 2]; 2];
    <[[char; 2]; 2] as Storage<char>>::set(&mut char_array, point, 'z');
    assert_eq!(
        *<[[char; 2]; 2] as Storage<char>>::get(&char_array, point),
        'z'
    );
}
