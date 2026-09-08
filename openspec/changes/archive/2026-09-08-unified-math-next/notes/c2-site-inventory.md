<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# C2 — the sites `deep_causality_stats` absorbs

Established by scanning every source file under all 29 library crates and 16 example crates, for
named statistical functions and for the inline idioms that carry no such name. Test files and
benches excluded; `.claude/worktrees/` excluded.

**Reconciled after the migration, 2026-09-07.** The original count of "31 sites: 23 in library
crates, 8 in examples" did not survive contact with the code, in three separate ways, and the
corrections are recorded here rather than in the count alone.

*Sites the scan missed.* Two Gaussian log-densities in BRCD (`brcd_gaussian.rs:561` `logpdf_rows`
and `:579` `single_logpdf`) — the section below named only `tensor`'s, so it listed a site that must
not migrate and none of the two that must. Two Bernoulli standard errors in `deep_causality_quantum`
(`qpu/shot_estimate.rs` and `decision/tolerance.rs`), which duplicated each other. Four hand-rolled
means in the examples (both CATE programs, the second Granger mean, and both halves of the physics
pipeline's average).

*A claim that was false.* "Every variance in the workspace is the corrected `n−1` form. There is no
population `÷n` variance at any site" — the basis for task 4.2a's decision not to ship one. Two
population variances exist: `ml_rca/utils.rs` `fit_standardizer` and
`chronometric_examples/gm_recovery/pipeline.rs`. The scan behind the claim covered library crates
only, while the inventory itself covers examples. `population_variance` and `population_std_dev`
now ship, on the strength of those two callers.

*An arithmetic error.* The "Descriptive — seven sites" heading listed eight, and the examples table
carried nine `file:line` references under an "eight sites" heading, because the Granger row names two
files.

**The count is not restated.** It was never checkable — the sections did not sum to it — and a number
that has to be maintained by hand alongside the list it summarises will drift again. What replaces it
is the closing section, which states the disposition of every site.

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


## Disposition — every site, after the migration

Migrated to `deep_causality_stats`, by consumer:

| Consumer | Sites |
|---|---|
| `deep_causality_algorithms` (SURD) | `entropy_nvars`, `cond_entropy`, `entropy_nvars_cdl`, `cond_entropy_cdl` |
| `deep_causality_algorithms` (BRCD) | three log-sum-exp, two Gaussian log-densities, both ridge forms, the logistic gate, `sigmoid`, `mean`, `variance_ddof1` |
| `deep_causality_algorithms` (mRMR) | `pearson_correlation` |
| `deep_causality_discovery` | `bin_equal_width`, `bin_equal_frequency` |
| `deep_causality_physics` | `shannon_entropy_kernel` (renamed to bits) |
| `deep_causality_tensor` | all five of `ext_stats.rs` |
| `deep_causality_uncertain` | `from_samples`, `standard_deviation`, `standard_deviation_qmc`, `expected_value`, `expected_value_qmc` |
| `deep_causality_quantum` | both Bernoulli standard errors |
| examples | twelve sites across seven crates |

Kept, each with the reason recorded at the site:

| Site | Reason |
|---|---|
| `qpu/bridge.rs` `shots_to_observable` | A frequency-weighted mean and variance. Expanding the histogram to reach the slice form does not reproduce the numbers: over 2000 random histograms the mean differed in 33% of cases and the `n−1` variance in 92%, because `value × count` is one multiplication where the expanded form adds `value` to a running sum `count` times |
| `shot_estimate.rs` `separation_bits` | A Bhattacharyya distance, which task 4.3 excluded from the crate by name |
| `ml_rca/model.rs` `sigmoid` | Element-wise over a `candle_core::Tensor`, inside candle's autodiff graph — a different function on a different type |
| `clinical_trial/model.rs` `average_arm` | Folds `Uncertain<f64>`, which implements neither `Real` nor `RealField`; `a + b` there builds a lazy computation graph rather than adding numbers |

Design decision **D7** — `tensor` keeping its five copies — was **reversed**. Its cost argument
(moving `tensor` to tier 5, `multivector` to 6, `topology` to 7) was accurate and the renumbering has
been done; the judgement changed because five duplicated statistics are a maintenance surface that
grows. There was never a cycle to prevent it: `deep_causality_stats` reaches only `num`, `algebra`,
`haft` and `linear`, none of which reach `tensor`. `deep_causality_uncertain` moved from tier 3 to
tier 5 on the same reasoning.

## Functions the migration added to the crate

Each is here because a caller needed it, which is the identity spec's rule:

| Function | Callers |
|---|---|
| `bernoulli_proportion`, `bernoulli_standard_error` | `qpu/shot_estimate.rs`, `decision/tolerance.rs` |
| `MeanAccumulator` | `uncertain`'s two `expected_value` forms, `ml_rca`'s `mean_score` — all of which generate observations rather than holding them, so a slice would cost memory the streaming form does not |
| `column_means`, `covariance_matrix`, `conditional_variance` | `tensor`'s `ext_stats.rs`, and through it BRCD's BOSS scorer and `topology`'s manifold covariance |
| `population_variance`, `population_std_dev` | `ml_rca/utils.rs`, `gm_recovery/pipeline.rs` |
| `Penalisation`, `RidgeConfig`, `LogisticConfig` | BRCD's logistic gate needs the intercept exempt from the penalty; see the note under 5.7 |
