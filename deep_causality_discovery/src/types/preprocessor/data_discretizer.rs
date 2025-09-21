/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::PreprocessError;
use crate::traits::data_preprocessor::DataPreprocessor;
use crate::types::config::{BinningStrategy, ColumnSelector, PreprocessConfig};
use deep_causality_tensor::CausalTensor;

pub struct DataDiscretizer;

impl DataPreprocessor for DataDiscretizer {
    fn process(
        &self,
        tensor: CausalTensor<f64>,
        config: &PreprocessConfig,
    ) -> Result<CausalTensor<f64>, PreprocessError> {
        let shape = tensor.shape();
        if shape.len() != 2 {
            return Err(PreprocessError::BinningError(
                "Tensor must be 2-dimensional".to_string(),
            ));
        }

        let n_rows = shape[0];
        let n_cols = shape[1];
        let mut new_data = tensor.as_slice().to_vec();

        let cols_to_process = match config.columns() {
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

            let column_data: Vec<f64> = (0..n_rows)
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

fn bin_equal_width(data: &[f64], num_bins: usize) -> Result<Vec<f64>, PreprocessError> {
    if num_bins < 2 {
        return Err(PreprocessError::ConfigError(
            "Number of bins must be at least 2".to_string(),
        ));
    }

    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    if (max - min).abs() < f64::EPSILON {
        // All values are the same, return a vector of zeros (single bin)
        return Ok(vec![0.0; data.len()]);
    }

    let bin_width = (max - min) / num_bins as f64;
    let mut binned_data = Vec::with_capacity(data.len());

    for &val in data {
        let mut bin_index = ((val - min) / bin_width) as usize;
        if bin_index >= num_bins {
            bin_index = num_bins - 1; // Clamp to last bin
        }
        binned_data.push(bin_index as f64);
    }

    Ok(binned_data)
}

fn bin_equal_frequency(data: &[f64], num_bins: usize) -> Result<Vec<f64>, PreprocessError> {
    if num_bins < 2 {
        return Err(PreprocessError::ConfigError(
            "Number of bins must be at least 2".to_string(),
        ));
    }
    let n = data.len();
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut indices: Vec<usize> = (0..n).collect();
    // Sort indices based on the data values, handling potential NaNs
    indices.sort_by(|&a, &b| {
        data[a]
            .partial_cmp(&data[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut binned_data = vec![0.0; n];
    let step = n as f64 / num_bins as f64;

    for i in 0..num_bins {
        let start_k = (i as f64 * step).round() as usize;
        let end_k = ((i + 1) as f64 * step).round() as usize;

        // Iterate over the relevant slice of sorted indices
        for &original_index in &indices[start_k..end_k.min(n)] {
            // Use the index to assign the bin number to the correct position in the final output
            binned_data[original_index] = i as f64;
        }
    }

    Ok(binned_data)
}
