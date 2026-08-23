[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DeepCausality Rational Numbers

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_num_rational

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_num_rational/latest/deep_causality_num_rational/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

## Summary

`Rational<T>` is **ℚ**, the field of fractions of an integral domain. It is exact: `1/3` is the pair
`(1, 3)`, never an approximation of it, so `(1/3) * 3` is exactly `1` and `0.1 + 0.2` is exactly
`0.3` — neither of which a binary float manages.

```rust
use deep_causality_num_rational::Rational;

let third = Rational::new(1_i64, 3);
assert_eq!(third + third + third, Rational::from_integer(1));

assert_eq!(Rational::new(6_i64, 8), Rational::new(3, 4));   // reduced on construction
assert_eq!(*Rational::new(1_i64, -2).numer(), -1);          // sign lives in the numerator
```

## Canonical form

Every value is held in canonical form, which is why the fields are private:

1. The denominator is strictly positive. A sign lives in the numerator, never the denominator.
2. Numerator and denominator are coprime.
3. Zero is exactly `0/1`.

That makes the representation unique, and therefore makes equality **structural** — two rationals
are equal exactly when their components match, with no cross-multiplication. It also lets `Ord`
compare `a/b` against `c/d` as `a·d` against `c·b`, since both denominators are known positive.

## Where it sits in the tower

`Rational<T>` is built over `T: EuclideanDomain`, because reducing to lowest terms needs a `gcd`
and a Euclidean domain is what supplies one. Every signed width qualifies: `i8`–`i128`, `isize`.

It is a [`Field`](../deep_causality_algebra/README.md) and deliberately **not** a `Real`. `Real` is
the analytic axis — `sqrt`, `exp`, `ln`, `sin` — and ℚ is closed under none of them. ℚ is
arithmetically complete and analytically empty, the exact mirror of `Dual`, which is analytic
without being a field.

This is also the step that ℤ cannot take on its own. Integer `/` truncates rather than inverts, so
the integers stop at `CommutativeRing`; constructing ℚ is precisely the act of supplying the
missing inverses.

## Overflow

This is the one sharp edge, and it is inherent to fixed-width rational arithmetic rather than
specific to this implementation. Adding `a/b + c/d` forms `a·d + c·b` over `b·d`, so denominators
grow multiplicatively.

The implementation cancels common factors **before** multiplying rather than after — addition works
over the least common denominator, and multiplication cross-cancels each numerator against the
other's denominator. That delays the problem substantially but cannot remove it: a long chain of
additions with coprime denominators will exhaust any fixed width.

When it does, the behaviour is `T`'s — a panic in debug builds, wrapping in release. Choose `T`
with headroom (`i128` where denominators are unpredictable), and prefer accumulating over a common
denominator where the problem admits one.

## Verification

The field laws, both canonical-form invariants, and density are machine-checked in Lean 4 against
Mathlib's `ℚ`, with one Rust witness test per theorem. See
[`LEAN_NUM_RATIONAL.md`](LEAN_NUM_RATIONAL.md).

## Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).
