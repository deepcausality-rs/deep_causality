/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{LogAddEntry, LogAppend, LogEffect, LogSize};

// CdlWarning Definition
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CdlWarning {
    DataIssue(String),
    FeatureIssue(String),
    ModelIssue(String),
    Generic(String),
}

impl From<&str> for CdlWarning {
    fn from(s: &str) -> Self {
        CdlWarning::Generic(s.to_string())
    }
}

// CdlWarningLog Definition
#[derive(Debug, Clone, Default)]
pub struct CdlWarningLog {
    pub entries: Vec<CdlWarning>,
}

// Implement traits from deep_causality_haft::effect_system::effect_log

impl LogAddEntry for CdlWarningLog {
    fn add_entry(&mut self, message: &str) {
        self.entries.push(CdlWarning::from(message));
    }
}

impl LogAppend for CdlWarningLog {
    fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }
}

impl LogSize for CdlWarningLog {
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

// Marker trait implementation
impl LogEffect for CdlWarningLog {}
