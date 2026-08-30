//! DESIGN A prototype: `linear` owns its own dense `Matrix<F: Field>`.
//!
//! Compiled against the REAL `deep_causality_algebra::Field`.

use deep_causality_algebra::Field;

mod gf2_scalar;
pub use gf2_scalar::Gf2;

/// A dense, row-major matrix owned by the `linear` crate.
#[derive(Clone, Debug, PartialEq)]
pub struct Matrix<F> {
    rows: usize,
    cols: usize,
    data: Vec<F>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LinearError {
    ShapeMismatch,
}

impl<F: Field> Matrix<F> {
    pub fn from_row_major(data: Vec<F>, rows: usize, cols: usize) -> Result<Self, LinearError> {
        if data.len() != rows * cols {
            return Err(LinearError::ShapeMismatch);
        }
        Ok(Self { rows, cols, data })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, r: usize, c: usize) -> F {
        self.data[r * self.cols + c].clone()
    }

    pub fn set(&mut self, r: usize, c: usize, v: F) {
        self.data[r * self.cols + c] = v;
    }

    fn swap_rows(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        for c in 0..self.cols {
            self.data.swap(a * self.cols + c, b * self.cols + c);
        }
    }

    /// Reduced row echelon form by Gauss-Jordan elimination. Returns the rank.
    ///
    /// Pivot rule: FIRST NON-ZERO. A bare `Field` has no ordering and no
    /// `epsilon`, so partial pivoting by magnitude is not expressible here.
    pub fn rref(&mut self) -> usize {
        let mut pivot_row = 0usize;
        for col in 0..self.cols {
            if pivot_row >= self.rows {
                break;
            }
            let mut found = None;
            for r in pivot_row..self.rows {
                if !self.get(r, col).is_zero() {
                    found = Some(r);
                    break;
                }
            }
            let p = match found {
                Some(p) => p,
                None => continue,
            };
            self.swap_rows(pivot_row, p);

            let inv = F::one() / self.get(pivot_row, col);
            for c in col..self.cols {
                let v = self.get(pivot_row, c) * inv.clone();
                self.set(pivot_row, c, v);
            }
            for r in 0..self.rows {
                if r == pivot_row {
                    continue;
                }
                let factor = self.get(r, col);
                if factor.is_zero() {
                    continue;
                }
                for c in col..self.cols {
                    let v = self.get(r, c) - factor.clone() * self.get(pivot_row, c);
                    self.set(r, c, v);
                }
            }
            pivot_row += 1;
        }
        pivot_row
    }

    pub fn rank(&self) -> usize {
        let mut m = self.clone();
        m.rref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_over_reals() {
        // rows 3 = row1 + row2 -> rank 2
        let m = Matrix::from_row_major(
            vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 7.0, 9.0],
            3,
            3,
        )
        .unwrap();
        assert_eq!(m.rank(), 2);
    }

    #[test]
    fn rank_over_gf2_differs_from_rank_over_reals() {
        // [[1,1,0],[0,1,1],[1,0,1]] : over R rank 2 (r3 = r1 - r2),
        // over F2 rank 2 as well (r1+r2+r3 = 0). Use a case that separates
        // the two: [[1,1],[1,1]] is rank 1 in both. Instead check the
        // classic even-weight dependency: over Z rank 3, over F2 rank 2.
        let real = Matrix::from_row_major(
            vec![1.0f64, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0],
            3,
            3,
        )
        .unwrap();
        assert_eq!(real.rank(), 3); // over R the determinant is 2, so full rank

        let f2 = Matrix::from_row_major(
            vec![
                Gf2::ONE,
                Gf2::ONE,
                Gf2::ZERO,
                Gf2::ZERO,
                Gf2::ONE,
                Gf2::ONE,
                Gf2::ONE,
                Gf2::ZERO,
                Gf2::ONE,
            ],
            3,
            3,
        )
        .unwrap();
        assert_eq!(f2.rank(), 2); // over F2 the rows sum to zero
    }
}
