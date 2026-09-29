[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DeepCausality Dual Numbers

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_num_dual

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_num_dual/latest/deep_causality_num_dual/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

## Summary

The `Dual` number type for the [DeepCausality project](http://www.deepcausality.com). A dual number carries a value and
its first derivative, so evaluating a function on a `Dual` computes the function and its derivative in one pass:
forward-mode automatic differentiation in the number itself.

`Dual` is generic over its real base. It implements the ordered scalar traits from `deep_causality_algebra`, including
`Real`, `Scalar`, and `ConjugateScalar`, so one generic model runs at `f64` for the value and at `Dual` for the
derivative. Because `Dual` is itself a `Real`, the type nests: `Dual<Dual<T>>` yields second derivatives.

This crate provides the differentiating *number*; the differentiation and integration *operators* live in
`deep_causality_calculus`.

The implementation is macro-free and unsafe-free. It depends on `deep_causality_num`, `deep_causality_algebra`, and
`deep_causality_haft`.

## Dependency

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_num_dual = "0.1"
```

## Build configurations

| Configuration | Cargo features | Supported |
|---|---|:-:|
| `std`, hosted | default | ✓ |
| `no-std` with a heap | `default-features = false, features = ["no-std"]` | ✓ |
| `no-std` on `core` alone, no heap | `default-features = false, features = ["no-std"]` | ✓ |

The crate needs only `core`: with `no-std` it links into a program that defines no `#[global_allocator]`.

CI builds every supported bare-metal configuration for `thumbv7em-none-eabihf` and links the `core` configuration without an allocator.
[README_NO_STD.md](https://github.com/deepcausality-rs/deep_causality/blob/main/README_NO_STD.md) describes the build configuration of every crate in the workspace.

## License

This project is licensed under the [MIT license](LICENSE).
