/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Unweighted choice over an integer range, and the modulo bias it exists to remove.

use deep_causality_rand::{Distribution, RngCore, Xoshiro256};
use deep_causality_stats::UniformInt;
use deep_causality_stats::utils_tests::sampling::{SUITE_SEED, expect_accepted, expect_refused};

/// Observed frequency of each index.
fn frequencies(d: &UniformInt, n: u64, seed: u64) -> Vec<f64> {
    let mut g = Xoshiro256::from_seed(seed);
    let mut counts = vec![0u64; d.len() as usize];
    for _ in 0..n {
        let i: u64 = d.sample(&mut g);
        assert!(i < d.len(), "index {i} is outside 0..{}", d.len());
        counts[i as usize] += 1;
    }
    counts.into_iter().map(|c| c as f64 / n as f64).collect()
}

// ---------------------------------------------------------------------------------------------
// Uniformity at a size that exposes the bias
// ---------------------------------------------------------------------------------------------

#[test]
fn a_range_of_seven_is_uniform() {
    // Seven divides no power of two. That makes the range awkward for any block-based map, and a
    // sampler that rounds a block boundary the wrong way shows up here. It does not test modulo
    // bias: at a range this far below the word size the bias is 4e-19 relative, which no sample
    // size can see. `a_range_near_the_word_size_is_uniform` is the test that sees it.
    let d = expect_accepted(UniformInt::new(7), "range 7");
    let f = frequencies(&d, 700_000, SUITE_SEED);
    for (i, p) in f.iter().enumerate() {
        let rel = (p - 1.0 / 7.0).abs() / (1.0 / 7.0);
        assert!(rel < 0.01, "index {i} took {p}, {rel} away from 1/7");
    }
}

#[test]
fn other_awkward_sizes_are_uniform_too() {
    // Three and five are the other small sizes that divide no power of two.
    for n in [3u64, 5, 11] {
        let d = expect_accepted(UniformInt::new(n), "awkward range");
        let f = frequencies(&d, 300_000, SUITE_SEED);
        let want = 1.0 / n as f64;
        for (i, p) in f.iter().enumerate() {
            let rel = (p - want).abs() / want;
            assert!(rel < 0.02, "range {n}, index {i} took {p}, {rel} away from {want}");
        }
    }
}

#[test]
fn a_power_of_two_range_is_uniform() {
    // The case modulo happens to get right, included so the suite is not only testing the hard
    // sizes.
    let d = expect_accepted(UniformInt::new(8), "range 8");
    let f = frequencies(&d, 400_000, SUITE_SEED);
    for (i, p) in f.iter().enumerate() {
        let rel = (p - 0.125).abs() / 0.125;
        assert!(rel < 0.02, "index {i} took {p}");
    }
}

/// A range for which `2^64 mod len` is half the range, so modulo double-counts the lower half.
const BIASED_LEN: u64 = 12_297_829_382_473_034_411;

/// `2^64 mod BIASED_LEN`, the first index modulo would not reach twice.
const BIASED_SHORT_BLOCK: u64 = 6_148_914_691_236_517_205;

#[test]
fn a_range_near_the_word_size_is_uniform() {
    // Modulo bias is a function of how many whole copies of the range fit in a word. At range 7
    // that is 2^61 copies and the bias vanishes; here exactly one whole copy fits and half a
    // second, so `w % len` hands the lower half two chances and the upper half one. A uniform
    // draw puts 1/2 of its mass below the short block; `w % len` puts 2/3 there.
    let d = expect_accepted(UniformInt::new(BIASED_LEN), "a range near the word size");
    let mut g = Xoshiro256::from_seed(SUITE_SEED);
    let n = 20_000;
    let below = (0..n)
        .filter(|_| {
            let i: u64 = d.sample(&mut g);
            assert!(i < BIASED_LEN, "index {i} is outside the range");
            i < BIASED_SHORT_BLOCK
        })
        .count() as f64
        / n as f64;
    assert!(
        (below - 0.5).abs() < 0.02,
        "{below} of the draws fell in the lower half; a uniform draw gives 1/2 and `w % len` 2/3"
    );
}

/// A generator that hands out a fixed script of words, so the rejection can be watched directly.
struct ScriptedRng {
    words: Vec<u64>,
    next: usize,
}

impl RngCore for ScriptedRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        let w = *self
            .words
            .get(self.next)
            .expect("the sampler drew more words than the script holds");
        self.next += 1;
        w
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0);
    }
}

impl deep_causality_rand::Rng for ScriptedRng {}

#[test]
fn a_word_from_the_short_block_is_redrawn() {
    // The statistical tests cannot separate a correct sampler from one that skips the rejection:
    // dropping it moves each index's probability by at most one part in 2^64. Watching a single
    // word can. Word zero falls in the short block and must be thrown away.
    let d = expect_accepted(UniformInt::new(BIASED_LEN), "a range near the word size");
    let mut rng = ScriptedRng {
        words: vec![0, 1 << 63],
        next: 0,
    };
    let i: u64 = d.sample(&mut rng);
    assert_eq!(
        i, BIASED_SHORT_BLOCK,
        "the short-block word was kept instead of redrawn"
    );
    assert_eq!(rng.next, 2, "the sampler did not draw a second word");
}

// ---------------------------------------------------------------------------------------------
// The range
// ---------------------------------------------------------------------------------------------

#[test]
fn a_range_of_one_always_returns_zero() {
    let d = expect_accepted(UniformInt::new(1), "range 1");
    let f = frequencies(&d, 1_000, SUITE_SEED);
    assert_eq!(f[0], 1.0);
}

#[test]
fn every_draw_stays_in_range() {
    // `frequencies` asserts this on every draw; this test names the sizes it matters for. An
    // off-by-one giving `1..=n` steps past the last index and is caught here.
    for n in [1u64, 2, 1000] {
        let d = expect_accepted(UniformInt::new(n), "range");
        let f = frequencies(&d, 10_000, SUITE_SEED);
        assert_eq!(f.len(), n as usize);
        assert!(f.iter().sum::<f64>() > 0.99, "range {n} lost draws");
    }
}

#[test]
fn the_last_index_is_reachable() {
    // A rejection loop that retries on the wrong condition silently narrows the range, which a
    // uniformity test at a coarse tolerance can miss but this cannot.
    let d = expect_accepted(UniformInt::new(4), "range 4");
    let f = frequencies(&d, 40_000, SUITE_SEED);
    assert!(f[3] > 0.2, "the last index took only {}", f[3]);
}

#[test]
fn the_usize_draw_agrees_with_the_u64_draw() {
    // Two implementations of the same draw must not diverge.
    let d = expect_accepted(UniformInt::new(6), "range 6");
    let mut a = Xoshiro256::from_seed(SUITE_SEED);
    let mut b = Xoshiro256::from_seed(SUITE_SEED);
    for _ in 0..1_000 {
        let x: u64 = d.sample(&mut a);
        let y: usize = d.sample(&mut b);
        assert_eq!(x as usize, y, "the two index widths disagreed");
    }
}

// ---------------------------------------------------------------------------------------------
// Degenerate generators
// ---------------------------------------------------------------------------------------------

/// A generator that walks the very top of the word range, where a rejection sampler does its work.
struct NearMaxRng(u64);

impl RngCore for NearMaxRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        u64::MAX - (self.0 % 8)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0xFF);
    }
}

impl deep_causality_rand::Rng for NearMaxRng {}

#[test]
fn words_at_the_top_of_the_range_still_yield_an_index() {
    // The rejection zone lives at the top of the word range. A sampler that rejects there must
    // still terminate and still return something in range.
    let d = expect_accepted(UniformInt::new(7), "range 7");
    let mut rng = NearMaxRng(0);
    for _ in 0..1_000 {
        let i: u64 = d.sample(&mut rng);
        assert!(i < 7, "a top-of-range word gave index {i}");
    }
}

// ---------------------------------------------------------------------------------------------
// Parameters outside the support
// ---------------------------------------------------------------------------------------------

#[test]
fn an_empty_range_is_refused() {
    expect_refused(UniformInt::new(0), "an empty range");
}

#[test]
fn a_valid_range_is_kept_verbatim() {
    let d = expect_accepted(UniformInt::new(42), "range 42");
    assert_eq!(d.len(), 42);
    assert!(!d.is_empty());
}
