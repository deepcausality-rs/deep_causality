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
4. The numerator is never `T::MIN`.

The first three make the representation unique, and therefore make equality **structural** — two
rationals are equal exactly when their components match, with no cross-multiplication.

The fourth is about the machine rather than about ℚ. `T::MIN` has no representable negation —
`-i64::MIN` is `2⁶³`, one past the top of the range — so a numerator holding it would make `Neg`
partial, and with it subtraction and the `AbelianGroup` law `a + (-a) = 0`. Construction refuses
that one value per width instead, which costs `Rational::<i64>::from_integer(i64::MIN)` and buys a
negation that cannot fail. The represented set stays symmetric, so the field laws are untouched.

Ordering does **not** cross-multiply. `a·d` against `c·b` overflows for large scalars, and in
release wraps rather than aborting: `i64::MAX/1` would compare as *less than* `1/2`. `Ord` instead
descends the continued fraction — compare the floor quotients, then recurse on the remainders —
which is exact, total, and multiplies nothing.

## Where it sits in the tower

`Rational<T>` is built over `T: EuclideanDomain + SignedInt`. The Euclidean domain is what
supplies the `gcd` that reduction to lowest terms needs; `SignedInt` is what supplies the rest of
canonical form — a **total** order, because `Ord` has no way to answer "incomparable"; a sign,
because invariant 1 moves it into the numerator; and the end of the range, because `T::MIN` is the
one value whose negation does not fit and the difference between an honest type and a wrong one is
whether that is detected. Every signed width qualifies: `i8`–`i128`, `isize`.

It is a [`Field`](../deep_causality_algebra/README.md) and deliberately **not** a `Real`. `Real` is
the analytic axis — `sqrt`, `exp`, `ln`, `sin` — and ℚ is closed under none of them. ℚ is
arithmetically complete and analytically empty, the exact mirror of `Dual`, which is analytic
without being a field.

This is also the step that ℤ cannot take on its own. Integer `/` truncates rather than inverts, so
the integers stop at `CommutativeRing`; constructing ℚ is precisely the act of supplying the
missing inverses.

## Overflow

`T` is fixed-width, so not every exact result is representable. That much is inherent. Which
operations can fail is not, so here it is exactly.

**Total — cannot overflow, on any input:**

- **Construction.** `try_new` returns a correct value or `None`, never a wrong one. It reduces by
  the gcd *before* moving a sign out of the denominator, so `try_new(i64::MIN, -2)` is `2⁶²`
  rather than an overflow on an intermediate `-i64::MIN` that was never needed.
- **Comparison.** `Ord` descends the continued fraction and multiplies nothing.
- **Negation and `recip`.** Guaranteed by invariant 4.

**Partial — can overflow, and here is when:**

- **Addition and subtraction.** Each operand is split into an integer part and a proper fraction,
  `a/b = q + r/b`, and the two are carried separately, so a sum with a large integer part no
  longer overflows on the numerator: `MAX/2 + MAX/2` is exactly `MAX`, where forming `MAX + MAX`
  would not fit. It overflows when `lcm(b, d)` does not fit, when `q₁ + q₂` does not, or when the
  fractional numerator — bounded by `2·lcm(b, d)` — does not. Two coprime denominators near
  `√T::MAX` still overflow, and so does any chain whose *intermediate* needs a numerator wider
  than `T`, even where the final answer would fit.
- **Multiplication and division.** Each numerator is cross-cancelled against the other's
  denominator before either product is formed, so `2/3 · 3/2` never builds `6/6`. What survives
  the cancelling must fit.

When an operation does overflow, the behaviour is `T`'s — a panic in debug builds, wrapping in
release — except where the type can see the failure itself, which is when it panics with a message
naming the cause. Choose `T` with headroom (`i128` where denominators are unpredictable), and
prefer accumulating over a common denominator where the problem admits one.

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
