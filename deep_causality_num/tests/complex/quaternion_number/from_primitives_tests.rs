use deep_causality_num::FromPrimitive;
use deep_causality_num::Quaternion;

#[test]
fn test_from_isize() {
    let q = Quaternion::<f64>::from_isize(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_i8() {
    let q = Quaternion::<f64>::from_i8(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_i16() {
    let q = Quaternion::<f64>::from_i16(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_i32() {
    let q = Quaternion::<f64>::from_i32(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_i64() {
    let q = Quaternion::<f64>::from_i64(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_i128() {
    let q = Quaternion::<f64>::from_i128(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_usize() {
    let q = Quaternion::<f64>::from_usize(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_u8() {
    let q = Quaternion::<f64>::from_u8(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_u16() {
    let q = Quaternion::<f64>::from_u16(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_u32() {
    let q = Quaternion::<f64>::from_u32(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_u64() {
    let q = Quaternion::<f64>::from_u64(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_from_u128() {
    let q = Quaternion::<f64>::from_u128(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}

fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
    assert!(
        (a - b).abs() < epsilon,
        "Assertion failed: {} is not approximately equal to {}",
        a,
        b
    );
}

#[test]
fn test_from_f32() {
    let q = Quaternion::<f64>::from_f32(123.45).unwrap();
    assert_approx_eq(q.w, 123.45, 1.0e-5);
    assert_eq!(q.x, 0.0);
    assert_eq!(q.y, 0.0);
    assert_eq!(q.z, 0.0);
}

#[test]
fn test_from_f64() {
    let q = Quaternion::<f64>::from_f64(123.45).unwrap();
    assert_eq!(q, Quaternion::new(123.45, 0.0, 0.0, 0.0));
}
