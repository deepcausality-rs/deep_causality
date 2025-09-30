/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CDL, CdlError, DataCleaner, DataPreprocessor, FeatureSelector, OptionNoneDataCleaner};
use crate::types::cdl::{WithData, WithFeatures};

// After data is loaded
impl CDL<WithData> {
    /// An optional step to preprocess the loaded data.
    ///
    /// This method is a self-transition, returning the `CDL` in the same `WithData`
    /// state, allowing it to be chained or skipped.
    ///
    /// # Arguments
    /// * `preprocessor` - An implementation of `DataPreprocessor` (e.g., `DataDiscretizer`).
    ///
    /// # Returns
    /// A `CDL` instance in the `WithData` state, or a `CdlError` if preprocessing fails.
    pub fn preprocess<P>(self, preprocessor: P) -> Result<CDL<WithData>, CdlError>
    where
        P: DataPreprocessor,
    {
        if let Some(config) = self.config.preprocess_config() {
            let processed_tensor = preprocessor.process(self.state.0, config)?;
            Ok(CDL {
                state: WithData(processed_tensor),
                config: self.config,
            })
        } else {
            Ok(self) // If no config is present, pass through without changes.
        }
    }

    /// An optional step to select a subset of features from the data.
    ///
    /// # Arguments
    /// * `selector` - An implementation of `FeatureSelector` (e.g., `MrmrFeatureSelector`).
    ///
    /// # Returns
    /// A `CDL` instance in the `WithFeatures` state, or a `CdlError` if selection fails.
    pub fn feature_select<S>(self, selector: S) -> Result<CDL<WithFeatures>, CdlError>
    where
        S: FeatureSelector,
    {
        let feature_config = self
            .config
            .feature_selector_config()
            .as_ref()
            .ok_or(CdlError::MissingFeatureSelectorConfig)?;

        // Clean the data first to convert CausalTensor<f64> to CausalTensor<Option<f64>>
        let cleaner = OptionNoneDataCleaner;
        let cleaned_tensor = cleaner.process(self.state.0)?;

        let selected_tensor = selector.select(cleaned_tensor, feature_config)?;
        Ok(CDL {
            state: WithFeatures(selected_tensor),
            config: self.config,
        })
    }
}
