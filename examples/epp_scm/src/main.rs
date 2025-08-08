/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod rung1_association;
mod rung2_intervention;
mod rung3_counterfactual;

fn main() {
    rung1_association::run_rung1_association();
    rung2_intervention::run_rung2_intervention();
    rung3_counterfactual::run_rung3_counterfactual();
}
