/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::csr_matrix::CsrMatrix;
use core::fmt::{Display, Formatter, Result};
use deep_causality_algebra::CommutativeSemiring;

/// Renders the matrix as a padded grid, structural zeros included.
///
/// The format is the one the crate this moves from produces, character for character: a header line
/// with the shape, then one bracketed row per line with each entry right-padded to eight columns and
/// three decimal places. A zero-dimension matrix is a single line ending in `[Empty]`.
///
/// # Why the format is copied rather than chosen
///
/// This crate had no `Display` at all until porting the sparse suite found the gap, and the obvious
/// replacement — printing the three CSR arrays — is arguably more useful for debugging a sparse
/// structure. It is also a different string, and `linear-matrix-representations` requires that code
/// written against the old type keep producing identical results. A rendering is a result.
///
/// Printing the structural zeros means the cost is `rows * cols` rather than the stored count. That
/// is the old behaviour and it is kept; a caller printing a large sparse matrix was already paying
/// it.
impl<T> Display for CsrMatrix<T>
where
    T: Display + CommutativeSemiring + Copy + PartialEq,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (rows, cols) = self.shape();
        if rows == 0 || cols == 0 {
            return write!(f, "CsrMatrix ({rows}x{cols}) [Empty]");
        }

        writeln!(f, "CsrMatrix ({rows}x{cols})")?;
        for r in 0..rows {
            write!(f, "[")?;
            for c in 0..cols {
                write!(f, "{: >8.3}", self.get_value_at(r, c))?;
                if c < cols - 1 {
                    write!(f, ", ")?;
                }
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}
