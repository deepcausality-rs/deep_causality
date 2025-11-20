[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Macros

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

## Why?

Custom numerical traits with default implementation for the [DeepCausality project](http://www.deepcausality.com) based on the [rust-num](https://github.com/rust-num/num-traits) crate.

**For projects other than DeepCausality, please use the excellent [rust-num](https://github.com/rust-num/num-traits) crate directly, as this crate is very minimalistic and lacks many common features.**

This crate provides a reduced custom implementation of the `rust-num` main traits.

### Implemented Types

*   `Complex`
*   `Octonion`
*   `Quaternion`

* Zero external dependencies
* Zero unsafe
* Minimal macros (only used for testing)

## License

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC