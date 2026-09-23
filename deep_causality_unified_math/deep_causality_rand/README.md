[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DeepCausality Rand

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
 

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_rand

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_rand/latest/deep_causality_rand/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

 

## Why?

`deep_causality_rand` supplies entropy for the [DeepCausality project](http://www.deepcausality.com): the `Xoshiro256` generator, an OS-entropy generator behind the "os-random" feature, a Sobol sequence, and the draws that depend only on bits: a machine word (`StandardWord`), a Boolean (`StandardBool`), and a value uniform over a range (`Uniform`). It states no distribution. The shaped distributions (normal, exponential, Poisson, and others) live in `deep_causality_stats`. The traits reimplement a reduced set of the [rand](https://crates.io/crates/rand) main traits; [rand_distr](https://docs.rs/rand_distr/latest/rand_distr) is the counterpart of `deep_causality_stats`.

**For projects other than DeepCausality, use the [rand](https://crates.io/crates/rand) and [rand_distr](https://docs.rs/rand_distr/latest/rand_distr) crates directly. This crate is minimal, lacks many common features, and solves one problem only: cross-compiling to targets without a libc.**

The crate has these properties:

* Zero external dependencies
* Zero unsafe
* Zero macros in library code

It therefore cross-compiles to exotic hardware, links statically, builds without a libc, and stays easy to review. The API stays minimal to preserve these properties.

## Dependencies

The crate has no external dependencies by default. Internally it depends on the [num crate](../deep_causality_num) and on `deep_causality_algebra`. The "os-random" feature flag adds one optional dependency on `getrandom` and, through it, on libc. The next section explains how the crate handles libc.

## Libc

By default, this crate uses a pseudorandom number generator written in pure Rust, so the build needs no libc. This leaves three options:

* Overwrite the [RngCore](../deep_causality_rand/src/traits/rng_core.rs) and [Rng trait](../deep_causality_rand/src/traits/rng.rs).
* Use the RNG from the operating system ("os-random" feature flag).
* Stick with the default PRNG (Xoshiro256).

The first option gives the most flexibility; the tested Xoshiro256 PRNG serves as a reference implementation. It suits projects that support several targets, each with its own custom RNG. Nearly all operating systems expose a hardware-based random generator via libc, and the "os-random" feature flag uses the `getrandom` crate to bind to a large number of platforms. See the [documentation for a list of platforms](https://docs.rs/getrandom/latest/getrandom). The default PRNG exists for testing and development. Its statistical properties resemble those of a hardware-backed RNG, but it is not secure and must not run in production. Production builds either enable the "os-random" feature flag or use a custom implementation. A custom implementation also covers targets without a full libc and targets whose hardware RNG needs dedicated bindings.

## Macros

The `rand` traits are reimplemented without macros, because macros are hard to maintain, test, and review. Generic traits over generic types take their place, with bounds such as `FromPrimitive` from the internal [num crate](../deep_causality_num).

Macros have valid uses, and this crate uses them to generate tests where a generic trait is implemented for many types. Keeping the library code free of macros, unsafe, and external dependencies simplifies maintenance.

## No-Std

This crate builds without a standard library. Three feature levels select the target environment:

* `std` (the default) builds against the standard library. This level enables the thread-local generator behind `rng()` and host entropy for seeding.
* `alloc` adds the heap without the standard library. It is a level, not a complete configuration, because it does not say where floating-point math comes from; select it through `std` or `no-std`.
* `no-std` builds against `core` plus `alloc` and routes float math through the pure-Rust `libm` crate that `deep_causality_num` pulls in, so still no libc. Build with `cargo build --no-default-features --features no-std`, and cross-compile to a bare-metal target the same way, for example `--target aarch64-unknown-none`.

On bare metal, seeding needs care. A bare-metal target offers no ambient entropy and no thread identity, so `Xoshiro256::new()` falls back to a per-call counter mixed into a fixed base seed. Successive calls within one run yield distinct streams, but the whole sequence repeats identically after every reset, which makes the default seeding reproducible rather than random. When the stream has to differ per boot, seed explicitly with `Xoshiro256::from_seed` from whatever entropy the board offers, such as a hardware RNG peripheral, ADC noise, or a timer capture. This crate cannot know what a given board provides, so that entropy belongs to the firmware. The counter also uses a 32-bit atomic, so targets without atomic compare-and-swap must call `Xoshiro256::from_seed` directly.

`ThreadRng` and the thread-local generator it wraps require `std` and are therefore unavailable under `no-std`; there, `rng()` hands back an owned `Xoshiro256` that the caller keeps. The "os-random" feature flag is a separate matter: `getrandom` ships no backend for a bare-metal target, so that combination fails to build unless the firmware registers a custom backend.

## Contributions

Contributions are welcome but must keep the crate's goals of zero macros, zero unsafe, and zero external dependencies in the main codebase (everything under the `/src` folder).

## License

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).
