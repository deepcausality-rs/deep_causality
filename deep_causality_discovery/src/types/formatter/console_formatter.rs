/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::FinalizeError;
use crate::traits::process_result::{
    ProcessAnalysis, ProcessFormattedResult, ProcessResultFormatter,
};

pub struct ConsoleFormatter;

impl ProcessResultFormatter for ConsoleFormatter {
    fn format(&self, analysis: &ProcessAnalysis) -> Result<ProcessFormattedResult, FinalizeError> {
        println!("Formatting results for console...");
        let mut formatted_output = String::new();
        for line in &analysis.0 {
            // Access the Vec<String> inside ProcessAnalysis
            formatted_output.push_str(line);
            formatted_output.push('\n');
        }
        Ok(ProcessFormattedResult(formatted_output))
    }
}
