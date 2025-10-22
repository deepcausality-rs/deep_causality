/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, Float, Num};

impl<F> Num for Complex<F> where F: Float {}
