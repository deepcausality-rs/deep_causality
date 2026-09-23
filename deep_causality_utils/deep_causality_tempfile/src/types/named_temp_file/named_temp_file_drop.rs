/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::NamedTempFile;
use std::fs;

impl Drop for NamedTempFile {
    /// Removes the file. A failed removal is ignored.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
