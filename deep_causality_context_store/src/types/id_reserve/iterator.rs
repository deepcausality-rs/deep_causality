/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, IdReserve};

/// The next identifier, or `None` when the reserve is spent.
impl Iterator for IdReserve {
    type Item = ContextoidId;

    fn next(&mut self) -> Option<ContextoidId> {
        let id = self.ids().get(self.taken()).copied()?;
        self.advance();
        Some(id)
    }
}
