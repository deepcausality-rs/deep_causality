use deep_causality_num::NumCast;
use deep_causality_num::Quaternion;

#[test]
fn test_num_cast_from_f64() {
    let q = <Quaternion<f64> as NumCast>::from(123.45).unwrap();
    assert_eq!(q, Quaternion::new(123.45, 0.0, 0.0, 0.0));
}

#[test]
fn test_num_cast_from_i32() {
    let q = <Quaternion<f64> as NumCast>::from(123).unwrap();
    assert_eq!(q, Quaternion::new(123.0, 0.0, 0.0, 0.0));
}
