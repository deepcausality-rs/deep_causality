/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Test module for the `BFloat16` type.
//!
//! Every file here is bit-exact integer work or `f32` arithmetic on values the type represents
//! exactly, so none of it is gated off under Miri. The `Float` trait implementation is tested in
//! `tests/float/bfloat16_impl_tests.rs`, beside the `f32` and `f64` implementations.
//!
//! # Where the expected values come from
//!
//! The suite follows `openspec/changes/unified-math-next/tdd/`. No expected value is produced by
//! the code under test. Each bit pattern is derived by hand from the format — 8 significant bits,
//! the binary32 exponent range, subnormals kept, round to nearest with ties to even — and the
//! derivation stands in a comment beside the assertion. `bfloat16_tests.rs` also carries an
//! independent oracle, `oracle_round_to_nearest_even`, that rounds an `f64` by integer arithmetic
//! on its fields, and checks the implementation against it over generated families.
//!
//! # Corner-case enumeration
//!
//! | # | Class | Test |
//! |---|---|---|
//! | A | Empty input | `traits_num_tests::test_sum_and_product_of_nothing_are_the_identities` |
//! | B | Single element | `traits_num_tests::test_sum_and_product_of_one_element_are_that_element` |
//! | C | Two quantities coincide | every `*_ties_go_to_even` test; `traits_num_tests::test_sum_is_not_product`; `ops_comparison_tests::test_signed_zeros_compare_equal` |
//! | D | Index expression degenerates | `traits_num_tests::test_from_integers_at_the_eight_bit_boundary` (the shift is zero at exactly 8 bits and one at 9); `bfloat16_tests::test_round_from_f32_ties_go_to_even` (the tie bias with the kept bit 0 and 1) |
//! | E | Each threshold, both sides | `bfloat16_tests::test_round_from_f32_overflows_to_infinity_at_the_ieee_threshold`, `test_round_from_f32_at_the_subnormal_boundary`, `test_round_from_f64_handles_the_subnormal_tie_and_its_neighbours`; `float/bfloat16_impl_tests::test_round_half_away_from_zero_both_sides` |
//! | F | Zero | `ops_arithmetic_tests::test_div_special_values`, `test_mul_special_values`; `traits_algebra_tests::test_negative_zero_is_zero`; `float/bfloat16_impl_tests::test_sqrt`, `test_ln_and_logs` |
//! | G | Negative | `float/bfloat16_impl_tests::test_sqrt` (NaN), `test_cbrt_is_real_for_negative_input`, `test_powf`, `test_inverse_hyperbolic` |
//! | H | Exact domain boundary | `float/bfloat16_impl_tests::test_inverse_trigonometric_at_the_domain_boundary`, `test_inverse_hyperbolic`; `constants_tests::test_range_constants` |
//! | I | Non-finite | every `*_special_values` test; `ops_comparison_tests::test_nan_is_unordered_and_unequal_to_everything`; `float/bfloat16_impl_tests::test_max_and_min_return_the_other_operand_for_nan` |
//! | J | Overflow and underflow reach | `float/bfloat16_impl_tests::test_intermediates_beyond_the_type_do_not_break_representable_results`, `test_mul_add_is_correctly_rounded` |
//! | K | Every precision-generic path | `traits_num_tests::test_generic_code_runs_at_every_shipped_precision` |
#[cfg(test)]
mod attributes_tests;
#[cfg(test)]
mod bfloat16_tests;
#[cfg(test)]
mod constants_tests;
#[cfg(test)]
mod debug_tests;
#[cfg(test)]
mod display_tests;
#[cfg(test)]
mod from_tests;
#[cfg(test)]
mod getters_tests;
#[cfg(test)]
mod ops_arithmetic_tests;
#[cfg(test)]
mod ops_comparison_tests;
#[cfg(test)]
mod traits_algebra_tests;
#[cfg(test)]
mod traits_num_tests;
