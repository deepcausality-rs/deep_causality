use dcl_data_structures::prelude::PointIndex;

#[test]
fn test_new1d() {
    let point = PointIndex::new1d(5);
    assert_eq!(point.x, 5);
    assert_eq!(point.y, 0);
    assert_eq!(point.z, 0);
    assert_eq!(point.t, 0);
}

#[test]
fn test_new2d() {
    let point = PointIndex::new2d(5, 10);
    assert_eq!(point.x, 5);
    assert_eq!(point.y, 10);
    assert_eq!(point.z, 0);
    assert_eq!(point.t, 0);
}

#[test]
fn test_new3d() {
    let point = PointIndex::new3d(5, 10, 15);
    assert_eq!(point.x, 5);
    assert_eq!(point.y, 10);
    assert_eq!(point.z, 15);
    assert_eq!(point.t, 0);
}

#[test]
fn test_new4d() {
    let point = PointIndex::new4d(5, 10, 15, 20);
    assert_eq!(point.x, 5);
    assert_eq!(point.y, 10);
    assert_eq!(point.z, 15);
    assert_eq!(point.t, 20);
}

#[test]
fn test_copy_clone() {
    let point = PointIndex::new4d(1, 2, 3, 4);
    let copied = point;  // Test Copy
    assert_eq!(point.x, copied.x);
    assert_eq!(point.y, copied.y);
    assert_eq!(point.z, copied.z);
    assert_eq!(point.t, copied.t);

    let cloned = point.clone();  // Test Clone
    assert_eq!(point.x, cloned.x);
    assert_eq!(point.y, cloned.y);
    assert_eq!(point.z, cloned.z);
    assert_eq!(point.t, cloned.t);
}

#[test]
fn test_debug() {
    let point = PointIndex::new4d(1, 2, 3, 4);
    let debug_str = format!("{:?}", point);
    assert!(debug_str.contains("PointIndex"));
    assert!(debug_str.contains("1"));
    assert!(debug_str.contains("2"));
    assert!(debug_str.contains("3"));
    assert!(debug_str.contains("4"));
}