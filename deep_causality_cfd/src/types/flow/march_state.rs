/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`MarchState`]: one state, two transports.
//!
//! A coupled march's resumable state is a [`CoupledField`] (carried scalars, ambient, navigation
//! engine, regime, provenance log) plus the step index reached. `MarchState` is that pair, and it
//! is the single type behind three surfaces that used to be three shapes:
//!
//! * what a paused march **exports** — [`CompressiblePause::state`](crate::CompressiblePause);
//! * what a resumed march **accepts** as its initial field;
//! * what the checksummed snapshot **stores** and reloads ([`save`](Self::save) / [`load`](Self::load)).
//!
//! So what you pause is what you resume, whether the resume happens on the next line (in memory)
//! or from disk days later — bit-identically, because the disk transport round-trips the same
//! `(field, step)` the in-memory transport carries.

use crate::CfdScalar;
use crate::types::flow::CoupledField;
use crate::types::flow::state_snapshot::{load_resume_state, save_resume_state};
use deep_causality_file::BitCodec;
use deep_causality_num::FromPrimitive;
use deep_causality_physics::PhysicsError;
use std::path::Path;

/// A resumable coupled-march state: the [`CoupledField`] to resume from, and the step reached.
#[derive(Debug, Clone)]
pub struct MarchState<R: CfdScalar> {
    field: CoupledField<R>,
    step: usize,
}

impl<R: CfdScalar> MarchState<R> {
    /// Begin a march from an initial coupled field, at step 0.
    pub fn new(field: CoupledField<R>) -> Self {
        Self { field, step: 0 }
    }

    /// A state at a given field and step — what a pause exports.
    pub fn at(field: CoupledField<R>, step: usize) -> Self {
        Self { field, step }
    }

    /// The coupled field to resume from (the initial field a continued march reads).
    pub fn field(&self) -> &CoupledField<R> {
        &self.field
    }

    /// Consume the state, yielding its coupled field (for a resume that takes ownership).
    pub fn into_field(self) -> CoupledField<R> {
        self.field
    }

    /// The step index reached when this state was captured.
    pub fn step(&self) -> usize {
        self.step
    }
}

impl<R: CfdScalar + BitCodec> MarchState<R> {
    /// Suspend this state to disk as a checksummed, fingerprinted resume package (the disk
    /// transport). A different workflow reloads it with [`load`](Self::load).
    ///
    /// # Errors
    /// Packing and file failures surface as physics errors naming the cause.
    pub fn save(
        &self,
        path: impl AsRef<Path>,
        world_fingerprint: &[u8],
    ) -> Result<(), PhysicsError> {
        save_resume_state(path, &self.field, self.step, world_fingerprint)
    }
}

impl<R: CfdScalar + BitCodec + FromPrimitive> MarchState<R> {
    /// Reload a state saved by [`save`](Self::save), strictly verifying the checksum, scalar, and
    /// world fingerprint before rebuilding. The reloaded state resumes bit-identically to the
    /// in-memory one.
    ///
    /// # Errors
    /// Corrupt files, scalar mismatches, and stale world fingerprints refuse with the file named.
    pub fn load(path: impl AsRef<Path>, world_fingerprint: &[u8]) -> Result<Self, PhysicsError> {
        let (field, step) = load_resume_state(path, world_fingerprint)?;
        Ok(Self { field, step })
    }
}
