/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::flow::study_effect::{StudyEffect, StudyEffectWitness};
use crate::types::flow::study_error::StudyError;
use crate::types::flow::study_warning::StudyWarningLog;
use deep_causality_haft::Functor;

/// `fmap`: transform the carried value, preserving the error channel and the warnings.
impl Functor<StudyEffectWitness<StudyError, StudyWarningLog>>
    for StudyEffectWitness<StudyError, StudyWarningLog>
{
    fn fmap<A, B, Func>(m_a: StudyEffect<A>, f: Func) -> StudyEffect<B>
    where
        Func: FnMut(A) -> B,
    {
        let f = f;
        StudyEffect {
            inner: m_a.inner.map(f),
            warnings: m_a.warnings,
        }
    }
}
