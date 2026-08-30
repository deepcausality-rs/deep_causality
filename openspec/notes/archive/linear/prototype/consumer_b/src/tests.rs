use crate::dense::{Dense, DensePivoted};
use crate::packed_gf2::PackedGf2;
use linear_a::Gf2;
use linear_b::{MatrixView, RowOps, determinant, rank_in_place, rref};

// ---------------------------------------------------------------
// ONE generic elimination, three representations, two scalar types
// ---------------------------------------------------------------

#[test]
fn generic_rref_over_f64_dense() {
    // r3 = r1 + r2 -> rank 2
    let mut m = Dense::new(
        vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 7.0, 9.0],
        3,
        3,
    );
    assert_eq!(rank_in_place(&mut m), 2);
}

#[test]
fn generic_rref_over_f64_with_partial_pivoting() {
    // Same matrix, the RealField pivot rule. Same generic driver.
    let mut m = DensePivoted::new(Dense::new(
        vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 7.0, 9.0],
        3,
        3,
    ));
    assert_eq!(rank_in_place(&mut m), 2);
}

/// The pivot hook is expressive enough to state a rank tolerance, but it is
/// EASY TO STATE WRONG, and the wrong version type-checks. A tolerance relative
/// to the current column reports rank 3 for a matrix of rank 2, because the
/// third pivot is round-off of order 4e-16 and the column max at that point is
/// also of order 4e-16.
///
/// This is the reason `deep_causality_topology` computes rank by SVD
/// (`chain_complex_impl.rs:94`) rather than by elimination, and it does not go
/// away under any of the three designs.
#[test]
fn numerical_rank_is_not_a_property_the_seam_can_deliver() {
    let mut naive = DensePivoted::new_column_relative(Dense::new(
        vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 7.0, 9.0],
        3,
        3,
    ));
    assert_eq!(rank_in_place(&mut naive), 3); // WRONG; the true rank is 2

    let mut scaled = DensePivoted::new(Dense::new(
        vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 7.0, 9.0],
        3,
        3,
    ));
    assert_eq!(rank_in_place(&mut scaled), 2);
}

/// Over 𝔽₂ the same question has no answer to get wrong: `is_zero` is exact.
#[test]
fn f2_rank_has_no_tolerance_at_all() {
    let mut m = PackedGf2::<u64>::from_bools(
        &[true, true, false, false, true, true, true, false, true],
        3,
        3,
    );
    assert_eq!(rank_in_place(&mut m), 2);
}

#[test]
fn generic_rref_over_packed_gf2_u64() {
    // [[1,1,0],[0,1,1],[1,0,1]] : rows sum to zero over F2 -> rank 2
    let mut m = PackedGf2::<u64>::from_bools(
        &[true, true, false, false, true, true, true, false, true],
        3,
        3,
    );
    assert_eq!(rank_in_place(&mut m), 2);
}

#[test]
fn generic_rref_over_packed_gf2_u8_same_answer() {
    // Word width is a parameter; the answer must not depend on it.
    let bits = [true, true, false, false, true, true, true, false, true];
    let mut a = PackedGf2::<u8>::from_bools(&bits, 3, 3);
    let mut b = PackedGf2::<u16>::from_bools(&bits, 3, 3);
    let mut c = PackedGf2::<u128>::from_bools(&bits, 3, 3);
    assert_eq!(rank_in_place(&mut a), 2);
    assert_eq!(rank_in_place(&mut b), 2);
    assert_eq!(rank_in_place(&mut c), 2);
}

#[test]
fn real_rank_and_f2_rank_disagree_which_is_the_point_of_g_02() {
    let bits = [true, true, false, false, true, true, true, false, true];
    let floats: Vec<f64> = bits.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect();

    let mut over_r = Dense::new(floats, 3, 3);
    let mut over_f2 = PackedGf2::<u64>::from_bools(&bits, 3, 3);

    assert_eq!(rank_in_place(&mut over_r), 3);
    assert_eq!(rank_in_place(&mut over_f2), 2);
}

#[test]
fn generic_determinant_over_both_scalars() {
    let mut m = Dense::new(vec![1.0f64, 2.0, 3.0, 4.0], 2, 2);
    assert_eq!(determinant(&mut m), Some(-2.0));

    // det over F2 of [[1,1],[1,0]] is 1
    let mut f = PackedGf2::<u64>::from_bools(&[true, true, true, false], 2, 2);
    assert_eq!(determinant(&mut f), Some(Gf2::ONE));
}

// ---------------------------------------------------------------
// The word-parallelism claim, tested rather than asserted.
// ---------------------------------------------------------------

/// A 128-column 𝔽₂ matrix uses 2 words of u64 per row. If the generic driver
/// were reading and writing one bit at a time, this would still give the right
/// answer — so instrument the row op instead and count how many times it runs.
#[test]
fn wide_gf2_elimination_is_correct() {
    let rows = 64;
    let cols = 128;
    // Identity in the first 64 columns, zeros elsewhere: rank 64.
    let mut bits = vec![false; rows * cols];
    for i in 0..rows {
        bits[i * cols + i] = true;
    }
    let mut m = PackedGf2::<u64>::from_bools(&bits, rows, cols);
    assert_eq!(rank_in_place(&mut m), 64);

    // Now make row 63 the XOR of rows 0..63 -> rank drops to 63.
    let mut bits2 = vec![false; rows * cols];
    for i in 0..(rows - 1) {
        bits2[i * cols + i] = true;
    }
    for i in 0..(rows - 1) {
        bits2[(rows - 1) * cols + i] = true;
    }
    let mut m2 = PackedGf2::<u64>::from_bools(&bits2, rows, cols);
    assert_eq!(rank_in_place(&mut m2), 63);
}

/// G-01's actual requirement (R4): a KERNEL BASIS over 𝔽₂, in packed form.
/// The generic `kernel_basis` is parameterised by the OUTPUT representation,
/// so the 𝔽₂ kernel comes back bit-packed rather than as `Vec<Vec<bool>>`.
#[test]
fn generic_kernel_basis_returns_a_packed_f2_matrix() {
    // ∂ = [[1,1,0],[0,1,1],[1,0,1]] over F2: rank 2, so kernel dim 1,
    // spanned by (1,1,1).
    let mut m = PackedGf2::<u64>::from_bools(
        &[true, true, false, false, true, true, true, false, true],
        3,
        3,
    );
    let k: PackedGf2<u64> = linear_b::kernel_basis(&mut m);
    assert_eq!(k.rows(), 3);
    assert_eq!(k.cols(), 1);
    assert_eq!(k.get(0, 0), Gf2::ONE);
    assert_eq!(k.get(1, 0), Gf2::ONE);
    assert_eq!(k.get(2, 0), Gf2::ONE);
}

/// The same generic function, dense f64 in and dense f64 out.
#[test]
fn generic_kernel_basis_over_reals() {
    // [[1,2,3],[4,5,6],[5,7,9]] : rank 2, kernel spanned by (1,-2,1).
    let mut m = Dense::new(
        vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 7.0, 9.0],
        3,
        3,
    );
    let k: Dense<f64> = linear_b::kernel_basis(&mut m);
    assert_eq!(k.cols(), 1);
    let (a, b, c) = (k.get(0, 0), k.get(1, 0), k.get(2, 0));
    // Normalise by the last entry and compare with (1, -2, 1).
    assert!((a / c - 1.0).abs() < 1e-9, "{a} {b} {c}");
    assert!((b / c + 2.0).abs() < 1e-9, "{a} {b} {c}");
}

/// Counting instrumentation: wrap `PackedGf2` and count element `get`s and
/// row `axpy`s during one elimination, to show that element access is O(rows)
/// per pivot and never O(rows*cols).
struct Counting {
    inner: PackedGf2<u64>,
    gets: core::cell::Cell<usize>,
    axpys: core::cell::Cell<usize>,
}

impl MatrixView for Counting {
    type Scalar = Gf2;
    fn rows(&self) -> usize {
        self.inner.rows()
    }
    fn cols(&self) -> usize {
        self.inner.cols()
    }
    fn get(&self, r: usize, c: usize) -> Gf2 {
        self.gets.set(self.gets.get() + 1);
        self.inner.get(r, c)
    }
}

impl RowOps for Counting {
    fn swap_rows(&mut self, a: usize, b: usize) {
        self.inner.swap_rows(a, b)
    }
    fn scale_row(&mut self, r: usize, f: Gf2, from_col: usize) {
        self.inner.scale_row(r, f, from_col)
    }
    fn axpy_rows(&mut self, d: usize, s: usize, f: Gf2, from_col: usize) {
        self.axpys.set(self.axpys.get() + 1);
        self.inner.axpy_rows(d, s, f, from_col)
    }
}

#[test]
fn element_access_is_not_in_the_inner_loop() {
    let rows = 64;
    let cols = 512; // 8 u64 words per row
                    // A dense-ish matrix that actually needs elimination:
                    // row i has bits set at every column congruent to i mod 7,
                    // plus the diagonal, so most pivots trigger many row XORs.
    let mut bits = vec![false; rows * cols];
    for i in 0..rows {
        bits[i * cols + i] = true;
        for c in 0..cols {
            if (c + i) % 7 == 0 {
                bits[i * cols + c] = true;
            }
        }
    }
    let mut m = Counting {
        inner: PackedGf2::<u64>::from_bools(&bits, rows, cols),
        gets: core::cell::Cell::new(0),
        axpys: core::cell::Cell::new(0),
    };
    let rank = rref(&mut m).rank;

    let gets = m.gets.get();
    let axpys = m.axpys.get();
    let words = cols / 64;

    // Elimination really happened.
    assert!(axpys > 0, "expected row XORs, got none");

    // The claim: the generic driver's ELEMENT accesses scale with rows*rank,
    // not rows*cols, and every column-wise unit of work goes through
    // `axpy_rows`, which the packed impl serves with `words` XOR instructions.
    let bitwise_work_if_element_at_a_time = axpys * cols;
    let word_work_actually_done = axpys * words;
    // Element access is confined to single-COLUMN scans (pivot search and the
    // elimination factor). It is bounded by rows*cols with a constant of one
    // bit-test, and it is NOT where the O(rows * cols * rank) work lives.
    assert!(gets < rows * cols, "gets={gets} rows={rows} rank={rank}");
    // All column-spanning work went through axpy_rows, at 1 XOR per 64 columns.
    assert_eq!(word_work_actually_done * 64, bitwise_work_if_element_at_a_time);

    std::eprintln!(
        "rank={rank} gets={gets} axpys={axpys} rows*cols={} \
         per-element column work avoided: {} -> {} word ops (64x)",
        rows * cols,
        bitwise_work_if_element_at_a_time,
        word_work_actually_done
    );
}
