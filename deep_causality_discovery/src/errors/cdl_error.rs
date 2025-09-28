/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AnalyzeError, CausalDiscoveryError, DataCleaningError, DataLoadingError, FeatureSelectError,
    FinalizeError, PreprocessError,
};

use std::fmt;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum CdlError {
    ReadDataError(DataLoadingError),
    CleanDataError(DataCleaningError),
    PreprocessError(PreprocessError),
    FeatSelectError(FeatureSelectError),
    CausalDiscoveryError(CausalDiscoveryError),
    AnalyzeError(AnalyzeError),
    FinalizeError(FinalizeError),
    MissingDataLoaderConfig,
    MissingFeatureSelectorConfig,
    MissingCausalDiscoveryConfig,
    MissingAnalyzeConfig,
    MissingFinalizeConfig,
}

impl fmt::Display for CdlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CdlError::PreprocessError(e) => write!(f, "Step [Preprocessing] failed: {}", e),
            CdlError::ReadDataError(e) => write!(f, "Step [Data Loading] failed: {}", e),
            CdlError::FeatSelectError(e) => write!(f, "Step [Feature Selection] failed: {}", e),
            CdlError::CausalDiscoveryError(e) => write!(f, "Step [Causal Discovery] failed: {}", e),
            CdlError::AnalyzeError(e) => write!(f, "Step [Analysis] failed: {}", e),
            CdlError::FinalizeError(e) => write!(f, "Step [Finalization] failed: {}", e),
            CdlError::MissingDataLoaderConfig => write!(
                f,
                "Missing data loader configuration. Please provide a DataLoaderConfig."
            ),
            CdlError::MissingFeatureSelectorConfig => write!(
                f,
                "Missing feature selector configuration. Please provide a FeatureSelectorConfig."
            ),
            CdlError::MissingCausalDiscoveryConfig => write!(
                f,
                "Missing causal discovery configuration. Please provide a CausalDiscoveryConfig."
            ),
            CdlError::MissingAnalyzeConfig => write!(
                f,
                "Missing analysis configuration. Please provide an AnalyzeConfig."
            ),
            CdlError::MissingFinalizeConfig => write!(
                f,
                "Missing finalization configuration. Please provide a FinalizeConfig."
            ),
            CdlError::CleanDataError(e) => {
                write!(f, "Step [Cleaning] failed: {}", e)
            }
        }
    }
}

impl std::error::Error for CdlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CdlError::PreprocessError(e) => Some(e),
            CdlError::ReadDataError(e) => Some(e),
            CdlError::FeatSelectError(e) => Some(e),
            CdlError::CausalDiscoveryError(e) => Some(e),
            CdlError::AnalyzeError(e) => Some(e),
            CdlError::FinalizeError(e) => Some(e),
            CdlError::MissingDataLoaderConfig => None,
            CdlError::MissingFeatureSelectorConfig => None,
            CdlError::MissingCausalDiscoveryConfig => None,
            CdlError::MissingAnalyzeConfig => None,
            CdlError::MissingFinalizeConfig => None,
            CdlError::CleanDataError(e) => Some(e),
        }
    }
}

impl From<DataLoadingError> for CdlError {
    fn from(err: DataLoadingError) -> CdlError {
        CdlError::ReadDataError(err)
    }
}
impl From<DataCleaningError> for CdlError {
    fn from(err: DataCleaningError) -> CdlError {
        CdlError::CleanDataError(err)
    }
}
impl From<FeatureSelectError> for CdlError {
    fn from(err: FeatureSelectError) -> CdlError {
        CdlError::FeatSelectError(err)
    }
}
impl From<CausalDiscoveryError> for CdlError {
    fn from(err: CausalDiscoveryError) -> CdlError {
        CdlError::CausalDiscoveryError(err)
    }
}
impl From<AnalyzeError> for CdlError {
    fn from(err: AnalyzeError) -> CdlError {
        CdlError::AnalyzeError(err)
    }
}
impl From<FinalizeError> for CdlError {
    fn from(err: FinalizeError) -> CdlError {
        CdlError::FinalizeError(err)
    }
}

impl From<PreprocessError> for CdlError {
    fn from(err: PreprocessError) -> CdlError {
        CdlError::PreprocessError(err)
    }
}
