/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Snapshot save and load as lazy [`IoAction`]s, with the refusal rules of the spec.
//!
//! Strict load verifies, in order: the checksum (before any section is interpreted), the
//! format version, the scalar tag, and the world fingerprint. [`force_load_snapshot`] skips
//! the checksum and fingerprint refusals but reports every mismatch as a warning, so the user
//! is always informed; it never skips the scalar check, because a wrong scalar cannot be
//! reinterpreted, only refused.

use crate::DataLoadingError;
use crate::snapshot::container::{decode, encode};
use crate::types::snapshot_types::{ScalarTypeTag, SnapshotPackage};
use deep_causality_haft::IoAction;
use std::fs;
use std::path::{Path, PathBuf};

/// A lazy IO description that, when [`run`](IoAction::run), writes `package` to `path` with
/// the whole-package checksum computed at save time. Construct with [`save_snapshot`].
pub struct SaveSnapshot {
    path: PathBuf,
    package: SnapshotPackage,
}

impl IoAction for SaveSnapshot {
    type Output = ();
    type Error = DataLoadingError;

    fn run(self) -> Result<(), DataLoadingError> {
        fs::write(&self.path, encode(&self.package))?;
        Ok(())
    }
}

/// Describe (but do not perform) saving `package` to `path`.
pub fn save_snapshot(path: impl AsRef<Path>, package: SnapshotPackage) -> SaveSnapshot {
    SaveSnapshot {
        path: path.as_ref().to_path_buf(),
        package,
    }
}

/// A lazy IO description that, when [`run`](IoAction::run), loads and strictly verifies a
/// snapshot: checksum first, then version, scalar, and fingerprint. Construct with
/// [`load_snapshot`].
pub struct LoadSnapshot {
    path: PathBuf,
    expected_scalar: ScalarTypeTag,
    current_fingerprint: Option<u64>,
}

impl IoAction for LoadSnapshot {
    type Output = SnapshotPackage;
    type Error = DataLoadingError;

    fn run(self) -> Result<SnapshotPackage, DataLoadingError> {
        let shown = self.path.display().to_string();
        let bytes = fs::read(&self.path)?;
        let decoded = decode(&bytes, &shown)?;
        if !decoded.checksum_ok {
            return Err(DataLoadingError::corrupt(
                &shown,
                "checksum mismatch: the file content does not match its recorded checksum",
            ));
        }
        check_scalar(&decoded.package, self.expected_scalar, &shown)?;
        if self
            .current_fingerprint
            .is_some_and(|current| decoded.package.fingerprint() != current)
        {
            return Err(DataLoadingError::fingerprint_mismatch(&shown));
        }
        Ok(decoded.package)
    }
}

/// Describe (but do not perform) a strict snapshot load. `expected_scalar` is the program's
/// working scalar; `current_fingerprint` is the digest of the current world description
/// (pass `None` to skip world validation, e.g. for inspection tools).
pub fn load_snapshot(
    path: impl AsRef<Path>,
    expected_scalar: ScalarTypeTag,
    current_fingerprint: Option<u64>,
) -> LoadSnapshot {
    LoadSnapshot {
        path: path.as_ref().to_path_buf(),
        expected_scalar,
        current_fingerprint,
    }
}

/// A lazy IO description of a force load: checksum and fingerprint mismatches are downgraded
/// to reported warnings; structural damage, unknown versions, and a scalar mismatch still
/// refuse. Construct with [`force_load_snapshot`].
pub struct ForceLoadSnapshot {
    path: PathBuf,
    expected_scalar: ScalarTypeTag,
    current_fingerprint: Option<u64>,
}

impl IoAction for ForceLoadSnapshot {
    type Output = (SnapshotPackage, Vec<String>);
    type Error = DataLoadingError;

    fn run(self) -> Result<(SnapshotPackage, Vec<String>), DataLoadingError> {
        let shown = self.path.display().to_string();
        let bytes = fs::read(&self.path)?;
        let decoded = decode(&bytes, &shown)?;
        let mut warnings = Vec::new();
        if !decoded.checksum_ok {
            warnings.push(format!(
                "force load: {shown} failed its checksum; the file is corrupt and its content \
                 is not trustworthy"
            ));
        }
        check_scalar(&decoded.package, self.expected_scalar, &shown)?;
        if self
            .current_fingerprint
            .is_some_and(|current| decoded.package.fingerprint() != current)
        {
            warnings.push(format!(
                "force load: {shown} belongs to a different world description \
                 (fingerprint mismatch)"
            ));
        }
        Ok((decoded.package, warnings))
    }
}

/// Describe (but do not perform) a force load of a snapshot from `path`.
pub fn force_load_snapshot(
    path: impl AsRef<Path>,
    expected_scalar: ScalarTypeTag,
    current_fingerprint: Option<u64>,
) -> ForceLoadSnapshot {
    ForceLoadSnapshot {
        path: path.as_ref().to_path_buf(),
        expected_scalar,
        current_fingerprint,
    }
}

fn check_scalar(
    package: &SnapshotPackage,
    expected: ScalarTypeTag,
    shown: &str,
) -> Result<(), DataLoadingError> {
    if package.scalar() != expected {
        return Err(DataLoadingError::scalar_mismatch(
            shown,
            expected.name(),
            package.scalar().name(),
        ));
    }
    Ok(())
}
