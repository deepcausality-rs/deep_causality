//! The bit-packed 𝔽₂ matrix qcl-gaps.md G-01 asks for, generic over the word
//! type as §0 of that register suggests (`W: NaturalNumber`).
//!
//! Column `c` of row `r` lives at bit `c % W::BITS` of word `data[r * wpr + c / W::BITS]`.
//! The whole point: `axpy_rows` is one XOR per W::BITS columns.

use deep_causality_num::{NaturalNumber, One, Zero};
use linear_a::Gf2;
use linear_b::{MatrixBuild, MatrixView, RowOps};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedGf2<W> {
    rows: usize,
    cols: usize,
    wpr: usize,
    data: Vec<W>,
}

impl<W: NaturalNumber> PackedGf2<W> {
    fn bits() -> usize {
        W::BITS as usize
    }

    pub fn zeros(rows: usize, cols: usize) -> Self {
        let wpr = cols.div_ceil(Self::bits());
        Self {
            rows,
            cols,
            wpr,
            data: core::iter::repeat_with(W::zero).take(rows * wpr).collect(),
        }
    }

    /// Build from a row-major bool grid. This is the only place a caller pays
    /// for packing; after this, everything is word-parallel.
    pub fn from_bools(bits: &[bool], rows: usize, cols: usize) -> Self {
        assert_eq!(bits.len(), rows * cols);
        let mut m = Self::zeros(rows, cols);
        for r in 0..rows {
            for c in 0..cols {
                if bits[r * cols + c] {
                    m.set_bit(r, c);
                }
            }
        }
        m
    }

    fn set_bit(&mut self, r: usize, c: usize) {
        let b = Self::bits();
        let idx = r * self.wpr + c / b;
        self.data[idx] = self.data[idx] | (W::one() << ((c % b) as u32));
    }

    fn clear_bit(&mut self, r: usize, c: usize) {
        let b = Self::bits();
        let idx = r * self.wpr + c / b;
        self.data[idx] = self.data[idx] & !(W::one() << ((c % b) as u32));
    }

    /// Mask keeping bits at column index >= `from_col` within the word that
    /// contains `from_col`.
    fn head_mask(from_col: usize) -> W {
        let b = Self::bits();
        W::MAX << ((from_col % b) as u32)
    }

    /// The number of set bits in the whole matrix — used by tests to show the
    /// XOR path really ran.
    pub fn popcount(&self) -> u32 {
        self.data.iter().map(|w| w.count_ones()).sum()
    }
}

impl<W: NaturalNumber> MatrixView for PackedGf2<W> {
    type Scalar = Gf2;

    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }

    fn get(&self, r: usize, c: usize) -> Gf2 {
        let b = Self::bits();
        let w = self.data[r * self.wpr + c / b];
        Gf2(!((w >> ((c % b) as u32)) & W::one()).is_zero())
    }
}

impl<W: NaturalNumber> RowOps for PackedGf2<W> {
    fn swap_rows(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        for k in 0..self.wpr {
            self.data.swap(a * self.wpr + k, b * self.wpr + k);
        }
    }

    fn scale_row(&mut self, r: usize, factor: Gf2, from_col: usize) {
        // 𝔽₂ has exactly one unit, so scaling by 1 is a no-op and scaling by 0
        // clears the row. RREF only ever passes 1.
        if factor.is_one() {
            return;
        }
        let b = Self::bits();
        let first = from_col / b;
        self.data[r * self.wpr + first] = self.data[r * self.wpr + first] & !Self::head_mask(from_col);
        for k in (first + 1)..self.wpr {
            self.data[r * self.wpr + k] = W::zero();
        }
    }

    /// ONE XOR PER `W::BITS` COLUMNS. This is the method the whole design exists for.
    fn axpy_rows(&mut self, dst: usize, src: usize, factor: Gf2, from_col: usize) {
        if factor.is_zero() {
            return;
        }
        // factor == 1, so this is row[dst] ^= row[src].
        let b = Self::bits();
        let first = from_col / b;
        let (d, s) = (dst * self.wpr, src * self.wpr);

        let masked = self.data[s + first] & Self::head_mask(from_col);
        self.data[d + first] = self.data[d + first] ^ masked;

        for k in (first + 1)..self.wpr {
            self.data[d + k] = self.data[d + k] ^ self.data[s + k];
        }
    }
    // pivot_in_column: the DEFAULT is used verbatim. First non-zero is the
    // correct and optimal rule for an exact field, and it is a single-column
    // scan, so there is nothing word-parallel to gain by overriding it.
}

impl<W: NaturalNumber> MatrixBuild for PackedGf2<W> {
    fn zeros(rows: usize, cols: usize) -> Self {
        PackedGf2::zeros(rows, cols)
    }
    fn set(&mut self, r: usize, c: usize, v: Gf2) {
        if v.is_one() {
            self.set_bit(r, c);
        } else {
            self.clear_bit(r, c);
        }
    }
}
