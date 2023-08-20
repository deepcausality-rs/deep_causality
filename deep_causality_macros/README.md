[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Macros

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Audit][audit-url]
![Clippy][clippy-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/dcl_data_structures

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/dcl_data_structures/latest/dcl_data_structures/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[audit-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/audit.yml/badge.svg

[clippy-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust-clippy.yml/badge.svg

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

## 🤔 Why?

Write custom types with near-zero boilerplate code.
Rust is great, but when you write a lot of custom types, adding constructor
and getters becomes tedious. These macros solve this by generating
the boilerplate code for you.

## 🎁 Features

* Generates default constructor for structs and enums
* Generates getters for structs
* Getters can be renamed

## 🚀 Install

Just run:

```bash
cargo add deep_causality_macros
```

## ⭐ Usage

See:

* [Examples](examples)
* [Test](tests)

```rust
use deep_causality_macros::{Getters, Constructor};

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data<T>{
    #[getter(name = data_id)] // Rename getter methods as you wish
    id: u64,
    data: T,
    filled: bool,
}

pub fn main() {
    let d = Data::new(0, 42, true);
    assert_eq!(*d.data_id(), 0);
    assert_eq!(*d.data(), 42);
    assert_eq!(*d.filled(), true);
}
```

## 👨‍💻👩‍💻 Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT license without additional terms or conditions.

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 👮️ Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## 💻 Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC