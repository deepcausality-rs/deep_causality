/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The snapshot container: round trips, the checksum catching corruption before content is
//! interpreted, force-load semantics, and the refusal rules (scalar, version, fingerprint).

use deep_causality_file::{
    ScalarTypeTag, SnapshotPackage, SnapshotSection, SnapshotTier, fingerprint64,
    force_load_snapshot, load_snapshot, save_snapshot,
};
use deep_causality_haft::IoAction;
use std::fs;

fn sample_package(fingerprint: u64) -> SnapshotPackage {
    SnapshotPackage::new(
        ScalarTypeTag::F64,
        SnapshotTier::Resume,
        fingerprint,
        vec![
            SnapshotSection::new("fields", 1, vec![1, 2, 3, 4]),
            SnapshotSection::new("nav", 2, vec![9, 9]),
            SnapshotSection::new("log", 1, Vec::new()),
        ],
    )
}

fn save_to_temp(package: SnapshotPackage) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("state.dcsnap");
    save_snapshot(&path, package).run().expect("saves");
    (dir, path)
}

#[test]
fn a_package_round_trips_with_sections_in_order() {
    let fp = fingerprint64(b"world-v1");
    let (_d, path) = save_to_temp(sample_package(fp));
    let back = load_snapshot(&path, ScalarTypeTag::F64, Some(fp))
        .run()
        .expect("loads");
    assert_eq!(back.tier(), SnapshotTier::Resume);
    assert_eq!(back.fingerprint(), fp);
    assert_eq!(back.sections().len(), 3);
    assert_eq!(back.sections()[0].name(), "fields");
    assert_eq!(back.sections()[0].bytes(), &[1, 2, 3, 4]);
    assert_eq!(back.section("nav").expect("nav").version(), 2);
    assert!(back.section("log").expect("log").bytes().is_empty());
}

#[test]
fn a_flipped_byte_is_reported_as_corrupt() {
    let fp = fingerprint64(b"world-v1");
    let (_d, path) = save_to_temp(sample_package(fp));
    let mut bytes = fs::read(&path).expect("read");
    // Flip a fingerprint byte (body offset 4..12, file offset 20..28): content damage that
    // keeps the container structurally parseable, so the checksum is what catches it.
    bytes[20] ^= 0xFF;
    fs::write(&path, &bytes).expect("rewrite");

    let err = load_snapshot(&path, ScalarTypeTag::F64, Some(fp))
        .run()
        .expect_err("corrupt");
    let msg = err.to_string();
    assert!(msg.contains("corrupt"), "{msg}");
    assert!(msg.contains("state.dcsnap"), "names the file: {msg}");
}

#[test]
fn force_load_reports_the_corruption_and_proceeds() {
    let fp = fingerprint64(b"world-v1");
    let (_d, path) = save_to_temp(sample_package(fp));
    let mut bytes = fs::read(&path).expect("read");
    // Same content-level damage as the strict test: a flipped fingerprint byte.
    bytes[20] ^= 0xFF;
    fs::write(&path, &bytes).expect("rewrite");

    let (package, warnings) = force_load_snapshot(&path, ScalarTypeTag::F64, Some(fp))
        .run()
        .expect("force load proceeds");
    assert_eq!(package.sections().len(), 3);
    assert!(
        warnings.iter().any(|w| w.contains("checksum")),
        "{warnings:?}"
    );
}

#[test]
fn a_scalar_mismatch_refuses_even_under_force_load() {
    let fp = fingerprint64(b"world-v1");
    let (_d, path) = save_to_temp(sample_package(fp));
    let err = load_snapshot(&path, ScalarTypeTag::Float106, Some(fp))
        .run()
        .expect_err("scalar mismatch");
    assert!(err.to_string().contains("Float106"), "{err}");

    let err = force_load_snapshot(&path, ScalarTypeTag::Float106, Some(fp))
        .run()
        .expect_err("force load never overrides the scalar");
    assert!(err.to_string().contains("f64"), "{err}");
}

#[test]
fn a_stale_fingerprint_is_refused_and_force_load_reports_it() {
    let saved_fp = fingerprint64(b"world-v1");
    let edited_fp = fingerprint64(b"world-v2-edited-constants");
    let (_d, path) = save_to_temp(sample_package(saved_fp));

    let err = load_snapshot(&path, ScalarTypeTag::F64, Some(edited_fp))
        .run()
        .expect_err("stale world");
    assert!(err.to_string().contains("different world"), "{err}");

    let (_package, warnings) = force_load_snapshot(&path, ScalarTypeTag::F64, Some(edited_fp))
        .run()
        .expect("force load proceeds");
    assert!(warnings.iter().any(|w| w.contains("fingerprint")));
}

#[test]
fn an_unknown_format_version_is_refused_loudly() {
    let fp = fingerprint64(b"world-v1");
    let (_d, path) = save_to_temp(sample_package(fp));
    let mut bytes = fs::read(&path).expect("read");
    // The format version is the first body field: bytes 16..18 (after magic and checksum).
    bytes[16] = 0xFF;
    bytes[17] = 0xFF;
    fs::write(&path, &bytes).expect("rewrite");

    // Version refusal beats the (now broken) checksum in both modes.
    let err = force_load_snapshot(&path, ScalarTypeTag::F64, Some(fp))
        .run()
        .expect_err("unknown version");
    assert!(err.to_string().contains("unknown format version"), "{err}");
}

#[test]
fn an_empty_package_and_inspection_mode_work() {
    let fp = fingerprint64(b"anything");
    let (_d, path) = save_to_temp(SnapshotPackage::new(
        ScalarTypeTag::F32,
        SnapshotTier::Field,
        fp,
        Vec::new(),
    ));
    // None skips world validation: the inspection-tool mode.
    let back = load_snapshot(&path, ScalarTypeTag::F32, None)
        .run()
        .expect("loads");
    assert_eq!(back.tier(), SnapshotTier::Field);
    assert!(back.sections().is_empty());
}

#[test]
fn a_non_snapshot_file_is_corrupt_not_a_panic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("not_a_snapshot.bin");
    fs::write(&path, b"just some text, definitely not a snapshot").expect("write");
    let err = load_snapshot(&path, ScalarTypeTag::F64, None)
        .run()
        .expect_err("bad magic");
    assert!(err.to_string().contains("not a snapshot"), "{err}");
}
