<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# C2 — the sites `deep_causality_stats` absorbs

Established by scanning every source file under all 29 library crates and 16 example crates, for
named statistical functions and for the inline idioms that carry no such name. Test files and
benches excluded; `.claude/worktrees/` excluded.

**31 sites: 23 in library crates, 8 in examples.**

## Entropy — five sites, and the reason the crate exists

Three shipped implementations disagree on all three axes at once. Two of them disagree on the
*unit of the answer*.

| Site | Base | Normalisation | Zero policy | Errors |
|---|---|---|---|---|
| `surd_utils/mod.rs:111` `entropy_nvars` | log2 | none | skip `p > 0` | none |
| `surd_utils_cdl.rs:135` `entropy_nvars_cdl` | log2 | divides by the sum of `Some` values | skip `p_norm > eps` | none |
| `thermodynamics/stats.rs:154` `shannon_entropy_kernel` | **ln** | none | filter `p > 0` | typed on empty **and** on negative |

Plus the two conditional forms, both `H(XY) − H(Y)` over the above:
`surd_utils/mod.rs:152` `cond_entropy`, `surd_utils_cdl.rs:189` `cond_entropy_cdl`.

## Log-sum-exp — four sites, no semantic disagreement

`brcd_algo.rs:540` `logsumexp_slice`, `brcd_boss_bootstrap.rs:325` `logsumexp`,
`tensor/extensions/ext_stats.rs:154` `logsumexp`, and the two-term
`brcd_gaussian.rs:631` `logaddexp` (one caller, `brcd_gaussian.rs:372`).

All four take the max shift, return the max when it is non-finite, and agree on the empty case:
`neg_inf` and `T::zero().ln()` are the same value. The cheapest absorption in the stage.

## Regression — four sites

- `brcd_gaussian.rs:86` `fit_ridge` — normal equations, solved by `brcd_linalg`'s local
  `solve_linear`, not by `deep_causality_linear`. This is what justifies the `linear` dependency:
  the migration is a substitution, not a re-export.

  **Held to, after first being missed.** The crate's own ridge shipped with a local
  `solve_symmetric` — the same defect one level up, and `cargo machete` caught it: the declared
  `deep_causality_linear` dependency was unused, leaving the tier-4 SHALL unbacked. Both solve
  sites, the ridge normal equations and the logistic Newton step, now call
  `deep_causality_linear::solve`. Recorded under task 4.14b; not for group 6 to re-litigate.
- `brcd_gaussian.rs:481` `fit_ridge_streaming`
- `brcd_gate.rs:91` `fit_logistic_gate` (IRLS), `brcd_gate.rs:202` `sigmoid`

## Descriptive — seven sites

`brcd_gaussian.rs:661` `mean`, `brcd_gaussian.rs:670` `variance_ddof1`,
`ext_stats.rs:26` `sample_mean`, `ext_stats.rs:38` `sample_covariance`,
`uncertain_statistics.rs:33` `standard_deviation`, `uncertain_statistics.rs:86`
`standard_deviation_qmc`, `uncertain_f64.rs:19` (inline), and
`missing_value_imputer.rs:28` `impute_mean`.

**Every variance in the workspace is the corrected `n−1` form.** There is no population `÷n`
variance at any site. `brcd_gaussian.rs:590` `density_variance` is a floor, not a variance, and
`ext_stats.rs:81` `conditional_variance` is a Schur complement.

## Gaussian log-density, Pearson, binning — four sites

`ext_stats.rs:58` `gaussian_log_density`; `mrmr_utils.rs:29` `pearson_correlation`;
`data_discretizer.rs:72` `bin_equal_width` and `:121` `bin_equal_frequency`.

## Examples — eight sites

| Site | What |
|---|---|
| `causal_correction_examples/src/math_utils.rs:15` | `mean` |
| `causal_counterfactual_examples/src/math_utils.rs:15` | `mean` — **byte-identical file** to the above |
| `causal_correction_examples/corrective_ddos_detector/model.rs:99` | inline `n−1` variance |
| `avionics_examples/cfd/plasma_blackout/weather/model.rs:178` | `mean_sd` |
| `causal_discovery_examples/ml/ml_rca/model.rs:92` | `mean_score` |
| `causal_discovery_examples/ml/ml_rca/model.rs:102` | `sigmoid` |
| `causal_uncertain_examples/clinical_trial/model.rs:194` | `average_arm` |
| `classical_causality_examples/.../granger/main.rs:116` + `.../granger/model.rs:85` | `mean`, once named and once inline |

## Two sites that stay

- **`quantum/types/qpu/bridge.rs:92-107`** computes a **frequency-weighted** mean and `n−1`
  variance over `(outcome, count)` pairs, deliberately avoiding one sample per shot. That is a
  different function from a slice mean, and it is `f64` at both ends: the counts arrive as `usize`
  and the result feeds `Uncertain::normal(f64, f64)`. No other caller wants a weighted form.
- **`ext_stats.rs:81` `conditional_variance`** is a Schur complement over a covariance block with a
  ridge on the parent diagonal, not a descriptive statistic.

## Excluded functions: confirmed absent

`cross_entropy`, `kl_divergence`, `kullback`, `mutual_information`, `jensen_shannon` and
`hellinger` return zero definitions across the workspace. `bhattacharyya` returns one —
`quantum/types/qpu/shot_estimate.rs:166` — which is the two-outcome Bernoulli case in bits, not the
slice-shaped general coefficient, and is excluded on that ground.
