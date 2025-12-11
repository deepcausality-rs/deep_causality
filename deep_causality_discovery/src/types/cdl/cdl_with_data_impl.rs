use crate::types::cdl::WithData;
use crate::{CDL, CdlBuilder, CdlEffect, CdlError};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

impl CDL<WithData> {
    /// Filters the dataset rows based on a provided predicate.
    ///
    /// This method allows for creating cohorts or subsets of the data based on column values.
    /// The predicate receives a slice representing a single row of data.
    ///
    /// # Arguments
    ///
    /// * `predicate` - A closure that takes a row slice `&[f64]` and returns `bool`.
    ///                 Returns `true` to keep the row, `false` to discard it.
    ///
    /// # Returns
    ///
    /// A `CdlEffect` containing a new `CDL<WithData>` with the filtered tensor.
    pub fn filter_cohort<P>(self, predicate: P) -> CdlEffect<CDL<WithData>>
    where
        P: Fn(&[f64]) -> bool,
    {
        let tensor = &self.state.tensor;
        let rows = tensor.shape()[0];
        let cols = tensor.shape()[1];
        
        let mut filtered_data = Vec::with_capacity(tensor.data.len());
        let mut new_row_count = 0;

        for r in 0..rows {
            // Extract row slice
            // CausalTensor stores data in row-major order: index = r * cols + c
            let start_idx = r * cols;
            let end_idx = start_idx + cols;
            
            // Safety check for bounds
            if end_idx > tensor.data.len() {
                 return CdlEffect {
                    inner: Err(CdlError::CausalDiscoveryError(deep_causality_algorithms::causal_discovery::CausalDiscoveryError::TensorError(CausalTensorError::IndexOutOfBounds))),
                    warnings: Default::default(),
                };
            }

            let row_slice = &tensor.data[start_idx..end_idx];

            if predicate(row_slice) {
                filtered_data.extend_from_slice(row_slice);
                new_row_count += 1;
            }
        }

        // Create new tensor from filtered data
        match CausalTensor::new(filtered_data, vec![new_row_count, cols]) {
            Ok(new_tensor) => CdlBuilder::pure(CDL {
                state: WithData {
                    tensor: new_tensor,
                    records_count: new_row_count,
                },
                config: self.config,
            }),
            Err(e) => CdlEffect {
                inner: Err(CdlError::CausalDiscoveryError(deep_causality_algorithms::causal_discovery::CausalDiscoveryError::TensorError(e))),
                warnings: Default::default(),
            },
        }
    }
}
