[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DeepCausality NUM types and traits

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_num

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_num/latest/deep_causality_num/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

Custom numerical traits with default implementation and types for
the [DeepCausality project](http://www.deepcausality.com) based on
the [rust-num](https://github.com/rust-num/num-traits) crate. This custom implementation is free of macros,
free of unsafe code, free of external dependencies, and compiles for std, non-std, and non-std without float support.
This crate supports all three complex number systems: Complex, Octonion, and Quaternion, 
which makes it suitable for scientific computing.

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

* One / OneConst
* Zero / Zero Const

### Float Types

* Float - implemented for f32 and f64
* FloatOption - Abstract over float types (`f32`, `f64`) and their `Option` variants.

### Complex Types

This crate implements all three complex numerical types:

* `Complex`
* `Octonion`
* `Quaternion`

## non-std support

The `deep_causality_num` crate provides support for `no-std` environments. This is particularly useful for embedded
systems or other contexts where the standard library is not available. Note, the std feature is enabled by default thus
you need to opt-into non-std via feature flags.

To use this crate in a `no-std` environment, you need to disable the default `std` feature and, if your application
requires floating-point operations, enable the `libm_math` feature. The `libm_math` feature integrates the `libm` crate,
which provides software implementations of floating-point math functions for `no-std`.

### Cargo Build and Test for `no-std`

**1. Building for `no-std` with Floating-Point Math:**

To build the crate for `no-std` while including floating-point math support (via `libm`), use the following command:

```bash
cargo build --no-default-features --features libm_math -p deep_causality_num
```

**2. Testing for `no-std` with Floating-Point Math:**

To run tests in a `no-std` environment with floating-point math support, use:

```bash
cargo test --no-default-features --features libm_math -p deep_causality_num
```

There might be minor floating precision differences between std and non-std implementations that cause some tests to
fail. If you encounter these, please submit a PR with a fix.

**3. Building for `no-std` without Floating-Point Math (if not needed):**

If your `no-std` application does not require floating-point operations,
you can build without the `libm_math` feature:

```bash
cargo build --no-default-features -p deep_causality_num
```

**4. Testing for `no-std` without Floating-Point Math (if not needed):**

Similarly, to test without floating-point math functions:

```bash
cargo test --no-default-features -p deep_causality_num
```

However, this will cause about 138 tests because to fail since these tests are not configured for conditional test run
because non-std without floating-point math is considered a corner case. If you need better support for this particular
scenario, please open an issue.

### Bazel Build

For regular (std) builds, run:

```bash
   bazel build //deep_causality_num/...
```

and

```bash
   bazel test //deep_causality_num/...
```

for tests. When you want to build for non-std, use

```bash
   bazel build --@rules_rust//rust/settings:no_std=alloc //deep_causality_num/...
```

and

```bash
   bazel test --@rules_rust//rust/settings:no_std=alloc //deep_causality_num/...
```

## License

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC