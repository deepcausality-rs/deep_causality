/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # 𝔽₂: a field that is not a float
//!
//! Every other scalar in the tower approximates the reals. `Gf2` is a field with two elements, and
//! the whole algebra vocabulary applies to it unchanged:
//!
//! ```text
//! a + b   exclusive-or        a - b   exclusive-or, the same operation
//! a * b   conjunction         -a      a, every element is its own additive inverse
//! CHARACTERISTIC = 2          ORDER = 2
//! ```
//!
//! `CHARACTERISTIC = 2` is the fact everything else follows from: `1 + 1 = 0`, so `x + x = 0` for
//! every `x`, so subtraction and addition are one operation and negation is the identity. `Gf2` is
//! deliberately outside `DivisibleByIntegers` for the same reason — halving needs `2` to be
//! invertible, and here `2` is `0`.
//!
//! ## Where 𝔽₂ is the right field
//!
//! Wherever the quantity is presence, parity or incidence rather than magnitude:
//!
//! * **Erasure coding.** RAID-5 parity is one XOR across the stripe, and recovering a lost drive
//!   is one more. Section 2 does both.
//! * **Error-correcting codes.** Hamming, BCH, Reed–Muller and the LDPC codes in 5G NR, Wi-Fi 6
//!   and NVMe are 𝔽₂ linear codes; decoding is a linear solve. Section 3 is the smallest one.
//! * **CRC checksums.** Polynomial arithmetic over 𝔽₂ — every Ethernet frame and ZIP archive.
//! * **Persistent homology.** Topological data analysis computes homology with 𝔽₂ coefficients,
//!   because the incidence signs vanish, `∂∂ = 0` becomes exclusive-or, and the reduction runs
//!   bit-parallel. This is the closest use to the rest of this workspace: the boundary operators
//!   in `deep_causality_topology` hold `i8` signs, and over 𝔽₂ there are no signs to hold.
//! * **Integer factorization.** The quadratic sieve and the number field sieve both finish by
//!   finding a subset of relations whose exponent vector is zero mod 2, which is an 𝔽₂ nullspace.
//! * **Graph theory.** The cycle space and the cut space of a graph are 𝔽₂ vector spaces.
//! * **Shift registers.** Xorshift generators, scramblers and stream ciphers are 𝔽₂ linear maps.
//!
//! ## Why a scalar, when the matrices are bit-packed
//!
//! `deep_causality_linear` stores an 𝔽₂ matrix one bit per entry, because that is what makes the
//! reduction fast. Storage and element type are separate decisions: a packed matrix still answers
//! `get` with a value, and that value is an element of 𝔽₂. `Gf2` is its name, and `PackedGf2` is
//! where the bits go.

use deep_causality_algebra::{Characteristic, FiniteField};
use deep_causality_num::Gf2;

/// The RAID stripe: one byte per data block, written most significant bit first so the printed
/// rows read the same way the literals do.
const STRIPE: [u8; 4] = [0b1011_0010, 0b0110_1101, 0b1100_0111, 0b0010_1001];
const DATA_BLOCKS: usize = STRIPE.len();
const BLOCK_BITS: usize = 8;
/// The block the example loses and then rebuilds.
const LOST_BLOCK: usize = 2;

/// Hamming(7, 4): four data bits become a seven-bit codeword, and any single flip is corrected.
const CODE_BITS: usize = 7;
const DATA_BITS: usize = 4;
const PARITY_BITS: usize = 3;
/// The message the example encodes, and the bit position the channel corrupts. Positions are
/// one-based here, because that is what makes the syndrome read as the position directly.
const MESSAGE: [bool; DATA_BITS] = [true, false, true, true];
const CORRUPTED_POSITION: usize = 5;

/// One block of the stripe: `BLOCK_BITS` elements of 𝔽₂.
type Block = [Gf2; BLOCK_BITS];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. The field, and the one fact the rest follows from.
    // ---------------------------------------------------------------------
    let zero = Gf2::ZERO;
    let one = Gf2::ONE;
    // Two bindings rather than one repeated: the laws below hold for *any* two equal elements,
    // and writing them that way is what they say.
    let a = Gf2::ONE;
    let b = Gf2::ONE;
    print_field(
        <Gf2 as Characteristic>::CHARACTERISTIC,
        <Gf2 as FiniteField>::ORDER,
        a + b,
        -one,
        a - b,
        a / b,
    );

    // Characteristic 2: adding anything to itself lands on zero, so subtraction is addition.
    assert_eq!(a + b, zero);
    assert_eq!(-one, one);
    assert_eq!(a - b, zero);
    for &a in [zero, one].iter() {
        for &b in [zero, one].iter() {
            assert_eq!(a + b, a - b);
        }
    }

    // ---------------------------------------------------------------------
    // 2. Erasure coding: RAID-5 parity and recovery.
    // ---------------------------------------------------------------------
    // The parity block is the sum of the data blocks. Because every element is its own inverse,
    // the same sum run over the survivors plus the parity rebuilds whatever went missing.
    let blocks = stripe();
    let parity = sum_blocks(blocks.iter());
    print_stripe(&blocks, &parity);

    let survivors: Vec<Block> = blocks
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != LOST_BLOCK)
        .map(|(_, b)| *b)
        .collect();
    let rebuilt = sum_blocks(survivors.iter().chain(core::iter::once(&parity)));
    print_recovery(&blocks[LOST_BLOCK], &rebuilt);

    assert_eq!(rebuilt, blocks[LOST_BLOCK]);

    // ---------------------------------------------------------------------
    // 3. Error correction: Hamming(7, 4).
    // ---------------------------------------------------------------------
    // The parity-check matrix has the binary numeral for column `j` as its `j`-th column, so the
    // syndrome of a codeword with one flipped bit *is* the position of that bit.
    let codeword = encode(&MESSAGE);
    print_codeword(&codeword);

    // A clean codeword has a zero syndrome: that is what "codeword" means.
    let clean = syndrome(&codeword);
    assert_eq!(position_of(&clean), 0);

    let mut received = codeword;
    received[CORRUPTED_POSITION - 1] += Gf2::ONE;
    let flagged = syndrome(&received);
    let position = position_of(&flagged);
    print_syndrome(&received, &flagged, position);

    assert_eq!(position, CORRUPTED_POSITION);

    // Correcting is adding the error back, because adding it twice is adding zero.
    let mut corrected = received;
    corrected[position - 1] += Gf2::ONE;
    print_correction(&corrected, corrected == codeword);

    assert_eq!(corrected, codeword);
    assert_eq!(position_of(&syndrome(&corrected)), 0);

    print_footer();
    Ok(())
}

/// The data blocks, each byte spread across `BLOCK_BITS` elements of 𝔽₂.
fn stripe() -> [Block; DATA_BLOCKS] {
    core::array::from_fn(|block| {
        let byte = STRIPE[block];
        core::array::from_fn(|bit| Gf2::new(byte >> (BLOCK_BITS - 1 - bit) & 1 == 1))
    })
}

/// The bitwise sum of any number of blocks, which over 𝔽₂ is their exclusive-or.
fn sum_blocks<'a>(blocks: impl Iterator<Item = &'a Block>) -> Block {
    blocks.fold([Gf2::ZERO; BLOCK_BITS], |mut acc, block| {
        for (slot, &bit) in acc.iter_mut().zip(block.iter()) {
            *slot += bit;
        }
        acc
    })
}

/// The Hamming(7, 4) parity-check matrix `H`.
///
/// Column `j` holds the binary numeral for `j + 1`, so `H` has every non-zero 3-bit pattern
/// exactly once. That is the whole design: no two columns are equal, so no two single-bit errors
/// share a syndrome, and the syndrome read as a numeral is the position that flipped.
fn parity_check() -> [[Gf2; CODE_BITS]; PARITY_BITS] {
    core::array::from_fn(|row| core::array::from_fn(|col| Gf2::new((col + 1) >> row & 1 == 1)))
}

/// A four-bit message placed at the non-power-of-two positions, with the parity positions filled
/// so that every check sums to zero.
fn encode(message: &[bool; DATA_BITS]) -> [Gf2; CODE_BITS] {
    let mut code = [Gf2::ZERO; CODE_BITS];

    // Positions 3, 5, 6, 7 carry the message; 1, 2, 4 are the checks.
    for (slot, &bit) in [3usize, 5, 6, 7].iter().zip(message.iter()) {
        code[slot - 1] = Gf2::new(bit);
    }
    // Each check position is the sum of the data positions its row covers.
    let h = parity_check();
    for (row, check) in [1usize, 2, 4].iter().enumerate() {
        let sum = (0..CODE_BITS)
            .filter(|&col| col + 1 != *check)
            .fold(Gf2::ZERO, |acc, col| acc + h[row][col] * code[col]);
        code[check - 1] = sum;
    }

    code
}

/// `H · r`, the syndrome of a received word.
fn syndrome(received: &[Gf2; CODE_BITS]) -> [Gf2; PARITY_BITS] {
    let h = parity_check();

    core::array::from_fn(|row| {
        (0..CODE_BITS).fold(Gf2::ZERO, |acc, col| acc + h[row][col] * received[col])
    })
}

/// The syndrome read as a binary numeral, which is the one-based position that flipped. A zero
/// syndrome means the word is a codeword.
fn position_of(syndrome: &[Gf2; PARITY_BITS]) -> usize {
    syndrome
        .iter()
        .enumerate()
        .fold(0, |acc, (row, &bit)| acc | usize::from(bit.bit()) << row)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== F2: a field that is not a float ===\n");
    println!("  Two elements, exact arithmetic, and the same algebra vocabulary as the reals.\n");
}

fn print_field(
    characteristic: u32,
    order: u64,
    sum: Gf2,
    negated: Gf2,
    difference: Gf2,
    quotient: Gf2,
) {
    println!("--- 1. The field ---");
    println!("  CHARACTERISTIC   {characteristic}");
    println!("  ORDER            {order}");
    println!(
        "  1 + 1            {}      every element is its own additive inverse",
        show(sum)
    );
    println!(
        "  -1               {}      negation is the identity",
        show(negated)
    );
    println!(
        "  1 - 1            {}      subtraction is addition",
        show(difference)
    );
    println!(
        "  1 / 1            {}      the only unit is its own inverse",
        show(quotient)
    );
    println!();
    println!("  Gf2 is outside `DivisibleByIntegers`, because halving needs 2 to be invertible");
    println!("  and 2 is 0 here. The bound is what stops an averaging kernel compiling over F2.");
}

fn print_stripe(blocks: &[Block; DATA_BLOCKS], parity: &Block) {
    println!("\n--- 2. Erasure coding: a RAID-5 stripe ---");
    for (i, block) in blocks.iter().enumerate() {
        println!("  data {i}     {}", show_block(block));
    }
    println!(
        "  parity     {}   <- the sum of the blocks above",
        show_block(parity)
    );
}

fn print_recovery(lost: &Block, rebuilt: &Block) {
    println!("\n  drive {LOST_BLOCK} fails, and the survivors plus the parity are summed again:");
    println!("  lost       {}", show_block(lost));
    println!(
        "  rebuilt    {}   <- identical, because x + x = 0",
        show_block(rebuilt)
    );
}

fn print_codeword(codeword: &[Gf2; CODE_BITS]) {
    println!("\n--- 3. Error correction: Hamming(7, 4) ---");
    println!("  message    {}", show_bools(&MESSAGE));
    println!(
        "  codeword   {}   positions 1, 2, 4 are the checks",
        show_bits(codeword)
    );
}

fn print_syndrome(received: &[Gf2; CODE_BITS], syndrome: &[Gf2; PARITY_BITS], position: usize) {
    println!("\n  the channel flips bit {CORRUPTED_POSITION}:");
    println!("  received   {}", show_bits(received));
    println!(
        "  syndrome   {}       reads as {position}, which is the position that flipped",
        show_bits(syndrome)
    );
}

fn print_correction(corrected: &[Gf2; CODE_BITS], matches: bool) {
    println!(
        "  corrected  {}   matches the codeword: {matches}",
        show_bits(corrected)
    );
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  Both sections are linear algebra, and neither rounds. The parity block is a");
    println!("  sum, the syndrome is a matrix-vector product, and correcting is adding the");
    println!("  error back — sound only because adding a thing twice adds nothing. That is");
    println!("  characteristic 2, and it is why F2 earns a place in a tower of scalars.");
}

fn show(x: Gf2) -> u8 {
    u8::from(x.bit())
}

fn show_block(block: &Block) -> String {
    block.iter().map(|b| show(*b).to_string()).collect()
}

fn show_bits(bits: &[Gf2]) -> String {
    bits.iter().map(|b| show(*b).to_string()).collect()
}

fn show_bools(bits: &[bool]) -> String {
    bits.iter().map(|b| u8::from(*b).to_string()).collect()
}
