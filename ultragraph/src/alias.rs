// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{UltraGraphContainer, UltraMatrixGraph};

// Type alias for convenience and to shorten type annotations / inference.
// This also allows for simple swapping of the underlying storage type.
pub type UltraGraph<T> = UltraGraphContainer<UltraMatrixGraph<T>, T>;