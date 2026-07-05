/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`LogSink`]: the disk side of the `save_log` verb. It implements the carrier's
//! [`AuditFlush`](crate::types::flow::carrier::AuditFlush) seam by appending every not-yet-written
//! effect-log entry to an open file and flushing it, so an audited march's file grows one step at a
//! time and — if the process dies — ends at the last entry recorded before the death (the
//! abort-tail guarantee). A completed run's file therefore holds exactly the run's provenance
//! entries, in order, one per line.
//!
//! Under a concurrent fan-out (the weather ensemble, a counterfactual fork) each branch opens its
//! own `LogSink` on its own file, so no file is written by two threads and no entries interleave;
//! the caller names each per-branch file from the main path plus the fan-out round and case.

use crate::types::flow::carrier::AuditFlush;
use deep_causality_core::EffectLog;
use deep_causality_haft::LogSize;
use deep_causality_physics::PhysicsError;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// An open audit-log file that flushes each new effect-log entry the moment it is recorded.
///
/// The OS buffer is flushed after every write (the abort-tail guarantee); power-loss durability
/// (`fsync` per entry) is intentionally not offered — the spec makes it an optional `MAY`, off by
/// default, and the disk-buffer flush is enough for the post-mortem-audit use case.
pub struct LogSink {
    writer: BufWriter<File>,
    flushed: usize,
}

impl LogSink {
    /// Create (or truncate) an audit-log file at `path`.
    ///
    /// # Errors
    /// The file cannot be created.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, PhysicsError> {
        let file = File::create(path).map_err(io_err)?;
        Ok(Self {
            writer: BufWriter::new(file),
            flushed: 0,
        })
    }
}

impl AuditFlush for LogSink {
    fn flush(&mut self, log: &EffectLog) -> Result<(), PhysicsError> {
        for msg in log.messages().skip(self.flushed) {
            writeln!(self.writer, "{msg}").map_err(io_err)?;
        }
        self.flushed = log.len();
        self.writer.flush().map_err(io_err)
    }
}

/// Append a closing line to a main audit file (the fan-out spawn/rejoin narration the campaign
/// writes around a concurrent round). Opens for append so it never truncates the main log.
///
/// # Errors
/// The file cannot be opened or written.
pub fn append_line(path: impl AsRef<Path>, line: &str) -> Result<(), PhysicsError> {
    use std::fs::OpenOptions;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(io_err)?;
    writeln!(f, "{line}").map_err(io_err)?;
    f.flush().map_err(io_err)
}

fn io_err(e: std::io::Error) -> PhysicsError {
    PhysicsError::CalculationError(alloc::format!("audit-log sink IO: {e}"))
}
