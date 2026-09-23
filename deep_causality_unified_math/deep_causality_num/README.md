[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DeepCausality NUM types and traits

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_num

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_num/latest/deep_causality_num/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

## Summary

Numerical foundation for the [DeepCausality project](http://www.deepcausality.com):

- **Cast Traits:** Safe primitive type conversions (`AsPrimitive`, `FromPrimitive`, `ToPrimitive`, `NumCast`)
- **Lift Utilities:** The precision-boundary crossings for a program written against a `FloatType` alias: `lift` for an `f64` literal, `lift_count` for a `u64`, a `lift_<primitive>` for every primitive float and integer, `lower` back to `f64`, and `to_count` back to a rounded `u64`
- **Identity Traits:** Zero and One with const variants for compile-time evaluation
- **Float Types:** Standard floating-point abstractions plus `Float106` for double-double precision arithmetic
- **Integer Traits:** Type-safe abstractions over the primitive integer types

The implementation is **unsafe-free** and **dependency-free** (with optional `libm` for no-std float support). It
compiles for std, no-std, and no-std without float.

Related crates:

- [`deep_causality_algebra`](../deep_causality_algebra/README.md): the algebra trait tower (Magma → Group → Ring →
  Field), the scalar traits, and the isomorphism markers.
- [`deep_causality_num_complex`](../deep_causality_num_complex/README.md): `Complex`, `Quaternion`, and `Octonion`.
- [`deep_causality_num_dual`](../deep_causality_num_dual/README.md): the `Dual` number for forward-mode autodiff.
- [`deep_causality_num_rational`](../deep_causality_num_rational/README.md): the `Rational` number, exact ℚ arithmetic.

## The five number systems

The workspace covers ℕ, ℤ, ℚ, ℝ, and ℂ, each split across two layers:

- **This crate** holds the *representation* traits: abstractions over Rust's own primitives, concerned with what a
  machine number can do.
- **`deep_causality_algebra`** holds the *algebraic* traits: the laws a number obeys, independent of how it is stored.

A number system gets a crate of its own only when it introduces a **type Rust does not have**. ℂ and ℚ have crates;
ℕ, ℤ, and ℝ do not, because `u64`, `i64`, and `f64` already exist.

Each system has two names: the **set name** you write in code, and the **algebraic name** that
says what structure it has. The table lists both and where each lives.

| Set | Set name | Where | Algebraic name | Where | Concrete types |
|-----|----------|-------|----------------|-------|----------------|
| **ℕ** naturals | `NaturalNumber` | `deep_causality_num` | `CommutativeSemiring` | `deep_causality_algebra` | `u8`–`u128`, `usize` |
| **ℤ** integers | `Integer` | `deep_causality_num` | `EuclideanDomain` | `deep_causality_algebra` | `i8`–`i128`, `isize` |
| **ℚ** rationals | `Rational<T>` | [`deep_causality_num_rational`](../deep_causality_num_rational/README.md) | `Field` | `deep_causality_algebra` | `Rational<i64>`, … |
| **ℝ** reals | `Real` | `deep_causality_algebra` | `RealField` | `deep_causality_algebra` | `f32`, `f64`, `Float106` |
| **ℂ** complex | `Complex<T>` | [`deep_causality_num_complex`](../deep_causality_num_complex/README.md) | `ComplexField` | `deep_causality_algebra` | `Complex<f64>`, … |

Use the **set name** by default: `NaturalNumber`, `Integer`, `Rational`, `Real`, and `Complex`
read the way the mathematics is spoken. Use the **algebraic name** when the code needs a specific
structure rather than a specific set; a single `Field` bound lets one function serve ℚ, ℝ, and ℂ.

Two supporting traits sit beside `Integer` on the ℤ row and describe the *representation* rather
than the set: `SignedInt` (`abs`, `signum`, `checked_neg`) and `UnsignedInt` (`is_power_of_two`,
`next_power_of_two`). `NaturalNumber` builds on `UnsignedInt` and adds ℕ's own vocabulary:
`succ`, `pred`, `monus`, `checked_difference`, `div_rem`, `gcd`, `lcm`.

The tower stops each system where the mathematics does:

- **ℕ is not a ring.** The unsigned types are a `CommutativeSemiring`, but `3u64 - 5u64` has no value in ℕ, so there are
  no additive inverses. They stop before `AbelianGroup`, and therefore before `Ring`. `NaturalNumber` reflects that in
  its API: subtraction appears as `checked_difference` (returning `None`) and `monus` (truncating to zero), never as the
  `Sub` operator, which wraps or panics rather than reporting the absence.
- **ℤ is not a field.** Integer `/` is a truncating quotient: `1 / 5` is `0`, so `5 * (1 / 5)` is `0` rather than
  `1`. The integers lack the `Invertible` marker, which stops `CommutativeRing` from becoming `Field`. ℚ supplies the
  missing inverses.
- **ℚ is not a `Real`.** `Real` is the *analytic* axis (`sqrt`, `exp`, `ln`, `sin`), and ℚ is closed under none of them.
  ℚ is arithmetically complete and analytically empty; ℝ is the completion that fixes it.
- **ℍ and 𝕆 are neither.** `Quaternion` is not `Commutative`; `Octonion` is not `Associative` either. Both are still
  `DivisionAlgebra`, so they sit beside ℂ rather than under it.

Two further types round out the tower without being number systems in their own right: `Float106`
(double-double, ~31 digits) extends ℝ's precision, and `Dual` ([crate](../deep_causality_num_dual/README.md)) is
analytic without being a field, the exact mirror of ℚ being a field without being analytic.

Every claim above is machine-checked. See [`LEAN_NUM.md`](LEAN_NUM.md),
[`LEAN_ALGEBRA.md`](../deep_causality_algebra/LEAN_ALGEBRA.md),
[`LEAN_NUM_COMPLEX.md`](../deep_causality_num_complex/LEAN_NUM_COMPLEX.md),
[`LEAN_NUM_DUAL.md`](../deep_causality_num_dual/LEAN_NUM_DUAL.md), and
[`LEAN_NUM_RATIONAL.md`](../deep_causality_num_rational/LEAN_NUM_RATIONAL.md).

### Choosing a working type

Write numeric code against the algebraic bound, never a concrete width, and name a concrete type only through a
single alias. `FloatType = f64` is the precision parameter for ℝ; `IntType = i64` is the range parameter for ℤ. The two
knobs differ. Widening `FloatType` buys **accuracy**; its failure mode is rounding, bounded by `epsilon()`. Widening
`IntType` buys **headroom**; its failure mode is overflow, a hard error with no analogue of `epsilon()`.

### Integer Traits

Type-safe abstractions over Rust's primitive integer types:

| Trait           | Covers               | Key Operations                                                      |
|-----------------|----------------------|---------------------------------------------------------------------|
| **Integer**     | All primitives       | Bit ops, checked/saturating/wrapping arithmetic, Euclidean division |
| **SignedInt**   | `i8`–`i128`, `isize` | `abs`, `signum`, `is_negative`, `checked_neg`                       |
| **UnsignedInt** | `u8`–`u128`, `usize` | `is_power_of_two`, `next_power_of_two`                              |

These are representation traits: they say what the machine integer can do, not what laws it obeys. The algebraic side
lives in [`deep_causality_algebra`](../deep_causality_algebra/README.md), where the signed types implement
`EuclideanDomain` (φ, `div_euclid`, `rem_euclid`, `gcd`, `lcm`) and so reach `CommutativeRing`, while the unsigned types
stop at `CommutativeSemiring`. `EuclideanDomain` is a *ring* structure and cannot serve ℕ, so `NaturalNumber` carries its
own `gcd` and `lcm`.

### Float Types

| Type            | Description                                                | Key Traits                          |
|-----------------|------------------------------------------------------------|-------------------------------------|
| **Float**       | Trait for `f32` and `f64`                                  | `Float`, `Num`                      |
| **Float106**    | High-precision (~31 digits) using double-double arithmetic | `Float`, `Num`                      |
| **BFloat16**    | 16-bit brain float: the top 16 bits of an `f32`            | `Float`, `Num`                      |

The real fields (`f32`, `f64`, `BFloat16`, `Float106`) also implement the full algebra tower (`RealField`, `Field`, `Scalar`, and
the rest); those trait implementations live in [`deep_causality_algebra`](../deep_causality_algebra/README.md).

#### Float106 vs f128 Comparison

| Aspect           | Float106                   | f128 (IEEE binary128)                                                     |
|------------------|----------------------------|---------------------------------------------------------------------------|
| Mantissa         | 106 bits                   | 112 bits                                                                  |
| Precision        | ~32 decimal digits (10⁻³¹) | ~34 decimal digits (10⁻³⁴)                                                |
| Speed            | ~2-4× slower than f64      | ~10-100× slower (software emulated)                                       |
| Hardware support | None (pure software)       | Very rare (POWER9, some ARMs)                                             |
| Rust status      | **Available now**          | Nightly only ([#116909](https://github.com/rust-lang/rust/issues/116909)) |

**Physical scale context:**

| Type         | Precision      | Scale              | Physical Reference    |
|--------------|----------------|--------------------|-----------------------|
| f64          | ~15 digits     | 10⁻¹⁵ (femto)      | Proton size           |
| **Float106** | **~32 digits** | **10⁻³¹ (quecto)** | Near Planck length    |
| f128         | ~34 digits     | 10⁻³⁴              | Planck length (10⁻³⁵) |

Float106 comes close to f128 precision and runs faster on most hardware,
because it uses native f64 FMA operations.

### Numerical Traits:

**Cast Traits:**

* AsPrimitive
* FloatAsScalar
* IntAsScalar
* FromPrimitive
* ToPrimitive
* NumCast
* IntoFloat

**General traits:**

* Num
* NumOps

**Identity traits:**

* One / ConstOne
* Zero / ConstZero

## non-std support

The crate supports `no-std` environments such as embedded systems. The `std` feature is on by default, so opt into
`no-std` via feature flags: disable the default `std` feature and, if your application needs floating-point operations,
enable the `libm_math` feature. `libm_math` pulls in the `libm` crate, which provides software implementations of
floating-point math functions.

### Cargo Build and Test for `no-std`

**1. Building for `no-std` with Floating-Point Math:**

To build for `no-std` with floating-point math (via `libm`):

```bash
cargo build --no-default-features --features libm_math -p deep_causality_num
```

**2. Testing for `no-std` with Floating-Point Math:**

To test for `no-std` with floating-point math:

```bash
cargo test --no-default-features --features libm_math -p deep_causality_num
```

Small floating-point differences between the std and non-std implementations can make some tests fail. If you hit
one, please submit a PR with a fix.

**3. Building for `no-std` without Floating-Point Math (if not needed):**

Without floating-point operations, build without the `libm_math` feature:

```bash
cargo build --no-default-features -p deep_causality_num
```

**4. Testing for `no-std` without Floating-Point Math (if not needed):**

To test without floating-point math:

```bash
cargo test --no-default-features -p deep_causality_num
```

About 138 tests fail in this configuration, because they are not gated for no-std without floating-point math, a
corner case. If you need better support for it, please open an issue.

### Bazel Build

For regular (std) builds, run:

```bash
   bazel build //deep_causality_num/...
```

and

```bash
   bazel test //deep_causality_num/...
```

for tests. To build for non-std, use

```bash
   bazel build --@rules_rust//rust/settings:no_std=alloc //deep_causality_num/...
```

and

```bash
   bazel test --@rules_rust//rust/settings:no_std=alloc //deep_causality_num/...
```

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

