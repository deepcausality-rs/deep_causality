/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_fft::FftError;

#[test]
fn test_invalid_length_display() {
    let e = FftError::InvalidLength(0);
    assert_eq!(format!("{e}"), "FftError: invalid transform length 0");
}

#[test]
fn test_length_mismatch_display() {
    let e = FftError::LengthMismatch {
        expected: 8,
        got: 4,
    };
    assert_eq!(
        format!("{e}"),
        "FftError: buffer length mismatch, expected 8, got 4"
    );
}

#[test]
fn test_scratch_too_small_display() {
    let e = FftError::ScratchTooSmall {
        required: 64,
        got: 0,
    };
    assert_eq!(
        format!("{e}"),
        "FftError: scratch buffer too small, required 64, got 0"
    );
}

#[test]
fn test_eq_and_clone() {
    let e = FftError::InvalidLength(0);
    assert_eq!(e.clone(), e);
    assert_ne!(
        e,
        FftError::LengthMismatch {
            expected: 1,
            got: 2
        }
    );
}

#[test]
fn test_debug_and_error_trait() {
    let e = FftError::InvalidLength(3);
    assert!(format!("{e:?}").contains("InvalidLength"));
    let dyn_err: &dyn core::error::Error = &e;
    assert!(dyn_err.to_string().contains("invalid transform length"));
}
