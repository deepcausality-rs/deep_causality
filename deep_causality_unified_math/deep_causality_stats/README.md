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

Descriptive and information statistics and the shaped distributions for the
[DeepCausality project](http://www.deepcausality.com), over slices, generic in the scalar. The crate owns Shannon
entropy and its conditional form, the log-sum-exp reduction, the moments, covariance, Pearson correlation, ridge and
logistic regression, the Gaussian log-density, two binning strategies, and the distributions (normal, exponential,
Cauchy, Weibull, log-normal, Poisson, Bernoulli, categorical) with their inverse CDFs. It depends on
`deep_causality_num`, `deep_causality_algebra`, `deep_causality_linear` and `deep_causality_rand`, and on nothing else.
It re-exports the generator traits of `deep_causality_rand` by name, so a caller that draws from a distribution needs
no second dependency.

One crate holds one implementation of each statistic, so two call sites cannot disagree about what it computes.

## Surface

| | |
|---|---|
| Information | `entropy`, `conditional_entropy`, `log_sum_exp`, `log_add_exp` |
| Descriptive | `mean`, `variance`, `std_dev`, `pearson`, `pearson_pairwise_complete` |
| Regression | `fit_ridge`, `fit_ridge_streaming`, `fit_logistic`, `sigmoid` |
| Density | `gaussian_log_density` |
| Binning | `bin_equal_width`, `bin_equal_frequency` |
| Distributions | `Normal`, `Exponential`, `Cauchy`, `Weibull`, `LogNormal`, `Poisson`, `Bernoulli`, `Categorical` |

Anything with axes (marginalising a joint distribution, reducing a tensor) belongs with the container that has them.
This crate sits below those containers so they can delegate to it without a cycle; its surface is over slices.

## Precision as a parameter

Functions are generic over their scalar under the algebra tower's bounds, with the exceptions
listed here.

Four public functions name a concrete float:

- `standard_normal_inverse_cdf_at::<R>(u: f64)` takes the unit coordinate as `f64`, because it is a
  position on `[0, 1)` and rounding it into a narrow scalar before the transform is destructive at
  the endpoints. The result is in `R`.
- `standard_normal_inverse_cdf` (`f64`) and `standard_normal_inverse_cdf_f106` (`Float106`) are the
  same transform at one fixed precision each.
- `bernoulli_inverse_cdf(u: f64, p: f64)` compares two `f64` values.

Three generic paths pass through `f64` internally:

- `Bernoulli::new` lowers `p` to `f64` and quantises it to a multiple of `2^-64`, and `Bernoulli::p`
  returns that value through `f64`. At `Float106`, probability detail finer than `f64` is lost.
- `Poisson::new` lowers the rate to `f64` only to compare it with `MAX_RATE`; the stored rate stays
  in the caller's scalar.
- `bernoulli_proportion` divides in the caller's scalar when both counts are exact there, and
  otherwise divides in `f64` and rounds the quotient once into the scalar.

The suites run at four scalars, from ~2 to ~32 decimal digits:

| Scalar | Epsilon | Decimal digits | Integers exact to |
|---|---|---|---|
| `BFloat16` | `7.8e-3` | ~2 | 256 |
| `f32` | `1.19e-7` | ~7 | 2²⁴ |
| `f64` | `2.22e-16` | ~16 | 2⁵³ |
| `Float106` | `4.93e-32` | ~32 | 2¹⁰⁶ |

`BFloat16` is a truncated `f32` (the same eight exponent bits, the
mantissa cut from 23 to 7), so it has `f32`'s reach at a hundred-thousandth of its resolution and half its
memory footprint.

`sigmoid` is bounded on `Scalar` rather than `RealField`: the quotient needs division, which `Real` does not carry, but
not field invertibility. `Scalar` names exactly that middle, so a dual number flows through `sigmoid` and carries its
derivative.

## Conventions worth knowing before you call it

Three of these are deliberate and would otherwise look like defects.

**Zero variance in `pearson` returns `Ok((0, n))`, not an error.** The correlation is undefined there (the denominator
vanishes), but callers rank features by `|r|`, and a rank of zero says "carries no information", the right answer for
a constant column.

**A one-element sample refuses `variance` with `InsufficientSamples`.** No sentinel value takes the place of the
undefined corrected variance, so none can feed a Gaussian density.

**`gaussian_log_density` takes a variance, not a standard deviation.** Both are plausible readings of a scale argument
and they agree only at `σ = 1`. The suite pins the choice at `σ² = 4`, where they differ.

Two more are less surprising. Binning uses `[lower, upper)` for every bin except the last, which is closed, so the maximum
falls inside the range rather than one bin past its end; a constant column is not an error and every observation lands
in bin 0. And `log_sum_exp` is total: the empty slice is `−∞`, the identity of that reduction, rather than an error.

## Errors

`StatsError` wraps `StatsErrorEnum` and offers a named constructor per variant, so a call site reads as the failure it
reports:

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
