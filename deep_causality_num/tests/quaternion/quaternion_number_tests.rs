use deep_causality_num::Float;
use deep_causality_num::Quaternion;

#[test]
fn test_conjugate() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let expected = Quaternion::new(1.0, -2.0, -3.0, -4.0);
    assert_eq!(q.conjugate(), expected);
}

#[test]
fn test_norm_sqr() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q.norm_sqr(), 1.0 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0 + 4.0 * 4.0);
}

#[test]
fn test_norm() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(
        q.norm(),
        (1.0f64.powi(2) + 2.0f64.powi(2) + 3.0f64.powi(2) + 4.0f64.powi(2)).sqrt()
    );
}

#[test]
fn test_normalize() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let norm = q.norm();
    let expected = Quaternion::new(1.0 / norm, 2.0 / norm, 3.0 / norm, 4.0 / norm);
    let result = q.normalize();
    const EPSILON: f64 = 1e-9;
    assert!((result.w - expected.w).abs() < EPSILON);
    assert!((result.x - expected.x).abs() < EPSILON);
    assert!((result.y - expected.y).abs() < EPSILON);
    assert!((result.z - expected.z).abs() < EPSILON);

    // Test normalize zero quaternion
    let zero_q = Quaternion::new(0.0, 0.0, 0.0, 0.0);
    assert_eq!(zero_q.normalize(), zero_q);
}

#[test]
fn test_inverse() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let norm_sqr = q.norm_sqr();
    let expected = Quaternion::new(
        1.0 / norm_sqr,
        -2.0 / norm_sqr,
        -3.0 / norm_sqr,
        -4.0 / norm_sqr,
    );
    let result = q.inverse();
    const EPSILON: f64 = 1e-9;
    assert!((result.w - expected.w).abs() < EPSILON);
    assert!((result.x - expected.x).abs() < EPSILON);
    assert!((result.y - expected.y).abs() < EPSILON);
    assert!((result.z - expected.z).abs() < EPSILON);

    // Test inverse of zero quaternion
    let zero_q = Quaternion::new(0.0, 0.0, 0.0, 0.0);
    let inv_zero_q = zero_q.inverse();
    assert!(inv_zero_q.w.is_nan());
    assert!(inv_zero_q.x.is_nan());
    assert!(inv_zero_q.y.is_nan());
    assert!(inv_zero_q.z.is_nan());
}

#[test]
fn test_dot() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    let expected = 1.0 * 5.0 + 2.0 * 6.0 + 3.0 * 7.0 + 4.0 * 8.0;
    assert_eq!(q1.dot(&q2), expected);
}
