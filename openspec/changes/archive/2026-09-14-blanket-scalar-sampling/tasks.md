<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Tasks

Implemented in one pass; recorded here because the specs it modifies are already archived.

- [x] 1. `FloatKind` / `UnsignedKind` markers; `SampleUniform<Kind>` and `SampleRange<T, Kind>`.
- [x] 2. One blanket float binding over `RealField + FromPrimitive`; zero types named.
- [x] 3. One `UniformUnsigned<T>` body over the integer tower, replacing `uniform_u32`,
      `uniform_u64` and `uniform_usize`; `u8`, `u16` and `u128` gain samplers.
- [x] 4. `random_range<T, K, R>` keeps one signature; `Uniform<X, K = FloatKind>`.
- [x] 5. `RandFloat` → `RandScalar`, blanket, `WORDS` removed; width derived from
      `Real::epsilon()`. `deep_causality_stats::RandWidth` removed; `stats` re-exports
      `RandScalar` and delegates the accumulation.
- [x] 6. Rejection moved into `rand_float_gen`, which is the function promising `[0, 1)`.
- [x] 7. `UniformFloat::sample` rejects a result reaching the exclusive bound; `new_inclusive`
      unaffected. Docstring corrected to the contract now kept.
- [x] 8. `extensions/` removed: with no per-type implementations left there is nothing for it to
      hold. No macro in lib code.
- [x] 9. `parity_probe_tests.rs`: `BFloat16` draws though unnamed; all six unsigned widths draw;
      a `usize` range of `2^40` reaches past `2^32`.
- [x] 10. rand 112, stats 625, uncertain and topology green, clippy clean,
      `bazel test //...` 1 392 pass.
