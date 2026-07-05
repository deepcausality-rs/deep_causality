/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`StudyEffect`]: the effect carrier every campaign phase rides inside, mirroring the Causal
//! Discovery Language's `CdlEffect` so the two grammars share one substrate.
//!
//! It carries a `Result<T, StudyError>` (the value, short-circuiting on the first error, which is
//! tagged with the verb that failed) and a [`StudyWarningLog`] (non-fatal diagnostics that
//! accumulate and render in the final verdict). The [`StudyEffectWitness`] fixes the error and
//! warning types at the HKT level and carries the lawful `Functor` / `Applicative` / `Monad`
//! instances (in the sibling modules), so the fluent phase verbs compose monadically without any
//! witness type ever appearing in a signature a study author reads.

mod applicative;
mod functor;
mod monad;

use crate::types::flow::study_error::StudyError;
use crate::types::flow::study_warning::{StudyWarning, StudyWarningLog};
use deep_causality_haft::{HKT, HKT3, LogAppend, NoConstraint, Pure};
use std::marker::PhantomData;

/// A phase value inside the study effect: either the carried value or the first (verb-tagged)
/// error, plus the accumulated non-fatal warnings.
#[derive(Debug)]
pub struct StudyEffect<T> {
    inner: Result<T, StudyError>,
    warnings: StudyWarningLog,
}

impl<T> StudyEffect<T> {
    /// Lift a value into the effect with no warnings.
    pub fn pure(value: T) -> Self {
        StudyEffectWitness::<StudyError, StudyWarningLog>::pure(value)
    }

    /// Lift a fallible result into the effect with no warnings — the seam a phase verb uses to
    /// carry a stage's `Result` (its error already tagged with the verb) into the chain.
    pub fn from_result(result: Result<T, StudyError>) -> Self {
        Self {
            inner: result,
            warnings: StudyWarningLog::default(),
        }
    }

    /// `FnOnce` Kleisli composition — the engine behind the fluent phase verbs: short-circuits on
    /// a prior error, otherwise runs `f` on the value and threads (merges) the warning log. This
    /// is why a study reads `.sweep(..).record(..).gates(..)` with no `bind` wrapper per line.
    pub fn and_then<U>(self, f: impl FnOnce(T) -> StudyEffect<U>) -> StudyEffect<U> {
        match self.inner {
            Err(e) => StudyEffect {
                inner: Err(e),
                warnings: self.warnings,
            },
            Ok(val) => {
                let mut next = f(val);
                let mut merged = self.warnings;
                merged.append(&mut next.warnings);
                StudyEffect {
                    inner: next.inner,
                    warnings: merged,
                }
            }
        }
    }

    /// Transform the carried value, preserving the error channel and warnings.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> StudyEffect<U> {
        StudyEffect {
            inner: self.inner.map(f),
            warnings: self.warnings,
        }
    }

    /// Record a non-fatal warning, carried through to the verdict. Does not affect the value or
    /// the error channel.
    pub fn warn(mut self, warning: StudyWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Consume the effect into its result and accumulated warnings (the terminal seam a verdict
    /// reads).
    pub fn into_parts(self) -> (Result<T, StudyError>, StudyWarningLog) {
        (self.inner, self.warnings)
    }
}

// --- HKT witness: fixes the error + warning-log types at the type level -----------------------

/// A witness type that fixes the error `E` and warning-log `WLog` for [`StudyEffect`], carrying
/// the type-level function `Type<T> = StudyEffect<T>`. The lawful `Functor` / `Applicative` /
/// `Monad` instances live in the sibling modules; no consumer ever names this type.
pub struct StudyEffectWitness<E, WLog>(PhantomData<(E, WLog)>);

impl<E, WLog> HKT for StudyEffectWitness<E, WLog> {
    type Constraint = NoConstraint;
    type Type<T> = StudyEffect<T>;
}

impl<E, WLog> HKT3<E, WLog> for StudyEffectWitness<E, WLog>
where
    E: 'static,
    WLog: 'static,
{
    type Type<T> = StudyEffect<T>;
}
