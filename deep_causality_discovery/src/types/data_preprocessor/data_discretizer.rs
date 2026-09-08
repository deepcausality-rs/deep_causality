/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Precision;
use crate::errors::PreprocessError;
use crate::traits::data_preprocessor::DataPreprocessor;
use crate::types::config::{BinningStrategy, ColumnSelector, PreprocessConfig};
use deep_causality_stats::{StatsError, StatsErrorEnum};
use deep_causality_tensor::CausalTensor;

/// A concrete implementation of `DataPreprocessor` that discretizes continuous data into bins.
pub struct DataDiscretizer;

impl<T: Precision> DataPreprocessor<T> for DataDiscretizer {
    fn process(
        &self,
        tensor: CausalTensor<T>,
        config: &PreprocessConfig,
    ) -> Result<CausalTensor<T>, PreprocessError> {
        let shape = tensor.shape();
        if shape.len() != 2 {
            return Err(PreprocessError::BinningError(
                "Tensor must be 2-dimensional".to_string(),
            ));
        }

        let n_rows = shape[0];
        let n_cols = shape[1];
        let mut new_data = tensor.as_slice().to_vec();

        let cols_to_process: Vec<usize> = match config.columns() {
            ColumnSelector::All => (0..n_cols).collect(),
            ColumnSelector::ByIndex(indices) => indices.clone(),
            ColumnSelector::ByName(_) => {
                return Err(PreprocessError::ConfigError(
                    "ByName column selection is not yet implemented".to_string(),
                ));
            }
        };

        for &col_idx in &cols_to_process {
            if col_idx >= n_cols {
                return Err(PreprocessError::InvalidColumnIdentifier(format!(
                    "Column index {} is out of bounds for tensor with {} columns",
                    col_idx, n_cols
                )));
            }

            let column_data: Vec<T> = (0..n_rows)
                .map(|r| new_data[r * n_cols + col_idx])
                .collect();

            let binned_column = match config.strategy() {
                BinningStrategy::EqualWidth => bin_equal_width(&column_data, config.num_bins())?,
                BinningStrategy::EqualFrequency => {
                    bin_equal_frequency(&column_data, config.num_bins())?
                }
            };

            for r in 0..n_rows {
                new_data[r * n_cols + col_idx] = binned_column[r];
            }
        }

        CausalTensor::new(new_data, shape.to_vec())
            .map_err(|e| PreprocessError::BinningError(e.to_string()))
    }
}

/// Maps a refusal from the statistics crate onto this crate's preprocessing error.
///
/// The split follows what a caller can act on. A bin count the data cannot support is a
/// configuration mistake; everything else is a property of the column being binned.
fn binning_error(error: StatsError) -> PreprocessError {
    match error.kind() {
        StatsErrorEnum::InvalidBinCount(message) => PreprocessError::ConfigError(message.clone()),
        _ => PreprocessError::BinningError(error.to_string()),
    }
}

/// Equal-width bins, delegated to `deep_causality_stats`.
///
/// Same partition as the implementation it replaces — every bin is `[lower, upper)` except the
/// last, which is closed by clamping the maximum down into it — but **not value-identical**, in
/// three ways.
///
/// The quotient is formed differently. The shipped routine scales, `floor((x − lo) / (hi − lo) ·
/// bins)`; the replaced one divided by a pre-rounded width, `floor((x − lo) / fl((hi − lo) / bins))`.
/// A value within a rounding of a bin edge can therefore land one bin over: for the column
/// `[0.0, 0.7, 2.1]` in three bins, `0.7` is bin 1 here and was bin 0 before.
///
/// A near-constant column is treated as the column it is. The replaced code collapsed a range
/// below `T::epsilon()` to a single bin; the shipped routine does that only for a range of exactly
/// zero, so `[0.0, 1e-17]` in two bins is `[0, 1]` here and was `[0, 0]` before.
///
/// Two inputs are refused rather than answered. An empty column, and `bins > observations` — the
/// same refusal the equal-frequency path carries, since both share one validation. The refusal is
/// deliberate (unified-math-next task 5.10a) and the caller sees it as a `ConfigError`.
fn bin_equal_width<T: Precision>(data: &[T], num_bins: usize) -> Result<Vec<T>, PreprocessError> {
    deep_causality_stats::bin_equal_width(data, num_bins).map_err(binning_error)
}

/// Equal-frequency bins, delegated to `deep_causality_stats`.
///
/// **This changes results on tied values**, and the change is the point. The implementation it
/// replaces sliced sorted *positions* — bin `i` took ranks `[round(i·n/k), round((i+1)·n/k))` — so
/// equal observations landed in different bins purely by where they fell in the sort. The shipped
/// routine keeps a block of equal values together and gives the whole block the bin holding the
/// largest share of its ranks, ties to the lower bin. Two observations of the same value now always
/// receive the same bin, which is what a caller reading a discretised column expects.
///
/// The rank-to-bin map differs even without ties: `round(i·n/k)` against the shipped
/// `floor(r·k/n)`. The two agree exactly when `k` divides `n` and can differ by one rank otherwise.
fn bin_equal_frequency<T: Precision>(
    data: &[T],
    num_bins: usize,
) -> Result<Vec<T>, PreprocessError> {
    deep_causality_stats::bin_equal_frequency(data, num_bins).map_err(binning_error)
}

#[cfg(test)]
mod tests {
    // Private helper test b/c bin_equal_frequency is shielded from the public API
    // so that its error states cannot occur via the public API.
    use super::{PreprocessError, bin_equal_frequency};

    #[test]
    fn test_bin_equal_frequency_empty_data_is_refused() {
        // Behaviour change on delegation: an empty column used to come back as an empty vector,
        // which reads as "binned successfully into nothing". It is now refused. The column carries
        // no observations, so there is no partition to report, and the empty `Ok` was the more
        // misleading of the two answers. Unreachable through the public preprocessor, which only
        // ever passes a column of `n_rows` values.
        let data: Vec<f64> = vec![];
        let result = bin_equal_frequency(&data, 2);
        assert!(
            matches!(result, Err(PreprocessError::BinningError(_))),
            "an empty column has no partition, got {result:?}"
        );
    }

    #[test]
    fn test_bin_equal_frequency_less_than_two_bins() {
        // The variant survives the delegation — a bin count the data cannot support is still a
        // `ConfigError`, because it is the caller's configuration rather than the column's content.
        // Only the wording comes from the statistics crate now, so the assertion is on the variant.
        let data = vec![1.0_f64, 2.0, 3.0];
        let result = bin_equal_frequency(&data, 1);
        assert!(
            matches!(result, Err(PreprocessError::ConfigError(_))),
            "a bin count below two is a configuration error, got {result:?}"
        );
    }

    #[test]
    fn test_bin_equal_frequency_more_bins_than_observations() {
        // New refusal the delegation brings: `n` observations support at most `n` non-empty bins,
        // so asking for more is a configuration error rather than a silent set of empty bins.
        let data = vec![1.0_f64, 2.0, 3.0];
        let result = bin_equal_frequency(&data, 5);
        assert!(
            matches!(result, Err(PreprocessError::ConfigError(_))),
            "more bins than observations is a configuration error, got {result:?}"
        );
    }

    #[test]
    fn test_bin_equal_frequency_all_same_value() {
        let data = vec![5.0_f64, 5.0, 5.0, 5.0];
        let num_bins = 2;
        let result = bin_equal_frequency(&data, num_bins).unwrap();
        // All values are the same, so they should all fall into the first bin (0.0)
        assert_eq!(result, vec![0.0, 0.0, 0.0, 0.0]);
    }
}
