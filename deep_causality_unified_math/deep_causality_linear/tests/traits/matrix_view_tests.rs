/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The defaulted half of [`MatrixView`].
//!
//! Every representation in the crate overrides `to_row_major` with a copy of its own buffer, so the
//! default body is reached only by a foreign view. These define such a view and hold the default to
//! the contract the overrides are written against.

use deep_causality_linear::{LinearError, LinearErrorEnum, MatrixView};

/// A view that computes its entries instead of storing them, so it has no buffer to copy and takes
/// the defaulted `to_row_major`.
///
/// Entry `(i, j)` is the Hilbert entry `1 / (i + j + 1)`.
struct HilbertView {
    rows: usize,
    cols: usize,
}

impl MatrixView for HilbertView {
    type Scalar = f64;

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn get(&self, row: usize, col: usize) -> Result<f64, LinearError> {
        if row >= self.rows || col >= self.cols {
            return Err(LinearError::IndexOutOfBounds(
                (row, col),
                (self.rows, self.cols),
            ));
        }
        Ok(1.0 / (row + col + 1) as f64)
    }
}

/// A view whose declared shape runs past the buffer it holds, so `get` fails at a position that is
/// inside the reported shape.
struct ShortBuffer {
    data: Vec<f64>,
    rows: usize,
    cols: usize,
}

impl MatrixView for ShortBuffer {
    type Scalar = f64;

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn get(&self, row: usize, col: usize) -> Result<f64, LinearError> {
        self.data
            .get(row * self.cols + col)
            .copied()
            .ok_or_else(|| LinearError::IndexOutOfBounds((row, col), (self.rows, self.cols)))
    }
}

#[test]
fn test_the_default_row_major_buffer_is_the_entries_read_across_each_row_in_turn() {
    // A 2x3 Hilbert block: row 0 is 1, 1/2, 1/3 and row 1 is 1/2, 1/3, 1/4. Row-major means the
    // rows follow one another; a column-major default would give 1, 1/2, 1/2, 1/3, 1/3, 1/4, which
    // has the same multiset and a different order.
    let v = HilbertView { rows: 2, cols: 3 };
    let flat = v.to_row_major().unwrap();
    let want = [1.0, 1.0 / 2.0, 1.0 / 3.0, 1.0 / 2.0, 1.0 / 3.0, 1.0 / 4.0];
    assert_eq!(flat.len(), 6, "rows * cols entries, not the stored count");
    for (k, w) in want.iter().enumerate() {
        assert_eq!(flat[k], *w, "entry {k}");
    }
}

#[test]
fn test_the_default_row_major_buffer_agrees_with_get_at_every_position() {
    // The kernels index the buffer as `i * cols + j` after reading the shape off the view. That
    // indexing is only correct if the buffer and `get` agree, which is the contract the default
    // establishes for any view that does not override it.
    let v = HilbertView { rows: 4, cols: 3 };
    let flat = v.to_row_major().unwrap();
    for i in 0..4 {
        for j in 0..3 {
            assert_eq!(flat[i * 3 + j], v.get(i, j).unwrap(), "at ({i}, {j})");
        }
    }
}

#[test]
fn test_the_default_row_major_buffer_of_a_shape_holding_nothing_is_empty() {
    // `0xn` and `nx0` are distinct shapes that both describe no entries, and neither is an error.
    let wide = HilbertView { rows: 0, cols: 4 };
    assert!(wide.is_empty());
    assert!(wide.to_row_major().unwrap().is_empty());

    let tall = HilbertView { rows: 4, cols: 0 };
    assert!(tall.is_empty());
    assert!(tall.to_row_major().unwrap().is_empty());
}

#[test]
fn test_the_default_row_major_buffer_stops_at_the_first_entry_the_view_refuses() {
    // A view that fails inside its own shape must not yield a buffer shorter than `rows * cols` --
    // a kernel indexing it by the reported shape would then read past the end. The error surfaces
    // instead, carrying the position that failed.
    let v = ShortBuffer {
        data: vec![1.0, 2.0, 3.0],
        rows: 2,
        cols: 2,
    };
    assert_eq!(v.len(), 4, "the shape describes four entries");
    let e = v.to_row_major().unwrap_err();
    assert!(
        matches!(
            e.kind(),
            LinearErrorEnum::IndexOutOfBounds {
                index: (1, 1),
                shape: (2, 2)
            }
        ),
        "got {e:?}"
    );
}
