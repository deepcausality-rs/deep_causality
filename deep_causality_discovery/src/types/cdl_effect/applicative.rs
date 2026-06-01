/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl_effect::{CdlEffect, CdlEffectWitness};
use crate::{CdlError, CdlWarningLog};
use deep_causality_haft::{Applicative, LogAppend, NoConstraint, Pure, Satisfies};

// Pure: lift a plain value into the effect context.
impl Pure<CdlEffectWitness<CdlError, CdlWarningLog>> for CdlEffectWitness<CdlError, CdlWarningLog> {
    fn pure<T>(value: T) -> CdlEffect<T>
    where
        T: Satisfies<NoConstraint>,
    {
        CdlEffect {
            inner: Ok(value),
            warnings: CdlWarningLog::default(),
        }
    }
}

// Applicative: apply
impl Applicative<CdlEffectWitness<CdlError, CdlWarningLog>>
    for CdlEffectWitness<CdlError, CdlWarningLog>
{
    fn apply<A, B, Func>(
        f_ab: CdlEffect<Func>, // The container holding the function
        mut m_a: CdlEffect<A>, // The container holding the value
    ) -> CdlEffect<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
        B: Satisfies<NoConstraint>,
        Func: Satisfies<NoConstraint>,
    {
        let mut combined_warnings = f_ab.warnings;
        // Append warnings from m_a
        combined_warnings.append(&mut m_a.warnings);

        let new_inner = match (f_ab.inner, m_a.inner) {
            (Ok(mut func), Ok(val)) => Ok(func(val)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        };

        CdlEffect {
            inner: new_inner,
            warnings: combined_warnings,
        }
    }
}
