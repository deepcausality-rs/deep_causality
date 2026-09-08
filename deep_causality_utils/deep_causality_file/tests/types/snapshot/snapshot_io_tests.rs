/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The snapshot container: round trips, the checksum catching corruption before content is
//! interpreted, force-load semantics, and the refusal rules (scalar, version, fingerprint).

use deep_causality_file::{
    ScalarTypeTag, SnapshotPackage, SnapshotSection, SnapshotTier, fingerprint64, fnv1a64,
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

/// Byte offsets into the container: 8 magic + 8 checksum, then the body's fixed header.
const SCALAR_TAG_OFFSET: usize = 18;
const TIER_TAG_OFFSET: usize = 19;
const SECTION_COUNT_OFFSET: usize = 28;

/// Save `sample_package`, hand its bytes to `damage`, and force-load the result. Force load is
/// what reaches the structural checks: it treats the checksum the edit invalidates as a warning
/// rather than refusing before the body is parsed.
fn force_load_damaged(
    damage: impl FnOnce(&mut Vec<u8>),
) -> (tempfile::TempDir, deep_causality_file::DataLoadingError) {
    let fp = fingerprint64(b"world-v1");
    let (dir, path) = save_to_temp(sample_package(fp));
    let mut bytes = fs::read(&path).expect("read");
    damage(&mut bytes);
    fs::write(&path, &bytes).expect("rewrite");
    let err = force_load_snapshot(&path, ScalarTypeTag::F64, None)
        .run()
        .expect_err("damaged container");
    (dir, err)
}

#[test]
fn a_file_shorter_than_the_header_is_corrupt() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("stub.dcsnap");
    // The magic alone: 8 bytes, short of the 16 the magic-plus-checksum header needs.
    fs::write(&path, b"DCFSNP01").expect("write");
    let err = load_snapshot(&path, ScalarTypeTag::F64, None)
        .run()
        .expect_err("truncated header");
    assert!(
        err.to_string().contains("shorter than the snapshot header"),
        "{err}"
    );
}

#[test]
fn an_implausible_section_count_is_refused_before_allocating() {
    // A count no remaining body could hold is a malformed header, and it must be rejected on
    // arithmetic rather than by attempting the allocation it asks for.
    let (_d, err) = force_load_damaged(|bytes| {
        bytes[SECTION_COUNT_OFFSET..SECTION_COUNT_OFFSET + 4]
            .copy_from_slice(&u32::MAX.to_le_bytes());
    });
    let msg = err.to_string();
    assert!(msg.contains("section count 4294967295"), "{msg}");
    assert!(msg.contains("body bytes can hold"), "{msg}");
}

#[test]
fn trailing_bytes_after_the_last_section_are_corrupt() {
    // Every body byte must be accounted for by a section; a tail that no section claims means
    // the file is not what its own header describes.
    let (_d, err) = force_load_damaged(|bytes| bytes.extend_from_slice(b"tail"));
    assert!(
        err.to_string()
            .contains("trailing bytes after the last section"),
        "{err}"
    );
}

#[test]
fn an_invalid_scalar_tag_is_corrupt() {
    let (_d, err) = force_load_damaged(|bytes| bytes[SCALAR_TAG_OFFSET] = 0x7F);
    assert!(err.to_string().contains("invalid scalar tag 127"), "{err}");
}

#[test]
fn an_invalid_tier_tag_is_corrupt() {
    let (_d, err) = force_load_damaged(|bytes| bytes[TIER_TAG_OFFSET] = 0x7F);
    assert!(err.to_string().contains("invalid tier tag 127"), "{err}");
}

#[test]
fn a_float106_package_round_trips() {
    // The third scalar tag: written as 3 and read back as Float106.
    let fp = fingerprint64(b"world-v1");
    let (_d, path) = save_to_temp(SnapshotPackage::new(
        ScalarTypeTag::Float106,
        SnapshotTier::Field,
        fp,
        vec![SnapshotSection::new("fields", 1, vec![7, 7])],
    ));
    assert_eq!(fs::read(&path).expect("read")[SCALAR_TAG_OFFSET], 3);

    let back = load_snapshot(&path, ScalarTypeTag::Float106, Some(fp))
        .run()
        .expect("loads");
    assert_eq!(back.scalar(), ScalarTypeTag::Float106);
    assert_eq!(back.section("fields").expect("fields").bytes(), &[7, 7]);
}

#[test]
fn a_scalar_mismatch_names_the_expected_and_the_found_scalar() {
    let fp = fingerprint64(b"world-v1");
    let (_d, path) = save_to_temp(SnapshotPackage::new(
        ScalarTypeTag::F32,
        SnapshotTier::Field,
        fp,
        Vec::new(),
    ));
    let err = load_snapshot(&path, ScalarTypeTag::F64, Some(fp))
        .run()
        .expect_err("scalar mismatch");
    // Both scalars appear, so only their roles distinguish a correct message from one that has
    // them the wrong way round.
    assert_eq!(
        err.to_string(),
        format!(
            "data loading: snapshot {} was saved at scalar f32, this program runs at f64",
            path.display()
        )
    );
}

#[test]
fn the_digest_matches_the_published_fnv_1a_64_vectors() {
    // Independent oracle: the reference FNV-1a-64 values, not this implementation's own output.
    // Every other observation of the digest compares it against itself.
    assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    // The fingerprint is that digest over the caller's world bytes, not a second hash.
    assert_eq!(fingerprint64(b"foobar"), fnv1a64(b"foobar"));
}

#[test]
fn the_saved_bytes_match_the_documented_container_layout() {
    // `encode` is otherwise only ever checked against `decode`, the matching half of the same
    // module: a magic, an endianness or a field order wrong in both directions round trips.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("layout.dcsnap");
    save_snapshot(
        &path,
        SnapshotPackage::new(
            ScalarTypeTag::F64,
            SnapshotTier::Resume,
            0x0102_0304_0506_0708,
            vec![SnapshotSection::new("ab", 7, vec![0xAA, 0xBB])],
        ),
    )
    .run()
    .expect("saves");
    let bytes = fs::read(&path).expect("read");

    assert_eq!(&bytes[..8], b"DCFSNP01", "magic");
    let body = &bytes[16..];
    assert_eq!(
        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        fnv1a64(body),
        "the checksum covers exactly the body"
    );
    assert_eq!(&body[0..2], &[1, 0], "format version 1, little-endian u16");
    assert_eq!(body[2], 2, "scalar tag f64");
    assert_eq!(body[3], 2, "tier tag resume");
    assert_eq!(
        &body[4..12],
        &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
        "fingerprint, little-endian u64"
    );
    assert_eq!(
        &body[12..16],
        &[1, 0, 0, 0],
        "section count, little-endian u32"
    );
    assert_eq!(&body[16..20], &[2, 0, 0, 0], "name length");
    assert_eq!(&body[20..22], b"ab", "name bytes");
    assert_eq!(body[22], 7, "section version");
    assert_eq!(
        &body[23..31],
        &[2, 0, 0, 0, 0, 0, 0, 0],
        "data length, little-endian u64"
    );
    assert_eq!(&body[31..33], &[0xAA, 0xBB], "data bytes");
    assert_eq!(body.len(), 33, "no padding and no trailing bytes");
}

/// A container whose fixed 16-byte body header is followed by `filler` arbitrary bytes and which
/// declares `count` sections. The checksum is written correctly so only the declared count is
/// under test.
fn container_with_declared_count(count: u32, filler: usize) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(&1u16.to_le_bytes()); // format version
    body.push(2); // scalar tag: f64
    body.push(1); // tier tag: field
    body.extend_from_slice(&0u64.to_le_bytes()); // fingerprint
    body.extend_from_slice(&count.to_le_bytes());
    body.resize(16 + filler, 0);

    let mut out = Vec::new();
    out.extend_from_slice(b"DCFSNP01");
    out.extend_from_slice(&fnv1a64(&body).to_le_bytes());
    out.extend_from_slice(&body);
    out
}

fn load_bytes(bytes: &[u8]) -> Result<SnapshotPackage, deep_causality_file::DataLoadingError> {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("crafted.dcsnap");
    fs::write(&path, bytes).expect("write");
    load_snapshot(&path, ScalarTypeTag::F64, None).run()
}

#[test]
fn the_section_count_bound_is_exactly_the_bytes_each_section_needs() {
    // A section costs at least 4 + 1 + 8 = 13 body bytes of metadata. 24 filler bytes therefore
    // admit one section, not two, and 26 admit two. Pinning both sides fixes the constant, the
    // subtraction that computes the remaining bytes, and the division that turns them into a
    // count — none of which the u32::MAX case above can distinguish.
    let err = load_bytes(&container_with_declared_count(2, 24)).expect_err("2 will not fit in 24");
    assert!(
        err.to_string()
            .contains("section count 2 exceeds what the remaining 24 body bytes can hold"),
        "{err}"
    );

    // 26 bytes are exactly two sections' metadata, and zeroed metadata describes two empty,
    // unnamed sections — so at the bound the count is accepted and the body decodes completely.
    let package = load_bytes(&container_with_declared_count(2, 26)).expect("2 fit in 26");
    assert_eq!(package.sections().len(), 2);
    assert!(package.sections().iter().all(|s| s.bytes().is_empty()));
}

#[test]
fn a_sixteen_byte_file_clears_the_header_check_and_fails_on_its_empty_body() {
    // The length gate is `< magic + checksum`, so exactly 16 bytes is enough header and the
    // refusal must come from the empty body behind it, not from the gate.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("header_only.dcsnap");
    let mut bytes = b"DCFSNP01".to_vec();
    bytes.extend_from_slice(&fnv1a64(&[]).to_le_bytes());
    assert_eq!(bytes.len(), 16);
    fs::write(&path, &bytes).expect("write");

    let err = load_snapshot(&path, ScalarTypeTag::F64, None)
        .run()
        .expect_err("empty body");
    assert!(err.to_string().contains("truncated version"), "{err}");
}

#[test]
fn a_strict_load_refuses_on_the_checksum_before_it_reads_any_tag() {
    // The documented ordering: a corrupt file must not drive the tag and section parse at all.
    // Damage the checksum and the scalar tag together — the strict refusal names the checksum,
    // while the force load, which does parse, reaches and names the invalid tag.
    let fp = fingerprint64(b"world-v1");
    let (_d, path) = save_to_temp(sample_package(fp));
    let mut bytes = fs::read(&path).expect("read");
    bytes[SCALAR_TAG_OFFSET] = 0x7F;
    fs::write(&path, &bytes).expect("rewrite");

    let strict = load_snapshot(&path, ScalarTypeTag::F64, None)
        .run()
        .expect_err("checksum first");
    assert!(strict.to_string().contains("checksum mismatch"), "{strict}");

    let forced = force_load_snapshot(&path, ScalarTypeTag::F64, None)
        .run()
        .expect_err("parses, then finds the tag");
    assert!(
        forced.to_string().contains("invalid scalar tag"),
        "{forced}"
    );
}

#[test]
fn the_tier_tag_is_written_as_the_documented_byte() {
    // The tier byte is otherwise only ever exercised as a to_byte/from_byte inverse pair, which
    // agrees for any consistent pair of wrong mappings.
    for (tier, byte) in [(SnapshotTier::Field, 1u8), (SnapshotTier::Resume, 2)] {
        let (_d, path) = save_to_temp(SnapshotPackage::new(
            ScalarTypeTag::F64,
            tier,
            0,
            Vec::new(),
        ));
        assert_eq!(fs::read(&path).expect("read")[TIER_TAG_OFFSET], byte);
        let back = load_snapshot(&path, ScalarTypeTag::F64, None)
            .run()
            .expect("loads");
        assert_eq!(back.tier(), tier);
    }
}
