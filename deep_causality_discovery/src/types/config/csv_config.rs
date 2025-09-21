/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

#[derive(Debug, Clone)]
pub struct CsvConfig {
    has_headers: bool,
    delimiter: u8,
    skip_rows: usize,
    columns: Option<Vec<String>>,
}

impl CsvConfig {
    pub fn new(
        has_headers: bool,
        delimiter: u8,
        skip_rows: usize,
        columns: Option<Vec<String>>,
    ) -> Self {
        Self {
            has_headers,
            delimiter,
            skip_rows,
            columns,
        }
    }

    pub fn has_headers(&self) -> bool {
        self.has_headers
    }

    pub fn delimiter(&self) -> u8 {
        self.delimiter
    }

    pub fn skip_rows(&self) -> usize {
        self.skip_rows
    }

    pub fn columns(&self) -> &Option<Vec<String>> {
        &self.columns
    }
}

impl fmt::Display for CsvConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CsvConfig(headers: {}, delimiter: {}, skip: {}, columns: {:?})",
            self.has_headers, self.delimiter as char, self.skip_rows, self.columns
        )
    }
}
