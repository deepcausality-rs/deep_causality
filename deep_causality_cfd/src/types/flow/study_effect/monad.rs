/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::flow::study_effect::{StudyEffect, StudyEffectWitness};
use crate::types::flow::study_error::StudyError;
use crate::types::flow::study_warning::StudyWarningLog;
use deep_causality_haft::{LogAppend, Monad, NoConstraint, Satisfies};

/// `bind`: the lawful monadic sequencing — short-circuit on a prior error, otherwise run the
/// continuation on the value and thread (merge) the warning log. The fluent `and_then` engine on
/// `StudyEffect` is the `FnOnce` specialization of this.
impl Monad<StudyEffectWitness<StudyError, StudyWarningLog>>
    for StudyEffectWitness<StudyError, StudyWarningLog>
{
    fn bind<A, B, Func>(m_a: StudyEffect<A>, f: Func) -> StudyEffect<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> StudyEffect<B>,
    {
        let mut f = f;
        match m_a.inner {
            Err(e) => StudyEffect {
                inner: Err(e),
                warnings: m_a.warnings,
            },
            Ok(val) => {
                let mut m_b = f(val);
                let mut merged = m_a.warnings;
                merged.append(&mut m_b.warnings);
                StudyEffect {
                    inner: m_b.inner,
                    warnings: merged,
                }
            }
        }
    }
}
