<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Replacing circular tests in `deep_causality_num`

## The two classes found

A scan of the 74 test files (13,406 lines) turned up two patterns.

**173 high-word-only assertions**, all in the `float_double` suites, of the form
`assert!((result.hi() - 5.0).abs() < EPSILON)` with `EPSILON` at `1e-14` or `1e-15`. `Float106`
stores a value as two `f64` words and carries roughly 106 bits; reading `hi()` discards the second
word, so an implementation with a wholly wrong low word passes. The archetype was
`test_pi_constant`, which checked `pi.hi()` against `f64::consts::PI` and then only that
`pi.lo() != 0.0` — any non-zero low word satisfied it.

**7 forwarder-versus-source assertions** in `integer_all_types_tests.rs`, of the form
`assert_eq!(Integer::count_ones(x), x.count_ones())`. The trait implementation forwards to the
inherent method, so each compares a value with itself. Each also ran on a single input.

## The oracles used instead

- **Reference values** from mpmath at 60 decimal places, split into the exact `(hi, lo)` `f64`
  pair the type stores. Each argument is given to mpmath as the `f64` the test constructs rather
  than as a decimal literal — an early draft compared `Float106::from(0.05)` against the decimal
  0.05 and measured the `2.8e-18` conversion gap instead of the function, which read as a
  uniform `5e-17` error across `atan`, `asin` and `acos` that was not there.
- **Published constants**, checked in both words: π, e, ln 2 and ln 10 against their decimal
  expansions.
- **Algebraic invariants**: `exp(ln x) = x`, `sin² + cos² = 1`, `cosh + sinh = eˣ`,
  `asin x + acos x = π/2`, `trunc x + fract x = x`, byte reversal as an involution.
- **Exactness**, where the answer is representable: `sqrt(4)`, `cbrt(27)`, `log2(8)`, `ln(1)` and
  every `From` conversion must return a zero low word, not merely a close high word.
- **Hand-derived bit counts** over every bit position, replacing the seven tautologies.

## Defects the new tests exposed

All five were passing before, and all five are fixed in this change.

| Defect | Before | After |
|---|---|---|
| `cbrt` computed `1/3` in `f64` and widened it, capping Newton at `f64` | `1.67e-16`, identical for every input | exact on cubes, `1.0e-32` otherwise |
| `atan` applied its argument reduction once, leaving the series argument near 1 for large `x`, where 80 terms do not converge | `atan(100)` off by `8.1e-4` | exact |
| `asin`/`acos` reach `atan` through a ratio that grows near \|x\|=1, inheriting the above | `4.2e-5` at `x = 0.99875` | `4.1e-31` |
| `ln(+∞)` evaluated `inf + inf/inf − 1` in the Newton step | `NaN` | `+∞` |
| `tanh` on a signed argument: overflows above `x ≈ 355`, and rounds asymmetrically in the last bits | `tanh(400) = NaN`, not odd | `1`, exactly odd |

`atan` also short-circuited to π/4 for any argument within `1e-15` of 1, so `atan(1 + ε)` returned
a value that is not its arctangent. Removing the reduction defect made the shortcut unnecessary.

Two apparent defects were withdrawn after checking: `acos(cos y)` and `atan(tan y)` round-trips
outside the principal branch, and `cosh² − sinh² = 1` at large `x`, are properties of the
identities rather than of the implementations.

## Negative control

Each fix was reverted in turn to confirm the new suite rejects it.

| Reverted | Tests failing | Old suite |
|---|---|---|
| `cbrt` third widened from `f64` | 2 | passed |
| `atan` single halving | 10 | passed |
| `ln(+∞)` guard | 1 | passed |
| `tanh` sign symmetry | 1 | passed |
| π low word corrupted | 3 | passed |
| e low word zeroed | 1 | caught |
| ln 2 low word corrupted | 2 | passed |

The old suite caught one of the seven.

## Tolerance

`TOL = 1e-29`, from the measured worst case of about `5e-31` across every reference table. The
previous `1e-14` admits an answer with no correct low word at all.
