/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # deep_causality_tempfile
//!
//! Scratch files and directories under `std::env::temp_dir()`, removed on drop. The crate
//! depends only on `std`.
//!
//! * [`TempDir`]: a new, empty directory, removed with its contents on drop.
//! * [`NamedTempFile`]: a new, empty file, written through `std::io::Write` and removed on drop.
//!   [`NamedTempFile::with_suffix`] gives the name an extension such as `".csv"`.

mod types;
mod utils;

pub use types::named_temp_file::NamedTempFile;
pub use types::temp_dir::TempDir;
