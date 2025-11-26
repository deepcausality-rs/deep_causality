use core::cmp::Ordering;
use deep_causality_num::Quaternion;

#[test]
fn test_partial_ord_equal() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q1.partial_cmp(&q2), Some(Ordering::Equal));
}

#[test]
fn test_partial_ord_less() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::new(1.0, 2.0, 3.0, 5.0);
    assert_eq!(q1.partial_cmp(&q2), Some(Ordering::Less));

    let q3 = Quaternion::new(1.0, 2.0, 2.0, 4.0);
    let q4 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q3.partial_cmp(&q4), Some(Ordering::Less));
}

#[test]
fn test_partial_ord_greater() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 5.0);
    let q2 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q1.partial_cmp(&q2), Some(Ordering::Greater));
}

#[test]
fn test_partial_ord_nan() {
    let q1 = Quaternion::new(1.0, f64::NAN, 3.0, 4.0);
    let q2 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q1.partial_cmp(&q2), None);
}
