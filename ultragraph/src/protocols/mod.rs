// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

#![forbid(unsafe_code)]

use crate::prelude::{GraphLike, GraphRoot, GraphStorage};

pub mod graph_like;
pub mod graph_root;
pub mod graph_storage;

/// Super trait for custom graph implementations.
pub trait UltraGraphable<T>: GraphLike<T> + GraphStorage<T> + GraphRoot<T> where T: Copy, {}