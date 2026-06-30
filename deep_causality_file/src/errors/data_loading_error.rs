/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The crate's stable public error type for all data loading.
//!
//! Design: a **public opaque struct** [`DataLoadingError`] wrapping a **private** representation enum
//! [`DataLoadingErrorKind`]. The public type is the invariant API surface (its `Display` /
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
    // Future formats add variants here (e.g. `Csv { line, detail }`, `Parquet(...)`,
    // `SchemaMismatch { expected, found }`) without changing the public `DataLoadingError` type.
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
}

impl fmt::Display for DataLoadingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DataLoadingErrorKind::Io(e) => write!(f, "data loading: I/O error: {e}"),
            DataLoadingErrorKind::Parse { context, detail } => {
                write!(f, "data loading: parse error in {context}: {detail}")
            }
            DataLoadingErrorKind::Unknown(d) => write!(f, "data loading: unknown identifier: {d}"),
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
