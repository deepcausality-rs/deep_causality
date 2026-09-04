/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `BFloat16` layout, constructors and the two rounding kernels behind them.
//!
//! Expected bit patterns are derived by hand in the comment beside each assertion. The rule for
//! an `f32` input is: keep the top 16 bits, add one to them when the discarded low half is above
//! `0x8000`, or equals `0x8000` and the kept bit is odd. The families at the end are checked
//! against [`oracle_round_to_nearest_even`], which rounds an `f64` by integer arithmetic on its
//! sign, exponent and significand fields and never goes through `f32`.

use deep_causality_num::BFloat16;

// =============================================================================
// The independent oracle
// =============================================================================

/// Round an `f64` to bf16 bits, to nearest with ties to even, by integer arithmetic on the fields.
///
/// A different algorithm from the implementation, which adds a bias to the `f32` pattern and
/// takes an `f64` through a round-to-odd `f32`. Here the value is `m · 2^e` with `m` an integer,
/// the bf16 quantum is chosen from the exponent, and the remainder below the quantum decides the
/// rounding by an exact integer comparison against half a quantum.
fn oracle_round_to_nearest_even(x: f64) -> u16 {
    let bits = x.to_bits();
    let sign = ((bits >> 63) as u16) << 15;
    let exp = ((bits >> 52) & 0x7FF) as i32;
    let frac = bits & ((1u64 << 52) - 1);
    if exp == 0x7FF {
        // Infinity keeps its sign; a NaN keeps its sign and the top seven significand bits
        // (51..45, of which 51 is the quiet bit) and is quieted.
        return if frac == 0 {
            sign | 0x7F80
        } else {
            sign | 0x7FC0 | ((frac >> 45) as u16 & 0x7F)
        };
    }
    if exp == 0 && frac == 0 {
        return sign;
    }
    let (m, e) = if exp == 0 {
        (frac, -1074)
    } else {
        (frac | (1u64 << 52), exp - 1075)
    };
    let top = 63 - m.leading_zeros() as i32;
    let value_exp = e + top;
    // The quantum is 2^(value_exp - 7) for a normal result and 2^-133 in the subnormal range.
    let quantum_exp = if value_exp >= -126 {
        value_exp - 7
    } else {
        -133
    };
    let shift = quantum_exp - e;
    debug_assert!(
        shift > 0,
        "an f64 always carries more bits than a bf16 quantum admits"
    );
    let mut n: u128 = if shift >= 128 {
        // The value is below half a quantum and rounds to zero.
        0
    } else {
        let m = m as u128;
        let half = 1u128 << (shift - 1);
        let rem = m & ((1u128 << shift) - 1);
        let n = m >> shift;
        if rem > half || (rem == half && n & 1 == 1) {
            n + 1
        } else {
            n
        }
    };
    let mut quantum_exp = quantum_exp;
    if value_exp < -126 {
        // Subnormal: n quanta of 2^-133. n == 128 is exactly the smallest normal, 0x0080.
        return sign | n as u16;
    }
    if n == 256 {
        n = 128;
        quantum_exp += 1;
    }
    let biased = quantum_exp + 7 + 127;
    if biased >= 255 {
        return sign | 0x7F80;
    }
    sign | ((biased as u16) << 7) | ((n as u16) & 0x7F)
}

/// A deterministic generator for families of `f64` inputs; no external crate, no time seed.
struct Lcg(u64);

impl Lcg {
    fn next_u64(&mut self) -> u64 {
        // Knuth's MMIX multiplier and increment.
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
}

#[test]
fn test_oracle_agrees_with_closed_forms() {
    // The oracle is itself checked against literals derived by hand before it judges anything.
    // 1.0 is 0x3F80; pi = 0x40490FDB in f32, low half 0x0FDB is below the midpoint: 0x4049.
    assert_eq!(oracle_round_to_nearest_even(1.0), 0x3F80);
    assert_eq!(oracle_round_to_nearest_even(core::f64::consts::PI), 0x4049);
    // 1 + 2^-8 is the tie between 1.0 (even) and 1 + 2^-7 (odd).
    assert_eq!(oracle_round_to_nearest_even(1.0 + 2f64.powi(-8)), 0x3F80);
    // 2^-133 is the smallest subnormal; 2^-134 is the tie with zero.
    assert_eq!(oracle_round_to_nearest_even(2f64.powi(-133)), 0x0001);
    assert_eq!(oracle_round_to_nearest_even(2f64.powi(-134)), 0x0000);
    // 2^128 - 2^120 is MAX; 2^128 is infinity.
    assert_eq!(oracle_round_to_nearest_even(3.3895313892515355e38), 0x7F7F);
    assert_eq!(oracle_round_to_nearest_even(2f64.powi(128)), 0x7F80);
    assert_eq!(oracle_round_to_nearest_even(-0.0), 0x8000);
    assert_eq!(oracle_round_to_nearest_even(f64::NEG_INFINITY), 0xFF80);
    assert_eq!(oracle_round_to_nearest_even(f64::NAN) & 0x7FC0, 0x7FC0);
}

// =============================================================================
// Layout and bit-level constructors
// =============================================================================

#[test]
fn test_layout_is_a_bare_u16() {
    // half-rs 1.4.1 added `repr(transparent)` to remove undefined behaviour in its layout; the
    // type here is two bytes with the alignment of `u16` and nothing else.
    assert_eq!(core::mem::size_of::<BFloat16>(), 2);
    assert_eq!(core::mem::align_of::<BFloat16>(), 2);
}

#[test]
fn test_from_bits_to_bits_round_trip_every_pattern() {
    for bits in 0..=u16::MAX {
        assert_eq!(BFloat16::from_bits(bits).to_bits(), bits);
    }
}

#[test]
fn test_default_is_positive_zero() {
    assert_eq!(BFloat16::default().to_bits(), 0x0000);
}

#[test]
fn test_copy_and_clone_preserve_bits() {
    let x = BFloat16::from_bits(0x4049);
    let y = x;
    #[allow(clippy::clone_on_copy)]
    let z = x.clone();
    assert_eq!(y.to_bits(), 0x4049);
    assert_eq!(z.to_bits(), 0x4049);
}

#[test]
fn test_constructors_and_getters_are_const() {
    // half-rs #109 asks for const conversions; here every bit-level crossing is `const fn`.
    const HALF: BFloat16 = BFloat16::round_from_f32(0.5);
    const THIRD: BFloat16 = BFloat16::round_from_f64(1.0 / 3.0);
    const RAW: BFloat16 = BFloat16::from_bits(0x3F80);
    const BITS: u16 = RAW.to_bits();
    const WIDE: f32 = RAW.to_f32();
    const WIDER: f64 = HALF.to_f64();
    // 0.5 = 2^-1: biased exponent 126 = 0x7E, so 0x7E << 7 = 0x3F00.
    assert_eq!(HALF.to_bits(), 0x3F00);
    // 1/3 = 0x3EAAAAAB in f32; the low half 0xAAAB is above the midpoint: 0x3EAB.
    assert_eq!(THIRD.to_bits(), 0x3EAB);
    assert_eq!(BITS, 0x3F80);
    assert_eq!(WIDE, 1.0);
    assert_eq!(WIDER, 0.5);
}

// =============================================================================
// f32 -> bf16: round to nearest, ties to even
// =============================================================================

#[test]
fn test_round_from_f32_exact_values_pass_through() {
    // Each value has at most 8 significant bits, so its low half-word is zero.
    assert_eq!(BFloat16::round_from_f32(1.0).to_bits(), 0x3F80);
    assert_eq!(BFloat16::round_from_f32(-2.0).to_bits(), 0xC000);
    // 0.375 = 1.1b * 2^-2: exponent 125 = 0x7D, significand 0x40: 0x3EC0.
    assert_eq!(BFloat16::round_from_f32(0.375).to_bits(), 0x3EC0);
    // 255 = 1.1111111b * 2^7: exponent 134 = 0x86, significand 0x7F: 0x437F.
    assert_eq!(BFloat16::round_from_f32(255.0).to_bits(), 0x437F);
    // 1000 = 1.111101b * 2^9: exponent 136 = 0x88, significand 0x7A: 0x447A.
    assert_eq!(BFloat16::round_from_f32(1000.0).to_bits(), 0x447A);
}

#[test]
fn test_round_from_f32_rounds_to_nearest() {
    // 0.1f32 = 0x3DCCCCCD; the discarded half-word 0xCCCD is above the midpoint.
    assert_eq!(BFloat16::round_from_f32(0.1).to_bits(), 0x3DCD);
    // 1/3 = 0x3EAAAAAB; discarded 0xAAAB is above the midpoint.
    assert_eq!(BFloat16::round_from_f32(1.0 / 3.0).to_bits(), 0x3EAB);
    // pi = 0x40490FDB; discarded 0x0FDB is below the midpoint.
    assert_eq!(
        BFloat16::round_from_f32(core::f32::consts::PI).to_bits(),
        0x4049
    );
    // e = 0x402DF854; discarded 0xF854 is above the midpoint.
    assert_eq!(
        BFloat16::round_from_f32(core::f32::consts::E).to_bits(),
        0x402E
    );
}

#[test]
fn test_round_from_f32_ties_go_to_even() {
    // 1 + 2^-8 = 0x3F808000 sits exactly between 1.0 (0x3F80, even) and 1 + 2^-7 (0x3F81, odd).
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x3F80_8000)).to_bits(),
        0x3F80
    );
    // 1 + 3*2^-8 = 0x3F818000 sits between 1 + 2^-7 (0x3F81, odd) and 1 + 2^-6 (0x3F82, even).
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x3F81_8000)).to_bits(),
        0x3F82
    );
    // 257 = 0x43808000 sits between 256 (0x4380, even) and 258 (0x4381, odd).
    assert_eq!(BFloat16::round_from_f32(257.0).to_bits(), 0x4380);
    // 259 = 0x43818000 sits between 258 (0x4381, odd) and 260 (0x4382, even).
    assert_eq!(BFloat16::round_from_f32(259.0).to_bits(), 0x4382);
}

#[test]
fn test_round_from_f32_one_ulp_off_a_tie_is_not_a_tie() {
    // One f32 ulp above the tie at 1 + 2^-8 rounds up; one below rounds down.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x3F80_8001)).to_bits(),
        0x3F81
    );
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x3F80_7FFF)).to_bits(),
        0x3F80
    );
}

#[test]
fn test_round_from_f32_preserves_signed_zero() {
    // half-rs 1.0.2 fixed the sign of zero; both zeros must survive with their sign.
    assert_eq!(BFloat16::round_from_f32(0.0).to_bits(), 0x0000);
    assert_eq!(BFloat16::round_from_f32(-0.0).to_bits(), 0x8000);
}

#[test]
fn test_round_from_f32_preserves_infinities() {
    assert_eq!(BFloat16::round_from_f32(f32::INFINITY).to_bits(), 0x7F80);
    assert_eq!(
        BFloat16::round_from_f32(f32::NEG_INFINITY).to_bits(),
        0xFF80
    );
}

#[test]
fn test_round_from_f32_overflows_to_infinity_at_the_ieee_threshold() {
    // MAX = 0x7F7F0000. The ulp there is 0x00010000, so MAX + half an ulp is 0x7F7F8000, the
    // first value that rounds to infinity (a tie whose even neighbour is the exponent above).
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x7F7F_8000)).to_bits(),
        0x7F80
    );
    // One f32 ulp below the threshold still rounds down to MAX.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x7F7F_7FFF)).to_bits(),
        0x7F7F
    );
    assert_eq!(BFloat16::round_from_f32(f32::MAX).to_bits(), 0x7F80);
    assert_eq!(BFloat16::round_from_f32(f32::MIN).to_bits(), 0xFF80);
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0xFF7F_8000)).to_bits(),
        0xFF80
    );
}

#[test]
fn test_round_from_f32_keeps_subnormals() {
    // half-rs 1.1.1 fixed subnormal conversions in the software path.
    // 2^-127 = 0x00400000 is subnormal in bf16 (the smallest normal is 2^-126 = 0x0080).
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x0040_0000)).to_bits(),
        0x0040
    );
    // The smallest bf16 subnormal, 2^-133 = 0x00010000.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x0001_0000)).to_bits(),
        0x0001
    );
    // The smallest f32 subnormal, 2^-149, is far below half of 2^-133 and rounds to zero.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x0000_0001)).to_bits(),
        0x0000
    );
    // 2^-134 = 0x00008000 is the exact tie between 0 (even) and 2^-133 (odd).
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x0000_8000)).to_bits(),
        0x0000
    );
    // One f32 ulp above that tie rounds to the smallest subnormal.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x0000_8001)).to_bits(),
        0x0001
    );
}

#[test]
fn test_round_from_f32_at_the_subnormal_boundary() {
    // The largest subnormal is 0x007F; half an ulp above it, 0x007F8000, is the tie with the
    // smallest normal 0x0080 (even), so the tie rounds up into the normal range.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x007F_8000)).to_bits(),
        0x0080
    );
    // One f32 ulp below the tie stays the largest subnormal.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x007F_7FFF)).to_bits(),
        0x007F
    );
}

#[test]
fn test_round_from_f32_nan_stays_nan_even_when_the_payload_is_only_in_the_low_half() {
    // 0x7F800001 is a NaN whose payload lives entirely in the bits truncation discards; plain
    // truncation would yield 0x7F80, which is +infinity. half-rs 1.0.2 fixed exactly this.
    let quieted = BFloat16::round_from_f32(f32::from_bits(0x7F80_0001));
    assert!(quieted.is_nan());
    assert_eq!(quieted.to_bits(), 0x7FC0);

    // half-rs 1.4.0: the sign of a NaN survives conversion, as it does for f32 -> f64 in std.
    let negative = BFloat16::round_from_f32(f32::from_bits(0xFF80_0001));
    assert!(negative.is_nan());
    assert!(negative.is_sign_negative());
    assert_eq!(negative.to_bits(), 0xFFC0);
}

#[test]
fn test_round_from_f32_nan_keeps_the_high_payload_bits_and_sets_the_quiet_bit() {
    // Quiet NaN 0x7FEA1234: the top half 0x7FEA already has the quiet bit 0x0040 set.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x7FEA_1234)).to_bits(),
        0x7FEA
    );
    // Signalling NaN 0x7FAA0000: the quiet bit is forced on, 0x7FAA | 0x0040 = 0x7FEA.
    assert_eq!(
        BFloat16::round_from_f32(f32::from_bits(0x7FAA_0000)).to_bits(),
        0x7FEA
    );
}

#[test]
fn test_round_from_f32_is_the_identity_on_every_representable_value() {
    for bits in 0..=u16::MAX {
        let x = BFloat16::from_bits(bits);
        let back = BFloat16::round_from_f32(x.to_f32());
        if x.is_nan() {
            // A bf16 NaN comes back with the same sign and payload, quieted.
            assert_eq!(back.to_bits(), bits | 0x0040, "bits {bits:#06x}");
        } else {
            assert_eq!(back.to_bits(), bits, "bits {bits:#06x}");
        }
    }
}

#[test]
fn test_round_from_f32_agrees_with_the_oracle_on_a_generated_family() {
    // Every f32 pattern is an exact f64, so the oracle's f64 rounding is the expected value.
    let mut lcg = Lcg(0x5EED_BF16_0000_0001);
    for _ in 0..200_000 {
        let pattern = (lcg.next_u64() >> 32) as u32;
        let x = f32::from_bits(pattern);
        if x.is_nan() {
            continue;
        }
        assert_eq!(
            BFloat16::round_from_f32(x).to_bits(),
            oracle_round_to_nearest_even(x as f64),
            "f32 pattern {pattern:#010x}"
        );
    }
}

#[test]
fn test_round_from_f32_agrees_with_the_oracle_around_every_bf16_midpoint() {
    // For every finite bf16 value v, the f32 patterns v + 0x7FFF, v + 0x8000 and v + 0x8001
    // are one ulp below the tie, the tie, and one ulp above it.
    for bits in 0..=u16::MAX {
        if bits & 0x7F80 == 0x7F80 {
            continue;
        }
        let base = (bits as u32) << 16;
        for offset in [0x7FFFu32, 0x8000, 0x8001, 0x0001, 0xFFFF] {
            let pattern = base | offset;
            let x = f32::from_bits(pattern);
            assert_eq!(
                BFloat16::round_from_f32(x).to_bits(),
                oracle_round_to_nearest_even(x as f64),
                "f32 pattern {pattern:#010x}"
            );
        }
    }
}

// =============================================================================
// f64 -> bf16: correctly rounded, not rounded twice
// =============================================================================

#[test]
fn test_round_from_f64_exact_values_pass_through() {
    assert_eq!(BFloat16::round_from_f64(1.0).to_bits(), 0x3F80);
    assert_eq!(BFloat16::round_from_f64(-0.0).to_bits(), 0x8000);
    assert_eq!(BFloat16::round_from_f64(1000.0).to_bits(), 0x447A);
    // 1/3 in f64 rounds to the same bf16 as in f32: 0x3EAB.
    assert_eq!(BFloat16::round_from_f64(1.0 / 3.0).to_bits(), 0x3EAB);
}

#[test]
fn test_round_from_f64_does_not_round_twice_above_a_tie() {
    // half-rs #151: inputs just above a rounding tie misround when the software path goes
    // through a narrower intermediate. 1 + 2^-8 + 2^-30 lies just above the bf16 tie at
    // 1 + 2^-8; rounding to f32 first lands exactly on the tie and the tie-to-even rule then
    // gives 1.0. The correct answer is the value above the tie, 1 + 2^-7 = 0x3F81.
    let x = 1.0 + 2f64.powi(-8) + 2f64.powi(-30);
    assert_eq!(BFloat16::round_from_f64(x).to_bits(), 0x3F81);
}

#[test]
fn test_round_from_f64_does_not_round_twice_below_a_tie() {
    // 1 + 3*2^-8 - 2^-30 lies just below the tie at 1 + 3*2^-8, whose even neighbour is above
    // it; a first rounding to f32 would land on the tie and then round up to 0x3F82.
    let x = 1.0 + 3.0 * 2f64.powi(-8) - 2f64.powi(-30);
    assert_eq!(BFloat16::round_from_f64(x).to_bits(), 0x3F81);
}

#[test]
fn test_round_from_f64_exact_ties_still_go_to_even() {
    assert_eq!(
        BFloat16::round_from_f64(1.0 + 2f64.powi(-8)).to_bits(),
        0x3F80
    );
    assert_eq!(
        BFloat16::round_from_f64(1.0 + 3.0 * 2f64.powi(-8)).to_bits(),
        0x3F82
    );
    assert_eq!(BFloat16::round_from_f64(259.0).to_bits(), 0x4382);
}

#[test]
fn test_round_from_f64_at_the_top_of_a_binade() {
    // half-rs #116, second case: a value just below the tie at the top of a binade must not be
    // carried into the next power of two. 255.5 is the tie between 255 (0x437F, odd) and 256
    // (0x4380, even); 255.5 - 2^-40 is below it and rounds to 255.
    assert_eq!(
        BFloat16::round_from_f64(255.5 - 2f64.powi(-40)).to_bits(),
        0x437F
    );
    assert_eq!(BFloat16::round_from_f64(255.5).to_bits(), 0x4380);
}

#[test]
fn test_round_from_f64_at_the_subnormal_boundary() {
    // half-rs #116, first case: a value just below the tie between the largest subnormal and
    // the smallest normal must stay subnormal. The tie is 2^-126 - 2^-134; 2^-160 below it
    // rounds to the largest subnormal 0x007F, and the tie itself rounds to the even 0x0080.
    let tie = 2f64.powi(-126) - 2f64.powi(-134);
    assert_eq!(
        BFloat16::round_from_f64(tie - 2f64.powi(-160)).to_bits(),
        0x007F
    );
    assert_eq!(BFloat16::round_from_f64(tie).to_bits(), 0x0080);
}

#[test]
fn test_round_from_f64_handles_the_subnormal_tie_and_its_neighbours() {
    // 2^-134 is the exact tie between 0 and the smallest subnormal 2^-133.
    assert_eq!(BFloat16::round_from_f64(2f64.powi(-134)).to_bits(), 0x0000);
    // Just above the tie the smallest subnormal wins (half-rs PR #145's trailing-bit case).
    assert_eq!(
        BFloat16::round_from_f64(2f64.powi(-134) + 2f64.powi(-160)).to_bits(),
        0x0001
    );
    // 1e-40 lies between 2^-133 = 9.18e-41 and 2^-132 = 1.84e-40, nearer the former.
    assert_eq!(BFloat16::round_from_f64(1e-40).to_bits(), 0x0001);
    // Below half the smallest subnormal the value is zero with its sign kept.
    assert_eq!(BFloat16::round_from_f64(1e-50).to_bits(), 0x0000);
    assert_eq!(BFloat16::round_from_f64(-1e-50).to_bits(), 0x8000);
}

#[test]
fn test_round_from_f64_overflow_and_non_finite() {
    assert_eq!(BFloat16::round_from_f64(f64::MAX).to_bits(), 0x7F80);
    assert_eq!(BFloat16::round_from_f64(f64::MIN).to_bits(), 0xFF80);
    assert_eq!(BFloat16::round_from_f64(1e39).to_bits(), 0x7F80);
    assert_eq!(BFloat16::round_from_f64(f64::INFINITY).to_bits(), 0x7F80);
    assert_eq!(
        BFloat16::round_from_f64(f64::NEG_INFINITY).to_bits(),
        0xFF80
    );
    // f32::MAX as an f64 is beyond the bf16 overflow threshold 0x7F7F8000.
    assert_eq!(BFloat16::round_from_f64(f32::MAX as f64).to_bits(), 0x7F80);
    // Half an f64 ulp of MAX below the threshold still rounds down.
    let threshold = f32::from_bits(0x7F7F_8000) as f64;
    assert_eq!(
        BFloat16::round_from_f64(threshold - 2f64.powi(75)).to_bits(),
        0x7F7F
    );
    assert_eq!(BFloat16::round_from_f64(threshold).to_bits(), 0x7F80);
}

#[test]
fn test_round_from_f64_nan_keeps_sign_and_high_payload_and_is_quieted() {
    // An f64 NaN's payload starts at bit 51; the top six bits land in the bf16 significand.
    // 0x7FF8_0000_0000_0000 is the quiet NaN with an empty payload: 0x7FC0.
    assert_eq!(
        BFloat16::round_from_f64(f64::from_bits(0x7FF8_0000_0000_0000)).to_bits(),
        0x7FC0
    );
    // A signalling NaN with payload only in the low bits must not become an infinity.
    let low_payload = BFloat16::round_from_f64(f64::from_bits(0x7FF0_0000_0000_0001));
    assert!(low_payload.is_nan());
    assert_eq!(low_payload.to_bits(), 0x7FC0);
    // Sign and the top significand bits survive. 0xFFF5_4000_0000_0000 has sign 1 and f64
    // significand 0x5_4000_0000_0000, whose top seven bits (51..45) are 0101010b = 0x2A;
    // quieting sets bit 6, giving 0x6A, so the result is 0x8000 | 0x7F80 | 0x6A = 0xFFEA.
    let negative = BFloat16::round_from_f64(f64::from_bits(0xFFF5_4000_0000_0000));
    assert!(negative.is_nan());
    assert!(negative.is_sign_negative());
    assert_eq!(negative.to_bits(), 0xFFEA);
    // A quiet NaN whose top seven significand bits are 1010101b = 0x55 keeps all of them.
    assert_eq!(
        BFloat16::round_from_f64(f64::from_bits(0x7FFA_A000_0000_0000)).to_bits(),
        0x7FD5
    );
}

#[test]
fn test_round_from_f64_is_the_identity_on_every_representable_value() {
    for bits in 0..=u16::MAX {
        let x = BFloat16::from_bits(bits);
        let back = BFloat16::round_from_f64(x.to_f64());
        if x.is_nan() {
            // `to_f64` widens through an `f32 -> f64` cast, and Rust does not promise which
            // payload a cast NaN carries, so only NaN-ness is asserted on this path. The kernel's
            // own NaN handling is pinned bit for bit by
            // `test_round_from_f64_nan_keeps_sign_and_high_payload_and_is_quieted`.
            assert!(back.is_nan(), "bits {bits:#06x}");
        } else {
            assert_eq!(back.to_bits(), bits, "bits {bits:#06x}");
        }
    }
}

#[test]
fn test_round_from_f64_agrees_with_the_oracle_on_a_generated_family() {
    let mut lcg = Lcg(0xBF16_5EED_0000_0002);
    for _ in 0..200_000 {
        let pattern = lcg.next_u64();
        let x = f64::from_bits(pattern);
        if x.is_nan() {
            continue;
        }
        assert_eq!(
            BFloat16::round_from_f64(x).to_bits(),
            oracle_round_to_nearest_even(x),
            "f64 pattern {pattern:#018x}"
        );
    }
}

#[test]
fn test_round_from_f64_agrees_with_the_oracle_around_every_bf16_midpoint() {
    // For every finite bf16 value v with successor s, the midpoint m = (v + s) / 2 is exact in
    // f64. The neighbours m ± 2^-40·ulp are the inputs a doubly rounding path misjudges.
    for bits in 0..u16::MAX {
        if bits & 0x7F80 == 0x7F80 || bits & 0x8000 != 0 {
            continue;
        }
        let v = BFloat16::from_bits(bits).to_f64();
        let s = BFloat16::from_bits(bits + 1).to_f64();
        if !s.is_finite() {
            continue;
        }
        let ulp = s - v;
        let m = v + ulp / 2.0;
        // The last two inputs sit 0.7 of an f32 ulp below and above the f32 values that neighbour
        // the midpoint, so the nearest f32 to each has an odd significand and must be returned
        // unchanged by round to odd; nudging it would land exactly on the tie.
        let f32_ulp = ulp * 2f64.powi(-16);
        for x in [
            m,
            m - ulp * 2f64.powi(-40),
            m + ulp * 2f64.powi(-40),
            -m,
            m - f32_ulp * 0.7,
            m + f32_ulp * 0.7,
        ] {
            assert_eq!(
                BFloat16::round_from_f64(x).to_bits(),
                oracle_round_to_nearest_even(x),
                "input {x:e} near bf16 {bits:#06x}"
            );
        }
    }
}

// =============================================================================
// Exact (lossless) conversions
// =============================================================================

#[test]
fn test_from_f32_exact_accepts_only_representable_values() {
    // half-rs #90 asks for a conversion that never rounds. 0.375 has three significant bits.
    assert_eq!(
        BFloat16::from_f32_exact(0.375).map(|x| x.to_bits()),
        Some(0x3EC0)
    );
    assert_eq!(
        BFloat16::from_f32_exact(-0.0).map(|x| x.to_bits()),
        Some(0x8000)
    );
    assert_eq!(
        BFloat16::from_f32_exact(f32::INFINITY),
        Some(BFloat16::INFINITY)
    );
    // The smallest subnormal is representable; half of it is not.
    assert_eq!(
        BFloat16::from_f32_exact(f32::from_bits(0x0001_0000)).map(|x| x.to_bits()),
        Some(0x0001)
    );
    assert_eq!(BFloat16::from_f32_exact(f32::from_bits(0x0000_8000)), None);
    // 0.1 and 257 carry more than 8 significant bits.
    assert_eq!(BFloat16::from_f32_exact(0.1), None);
    assert_eq!(BFloat16::from_f32_exact(257.0), None);
    // A NaN never round-trips to an equal value, so it is not lossless.
    assert_eq!(BFloat16::from_f32_exact(f32::NAN), None);
}

#[test]
fn test_from_f64_exact_accepts_only_representable_values() {
    assert_eq!(
        BFloat16::from_f64_exact(3.140625).map(|x| x.to_bits()),
        Some(0x4049)
    );
    assert_eq!(
        BFloat16::from_f64_exact(2f64.powi(-133)).map(|x| x.to_bits()),
        Some(0x0001)
    );
    assert_eq!(BFloat16::from_f64_exact(2f64.powi(-134)), None);
    assert_eq!(BFloat16::from_f64_exact(1e-50), None);
    assert_eq!(BFloat16::from_f64_exact(1e39), None);
    assert_eq!(BFloat16::from_f64_exact(core::f64::consts::PI), None);
    assert_eq!(BFloat16::from_f64_exact(f64::NAN), None);
    assert_eq!(
        BFloat16::from_f64_exact(f64::NEG_INFINITY),
        Some(BFloat16::NEG_INFINITY)
    );
}

#[test]
fn test_exact_conversions_round_trip_every_non_nan_pattern() {
    for bits in 0..=u16::MAX {
        let x = BFloat16::from_bits(bits);
        if x.is_nan() {
            assert_eq!(BFloat16::from_f32_exact(x.to_f32()), None);
            continue;
        }
        assert_eq!(
            BFloat16::from_f32_exact(x.to_f32()),
            Some(x),
            "bits {bits:#06x}"
        );
        assert_eq!(
            BFloat16::from_f64_exact(x.to_f64()),
            Some(x),
            "bits {bits:#06x}"
        );
    }
}
