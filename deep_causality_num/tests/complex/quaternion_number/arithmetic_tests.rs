use deep_causality_num::Float;
use deep_causality_num::Quaternion;

#[test]
fn test_add() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    let expected = Quaternion::new(6.0, 8.0, 10.0, 12.0);
    assert_eq!(q1 + q2, expected);
}

#[test]
fn test_sub() {
    let q1 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    let q2 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let expected = Quaternion::new(4.0, 4.0, 4.0, 4.0);
    assert_eq!(q1 - q2, expected);
}

#[test]
fn test_mul_quaternion() {
    let q2 = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let q3 = Quaternion::new(0.0, 0.0, 1.0, 0.0); // j
    let q4 = Quaternion::new(0.0, 0.0, 0.0, 1.0); // k

    // i * j = k
    let expected = Quaternion::new(0.0, 0.0, 0.0, 1.0);
    assert_eq!(q2 * q3, expected);

    // j * k = i
    let expected = Quaternion::new(0.0, 1.0, 0.0, 0.0);
    assert_eq!(q3 * q4, expected);

    // k * i = j
    let expected = Quaternion::new(0.0, 0.0, 1.0, 0.0);
    assert_eq!(q4 * q2, expected);

    // j * i = -k
    let expected = Quaternion::new(0.0, 0.0, 0.0, -1.0);
    assert_eq!(q3 * q2, expected);

    // General case
    let qa = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let qb = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    let expected_w = 1.0 * 5.0 - 2.0 * 6.0 - 3.0 * 7.0 - 4.0 * 8.0; // 5 - 12 - 21 - 32 = -60
    let expected_x = 1.0 * 6.0 + 2.0 * 5.0 + 3.0 * 8.0 - 4.0 * 7.0; // 6 + 10 + 24 - 28 = 12
    let expected_y = 1.0 * 7.0 - 2.0 * 8.0 + 3.0 * 5.0 + 4.0 * 6.0; // 7 - 16 + 15 + 24 = 30
    let expected_z = 1.0 * 8.0 + 2.0 * 7.0 - 3.0 * 6.0 + 4.0 * 5.0; // 8 + 14 - 18 + 20 = 24
    let expected = Quaternion::new(expected_w, expected_x, expected_y, expected_z);
    assert_eq!(qa * qb, expected);
}

#[test]
fn test_mul_scalar() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let scalar = 2.0;
    let expected = Quaternion::new(2.0, 4.0, 6.0, 8.0);
    assert_eq!(q * scalar, expected);
}

#[test]
fn test_div_quaternion() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::new(1.0, 0.0, 0.0, 0.0); // Identity
    assert_eq!(q1 / q2, q1);

    let q_i = Quaternion::new(0.0, 1.0, 0.0, 0.0);
    let q_j = Quaternion::new(0.0, 0.0, 1.0, 0.0);

    // i / j = -k
    let expected = Quaternion::new(0.0, 0.0, 0.0, -1.0);
    let result = q_i / q_j;
    assert!((result.w - expected.w).abs() < 1e-9);
    assert!((result.x - expected.x).abs() < 1e-9);
    assert!((result.y - expected.y).abs() < 1e-9);
    assert!((result.z - expected.z).abs() < 1e-9);
}

#[test]
fn test_div_scalar() {
    let q = Quaternion::new(2.0, 4.0, 6.0, 8.0);
    let scalar = 2.0;
    let expected = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q / scalar, expected);

    let q_inf = Quaternion::new(1.0, 1.0, 1.0, 1.0);
    let scalar_zero = 0.0;
    let result = q_inf / scalar_zero;
    assert!(result.w.is_infinite());
    assert!(result.x.is_infinite());
    assert!(result.y.is_infinite());
    assert!(result.z.is_infinite());
}

#[test]
fn test_rem() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    // Remainder for quaternions is not standard. The current implementation of Rem<Quaternion> for Quaternion returns self.
    assert_eq!(q1 % q2, q1);
}

#[test]
fn test_sum() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    let q3 = Quaternion::new(9.0, 10.0, 11.0, 12.0);
    let quaternions = vec![q1, q2, q3];
    let expected = Quaternion::new(15.0, 18.0, 21.0, 24.0);
    assert_eq!(quaternions.into_iter().sum::<Quaternion<f64>>(), expected);
}

#[test]
fn test_product() {
    let q1 = Quaternion::new(1.0, 0.0, 0.0, 0.0); // 1
    let q2 = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let q3 = Quaternion::new(0.0, 0.0, 1.0, 0.0); // j
    let quaternions = vec![q1, q2, q3];
    // 1 * i * j = k
    let expected = Quaternion::new(0.0, 0.0, 0.0, 1.0);
    let result = quaternions.into_iter().product::<Quaternion<f64>>();
    const EPSILON: f64 = 1e-9;
    assert!((result.w - expected.w).abs() < EPSILON);
    assert!((result.x - expected.x).abs() < EPSILON);
    assert!((result.y - expected.y).abs() < EPSILON);
    assert!((result.z - expected.z).abs() < EPSILON);
}
