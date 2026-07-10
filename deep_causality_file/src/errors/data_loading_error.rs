/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The crate's stable public error type for all data loading.
//!
//! Design: a **public opaque struct** [`DataLoadingError`] wrapping a **private** representation enum
//! `DataLoadingErrorKind`. The public type is the invariant API surface (its `Display` /
//! [`std::error::Error`] behaviour); the private enum can grow new variants — a CSV parser, a parquet
//! reader, a schema-mismatch case — **without a breaking change** to any consumer. New failure modes
//! evolve behind the curtain; the public error stays the same.

use std::fmt;

/// The error type returned by every loader in this crate. Opaque by design: match on it via
/// [`Display`](fmt::Display) / [`std::error::Error`], not on internal variants, so the crate can add
/// new formats and failure modes without breaking your code.
#[derive(Debug)]
pub struct DataLoadingError {
    kind: DataLoadingErrorKind,
}

/// Private, evolvable representation. New loaders (CSV, parquet, …) add variants here; because this
/// enum is not part of the public API, doing so is never a breaking change.
#[derive(Debug)]
enum DataLoadingErrorKind {
    /// Underlying filesystem / I/O failure (open, read).
    Io(std::io::Error),
    /// A record field could not be parsed. `context` names the field; `detail` is the cause.
    Parse { context: String, detail: String },
    /// An identifier (e.g. a satellite code) was not recognised.
    Unknown(String),
    /// A delimited numeric table violated its shape or a cell failed to parse. `row` is
    /// one-based over significant (non-comment, non-empty) lines.
    Table {
        path: String,
        row: usize,
        detail: String,
    },
    /// A snapshot package failed its whole-package integrity checksum: the file is corrupt.
    Corrupt { path: String, detail: String },
    /// A snapshot package belongs to a different world description than the one loading it.
    FingerprintMismatch { path: String },
    /// A snapshot package was saved at a different working scalar than the loader expects.
    ScalarMismatch {
        path: String,
        expected: String,
        found: String,
    },
    /// A snapshot package carries a format version this build does not understand.
    UnknownVersion { path: String, found: u16 },
    /// A named column was requested from a table that does not carry it.
    MissingColumn { name: String },
}

impl DataLoadingError {
    /// A parse failure in a named field.
    pub(crate) fn parse(context: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind: DataLoadingErrorKind::Parse {
                context: context.into(),
                detail: detail.into(),
            },
        }
    }

    /// An unrecognised identifier (satellite code, column name, …).
    pub(crate) fn unknown(detail: impl Into<String>) -> Self {
        Self {
            kind: DataLoadingErrorKind::Unknown(detail.into()),
        }
    }

    /// A table shape or cell failure at a one-based significant row.
    pub(crate) fn table(path: impl Into<String>, row: usize, detail: impl Into<String>) -> Self {
        Self {
            kind: DataLoadingErrorKind::Table {
                path: path.into(),
                row,
                detail: detail.into(),
            },
        }
    }

    /// A snapshot whose checksum did not match its content.
    pub(crate) fn corrupt(path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind: DataLoadingErrorKind::Corrupt {
                path: path.into(),
                detail: detail.into(),
            },
        }
    }

    /// A snapshot saved under a different world description.
    pub(crate) fn fingerprint_mismatch(path: impl Into<String>) -> Self {
        Self {
            kind: DataLoadingErrorKind::FingerprintMismatch { path: path.into() },
        }
    }

    /// A snapshot saved at a different working scalar.
    pub(crate) fn scalar_mismatch(
        path: impl Into<String>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self {
            kind: DataLoadingErrorKind::ScalarMismatch {
                path: path.into(),
                expected: expected.into(),
                found: found.into(),
            },
        }
    }

    /// A snapshot with an unknown format version.
    pub(crate) fn unknown_version(path: impl Into<String>, found: u16) -> Self {
        Self {
            kind: DataLoadingErrorKind::UnknownVersion {
                path: path.into(),
                found,
            },
        }
    }

    /// A named column absent from a table.
    pub(crate) fn missing_column(name: impl Into<String>) -> Self {
        Self {
            kind: DataLoadingErrorKind::MissingColumn { name: name.into() },
        }
    }
}

impl fmt::Display for DataLoadingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DataLoadingErrorKind::Io(e) => write!(f, "data loading: I/O error: {e}"),
            DataLoadingErrorKind::Parse { context, detail } => {
                write!(f, "data loading: parse error in {context}: {detail}")
            }
            DataLoadingErrorKind::Unknown(d) => write!(f, "data loading: unknown identifier: {d}"),
            DataLoadingErrorKind::Table { path, row, detail } => {
                write!(
                    f,
                    "data loading: table error in {path} at row {row}: {detail}"
                )
            }
            DataLoadingErrorKind::Corrupt { path, detail } => {
                write!(f, "data loading: corrupt file {path}: {detail}")
            }
            DataLoadingErrorKind::FingerprintMismatch { path } => {
                write!(
                    f,
                    "data loading: snapshot {path} belongs to a different world description \
                     (fingerprint mismatch)"
                )
            }
            DataLoadingErrorKind::ScalarMismatch {
                path,
                expected,
                found,
            } => {
                write!(
                    f,
                    "data loading: snapshot {path} was saved at scalar {found}, this program \
                     runs at {expected}"
                )
            }
            DataLoadingErrorKind::UnknownVersion { path, found } => {
                write!(
                    f,
                    "data loading: snapshot {path} has unknown format version {found}"
                )
            }
            DataLoadingErrorKind::MissingColumn { name } => {
                write!(f, "data loading: table has no column named '{name}'")
            }
        }
    }
}

impl std::error::Error for DataLoadingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            DataLoadingErrorKind::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DataLoadingError {
    fn from(e: std::io::Error) -> Self {
        Self {
            kind: DataLoadingErrorKind::Io(e),
        }
    }
}
