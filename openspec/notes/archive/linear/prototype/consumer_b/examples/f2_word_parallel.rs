//! Does the Design-B seam actually preserve word-parallel XOR?
//!
//! ONE generic `linear_b::rref`, run over two representations of the SAME
//! 𝔽₂ matrix:
//!   * `Dense<Gf2>`     — one BYTE per bit (what Design A and Design C force)
//!   * `PackedGf2<u64>` — one bit per bit, one XOR per 64 columns
//!
//! If the seam leaked per-element access into the inner loop, the two would
//! run at the same speed.

use consumer_b::{Dense, PackedGf2};
use linear_a::Gf2;
use linear_b::rank_in_place;
use std::time::Instant;

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024);
    let mut seed = 0x2026_0823_u64;
    let mut bits = vec![false; n * n];
    for b in bits.iter_mut() {
        // The low bits of a power-of-two LCG are degenerate; take a high bit.
        *b = (lcg(&mut seed) >> 40) & 1 == 1;
    }

    let mut packed = PackedGf2::<u64>::from_bools(&bits, n, n);
    let scalars: Vec<Gf2> = bits.iter().map(|&b| Gf2(b)).collect();
    let mut unpacked = Dense::new(scalars, n, n);

    let t0 = Instant::now();
    let r_packed = rank_in_place(&mut packed);
    let d_packed = t0.elapsed();

    let t1 = Instant::now();
    let r_unpacked = rank_in_place(&mut unpacked);
    let d_unpacked = t1.elapsed();

    // A hand-written, NON-generic packed elimination: the "just write 200 lines
    // over u64" that qcl-gaps.md G-01 proposes. This prices the seam itself.
    let mut hand: Vec<u64> = vec![0; n * n.div_ceil(64)];
    let wpr = n.div_ceil(64);
    for r in 0..n {
        for c in 0..n {
            if bits[r * n + c] {
                hand[r * wpr + c / 64] |= 1u64 << (c % 64);
            }
        }
    }
    let t3 = Instant::now();
    let r_hand = hand_written_rref(&mut hand, n, n, wpr);
    let d_hand = t3.elapsed();

    // Design C's shape: free function over &mut [Vec<F>].
    let mut rows_of_rows: Vec<Vec<Gf2>> = bits
        .chunks(n)
        .map(|r| r.iter().map(|&b| Gf2(b)).collect())
        .collect();
    let t2 = Instant::now();
    let r_c = linear_c::rref(&mut rows_of_rows);
    let d_c = t2.elapsed();

    assert_eq!(r_packed, r_unpacked, "the two must agree on the rank");
    assert_eq!(r_packed, r_c, "design C must agree too");
    assert_eq!(r_packed, r_hand, "the hand-written version must agree too");

    println!("{n}x{n} F2 matrix, rank = {r_packed}");
    println!("  B: PackedGf2<u64> (1 bit/entry)   : {d_packed:?}");
    println!("  hand-written packed, no trait     : {d_hand:?}");
    println!("  B: Dense<Gf2>     (1 byte/entry)  : {d_unpacked:?}");
    println!("  C: Vec<Vec<Gf2>>  (1 byte/entry)  : {d_c:?}");
    println!(
        "  packed vs dense-scalar speedup: {:.1}x   packed vs design C: {:.1}x",
        d_unpacked.as_secs_f64() / d_packed.as_secs_f64(),
        d_c.as_secs_f64() / d_packed.as_secs_f64()
    );
    println!(
        "  cost of the seam (generic / hand-written): {:.2}x",
        d_packed.as_secs_f64() / d_hand.as_secs_f64()
    );
    println!(
        "  memory: {} KiB packed vs {} KiB per scalar entry",
        (n * n) / 8 / 1024,
        (n * n) / 1024
    );
}

/// The non-generic reference: exactly the algorithm `linear_b::rref` runs,
/// written directly against `&mut [u64]` with no trait in sight.
fn hand_written_rref(m: &mut [u64], rows: usize, cols: usize, wpr: usize) -> usize {
    let mut pivot_row = 0usize;
    for col in 0..cols {
        if pivot_row >= rows {
            break;
        }
        let (w, b) = (col / 64, col % 64);
        let mut found = None;
        for r in pivot_row..rows {
            if m[r * wpr + w] >> b & 1 == 1 {
                found = Some(r);
                break;
            }
        }
        let p = match found {
            Some(p) => p,
            None => continue,
        };
        if p != pivot_row {
            for k in 0..wpr {
                m.swap(pivot_row * wpr + k, p * wpr + k);
            }
        }
        for r in 0..rows {
            if r == pivot_row || m[r * wpr + w] >> b & 1 == 0 {
                continue;
            }
            let mask = u64::MAX << b;
            let head = m[pivot_row * wpr + w] & mask;
            m[r * wpr + w] ^= head;
            for k in (w + 1)..wpr {
                m[r * wpr + k] ^= m[pivot_row * wpr + k];
            }
        }
        pivot_row += 1;
    }
    pivot_row
}
