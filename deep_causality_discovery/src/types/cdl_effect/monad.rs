/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl_effect::{CdlEffect, CdlEffectWitness};
use crate::{CdlError, CdlWarningLog};
use deep_causality_haft::{LogAppend, Monad, NoConstraint, Satisfies};

// Monad: bind
impl Monad<CdlEffectWitness<CdlError, CdlWarningLog>>
    for CdlEffectWitness<CdlError, CdlWarningLog>
{
    fn bind<A, B, Func>(m_a: CdlEffect<A>, f: Func) -> CdlEffect<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> CdlEffect<B>,
    {
        let mut f = f;
        match m_a.inner {
            Err(e) => CdlEffect {
                inner: Err(e),
                warnings: m_a.warnings,
            },
            Ok(val) => {
                let mut m_b = f(val);
                let mut combined_warnings = m_a.warnings;
                // Append warnings from the result of the bound function
                combined_warnings.append(&mut m_b.warnings);

                CdlEffect {
                    inner: m_b.inner,
                    warnings: combined_warnings,
                }
            }
        }
    }
}
