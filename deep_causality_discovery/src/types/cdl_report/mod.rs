/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::causal_discovery::surd::SurdResult;
use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use std::fmt::{self, Display, Formatter};

/// Aggregates all significant findings from a CDL pipeline execution.
#[derive(Debug)]
pub struct CdlReport {
    // 1. Data Metadata
    pub dataset_path: String,
    pub records_processed: usize,

    // 2. Feature Selection Result
    pub feature_selection: MrmrResult,

    // 3. Causal Discovery Result (assuming f64 precision for this example)
    pub causal_analysis: SurdResult<f64>,
}

impl Display for CdlReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "=========================================================="
        )?;
        writeln!(
            f,
            "               DEEP CAUSALITY: ANALYSIS REPORT             "
        )?;
        writeln!(
            f,
            "=========================================================="
        )?;

        writeln!(f, "\n[1] DATASET SUMMARY")?;
        writeln!(f, "    File: .............. {}", self.dataset_path)?;
        writeln!(f, "    Records: ........... {}", self.records_processed)?;

        writeln!(f, "\n[2] FEATURE SELECTION (MRMR)")?;
        // Delegate to MrmrResult's Display implementation
        write!(f, "{}", self.feature_selection)?;

        writeln!(f, "\n[3] CAUSAL DISCOVERY (SURD)")?;
        // Delegate to SurdResult's Display implementation
        write!(f, "{}", self.causal_analysis)?;

        writeln!(
            f,
            "\n=========================================================="
        )?;
        Ok(())
    }
}
