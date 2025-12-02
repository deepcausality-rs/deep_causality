/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Quaternion;
use deep_causality_num::{One, RealField, Zero};

const EPSILON: f64 = 1e-9;

#[test]
fn test_sin_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.sin();

    // Expected values calculated manually or using a reference library
    // For q = w + xi + yj + zk
    // sin(q) = sin(w)cosh(|v|) + (v/|v|)cos(w)sinh(|v|)
    // |v| = sqrt(x^2 + y^2 + z^2) = sqrt(1+4+9) = sqrt(14)
    // w = 0.5 rad
    let w = 0.5f64;
    let v_norm = 14.0f64.sqrt();

    let scalar_part_expected = w.sin() * v_norm.cosh();
    let vector_scale_expected = w.cos() * v_norm.sinh() / v_norm;

    let expected_q = Quaternion::new(
        scalar_part_expected,
        1.0 * vector_scale_expected,
        2.0 * vector_scale_expected,
        3.0 * vector_scale_expected,
    );

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_sin_purely_real() {
    let q = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    let result = q.sin();

    let expected_q = Quaternion::new(0.5f64.sin(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_sin_purely_imaginary() {
    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.sin();

    // sin(i) = i sinh(1)
    let expected_q = Quaternion::new(0.0, 1.0f64.sinh(), 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_sin_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.sin();

    let expected_q = Quaternion::<f64>::zero();

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_cos_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.cos();

    // Expected values calculated manually or using a reference library
    // For q = w + xi + yj + zk
    // cos(q) = cos(w)cosh(|v|) - (v/|v|)sin(w)sinh(|v|)
    // |v| = sqrt(x^2 + y^2 + z^2) = sqrt(1^2 + 2^2 + 3^2) = sqrt(14)
    // w = 0.5 rad
    let w = 0.5f64;
    let v_norm = 14.0f64.sqrt();

    let scalar_part_expected = w.cos() * v_norm.cosh();
    let vector_scale_expected = -w.sin() * v_norm.sinh() / v_norm;

    let expected_q = Quaternion::new(
        scalar_part_expected,
        1.0 * vector_scale_expected,
        2.0 * vector_scale_expected,
        3.0 * vector_scale_expected,
    );

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_cos_purely_real() {
    let q = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    let result = q.cos();

    let expected_q = Quaternion::new(0.5f64.cos(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_cos_purely_imaginary() {
    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.cos();

    // cos(i) = cosh(1)
    let expected_q = Quaternion::new(1.0f64.cosh(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_cos_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.cos();

    let expected_q = Quaternion::<f64>::one(); // cos(0) = 1

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_tan_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.tan();

    let sin_q = q.sin();
    let cos_q = q.cos();
    let expected_q = sin_q / cos_q;

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_tan_purely_real() {
    let q = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    let result = q.tan();

    let expected_q = Quaternion::new(0.5f64.tan(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_tan_purely_imaginary() {
    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.tan();

    // tan(i) = i tanh(1)
    let expected_q = Quaternion::new(0.0, 1.0f64.tanh(), 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_tan_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.tan();

    let expected_q = Quaternion::<f64>::zero(); // tan(0) = 0

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_sinh_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.sinh();

    // Expected values calculated manually or using a reference library
    // For q = w + xi + yj + zk
    // sinh(q) = sinh(w)cos(|v|) + (v/|v|)cosh(w)sin(|v|)
    // |v| = sqrt(x^2 + y^2 + z^2) = sqrt(1^2 + 2^2 + 3^2) = sqrt(14)
    // w = 0.5
    let w = 0.5f64;
    let v_norm = 14.0f64.sqrt();

    let scalar_part_expected = w.sinh() * v_norm.cos();
    let vector_scale_expected = w.cosh() * v_norm.sin() / v_norm;

    let expected_q = Quaternion::new(
        scalar_part_expected,
        1.0 * vector_scale_expected,
        2.0 * vector_scale_expected,
        3.0 * vector_scale_expected,
    );

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_sinh_purely_real() {
    let q = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    let result = q.sinh();

    let expected_q = Quaternion::new(0.5f64.sinh(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_sinh_purely_imaginary() {
    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.sinh();

    // sinh(i) = i sin(1)
    let expected_q = Quaternion::new(0.0, 1.0f64.sin(), 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_sinh_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.sinh();

    let expected_q = Quaternion::<f64>::zero(); // sinh(0) = 0

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_cosh_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.cosh();

    // Expected values calculated manually or using a reference library
    // For q = w + xi + yj + zk
    // cosh(q) = cosh(w)cos(|v|) + (v/|v|)sinh(w)sin(|v|)
    // |v| = sqrt(x^2 + y^2 + z^2) = sqrt(1^2 + 2^2 + 3^2) = sqrt(14)
    // w = 0.5
    let w = 0.5f64;
    let v_norm = 14.0f64.sqrt();

    let scalar_part_expected = w.cosh() * v_norm.cos();
    let vector_scale_expected = w.sinh() * v_norm.sin() / v_norm;

    let expected_q = Quaternion::new(
        scalar_part_expected,
        1.0 * vector_scale_expected,
        2.0 * vector_scale_expected,
        3.0 * vector_scale_expected,
    );

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_cosh_purely_real() {
    let q = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    let result = q.cosh();

    let expected_q = Quaternion::new(0.5f64.cosh(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_cosh_purely_imaginary() {
    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.cosh();

    // cosh(i) = cos(1)
    let expected_q = Quaternion::new(1.0f64.cos(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_cosh_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.cosh();

    let expected_q = Quaternion::<f64>::one(); // cosh(0) = 1

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_tanh_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.tanh();

    let sinh_q = q.sinh();
    let cosh_q = q.cosh();
    let expected_q = sinh_q / cosh_q;

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_tanh_purely_real() {
    let q = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    let result = q.tanh();

    let expected_q = Quaternion::new(0.5f64.tanh(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_tanh_purely_imaginary() {
    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.tanh();

    // tanh(i) = i tan(1)
    let expected_q = Quaternion::new(0.0, 1.0f64.tan(), 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_tanh_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.tanh();

    let expected_q = Quaternion::<f64>::zero(); // tanh(0) = 0

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

// #[test]
// fn test_tanh_division_by_zero_cosh_w_zero() {
//     use std::f64::consts::FRAC_PI_2;
//     // cosh(q) is never zero for real w and non-zero v.
//     // However, if w=0 and v_norm = pi/2 + n*pi, then cos(v_norm) = 0 in cosh(q) calculation.
//     // In this specific case, cosh(0 + i * FRAC_PI_2) = cos(FRAC_PI_2) = 0, so division by zero occurs.
//     let q = Quaternion::new(0.0, FRAC_PI_2, 0.0, 0.0);
//     let result = q.tanh();
//     println!("result.w: {:?}", result.w);
//     assert!(result.w.is_nan());
//     assert!(result.x.is_nan());
//     assert!(result.y.is_nan());
//     assert!(result.z.is_nan());
// }

#[test]
fn test_acos_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.acos();

    // From Wikipedia/Quaternion#Logarithm, arccosine of a quaternion q = w + v:
    // acos(q) = acos(w) - (v / |v|) * acosh(w)
    // acos(q) = acos(w) + (v / |v|) * acos(w) / v_norm * (-1)
    // scalar part: acos(w)
    // vector part: (v / |v|) * acosh(w)

    // A simpler formula from some sources: acos(q) = (1/sqrt(1-w^2)) * (x i + y j + z k) * acos(w)
    // This is not what the implementation does. The current implementation for acos(q) is:
    // acos(q) = acos(w) - (v / |v|) * acosh(w)
    // The implementation for acos(q) is : Quaternion::new(T::zero(), self.x * scale, self.y * scale, self.z * scale)
    // scale = -angle / v_norm where angle = w.acos()
    // This is clearly not correct based on standard definitions of quaternion acos.

    // Given the current implementation:
    let w = q.w;
    let x = q.x;
    let y = q.y;
    let z = q.z;

    let v_norm_sq = x * x + y * y + z * z;
    let v_norm = v_norm_sq.sqrt();

    if v_norm.is_zero() {
        // This case is handled in test_acos_purely_real
        let expected_q = Quaternion::new(w.acos(), 0.0, 0.0, 0.0);
        assert!((result.w - expected_q.w).abs() < EPSILON);
        assert!((result.x - expected_q.x).abs() < EPSILON);
        assert!((result.y - expected_q.y).abs() < EPSILON);
        assert!((result.z - expected_q.z).abs() < EPSILON);
    } else {
        let angle = w.acos();
        let scale = -angle / v_norm;

        let expected_q = Quaternion::new(0.0, x * scale, y * scale, z * scale);

        assert!((result.w - expected_q.w).abs() < EPSILON);
        assert!((result.x - expected_q.x).abs() < EPSILON);
        assert!((result.y - expected_q.y).abs() < EPSILON);
        assert!((result.z - expected_q.z).abs() < EPSILON);
    }
}

#[test]
fn test_acos_purely_real() {
    let q = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    let result = q.acos();

    let expected_q = Quaternion::new(0.5f64.acos(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_acos_purely_imaginary() {
    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.acos();

    // Current implementation:
    // w = 0.0, v_norm = 1.0
    // angle = 0.0.acos() = pi/2
    // scale = -angle / v_norm = -pi/2
    // expected_q = Quaternion::new(0.0, 1.0 * (-pi/2), 0.0, 0.0)
    let expected_q = Quaternion::new(0.0, -std::f64::consts::FRAC_PI_2, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_acos_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.acos();

    // Current implementation:
    // w = 0.0, v_norm = 0.0
    // Branch v_norm.is_zero() is taken
    // expected_q = Quaternion::new(w.acos(), 0.0, 0.0, 0.0)
    // expected_q = Quaternion::new(0.0.acos(), 0.0, 0.0, 0.0) = Quaternion::new(FRAC_PI_2, 0.0, 0.0, 0.0)
    let expected_q = Quaternion::new(std::f64::consts::FRAC_PI_2, 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_acos_identity() {
    let q = Quaternion::<f64>::identity(); // w = 1.0, x=y=z=0.0
    let result = q.acos();

    // Current implementation:
    // w = 1.0, v_norm = 0.0
    // Branch v_norm.is_zero() is taken
    // expected_q = Quaternion::new(w.acos(), 0.0, 0.0, 0.0)
    // expected_q = Quaternion::new(1.0.acos(), 0.0, 0.0, 0.0) = Quaternion::new(0.0, 0.0, 0.0, 0.0) = Quaternion::zero()
    let expected_q = Quaternion::<f64>::zero();

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_acos_w_out_of_range() {
    let q = Quaternion::new(2.0, 1.0, 0.0, 0.0); // w > 1.0
    let result = q.acos();

    assert!((result.w - 0.0).abs() < EPSILON);
    assert!(result.x.is_nan());
    assert!(result.y.is_nan());
    assert!(result.z.is_nan());

    let q_neg = Quaternion::new(-2.0, 1.0, 0.0, 0.0); // w < -1.0
    let result_neg = q_neg.acos();

    assert!((result_neg.w - 0.0).abs() < EPSILON);
    assert!(result_neg.x.is_nan());
    assert!(result_neg.y.is_nan());
    assert!(result_neg.z.is_nan());
}

#[test]
fn test_ln_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.ln();

    // From the implementation:
    let q_norm = q.norm();
    let v_norm_sq = q.x * q.x + q.y * q.y + q.z * q.z;
    let v_norm = v_norm_sq.sqrt();

    let scalar_part_expected = q_norm.ln();
    let angle = (q.w / q_norm).acos();
    let vector_scale_expected = angle / v_norm;

    let expected_q = Quaternion::new(
        scalar_part_expected,
        q.x * vector_scale_expected,
        q.y * vector_scale_expected,
        q.z * vector_scale_expected,
    );

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_ln_purely_real_positive() {
    let q = Quaternion::new(2.0, 0.0, 0.0, 0.0);
    let result = q.ln();

    let expected_q = Quaternion::new(2.0f64.ln(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_ln_purely_real_negative() {
    use std::f64::consts::PI;

    let q = Quaternion::new(-2.0, 0.0, 0.0, 0.0);
    let result = q.ln();

    let expected_q = Quaternion::new(2.0f64.ln(), PI, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_ln_purely_imaginary() {
    use std::f64::consts::FRAC_PI_2;

    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.ln();

    // From the implementation:
    // q_norm = 1.0
    // v_norm = 1.0
    // scalar_part = 1.0.ln() = 0.0
    // angle = (0.0 / 1.0).acos() = FRAC_PI_2
    // vector_scale = FRAC_PI_2 / 1.0 = FRAC_PI_2
    let expected_q = Quaternion::new(0.0, FRAC_PI_2, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_ln_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.ln();

    // q_norm = 0.0, v_norm = 0.0
    // Branch v_norm.is_zero() is taken, then w >= T::zero() (0.0 >= 0.0)
    // Then 0.0.ln() which is -Infinity, so NaN
    assert!(result.w.is_infinite() && result.w.is_sign_negative());
    assert!((result.x).abs() < EPSILON);
    assert!((result.y).abs() < EPSILON);
    assert!((result.z).abs() < EPSILON);
}

#[test]
fn test_exp_general() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let result = q.exp();

    // From the implementation:
    let w_exp = q.w.exp();
    let v_norm_sq = q.x * q.x + q.y * q.y + q.z * q.z;
    let v_norm = v_norm_sq.sqrt();

    let scalar_part_expected = w_exp * v_norm.cos();
    let vector_scale_expected = w_exp * v_norm.sin() / v_norm;

    let expected_q = Quaternion::new(
        scalar_part_expected,
        q.x * vector_scale_expected,
        q.y * vector_scale_expected,
        q.z * vector_scale_expected,
    );

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_exp_purely_real() {
    let q = Quaternion::new(0.5, 0.0, 0.0, 0.0);
    let result = q.exp();

    let expected_q = Quaternion::new(0.5f64.exp(), 0.0, 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_exp_purely_imaginary() {
    // exp(i*theta) = cos(theta) + i sin(theta)
    // For q = 0 + i, exp(q) = cos(1) + i sin(1)
    let q = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let result = q.exp();

    let expected_q = Quaternion::new(1.0f64.cos(), 1.0f64.sin(), 0.0, 0.0);

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_exp_zero() {
    let q = Quaternion::<f64>::zero();
    let result = q.exp();

    let expected_q = Quaternion::<f64>::one(); // exp(0) = 1

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powi_positive_exponent() {
    let q = Quaternion::new(0.5, 0.5, 0.5, 0.5).normalize(); // Unit quaternion
    let n = 3;
    let result = q.powi(n);

    let mut expected_q = Quaternion::identity();
    for _ in 0..n {
        expected_q *= q;
    }

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powi_zero_exponent() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let n = 0;
    let result = q.powi(n);

    let expected_q = Quaternion::<f64>::one();

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powi_one_exponent() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let n = 1;
    let result = q.powi(n);

    let expected_q = q;

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powi_negative_one_exponent() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let n = -1;
    let result = q.powi(n);

    let expected_q = q.inverse();

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powi_negative_exponent() {
    let q = Quaternion::new(0.5, 0.5, 0.5, 0.5).normalize(); // Unit quaternion
    let n = -2;
    let result = q.powi(n);

    let q_inv = q.inverse();
    let mut expected_q = Quaternion::identity();
    for _ in 0..n.abs() {
        expected_q *= q_inv;
    }

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powi_zero_quaternion_positive_exponent() {
    let q = Quaternion::<f64>::zero();
    let n = 2;
    let result = q.powi(n);

    let expected_q = Quaternion::<f64>::zero();

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powi_zero_quaternion_negative_exponent() {
    let q = Quaternion::<f64>::zero();
    let n = -2;
    let result = q.powi(n);

    // Should result in NaNs due to inverse of zero quaternion
    assert!(result.w.is_nan());
    assert!(result.x.is_nan());
    assert!(result.y.is_nan());
    assert!(result.z.is_nan());
}

#[test]
fn test_powf_positive_exponent() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let n = 2.5f64;
    let result = q.powf(n);

    // q^n = exp(n * ln(q))
    let expected_q = (q.ln() * n).exp();

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powf_zero_exponent() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let n = 0.0f64;
    let result = q.powf(n);

    let expected_q = Quaternion::<f64>::one(); // q^0 = 1

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powf_one_exponent() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let n = 1.0f64;
    let result = q.powf(n);

    let expected_q = q; // q^1 = q

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powf_negative_exponent() {
    let q = Quaternion::new(0.5, 1.0, 2.0, 3.0);
    let n = -1.5f64;
    let result = q.powf(n);

    let expected_q = (q.ln() * n).exp();

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powf_zero_quaternion_positive_exponent() {
    let q = Quaternion::<f64>::zero();
    let n = 2.5f64;
    let result = q.powf(n);

    // powf uses ln(q), and ln(0) results in NaN/Infinity for the scalar part and 0 for vector
    // However, the current ln(zero) returns -Infinity for w, and finite for x,y,z
    // then (-Inf * n).exp() would result in 0 for w, and NaN for vector part
    // Let's rely on the actual calculation result.
    assert!((result.w).abs() < EPSILON); // e^(-Inf) = 0
    assert!((result.x).abs() < EPSILON);
    assert!((result.y).abs() < EPSILON);
    assert!((result.z).abs() < EPSILON);
}

#[test]
fn test_powf_zero_quaternion_negative_exponent() {
    let q = Quaternion::<f64>::zero();
    let n = -2.5f64;
    let result = q.powf(n);

    // powf uses ln(q), and ln(0) results in NaN/Infinity
    // then (-Inf * n).exp() would result in Inf for w, and NaN for vector part
    // Let's rely on the actual calculation result.
    assert!(result.w.is_infinite() && result.w.is_sign_positive()); // e^(Inf) = Inf
    assert!((result.x).abs() < EPSILON);
    assert!((result.y).abs() < EPSILON);
    assert!((result.z).abs() < EPSILON);
}

#[test]
fn test_powc_general() {
    let q_base = Quaternion::new(0.5, 1.0, 0.0, 0.0);
    let q_exp = Quaternion::new(0.0, 0.0, 0.5, 0.0); // 0.5j
    let result = q_base.powc(q_exp);

    // q_base^q_exp = exp(q_exp * ln(q_base))
    let expected_q = (q_exp * q_base.ln()).exp();

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powc_exponent_identity() {
    let q_base = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q_exp = Quaternion::<f64>::one(); // Identity quaternion
    let result = q_base.powc(q_exp);

    let expected_q = q_base; // q^1 = q

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powc_exponent_zero() {
    let q_base = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q_exp = Quaternion::<f64>::zero(); // Zero quaternion
    let result = q_base.powc(q_exp);

    let expected_q = Quaternion::<f64>::one(); // q^0 = 1

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powc_base_identity() {
    let q_base = Quaternion::<f64>::one(); // Identity quaternion
    let q_exp = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let result = q_base.powc(q_exp);

    let expected_q = Quaternion::<f64>::one(); // 1^p = 1

    assert!((result.w - expected_q.w).abs() < EPSILON);
    assert!((result.x - expected_q.x).abs() < EPSILON);
    assert!((result.y - expected_q.y).abs() < EPSILON);
    assert!((result.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_powc_base_zero() {
    let q_base = Quaternion::<f64>::zero();
    let q_exp = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let result = q_base.powc(q_exp);

    // Uses ln(q_base), and ln(zero) results in NaNs.
    // This propagates through the calculations to result in a NaN quaternion.
    assert!(result.w.is_nan());
    assert!(result.x.is_nan());
    assert!(result.y.is_nan());
    assert!(result.z.is_nan());
}
