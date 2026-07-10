/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    BaseContext, CausableGraph, CausalityError, CausalityErrorEnum, Causaloid, CausaloidGraph,
    ContextoidType, ContextuableGraph, Datable, Identifiable, IdentificationValue,
    PropagatingEffect, PropagatingProcess, Verdict,
};
use deep_causality_core::CausalEffect;
use std::sync::{Arc, RwLock};

// Contextoid IDs
pub(crate) const NICOTINE_ID: IdentificationValue = 1;
pub(crate) const TAR_ID: IdentificationValue = 2;

/// State struct for SCM causal chain
#[derive(Debug, Clone, Copy, Default)]
pub struct ScmState {
    pub nicotine_level: f64,
    pub has_high_nicotine: bool,
    pub has_tar: bool,
    pub cancer_risk: bool,
}

/// Graph reasoning requires the wire carrier to be a `Verdict` (the Stage-4 join bound:
/// reconvergent values fuse with the commutative `∇ = Verdict::join`). `ScmState` is the
/// **product lattice** of its fields — the `bool` fields are the Boolean carrier; the
/// unconstrained real field uses the extended-real min/max lattice (bounds `±∞`, complement
/// `1 − x`), which is lawful for arbitrary values, unlike the `[0, 1]` MV bounds. This
/// chain-shaped SCM has no reconvergent join, so the instance is a carrier bound, never
/// exercised as a merge.
impl Verdict for ScmState {
    fn bottom() -> Self {
        ScmState {
            nicotine_level: f64::NEG_INFINITY,
            has_high_nicotine: bool::bottom(),
            has_tar: bool::bottom(),
            cancer_risk: bool::bottom(),
        }
    }
    fn top() -> Self {
        ScmState {
            nicotine_level: f64::INFINITY,
            has_high_nicotine: bool::top(),
            has_tar: bool::top(),
            cancer_risk: bool::top(),
        }
    }
    fn meet(self, other: Self) -> Self {
        ScmState {
            nicotine_level: self.nicotine_level.meet(other.nicotine_level),
            has_high_nicotine: self.has_high_nicotine.meet(other.has_high_nicotine),
            has_tar: self.has_tar.meet(other.has_tar),
            cancer_risk: self.cancer_risk.meet(other.cancer_risk),
        }
    }
    fn join(self, other: Self) -> Self {
        ScmState {
            nicotine_level: self.nicotine_level.join(other.nicotine_level),
            has_high_nicotine: self.has_high_nicotine.join(other.has_high_nicotine),
            has_tar: self.has_tar.join(other.has_tar),
            cancer_risk: self.cancer_risk.join(other.cancer_risk),
        }
    }
    fn complement(self) -> Self {
        ScmState {
            nicotine_level: self.nicotine_level.complement(),
            has_high_nicotine: self.has_high_nicotine.complement(),
            has_tar: self.has_tar.complement(),
            cancer_risk: self.cancer_risk.complement(),
        }
    }
}

pub type ScmCausaloid = Causaloid<ScmState, ScmState, (), ()>;
pub type ScmGraph = CausaloidGraph<ScmCausaloid>;

pub(crate) fn get_causaloid_graph() -> (ScmGraph, usize, usize) {
    // 1. Build CausaloidGraph
    let mut graph = CausaloidGraph::new(1);
    let smoke_idx = graph.add_causaloid(get_smoking_causaloid()).unwrap();
    let tar_idx = graph.add_causaloid(get_tar_causaloid()).unwrap();
    let cancer_idx = graph.add_causaloid(get_cancer_risk_causaloid()).unwrap();

    graph.add_edge(smoke_idx, tar_idx).unwrap();
    graph.add_edge(tar_idx, cancer_idx).unwrap();
    graph.freeze();

    (graph, smoke_idx, cancer_idx)
}

// Define Causaloids
pub(crate) fn get_smoking_causaloid() -> ScmCausaloid {
    fn causal_fn(input: ScmState) -> PropagatingEffect<ScmState> {
        let threshold = 0.6;
        let high_nicotine_level = input.nicotine_level > threshold;

        let output = ScmState {
            nicotine_level: input.nicotine_level,
            has_high_nicotine: high_nicotine_level,
            has_tar: input.has_tar,
            cancer_risk: input.cancer_risk,
        };

        PropagatingEffect::pure(output)
    }

    Causaloid::new(1, causal_fn, "Smoking Status")
}

pub(crate) fn get_tar_causaloid() -> ScmCausaloid {
    fn causal_fn(input: ScmState) -> PropagatingEffect<ScmState> {
        // Tar in lungs follows from smoking (high nicotine)
        let output = ScmState {
            nicotine_level: input.nicotine_level,
            has_high_nicotine: input.has_high_nicotine,
            has_tar: input.has_high_nicotine, // Tar builds up if smoking
            cancer_risk: input.cancer_risk,
        };

        PropagatingEffect::pure(output)
    }

    Causaloid::new(2, causal_fn, "Tar in Lungs")
}

pub(crate) fn get_cancer_risk_causaloid() -> ScmCausaloid {
    fn causal_fn(input: ScmState) -> PropagatingEffect<ScmState> {
        // Cancer risk is high if tar is present
        let output = ScmState {
            nicotine_level: input.nicotine_level,
            has_high_nicotine: input.has_high_nicotine,
            has_tar: input.has_tar,
            cancer_risk: input.has_tar,
        };

        PropagatingEffect::pure(output)
    }

    Causaloid::new(3, causal_fn, "Cancer Risk")
}

/// Contextual causaloid for counterfactual analysis
pub type ContextualScmCausaloid = Causaloid<f64, bool, (), Arc<RwLock<BaseContext>>>;

/// A contextual causal function that determines cancer risk.
/// It prioritizes checking for tar, then for smoking.
pub(crate) fn contextual_cancer_risk_logic(
    _effect: CausalEffect<f64>,
    _state: (),
    context: Option<Arc<RwLock<BaseContext>>>,
) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
    let ctx_arc = match context {
        Some(c) => c,
        None => {
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Context is missing".into(),
            )));
        }
    };

    let mut tar_level = 0.0;
    let mut nicotine_level = 0.0;

    let ctx = ctx_arc.read().unwrap();

    // Scan the context for relevant data.
    for i in 0..ctx.number_of_nodes() {
        if let Some(node) = ctx.get_node(i)
            && let ContextoidType::Datoid(data_node) = node.vertex_type()
        {
            match data_node.id() {
                TAR_ID => tar_level = data_node.get_data(),
                NICOTINE_ID => nicotine_level = data_node.get_data(),
                _ => (),
            }
        }
    }

    if tar_level > 0.6 && nicotine_level > 0.6 {
        return PropagatingProcess::pure(true); // Highest risk
    }

    if tar_level > 0.6 {
        return PropagatingProcess::pure(true); // High tar = high risk
    }

    if nicotine_level > 0.6 {
        return PropagatingProcess::pure(true); // High nicotine = high risk
    }

    PropagatingProcess::pure(false) // Low risk
}

pub(crate) fn get_contextual_cancer_causaloid(
    context: Arc<RwLock<BaseContext>>,
) -> ContextualScmCausaloid {
    Causaloid::new_with_context(
        1,
        contextual_cancer_risk_logic,
        context,
        "Contextual Cancer Risk",
    )
}
