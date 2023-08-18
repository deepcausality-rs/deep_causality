// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{UltraGraphContainer, UltraMatrixGraph};

// Type alias for convenience and to shorten type annotations / inference.
// This also allows for simple swapping of the underlying storage type.
pub type UltraGraph<T> = UltraGraphContainer<UltraMatrixGraph<T>, T>;