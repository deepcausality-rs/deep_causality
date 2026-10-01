/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// What covers the ground at a point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Surface {
    Water,
    Grass,
    Road,
    Rock,
    Pad,
    Ravine,
    Trees,
}
