/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The snapshot package: a versioned, checksummed binary container of named sections.
//!
//! The container knows nothing about CFD. It stores named byte blobs under a header that
//! records the format version, the working scalar the values were encoded at, the snapshot
//! tier, and a digest of the world description the state belongs to. The CFD crate packs and
//! unpacks its own state into the sections; this crate owns the format, the checksum, and the
//! refusal rules.
//!
//! Every scalar value inside a section is stored as its raw bit pattern (see [`BitCodec`]),
//! which is what makes a suspended-and-resumed run bit-identical to one that never stopped.

use deep_causality_num::Float106;

/// The working scalar a snapshot's values were encoded at. The tag is authoritative: loading
/// a package into a program whose scalar differs is refused with no override, because a wrong
/// scalar cannot be reinterpreted, only refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarTypeTag {
    /// 32-bit IEEE 754, stored as one little-endian `u32` per value.
    F32,
    /// 64-bit IEEE 754, stored as one little-endian `u64` per value.
    F64,
    /// Double-double, stored as two little-endian `u64` bit patterns (hi, lo) per value.
    Float106,
}

impl ScalarTypeTag {
    pub(crate) fn to_byte(self) -> u8 {
        match self {
            ScalarTypeTag::F32 => 1,
            ScalarTypeTag::F64 => 2,
            ScalarTypeTag::Float106 => 3,
        }
    }

    pub(crate) fn from_byte(b: u8) -> Option<Self> {
        match b {
            1 => Some(ScalarTypeTag::F32),
            2 => Some(ScalarTypeTag::F64),
            3 => Some(ScalarTypeTag::Float106),
            _ => None,
        }
    }

    /// Human-readable name, used in refusal messages.
    pub fn name(self) -> &'static str {
        match self {
            ScalarTypeTag::F32 => "f32",
            ScalarTypeTag::F64 => "f64",
            ScalarTypeTag::Float106 => "Float106",
        }
    }
}

/// The two snapshot tiers: a field snapshot (tensor fields plus grid metadata, the
/// area-of-interest artifact) and a full resume package (the field snapshot plus the state's
/// passengers: carried scalars, navigation engine, provenance log, step index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotTier {
    /// Named tensor-train fields and grid metadata only.
    Field,
    /// The full state a different workflow needs to continue the run.
    Resume,
}

impl SnapshotTier {
    pub(crate) fn to_byte(self) -> u8 {
        match self {
            SnapshotTier::Field => 1,
            SnapshotTier::Resume => 2,
        }
    }

    pub(crate) fn from_byte(b: u8) -> Option<Self> {
        match b {
            1 => Some(SnapshotTier::Field),
            2 => Some(SnapshotTier::Resume),
            _ => None,
        }
    }
}

/// One named section: an opaque byte blob with its own version byte, so a single section's
/// layout can evolve without bumping the container format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotSection {
    name: String,
    version: u8,
    bytes: Vec<u8>,
}

impl SnapshotSection {
    /// A named, versioned blob.
    pub fn new(name: impl Into<String>, version: u8, bytes: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            version,
            bytes,
        }
    }

    /// The section name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The section's own layout version.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// The payload bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// A snapshot package in memory: what the saver serializes and the loader returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotPackage {
    scalar: ScalarTypeTag,
    tier: SnapshotTier,
    fingerprint: u64,
    sections: Vec<SnapshotSection>,
}

impl SnapshotPackage {
    /// Assemble a package. `fingerprint` is the digest of the world-description bytes the
    /// state belongs to (see [`fingerprint64`](crate::fingerprint64)).
    pub fn new(
        scalar: ScalarTypeTag,
        tier: SnapshotTier,
        fingerprint: u64,
        sections: Vec<SnapshotSection>,
    ) -> Self {
        Self {
            scalar,
            tier,
            fingerprint,
            sections,
        }
    }

    /// The scalar the values were encoded at.
    pub fn scalar(&self) -> ScalarTypeTag {
        self.scalar
    }

    /// The snapshot tier.
    pub fn tier(&self) -> SnapshotTier {
        self.tier
    }

    /// The stored world-description digest.
    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    /// The named sections, in save order.
    pub fn sections(&self) -> &[SnapshotSection] {
        &self.sections
    }

    /// A section by name, when present.
    pub fn section(&self, name: &str) -> Option<&SnapshotSection> {
        self.sections.iter().find(|s| s.name() == name)
    }
}

/// Bit-exact encoding of a working scalar into a section payload. Values are raw IEEE bit
/// patterns, little-endian, so encoding and decoding change no bits at any precision.
pub trait BitCodec: Sized {
    /// The tag a package of this scalar carries in its header.
    const SCALAR_TAG: ScalarTypeTag;

    /// Append this value's bit pattern to `out`.
    fn write_bits(&self, out: &mut Vec<u8>);

    /// Read one value's bit pattern at `*offset`, advancing it. `None` when `bytes` is short.
    fn read_bits(bytes: &[u8], offset: &mut usize) -> Option<Self>;
}

fn take<const N: usize>(bytes: &[u8], offset: &mut usize) -> Option<[u8; N]> {
    let end = offset.checked_add(N)?;
    let slice = bytes.get(*offset..end)?;
    *offset = end;
    let mut buf = [0u8; N];
    buf.copy_from_slice(slice);
    Some(buf)
}

impl BitCodec for f32 {
    const SCALAR_TAG: ScalarTypeTag = ScalarTypeTag::F32;

    fn write_bits(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_bits().to_le_bytes());
    }

    fn read_bits(bytes: &[u8], offset: &mut usize) -> Option<Self> {
        take::<4>(bytes, offset).map(|b| f32::from_bits(u32::from_le_bytes(b)))
    }
}

impl BitCodec for f64 {
    const SCALAR_TAG: ScalarTypeTag = ScalarTypeTag::F64;

    fn write_bits(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_bits().to_le_bytes());
    }

    fn read_bits(bytes: &[u8], offset: &mut usize) -> Option<Self> {
        take::<8>(bytes, offset).map(|b| f64::from_bits(u64::from_le_bytes(b)))
    }
}

impl BitCodec for Float106 {
    const SCALAR_TAG: ScalarTypeTag = ScalarTypeTag::Float106;

    fn write_bits(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.hi().to_bits().to_le_bytes());
        out.extend_from_slice(&self.lo().to_bits().to_le_bytes());
    }

    fn read_bits(bytes: &[u8], offset: &mut usize) -> Option<Self> {
        let hi = take::<8>(bytes, offset).map(|b| f64::from_bits(u64::from_le_bytes(b)))?;
        let lo = take::<8>(bytes, offset).map(|b| f64::from_bits(u64::from_le_bytes(b)))?;
        Some(Float106::new(hi, lo))
    }
}
