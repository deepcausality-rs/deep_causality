/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensorTrain;

/// The physical gradient triple `(∂/∂x, ∂/∂y, ∂/∂z)` a 3-D metric returns.
pub type PhysicalGradient3d<R> = (
    CausalTensorTrain<R>,
    CausalTensorTrain<R>,
    CausalTensorTrain<R>,
);
