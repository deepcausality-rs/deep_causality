# Data structures for DeepCausality 

Web: https://deepcausality.com/about/

GridArray and sliding window implementation used in [DeepCausality](https://github.com/deepcausality-rs/deep_causality).
A sliding window of fixed size n holds the newest element at position 0 and
the oldest one at position n. Adding one more element shifts the index so that
the oldest element will be dropped. This crate has two implementations, one over vector
and a second one over a const generic array. The const generic implementation is significantly
faster than the vector based version.


## 🤔 Why?



## 🙏 Credits

The project took inspiration from:
* [sliding_features](https://crates.io/crates/sliding_features)
* [sliding-window-aggregation](https://crates.io/crates/sliding-window-aggregation)
* [sliding_window_alt](https://crates.io/crates/sliding_window_alt)
* [sliding_windows](https://crates.io/crates/sliding_windows)

## 👨‍💻👩‍💻 Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask. 

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 💻 Author

* Marvin Hansen, [Emet-Labs](https://emet-labs.com/).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC