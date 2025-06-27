/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::sync::{Arc, RwLock};

// Thread safe Interior mutability in Rust
// https://ricardomartins.cc/2016/06/25/interior-mutability-thread-safety
pub type ArcRWLock<T> = Arc<RwLock<T>>;
