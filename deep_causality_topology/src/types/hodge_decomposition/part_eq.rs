/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::RealField;

use crate::types::hodge_decomposition::HodgeDecomposition;

impl<R: RealField> PartialEq for HodgeDecomposition<R> {
    fn eq(&self, other: &Self) -> bool {
        self.grade() == other.grade()
            && self.exact() == other.exact()
            && self.co_exact() == other.co_exact()
            && self.harmonic() == other.harmonic()
    }
}
