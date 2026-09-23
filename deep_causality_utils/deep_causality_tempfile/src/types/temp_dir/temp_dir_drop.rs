/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::TempDir;
use std::fs;

impl Drop for TempDir {
    /// Removes the directory and everything in it. A failed removal is ignored.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
