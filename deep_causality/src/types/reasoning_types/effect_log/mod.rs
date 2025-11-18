/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines the CausalEffectLog type.

use crate::traits::log_append::LogAppend;
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

// Internal struct, not exposed in the public API.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LogEntry {
    timestamp_ms: u128,
    message: String,
}

/// Represents an encapsulated, timestamped log of causal evaluation events.
///
/// This struct provides a high-level API for logging and ensures that all
/// log entries are automatically timestamped. It hides the internal storage
/// details to provide a clean and robust interface.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CausalEffectLog {
    entries: Vec<LogEntry>,
}

impl CausalEffectLog {
    /// Creates a new, empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new, empty log with a specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Adds a new timestamped entry to the log.
    ///
    /// # Arguments
    /// * `message` - The log message string slice.
    pub fn add_entry(&mut self, message: &str) {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();

        self.entries.push(LogEntry {
            timestamp_ms,
            message: message.to_string(),
        });
    }

    /// Merges another log's entries into this one. The entries from the other
    /// log are moved, and the other log will be empty after this operation.
    pub fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }

    /// Returns the number of entries in the log.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the log contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl LogAppend for CausalEffectLog {
    fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }
}

/// Provides a clean way to create a log with a single initial entry.
/// e.g., `let log: CausalEffectLog = "Initial message".to_string().into();`
impl From<String> for CausalEffectLog {
    fn from(message: String) -> Self {
        let mut log = Self::new();
        log.add_entry(&message);
        log
    }
}

/// Provides a clean way to create a log with a single initial entry from a string slice.
/// e.g., `let log: CausalEffectLog = "Initial message".into();`
impl From<&str> for CausalEffectLog {
    fn from(message: &str) -> Self {
        let mut log = Self::new();
        log.add_entry(message);
        log
    }
}

impl Display for CausalEffectLog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.entries.is_empty() {
            return write!(f, "CausalEffectLog: (empty)");
        }

        writeln!(f, "CausalEffectLog ({} entries):", self.entries.len())?;
        for entry in &self.entries {
            writeln!(f, "[ts_micros: {}] {}", entry.timestamp_ms, entry.message)?;
        }
        Ok(())
    }
}
