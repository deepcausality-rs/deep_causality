/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A schema-agnostic keyed lookup table with **value-bracketed** linear interpolation and end
//! clamping — the reusable core of a measured-parameter lookup (change
//! `plasma-retropulsion-cfd-contracts`, capability `weather-table-consumption`; the M5
//! retropulsion example binds it to the weather dispersion table keyed by temperature departure).
//!
//! It is the N-column generalization of [`DescentSchedule::sample`](crate::DescentSchedule) (which
//! brackets and clamps a fixed four-column atmosphere row by altitude): rows arrive in any order,
//! [`KeyedTable::new`] sorts them ascending by key and rejects duplicate keys, and
//! [`KeyedTable::interpolate`] brackets the query **by key value** (never by file order — that would
//! select non-adjacent neighbors for most inputs) and interpolates every column linearly, clamping
//! a query outside the tabulated range to the nearest row and reporting it through the
//! [`KeyedInterpolation::clamped`] marker. Provenance stamping (a clamp flown outside the tabulated
//! range) stays with the flight side, which owns the `EffectLog`; this type is pure.

use crate::CfdScalar;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_physics::PhysicsError;

/// The result of a value-bracketed table interpolation: the interpolated columns at the query key,
/// the indices of the bracketing rows (equal when the query clamped to a single end row), and
/// whether the query fell outside the tabulated range (the marker the flight side stamps into
/// provenance).
#[derive(Debug, Clone, PartialEq)]
pub struct KeyedInterpolation<R: CfdScalar> {
    values: Vec<R>,
    lower: usize,
    upper: usize,
    clamped: bool,
}

impl<R: CfdScalar> KeyedInterpolation<R> {
    /// The interpolated (or clamped) column values at the query key.
    pub fn values(&self) -> &[R] {
        &self.values
    }

    /// The index of the lower bracketing row (equal to [`upper`](Self::upper) when clamped to an end).
    pub fn lower(&self) -> usize {
        self.lower
    }

    /// The index of the upper bracketing row (equal to [`lower`](Self::lower) when clamped to an end).
    pub fn upper(&self) -> usize {
        self.upper
    }

    /// Whether the query fell outside the tabulated key range and was clamped to the nearest row.
    pub fn clamped(&self) -> bool {
        self.clamped
    }
}

/// A table of numeric rows keyed by an ascending scalar, supporting value-bracketed linear
/// interpolation with end clamping. Rows are sorted ascending by key at construction; duplicate
/// keys and ragged column counts are rejected.
#[derive(Debug, Clone)]
pub struct KeyedTable<R: CfdScalar> {
    rows: Vec<(R, Vec<R>)>,
}

impl<R: CfdScalar> KeyedTable<R> {
    /// Build a table from `(key, columns)` rows in any order. Sorts ascending by key.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if there are no rows, if any key is not finite, or
    /// if two rows share a key; [`PhysicsError::DimensionMismatch`] if the rows are not all the same
    /// width.
    pub fn new(mut rows: Vec<(R, Vec<R>)>) -> Result<Self, PhysicsError> {
        if rows.is_empty() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "KeyedTable: at least one row is required".into(),
            ));
        }
        let ncol = rows[0].1.len();
        for (key, cols) in &rows {
            if !key.is_finite() {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "KeyedTable: every key must be finite".into(),
                ));
            }
            if cols.len() != ncol {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "KeyedTable: every row must have {ncol} columns"
                )));
            }
        }
        rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
        for w in rows.windows(2) {
            if w[0].0 == w[1].0 {
                return Err(PhysicsError::PhysicalInvariantBroken(format!(
                    "KeyedTable: duplicate key {}",
                    w[0].0
                )));
            }
        }
        Ok(Self { rows })
    }

    /// The number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether the table has no rows (never true after [`new`](Self::new), which rejects empties).
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The sorted `(key, columns)` rows.
    pub fn rows(&self) -> &[(R, Vec<R>)] {
        &self.rows
    }

    /// Interpolate every column at `key`, bracketing by key value. A `key` at or beyond either end
    /// clamps to that end row with [`KeyedInterpolation::clamped`] set when it is strictly outside
    /// the tabulated range.
    pub fn interpolate(&self, key: R) -> KeyedInterpolation<R> {
        let last = self.rows.len() - 1;
        let (first_key, first_vals) = &self.rows[0];
        if key <= *first_key {
            return KeyedInterpolation {
                values: first_vals.clone(),
                lower: 0,
                upper: 0,
                clamped: key < *first_key,
            };
        }
        let (last_key, last_vals) = &self.rows[last];
        if key >= *last_key {
            return KeyedInterpolation {
                values: last_vals.clone(),
                lower: last,
                upper: last,
                clamped: key > *last_key,
            };
        }
        // Strictly inside the range: the first row whose key reaches `key` is the upper bracket.
        let upper = self
            .rows
            .iter()
            .position(|(k, _)| *k >= key)
            .expect("an in-range key has an upper bracket");
        let lower = upper - 1;
        let (klo, vlo) = &self.rows[lower];
        let (khi, vhi) = &self.rows[upper];
        let t = (key - *klo) / (*khi - *klo);
        let values = vlo
            .iter()
            .zip(vhi)
            .map(|(a, b)| *a + t * (*b - *a))
            .collect();
        KeyedInterpolation {
            values,
            lower,
            upper,
            clamped: false,
        }
    }
}
