/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`StudyError`]: the one error a campaign meets, wrapping the two causes a study can hit — a
//! [`PhysicsError`] from a march or reduction, and a [`DataLoadingError`] from reading a matrix
//! or writing a table — and tagged with the verb that failed, so `verdict()` names where the
//! study broke.

use core::fmt;
use deep_causality_file::DataLoadingError;
use deep_causality_physics::PhysicsError;

/// A study failure: its cause plus the verb (stage) it arose in. A bare `From` conversion leaves
/// the stage empty; the campaign's verbs re-tag with their own name via [`in_stage`](Self::in_stage).
#[derive(Debug)]
pub struct StudyError {
    stage: &'static str,
    kind: StudyErrorKind,
}

#[derive(Debug)]
enum StudyErrorKind {
    /// A physics failure from a march, reduction, or apparatus build.
    Physics(PhysicsError),
    /// A data failure from reading a matrix or writing a table.
    Data(DataLoadingError),
}

impl StudyError {
    /// Tag a cause with the verb it failed in (`"sweep"`, `"record"`, `"read"`, …).
    pub fn in_stage(stage: &'static str, cause: impl Into<StudyError>) -> Self {
        let mut e = cause.into();
        e.stage = stage;
        e
    }

    /// The verb this error was tagged with, or `""` if untagged.
    pub fn stage(&self) -> &'static str {
        self.stage
    }
}

impl From<PhysicsError> for StudyError {
    fn from(e: PhysicsError) -> Self {
        Self {
            stage: "",
            kind: StudyErrorKind::Physics(e),
        }
    }
}

impl From<DataLoadingError> for StudyError {
    fn from(e: DataLoadingError) -> Self {
        Self {
            stage: "",
            kind: StudyErrorKind::Data(e),
        }
    }
}

impl fmt::Display for StudyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.stage.is_empty() {
            match &self.kind {
                StudyErrorKind::Physics(e) => write!(f, "study failed: {e}"),
                StudyErrorKind::Data(e) => write!(f, "study failed: {e}"),
            }
        } else {
            match &self.kind {
                StudyErrorKind::Physics(e) => write!(f, "study failed in '{}': {e}", self.stage),
                StudyErrorKind::Data(e) => write!(f, "study failed in '{}': {e}", self.stage),
            }
        }
    }
}

impl std::error::Error for StudyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            StudyErrorKind::Physics(_) => None,
            StudyErrorKind::Data(e) => Some(e),
        }
    }
}
