/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Formatter};
use deep_causality_haft::{LogAddEntry, LogAppend, LogEffect, LogSize};

use crate::types::effect_log::log_entry::LogEntry;
#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents an encapsulated, timestamped log of causal evaluation events.
///
/// This struct provides a high-level API for logging and ensures that all
/// log entries are automatically timestamped. It hides the internal storage
/// details to provide a clean and robust interface.
#[derive(Debug, Default, Clone,Eq,  PartialEq)]
pub struct EffectLog {
    entries: Vec<LogEntry>,
}

impl EffectLog {
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
}

impl LogAddEntry for EffectLog {
    /// Adds a new timestamped entry to the log.
    ///
    /// # Arguments
    /// * `message` - The log message string slice.
    fn add_entry(&mut self, message: &str) {
        #[cfg(feature = "std")]
        let timestamp_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();

        #[cfg(not(feature = "std"))]
        let timestamp_micros = 0;
        // Add a non-std timestamp

        self.entries
            .push(LogEntry::new(timestamp_micros, message.to_string()))
    }
}

impl LogSize for EffectLog {
    /// Returns true if the log contains no entries.
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of entries in the log.
    fn len(&self) -> usize {
        self.entries.len()
    }
}

// Marker trait for full log implementation
impl LogEffect for EffectLog {}

impl LogAppend for EffectLog {
    /// Merges another log's entries into this one. The entries from the other
    /// log are moved, and the other log will be empty after this operation.
    fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }
}

/// Provides a clean way to create a log with a single initial entry.
/// e.g., `let log: CausalEffectLog = "Initial message".to_string().into();`
impl From<String> for EffectLog {
    fn from(message: String) -> Self {
        let mut log = Self::new();
        log.add_entry(&message);
        log
    }
}

/// Provides a clean way to create a log with a single initial entry from a string slice.
/// e.g., `let log: CausalEffectLog = "Initial message".into();`
impl From<&str> for EffectLog {
    fn from(message: &str) -> Self {
        let mut log = Self::new();
        log.add_entry(message);
        log
    }
}

impl Display for EffectLog {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.entries.is_empty() {
            return write!(f, "EffectLog: (empty)");
        }

        writeln!(f, "EffectLog ({} entries):", self.entries.len())?;
        for entry in &self.entries {
            writeln!(f, "[{}", entry.to_string())?;
        }
        Ok(())
    }
}
