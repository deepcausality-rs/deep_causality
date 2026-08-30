//! DESIGN C prototype: free functions over `&mut [Vec<F>]`.
//!
//! No type, no trait. The representation is fixed by the signature.

use deep_causality_algebra::Field;

/// Reduce a row-of-rows matrix to RREF in place. Returns the rank.
pub fn rref<F: Field>(m: &mut [Vec<F>]) -> usize {
    let rows = m.len();
    if rows == 0 {
        return 0;
    }
    let cols = m[0].len();
    let mut pivot_row = 0usize;

    for col in 0..cols {
        if pivot_row >= rows {
            break;
        }
        let p = match (pivot_row..rows).find(|&r| !m[r][col].is_zero()) {
            Some(p) => p,
            None => continue,
        };
        m.swap(pivot_row, p);

        let inv = F::one() / m[pivot_row][col].clone();
        for v in m[pivot_row][col..].iter_mut() {
            *v = v.clone() * inv.clone();
        }
        for r in 0..rows {
            if r == pivot_row {
                continue;
            }
            let factor = m[r][col].clone();
            if factor.is_zero() {
                continue;
            }
            // Two disjoint mutable rows: the same `split_at_mut` dance the
            // trait-based `[Vec<F>]` impl needs. Design C does not avoid it.
            let (lo, hi) = if r < pivot_row {
                (r, pivot_row)
            } else {
                (pivot_row, r)
            };
            let (head, tail) = m.split_at_mut(hi);
            let (a, b) = (&mut head[lo], &mut tail[0]);
            let (dst, src) = if r < pivot_row { (a, &*b) } else { (b, &*a) };
            for (dv, sv) in dst[col..].iter_mut().zip(src[col..].iter()) {
                *dv = dv.clone() - factor.clone() * sv.clone();
            }
        }

        pivot_row += 1;
    }
    pivot_row
}

/// The only 𝔽₂ shape this signature admits: one `Gf2` (one `bool`, one BYTE)
/// per matrix entry. `Vec<Vec<u64>>` would type-check too, but `u64` is not a
/// `Field`, so the generic function cannot be instantiated at it.
pub fn rref_gf2_unpacked(m: &mut [Vec<linear_a::Gf2>]) -> usize {
    rref(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use linear_a::Gf2;

    #[test]
    fn rref_f64() {
        let mut m = vec![
            vec![1.0f64, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![5.0, 7.0, 9.0],
        ];
        assert_eq!(rref(&mut m), 2);
    }

    #[test]
    fn rref_gf2_one_byte_per_bit() {
        let o = Gf2::ONE;
        let z = Gf2::ZERO;
        let mut m = vec![vec![o, o, z], vec![z, o, o], vec![o, z, o]];
        assert_eq!(rref_gf2_unpacked(&mut m), 2);
        // The storage cost this design forces:
        assert_eq!(core::mem::size_of::<Gf2>(), 1);
    }
}
