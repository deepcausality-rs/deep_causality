<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `BFloat16` (issue #769): TDD record

The record the protocol in `openspec/changes/unified-math-next/tdd/` asks each stage to keep,
for the `BFloat16` type in `deep_causality_num` and its law markers in `deep_causality_algebra`.
Branch `feature/bf16`, 2026-09-04.

## Phase 1: where every expected value came from

No expected value in the suite is produced by the code under test. The sources used, by
allow-list item:

| Source | Where |
|---|---|
| Closed form, written beside the assertion | Every bit pattern: the `f32` pattern of the input, the discarded half-word against `0x8000`, the kept bit's parity; for a transcendental value, the published constant, the grid step `2^(k-7)` in `[2^k, 2^(k+1))`, and the nearest multiple |
| Published value, cited | The IEEE 754 rules for NaN, signed zero, infinities; half-rs issue numbers and changelog versions beside the tests that pin each one |
| Demonstrably different algorithm | `oracle_round_to_nearest_even` in `tests/bfloat16/bfloat16_tests.rs`: integer arithmetic on the `f64` sign, exponent and significand fields, never through `f32`, against the implementation's bias-add kernel and round-to-odd step |
| Algebraic invariant | `from_bits`/`to_bits` and `round_from_*(x.to_*())` are the identity on all 65 536 patterns; `integer_decode` reconstructs every finite value; the constants are nearer the true value than either neighbour; ordering is monotone in the bit pattern for positive finite values |
| Property over a generated family | Two 200 000-sample LCG families (`f32` patterns, `f64` patterns) and the four neighbours of every bf16 midpoint, all against the oracle; every pair in a 144-value window for `+ - * /` against one rounding of the exact `f64` result; `mul_add` over a window where the exact result fits `f64` |

The oracle is itself pinned to hand-derived literals before it judges anything
(`test_oracle_agrees_with_closed_forms`).

## Phase 2: the corner-case enumeration

| # | Class | Test |
|---|---|---|
| A | Empty input | `traits_num_tests::test_sum_and_product_of_nothing_are_the_identities` |
| B | Single element | `traits_num_tests::test_sum_and_product_of_one_element_are_that_element` |
| C | Two distinct quantities coincide | every `*_ties_go_to_even` test; `traits_num_tests::test_sum_is_not_product` (1, 2, 4 sum to 7 and multiply to 8; 1, 2, 3 would not have told the two apart); `ops_comparison_tests::test_signed_zeros_compare_equal` |
| D | An index expression degenerates | `traits_num_tests::test_from_integers_at_the_eight_bit_boundary` (the shift is zero at exactly 8 bits and one at 9); `bfloat16_tests::test_round_from_f32_ties_go_to_even` (the tie bias with the kept bit 0 and 1) |
| E | Each documented threshold, both sides | `bfloat16_tests::test_round_from_f32_overflows_to_infinity_at_the_ieee_threshold`, `test_round_from_f32_at_the_subnormal_boundary`, `test_round_from_f64_at_the_top_of_a_binade`, `test_round_from_f64_handles_the_subnormal_tie_and_its_neighbours`; `float/bfloat16_impl_tests::test_round_half_away_from_zero_both_sides`, `test_clamp` (one step inside each bound) |
| F | Zero | `ops_arithmetic_tests::test_div_special_values`, `test_mul_special_values`; `traits_algebra_tests::test_negative_zero_is_zero`; `float/bfloat16_impl_tests::test_sqrt`, `test_ln_and_logs`, `test_recip` |
| G | Negative | `float/bfloat16_impl_tests::test_sqrt` (NaN), `test_cbrt_is_real_for_negative_input`, `test_powf`, `test_inverse_hyperbolic`, `test_ln_and_logs` |
| H | Exact domain boundary | `float/bfloat16_impl_tests::test_inverse_trigonometric_at_the_domain_boundary`, `test_inverse_hyperbolic`; `constants_tests::test_range_constants`; `traits_num_tests::test_from_wide_integers_round_once_at_the_top_of_the_range` |
| I | Non-finite | every `*_special_values` test; `ops_comparison_tests::test_nan_is_unordered_and_unequal_to_everything`; `float/bfloat16_impl_tests::test_max_and_min_return_the_other_operand_for_nan`, `test_rounding_functions_keep_special_values`; `traits_num_tests::test_sum_and_product_propagate_non_finite_values` |
| J | Overflow and underflow reach | `float/bfloat16_impl_tests::test_intermediates_beyond_the_type_do_not_break_representable_results` (`MAX.mul_add(2, MIN)` is `MAX`, `hypot(MAX, 0)` is `MAX`, `sqrt(MAX)`, `powi` and `exp` at both ends), `test_mul_add_is_correctly_rounded`; `ops_arithmetic_tests::test_mul_rounds_in_the_subnormal_range` |
| K | Every precision-generic path | `traits_num_tests::test_generic_code_runs_at_every_shipped_precision` (one body at `BFloat16`, `f32`, `f64`, `Float106`) |

No input serves two rows: the ties of row C are not the thresholds of row E, and the uniform
sum of row A is not the single element of row B.

## Phase 3: the defect audit

Run by a script that introduces one defect into the real source, runs
`cargo test -p deep_causality_num --test mod`, records the failing tests, and restores the file;
a SHA-256 over every source file before and after confirmed the tree came back byte for byte.
A defect counts as caught only when a failing test's subject is the defective behaviour; that test
is the one named. Two gaps surfaced while the defect list was being written, before the script
ran, and became tests first: `mul_add` with a negative-infinite sum, and inputs beside every
midpoint whose nearest `f32` has an odd significand (the case that separates round to odd from
"always nudge").

| # | Defect introduced | Tests failing | Test whose subject is the defect |
|---|---|---|---|
| 1 | tie bias `0x7FFF` -> `0x8000`: ties always round up | 19 | `test_round_from_f32_ties_go_to_even` |
| 1 | integer path `width <= 8` -> `< 8`: an exactly-8-bit integer takes the rounding branch | 2 | `test_from_integers_at_the_eight_bit_boundary` |
| 2 | integer path `dropped > half` -> `>=`: integer ties always round up | 2 | `test_from_integers_round_to_nearest_even` |
| 2 | integer path `exponent > 127` -> `>=`: 2^127 becomes infinity | 1 | `test_from_wide_integers_round_once_at_the_top_of_the_range` |
| 2 | `is_nan` `>` -> `>=`: infinity classified as NaN | 4 | `test_is_nan` |
| 3 | round to odd steps away from `x` instead of toward it | 15 | `test_round_from_f64_does_not_round_twice_above_a_tie` |
| 3 | integer path drops the sign | 3 | `test_from_small_integers_are_exact` |
| 3 | `abs` keeps the sign bit | 1 | `test_abs` |
| 3 | `copysign` keeps the magnitude's own sign | 1 | `test_copysign` |
| 4 | quiet-NaN bit `0x0040` -> `0x0020` | 4 | `test_round_from_f32_nan_stays_nan_even_when_the_payload_is_only_in_the_low_half` |
| 4 | `integer_decode` exponent bias `127 + 7` -> `127 + 8` | 2 | `test_integer_decode` |
| 4 | f64 NaN keeps six significand bits from bit 46 instead of seven from bit 45 | 1 | `test_round_from_f64_nan_keeps_sign_and_high_payload_and_is_quieted` |
| 5 | integer path never carries a `0x100` significand into the exponent | 1 | `test_from_wide_integers_round_once_at_the_top_of_the_range` |
| 6 | loosened tolerance | n/a | the suite has no tolerances; every assertion is bit-exact or an exact `f32`/`f64` equality |
| 7 | f32 kernel does not quiet a NaN (payload only in the low half becomes infinity) | 3 | `test_round_from_f32_nan_stays_nan_even_when_the_payload_is_only_in_the_low_half` |
| 7 | round to odd never returns an odd nearest unchanged (always nudges) | 1 | `test_round_from_f64_agrees_with_the_oracle_around_every_bf16_midpoint` |
| 8 | `mul_add` drops the non-finite guard | 1 | `test_mul_add_special_values` |
| 8 | `from_f32_exact` drops the exactness check | 1 | `test_from_f32_exact_accepts_only_representable_values` |
| 9 | `mul_add` rounds the f64 sum without the error term (two roundings) | 1 | `test_mul_add_is_correctly_rounded` |
| 9 | `round_from_f64` goes through a plain `as f32` cast (two roundings, half-rs #151) | 10 | `test_round_from_f64_does_not_round_twice_above_a_tie` |
| 9 | `max` returns `self` when `self` is NaN (half-rs #126) | 1 | `test_max_and_min_return_the_other_operand_for_nan` |
| 9 | `is_zero` compares bits, missing `-0.0` | 1 | `test_negative_zero_is_zero` |
| 9 | `Sum` multiplies (half-rs 2.3.0) | 2 | `test_sum_is_not_product` |
| 9 | `PartialOrd` compares bit patterns | 5 | `test_nan_is_unordered_and_unequal_to_everything` |
| 9 | `Neg` returns the input unchanged | 6 | `test_neg_flips_the_sign_bit_only` |

The one defect the suite itself found during development, rather than the audit: the `f64` NaN
branch took six significand bits from bit 46 and then set the quiet bit, counting bit 51 twice.
`test_round_from_f64_nan_keeps_sign_and_high_payload_and_is_quieted` failed on the first run; the
same slip was in the oracle's NaN branch, which no family test exercised because the families skip
NaN inputs. Both were corrected together, and the defect is row 4 above so it stays caught.

## Phase 5: mutation testing

Pending: `cargo mutants -p deep_causality_num --file 'src/bfloat16/*.rs' --file
'src/float/bfloat16_impl.rs' -j 8`; the result is recorded below when the run completes.

## The half-rs inventory

The issue asked that every open and known issue of the reference crate be fixed here.

| half-rs | What it reports | Here |
|---|---|---|
| #151 open | software `f64` conversion misrounds inputs just above a tie | direct `f64` kernel via round to odd; `test_round_from_f64_does_not_round_twice_*`, and every midpoint neighbourhood against the oracle |
| #141 open | `mul_add` computed in `f32` and rounded again | exact `f64` product, `two_sum` error term, round to odd, one final rounding; `test_mul_add_is_correctly_rounded` (259 ∓ 2⁻¹⁰⁰) |
| #116 open | `from_f64` corner cases at the subnormal boundary and the top of a binade | `test_round_from_f64_at_the_subnormal_boundary`, `test_round_from_f64_at_the_top_of_a_binade` |
| #112 open | `{:.1e}` ignores the precision | formatting delegates to `f32` with the caller's formatter; `test_lower_exp_honours_precision_like_f32` pins the case from the report |
| #90 open | lossless `TryFrom` | `from_f32_exact`, `from_f64_exact`; `TryFrom` itself is taken by the blanket over `From`, which rounds |
| #109 open | const conversions | `from_bits`, `to_bits`, `to_f32`, `to_f64`, `round_from_f32`, `round_from_f64` are `const fn`; `test_constructors_and_getters_are_const` |
| #126 closed 2.7.0 | `min`/`max` return NaN when `self` is NaN | IEEE `minNum`/`maxNum` through `f32`; `test_max_and_min_return_the_other_operand_for_nan` |
| 2.3.0 | `Sum` for `bf16` multiplied | `test_sum_is_not_product` |
| 1.4.1 | layout undefined behaviour, fixed by `repr(transparent)` | `repr(transparent)` over `u16`, no `unsafe` anywhere; `test_layout_is_a_bare_u16` |
| 1.4.0 | NaN sign lost in conversion | sign kept on both kernels; `test_round_from_f32_nan_stays_nan_*`, `test_round_from_f64_nan_keeps_sign_*` |
| 1.3.1 | wrong `EPSILON`, `MIN_EXP`, `MAX_EXP`, `MIN_10_EXP`, `MAX_10_EXP`, `NAN` | each derived and pinned in `constants_tests` |
| 1.1.1 | subnormal conversions wrong without intrinsics | `test_round_from_f32_keeps_subnormals`, the subnormal ties and boundaries |
| 1.0.2 | signed-zero comparison; NaN converted to infinity | `test_signed_zeros_compare_equal`; the low-payload NaN tests |
| 2.2.0 | software rounding did not match hardware ties-to-even | the whole `bfloat16_tests` file |
| #152, #144, #124, #120, #95, #85 | `rand_distr` ranges, `rand` version, serde speed, Cranelift, i686 CI, `num_traits::ApproxEq` | not applicable: no `rand`, `serde`, intrinsics or `num-traits` here. #152 is the trap to avoid when `BFloat16` gets `RandFloat`/`SampleUniform` in `deep_causality_rand` |

## Verification

| Check | Result |
|---|---|
| `cargo test -p deep_causality_num` | 3 838 passed (the `float_bfloat16` and `float` suites included) |
| `cargo test -p deep_causality_algebra` | 330 passed |
| `cargo clippy --all-targets --all-features -- -D warnings`, both crates | clean |
| `cargo build --no-default-features --features libm_math -p deep_causality_num` | builds |
| `cargo check --workspace` | passes |
| `bazel test` on `num:bfloat16`, `num:float`, `algebra:bfloat16` | 17 of 17 pass |
| `buildifier -mode=check -lint=warn` on both `BUILD.bazel` files | clean |
| Defect audit, 24 defects | every one caught by a subject test; tree restored byte for byte |

The `no-std` build without `libm_math` fails on `main` before this change with 68 errors; it is
not a target of this work and is unchanged.
