/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl_effect::{CdlEffect, CdlEffectWitness};
use crate::{CdlError, CdlWarningLog};
use deep_causality_haft::Functor;

// Functor: fmap
impl Functor<CdlEffectWitness<CdlError, CdlWarningLog>>
    for CdlEffectWitness<CdlError, CdlWarningLog>
{
    fn fmap<A, B, Func>(m_a: CdlEffect<A>, f: Func) -> CdlEffect<B>
    where
        Func: FnMut(A) -> B,
    {
        // fmap expects FnMut
        let f = f;
        CdlEffect {
            inner: m_a.inner.map(f),
            warnings: m_a.warnings, // Warnings are preserved
        }
    }
}
