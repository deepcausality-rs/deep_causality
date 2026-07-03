/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The bit codec and the snapshot value types: raw bit patterns round-trip at every scalar.

use deep_causality_file::{
    BitCodec, ScalarTypeTag, SnapshotPackage, SnapshotSection, SnapshotTier,
};
use deep_causality_num::Float106;

#[test]
fn f64_bits_round_trip_including_awkward_values() {
    let values = [0.1_f64, -0.0, 1e-308, f64::MAX, 2.0 / 3.0];
    let mut buf = Vec::new();
    for v in &values {
        v.write_bits(&mut buf);
    }
    let mut offset = 0;
    for v in &values {
        let back = f64::read_bits(&buf, &mut offset).expect("reads");
        assert_eq!(back.to_bits(), v.to_bits());
    }
    assert_eq!(offset, buf.len());
    assert_eq!(<f64 as BitCodec>::SCALAR_TAG, ScalarTypeTag::F64);
}

#[test]
fn f32_bits_round_trip() {
    let values = [0.1_f32, -0.0, f32::MIN_POSITIVE];
    let mut buf = Vec::new();
    for v in &values {
        v.write_bits(&mut buf);
    }
    let mut offset = 0;
    for v in &values {
        let back = f32::read_bits(&buf, &mut offset).expect("reads");
        assert_eq!(back.to_bits(), v.to_bits());
    }
    assert_eq!(<f32 as BitCodec>::SCALAR_TAG, ScalarTypeTag::F32);
}

#[test]
fn float106_components_round_trip_bit_exact() {
    // A value whose lo component is nonzero: 0.1 + a tiny correction.
    let v = Float106::new(0.1, 1e-18);
    let mut buf = Vec::new();
    v.write_bits(&mut buf);
    let mut offset = 0;
    let back = Float106::read_bits(&buf, &mut offset).expect("reads");
    assert_eq!(back.hi().to_bits(), v.hi().to_bits());
    assert_eq!(back.lo().to_bits(), v.lo().to_bits());
    assert_eq!(<Float106 as BitCodec>::SCALAR_TAG, ScalarTypeTag::Float106);
}

#[test]
fn truncated_bytes_read_as_none_not_panic() {
    let mut buf = Vec::new();
    1.5_f64.write_bits(&mut buf);
    buf.truncate(5);
    let mut offset = 0;
    assert!(f64::read_bits(&buf, &mut offset).is_none());
}

#[test]
fn package_accessors_expose_what_was_stored() {
    let package = SnapshotPackage::new(
        ScalarTypeTag::F64,
        SnapshotTier::Field,
        42,
        vec![SnapshotSection::new("grid", 3, vec![7])],
    );
    assert_eq!(package.scalar(), ScalarTypeTag::F64);
    assert_eq!(package.tier(), SnapshotTier::Field);
    assert_eq!(package.fingerprint(), 42);
    assert_eq!(package.section("grid").expect("grid").version(), 3);
    assert!(package.section("absent").is_none());
}
