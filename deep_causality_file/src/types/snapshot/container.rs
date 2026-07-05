/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Binary encoding and decoding of the snapshot container.
//!
//! Layout (all integers little-endian):
//!
//! ```text
//! magic:            8 bytes  b"DCFSNP01"
//! checksum:         u64      FNV-1a 64 over every byte after this field (the body)
//! body:
//!   format_version: u16      (currently 1; unknown versions are refused loudly)
//!   scalar_tag:     u8       (1 = f32, 2 = f64, 3 = Float106)
//!   tier:           u8       (1 = field snapshot, 2 = full resume)
//!   fingerprint:    u64      digest of the world-description bytes
//!   section_count:  u32
//!   per section:
//!     name_len:     u32, then name bytes (UTF-8)
//!     version:      u8       the section's own layout version
//!     data_len:     u64, then data bytes
//! ```

use crate::DataLoadingError;
use crate::types::snapshot::checksum::fnv1a64;
use crate::types::snapshot_types::{ScalarTypeTag, SnapshotPackage, SnapshotSection, SnapshotTier};

/// The container magic: identifies a DeepCausality file snapshot, format family 01.
pub(crate) const MAGIC: &[u8; 8] = b"DCFSNP01";
/// The container format version this build writes and understands.
pub(crate) const FORMAT_VERSION: u16 = 1;

/// The smallest number of body bytes any section can occupy: its fixed metadata, a `u32`
/// name length (4) + a `u8` version (1) + a `u64` data length (8), before any name or data
/// bytes. Used to reject an implausible section count before allocating for it.
const MIN_SECTION_BYTES: usize = 4 + 1 + 8;

/// Serialize a package to the container bytes, checksum included.
pub(crate) fn encode(package: &SnapshotPackage) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    body.push(package.scalar().to_byte());
    body.push(package.tier().to_byte());
    body.extend_from_slice(&package.fingerprint().to_le_bytes());
    let count = u32::try_from(package.sections().len()).unwrap_or(u32::MAX);
    body.extend_from_slice(&count.to_le_bytes());
    for section in package.sections() {
        let name = section.name().as_bytes();
        let name_len = u32::try_from(name.len()).unwrap_or(u32::MAX);
        body.extend_from_slice(&name_len.to_le_bytes());
        body.extend_from_slice(name);
        body.push(section.version());
        let data_len = u64::try_from(section.bytes().len()).unwrap_or(u64::MAX);
        body.extend_from_slice(&data_len.to_le_bytes());
        body.extend_from_slice(section.bytes());
    }

    let mut out = Vec::with_capacity(MAGIC.len() + 8 + body.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&fnv1a64(&body).to_le_bytes());
    out.extend_from_slice(&body);
    out
}

/// The outcome of decoding: the package plus whether the checksum matched. The caller decides
/// whether a mismatch is fatal (strict load) or a reported warning (`force_load`).
pub(crate) struct Decoded {
    pub package: SnapshotPackage,
    pub checksum_ok: bool,
}

fn read_u16(bytes: &[u8], offset: &mut usize) -> Option<u16> {
    let s = bytes.get(*offset..*offset + 2)?;
    *offset += 2;
    Some(u16::from_le_bytes([s[0], s[1]]))
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
    let s = bytes.get(*offset..*offset + 4)?;
    *offset += 4;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn read_u64(bytes: &[u8], offset: &mut usize) -> Option<u64> {
    let s = bytes.get(*offset..*offset + 8)?;
    *offset += 8;
    Some(u64::from_le_bytes([
        s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7],
    ]))
}

fn read_u8(bytes: &[u8], offset: &mut usize) -> Option<u8> {
    let b = *bytes.get(*offset)?;
    *offset += 1;
    Some(b)
}

/// Decode container bytes. Structural damage (bad magic, truncation, invalid tags) is always
/// a corrupt-file error; the checksum verdict is returned for the caller's policy. The format
/// version is checked here and an unknown version is refused regardless of load mode.
///
/// When `checksum_fatal` is set (the strict load path), a checksum mismatch refuses immediately,
/// before any section is interpreted or allocated for — so a corrupt file cannot drive the
/// section parse the strict path means to avoid. The force-load path passes `false` so it can
/// still parse and salvage, downgrading the mismatch to a reported warning.
pub(crate) fn decode(
    bytes: &[u8],
    shown_path: &str,
    checksum_fatal: bool,
) -> Result<Decoded, DataLoadingError> {
    let corrupt = |detail: &str| DataLoadingError::corrupt(shown_path, detail);

    if bytes.len() < MAGIC.len() + 8 {
        return Err(corrupt("file shorter than the snapshot header"));
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return Err(corrupt("bad magic: not a snapshot file"));
    }
    let mut offset = MAGIC.len();
    let stored_checksum =
        read_u64(bytes, &mut offset).ok_or_else(|| corrupt("truncated checksum"))?;
    let body = &bytes[offset..];
    let checksum_ok = fnv1a64(body) == stored_checksum;
    if checksum_fatal && !checksum_ok {
        return Err(corrupt(
            "checksum mismatch: the file content does not match its recorded checksum",
        ));
    }

    let mut o = 0usize;
    let version = read_u16(body, &mut o).ok_or_else(|| corrupt("truncated version"))?;
    if version != FORMAT_VERSION {
        return Err(DataLoadingError::unknown_version(shown_path, version));
    }
    let scalar_byte = read_u8(body, &mut o).ok_or_else(|| corrupt("truncated scalar tag"))?;
    let scalar = ScalarTypeTag::from_byte(scalar_byte)
        .ok_or_else(|| corrupt(&format!("invalid scalar tag {scalar_byte}")))?;
    let tier_byte = read_u8(body, &mut o).ok_or_else(|| corrupt("truncated tier tag"))?;
    let tier = SnapshotTier::from_byte(tier_byte)
        .ok_or_else(|| corrupt(&format!("invalid tier tag {tier_byte}")))?;
    let fingerprint = read_u64(body, &mut o).ok_or_else(|| corrupt("truncated fingerprint"))?;
    let count = read_u32(body, &mut o).ok_or_else(|| corrupt("truncated section count"))?;

    // Reject an implausible section count before allocating for it: each section occupies at
    // least its fixed metadata (`MIN_SECTION_BYTES`) in the remaining body, so a count that
    // cannot possibly fit is a malformed header, not a huge-but-valid file.
    let count = count as usize;
    let remaining = body.len() - o;
    if count > remaining / MIN_SECTION_BYTES {
        return Err(corrupt(&format!(
            "section count {count} exceeds what the remaining {remaining} body bytes can hold"
        )));
    }

    let mut sections = Vec::with_capacity(count);
    for i in 0..count {
        let name_len = read_u32(body, &mut o)
            .ok_or_else(|| corrupt(&format!("section {i}: name len")))?
            as usize;
        let name_bytes = body
            .get(o..o + name_len)
            .ok_or_else(|| corrupt(&format!("section {i}: truncated name")))?;
        o += name_len;
        let name = std::str::from_utf8(name_bytes)
            .map_err(|_| corrupt(&format!("section {i}: name is not UTF-8")))?
            .to_string();
        let sec_version =
            read_u8(body, &mut o).ok_or_else(|| corrupt(&format!("section {i}: version")))?;
        let data_len = read_u64(body, &mut o)
            .ok_or_else(|| corrupt(&format!("section {i}: data len")))?
            as usize;
        let data = body
            .get(o..o + data_len)
            .ok_or_else(|| corrupt(&format!("section {i}: truncated data")))?
            .to_vec();
        o += data_len;
        sections.push(SnapshotSection::new(name, sec_version, data));
    }
    if o != body.len() {
        return Err(corrupt("trailing bytes after the last section"));
    }

    Ok(Decoded {
        package: SnapshotPackage::new(scalar, tier, fingerprint, sections),
        checksum_ok,
    })
}
