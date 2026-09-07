[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# deep_causality_stats

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_stats

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_stats/latest/deep_causality_stats/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

## Summary

Descriptive and information statistics for the [DeepCausality project](http://www.deepcausality.com), over slices,
generic in the scalar. The crate owns Shannon entropy and its conditional form, the log-sum-exp reduction, the moments,
Pearson correlation, ridge and logistic regression, the Gaussian log-density, and two binning strategies. It depends on
`deep_causality_num`, `deep_causality_algebra` and `deep_causality_linear`, and on nothing else.

It does not exist to remove duplication. It adds more source than it removes, and says so in its own change notes. It
exists because shipped implementations of the same statistic disagreed about what they compute.

## Surface

| | |
|---|---|
| Information | `entropy`, `conditional_entropy`, `log_sum_exp`, `log_add_exp` |
| Descriptive | `mean`, `variance`, `std_dev`, `pearson`, `pearson_pairwise_complete` |
| Regression | `fit_ridge`, `fit_ridge_streaming`, `fit_logistic`, `sigmoid` |
| Density | `gaussian_log_density` |
| Binning | `bin_equal_width`, `bin_equal_frequency` |

Anything with axes — marginalising a joint distribution, reducing a tensor — belongs with the container that has them.
This crate sits below those containers so they can delegate to it without a cycle, and its surface is over slices.

## Precision as a parameter

Every function is generic over its scalar under the algebra tower's bounds, and no public signature
names a concrete float. Two of the implementations this crate absorbs compute in `f64` behind a
generic signature, which silently discards the caller's precision.

The suites run at four scalars, and the widest and narrowest are three orders of magnitude apart in
*digits*:

| Scalar | Epsilon | Decimal digits | Integers exact to |
|---|---|---|---|
| `BFloat16` | `7.8e-3` | ~2 | 256 |
| `f32` | `1.19e-7` | ~7 | 2²⁴ |
| `f64` | `2.22e-16` | ~16 | 2⁵³ |
| `Float106` | `4.93e-32` | ~32 | 2¹⁰⁶ |

`BFloat16` is a truncated `f32` — the same eight exponent bits, the
mantissa cut from 23 to 7 — so it has `f32`'s reach at a hundred-thousandth of its resolution, but at half the 
memory footprint. 

`sigmoid` is bounded on `Scalar` rather than `RealField`: the quotient needs division, which `Real` does not carry, but
it does not need field invertibility. That is exactly the middle `Scalar` names, so a dual number flows through it and
the derivative comes with it.

## Conventions worth knowing before you call it

Three of these are deliberate and would look like defects otherwise.

**Zero variance in `pearson` returns `Ok((0, n))`, not an error.** The correlation is undefined there — the denominator
vanishes — but the absorbed caller ranks features by `|r|`, and a rank of zero says "carries no information", which is
the right answer for a constant column.

**A one-element sample refuses `variance` with `InsufficientSamples`.** This is a behaviour change. The absorbed
`variance_ddof1` returned `T::one()` there, and that sentinel fed a Gaussian density.

**`gaussian_log_density` takes a variance, not a standard deviation.** Both are plausible readings of a scale argument
and they agree only at `σ = 1`. The suite pins the choice at `σ² = 4`, where they differ.

Two more, less surprising. Binning uses `[lower, upper)` for every bin except the last, which is closed, so the maximum
falls inside the range rather than one bin past its end; a constant column is not an error and every observation lands
in bin 0. And `log_sum_exp` is total: the empty slice is `−∞`, the identity of that reduction, rather than an error.

## Errors

`StatsError` wraps `StatsErrorEnum` and offers a named constructor per variant, so a call site reads as the failure it
reports rather than as a nested construction:

```rust
return Err(StatsError::InsufficientSamples(
    "the corrected variance needs two observations: with one the divisor n − 1 is zero",
));
```

The variants separate three kinds of cause: an input the mathematics does not admit (`EmptyInput`,
`NegativeProbability`, `NonPositiveScale`, `NonFiniteInput`), a shape the caller got wrong (`DimensionMismatch`,
`InvalidBinCount`, `InsufficientSamples`), and a computation that could not reach an answer (`RankDeficient`,
`NotConverged`, `ConversionFailed`). A caller can act on the second and third; the first is a statement about the data.

`NotConverged` carries its iteration count, so a cap that is merely too low is distinguishable from a problem that does
not converge at all.


## Dependency

```toml
[dependencies]
deep_causality_stats = "0.1"
```

## Licence

MIT. See [LICENSE](https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE).
