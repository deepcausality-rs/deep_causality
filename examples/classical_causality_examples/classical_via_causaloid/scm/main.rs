/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod model;
mod rung1_association;
mod rung2_intervention;
mod rung3_counterfactual;

// For more detailed explanations of the automated reasoning, set this flag to true.
const EXAPLAIN: bool = false;

fn main() {
    rung1_association::run_rung1_association(EXAPLAIN);
    rung2_intervention::run_rung2_intervention();
    rung3_counterfactual::run_rung3_counterfactual(EXAPLAIN);
}
