/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::flow::study_effect::{StudyEffect, StudyEffectWitness};
use crate::types::flow::study_error::StudyError;
use crate::types::flow::study_warning::StudyWarningLog;
use deep_causality_haft::{Applicative, LogAppend, NoConstraint, Pure, Satisfies};

/// `pure`: lift a plain value into the effect with an empty warning log.
impl Pure<StudyEffectWitness<StudyError, StudyWarningLog>>
    for StudyEffectWitness<StudyError, StudyWarningLog>
{
    fn pure<T>(value: T) -> StudyEffect<T>
    where
        T: Satisfies<NoConstraint>,
    {
        StudyEffect {
            inner: Ok(value),
            warnings: StudyWarningLog::default(),
        }
    }
}

/// `apply`: apply a wrapped function to a wrapped value, merging warnings and short-circuiting on
/// the first error in either.
impl Applicative<StudyEffectWitness<StudyError, StudyWarningLog>>
    for StudyEffectWitness<StudyError, StudyWarningLog>
{
    fn apply<A, B, Func>(f_ab: StudyEffect<Func>, mut m_a: StudyEffect<A>) -> StudyEffect<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
        B: Satisfies<NoConstraint>,
        Func: Satisfies<NoConstraint>,
    {
        let mut merged = f_ab.warnings;
        merged.append(&mut m_a.warnings);

        let inner = match (f_ab.inner, m_a.inner) {
            (Ok(mut func), Ok(val)) => Ok(func(val)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        };

        StudyEffect {
            inner,
            warnings: merged,
        }
    }
}
