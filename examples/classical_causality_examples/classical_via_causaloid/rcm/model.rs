/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{CausableGraph, Causaloid, CausaloidGraph, PropagatingEffect, Verdict};

/// State struct to pass between causaloids in the RCM graph
#[derive(Debug, Clone, Copy, Default)]
pub struct RcmState {
    pub initial_bp: f64,
    pub drug_administered: bool,
    pub drug_effect: f64,
    pub final_bp: f64,
}

/// Graph reasoning requires the wire carrier to be a `Verdict` (the Stage-4 join bound:
/// reconvergent values fuse with the commutative `∇ = Verdict::join`). `RcmState` is the
/// **product lattice** of its fields — the `bool` field is the Boolean carrier; the blood-
/// pressure reals use the extended-real min/max lattice (bounds `±∞`, complement `1 − x`),
/// lawful for arbitrary values. This chain-shaped RCM has no reconvergent join, so the instance
/// is a carrier bound, never exercised as a merge.
impl Verdict for RcmState {
    fn bottom() -> Self {
        RcmState {
            initial_bp: f64::NEG_INFINITY,
            drug_administered: bool::bottom(),
            drug_effect: f64::NEG_INFINITY,
            final_bp: f64::NEG_INFINITY,
        }
    }
    fn top() -> Self {
        RcmState {
            initial_bp: f64::INFINITY,
            drug_administered: bool::top(),
            drug_effect: f64::INFINITY,
            final_bp: f64::INFINITY,
        }
    }
    fn meet(self, other: Self) -> Self {
        RcmState {
            initial_bp: self.initial_bp.min(other.initial_bp),
            drug_administered: self.drug_administered.meet(other.drug_administered),
            drug_effect: self.drug_effect.min(other.drug_effect),
            final_bp: self.final_bp.min(other.final_bp),
        }
    }
    fn join(self, other: Self) -> Self {
        RcmState {
            initial_bp: self.initial_bp.max(other.initial_bp),
            drug_administered: self.drug_administered.join(other.drug_administered),
            drug_effect: self.drug_effect.max(other.drug_effect),
            final_bp: self.final_bp.max(other.final_bp),
        }
    }
    fn complement(self) -> Self {
        RcmState {
            initial_bp: 1.0 - self.initial_bp,
            drug_administered: self.drug_administered.complement(),
            drug_effect: 1.0 - self.drug_effect,
            final_bp: 1.0 - self.final_bp,
        }
    }
}

pub type RCMCausaloid = Causaloid<RcmState, RcmState, (), ()>;
pub type RCMCausalGraph = CausaloidGraph<RCMCausaloid>;

pub(crate) fn get_causaloid_graph() -> RCMCausalGraph {
    let drug_effect_causaloid = get_drug_effect_causaloid();
    let final_bp_causaloid = get_final_bp_causaloid();

    // Create the CausaloidGraph
    let mut causaloid_graph = CausaloidGraph::new(0);

    // Add the Drug effect causaloid
    let drug_effect_idx = causaloid_graph
        .add_causaloid(drug_effect_causaloid)
        .unwrap();

    // Add the blood pressure causaloid
    let final_bp_idx = causaloid_graph.add_causaloid(final_bp_causaloid).unwrap();
    causaloid_graph
        .add_edge(drug_effect_idx, final_bp_idx)
        .unwrap();

    // Freeze the graph to ensure high performance reasoning
    causaloid_graph.freeze();

    causaloid_graph
}

fn get_drug_effect_causaloid() -> RCMCausaloid {
    let drug_effect_causaloid_id = 1;
    let drug_effect_causaloid_description = "Determines drug effect based on administration";

    fn causal_fn(input: RcmState) -> PropagatingEffect<RcmState> {
        let drug_effect_value = if input.drug_administered { -10.0 } else { 0.0 };

        let output = RcmState {
            initial_bp: input.initial_bp,
            drug_administered: input.drug_administered,
            drug_effect: drug_effect_value,
            final_bp: 0.0, // Not yet calculated
        };

        PropagatingEffect::pure(output)
    }

    Causaloid::new(
        drug_effect_causaloid_id,
        causal_fn,
        drug_effect_causaloid_description,
    )
}

fn get_final_bp_causaloid() -> RCMCausaloid {
    let final_bp_causaloid_id = 2;
    let final_bp_causaloid_description = "Calculates final blood pressure";

    fn causal_fn(input: RcmState) -> PropagatingEffect<RcmState> {
        let final_bp = input.initial_bp + input.drug_effect;

        let output = RcmState {
            initial_bp: input.initial_bp,
            drug_administered: input.drug_administered,
            drug_effect: input.drug_effect,
            final_bp,
        };

        PropagatingEffect::pure(output)
    }

    Causaloid::new(
        final_bp_causaloid_id,
        causal_fn,
        final_bp_causaloid_description,
    )
}
