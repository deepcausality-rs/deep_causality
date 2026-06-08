/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl_discovery_outcome::CdlDiscoveryOutcome;
use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use std::fmt::{self, Debug, Display, Formatter};

/// Aggregates all significant findings from a CDL pipeline execution.
///
/// The report is algorithm-neutral: `causal_analysis` carries the polymorphic
/// [`DiscoveryOutcome`], and `feature_selection` is present only for the SURD
/// lineage (BRCD performs no MRMR step).
#[derive(Debug)]
pub struct CdlReport<T> {
    // 1. Data Metadata
    pub dataset_path: String,
    pub records_processed: usize,

    // 2. Feature Selection Result (SURD only; `None` for BRCD).
    pub feature_selection: Option<MrmrResult>,

    // 3. Causal Discovery Result, carried at the pipeline's precision `T`.
    pub causal_analysis: CdlDiscoveryOutcome<T>,
}

impl<T: Debug> Display for CdlReport<T> {
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

        // Feature selection is SURD-only; BRCD reports none.
        if let Some(feature_selection) = &self.feature_selection {
            writeln!(f, "\n[2] FEATURE SELECTION (MRMR)")?;
            write!(f, "{}", feature_selection)?;
        }

        match &self.causal_analysis {
            CdlDiscoveryOutcome::Surd(surd_result) => {
                writeln!(f, "\n[3] CAUSAL DISCOVERY (SURD)")?;
                write!(f, "{}", surd_result)?;
            }
            CdlDiscoveryOutcome::Brcd(brcd_result) => {
                writeln!(f, "\n[3] ROOT-CAUSE DISCOVERY (BRCD)")?;
                write!(f, "{}", brcd_result)?;
            }
        }

        writeln!(
            f,
            "\n=========================================================="
        )?;
        Ok(())
    }
}
