/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::model;
use deep_causality::{
    BaseCausaloid, BaseContext, CausableGraph, CausalEffectLog, CausalFnOutput, CausalityError,
    Causaloid, CausaloidGraph, ContextoidType, ContextuableGraph, Datable, EffectValue,
    Identifiable, IdentificationValue, NumericalValue,
};
use std::sync::{Arc, RwLock};

// Contextoid IDs
pub(crate) const NICOTINE_ID: IdentificationValue = 1;
pub(crate) const TAR_ID: IdentificationValue = 2;

pub(crate) fn get_causaloid_graph() -> (
    CausaloidGraph<BaseCausaloid<EffectValue, EffectValue>>,
    usize,
    usize,
) {
    // 1. Build CausaloidGraph
    let mut graph = CausaloidGraph::new(1);
    let smoke_idx = graph.add_causaloid(model::get_smoking_causaloid()).unwrap();
    let tar_idx = graph.add_causaloid(model::get_tar_causaloid()).unwrap();
    let cancer_idx = graph
        .add_causaloid(model::get_cancer_risk_causaloid())
        .unwrap();

    graph.add_edge(smoke_idx, tar_idx).unwrap();
    graph.add_edge(tar_idx, cancer_idx).unwrap();
    graph.freeze();

    (graph, smoke_idx, cancer_idx)
}

// Define Causaloids
pub(crate) fn get_smoking_causaloid() -> BaseCausaloid<EffectValue, EffectValue> {
    fn causal_fn(effect: EffectValue) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
        let mut log = CausalEffectLog::new();
        let nicotine_level = effect.as_numerical().unwrap_or(&0.0);
        let threshold: NumericalValue = 0.6;
        let high_nicotine_level = nicotine_level > &threshold;
        log.add_entry(&format!(
            "Nicotine level {} is higher than threshold {}: {}",
            effect, threshold, high_nicotine_level
        ));
        Ok(CausalFnOutput::new(
            EffectValue::Boolean(high_nicotine_level),
            log,
        ))
    }

    Causaloid::new(1, causal_fn, "Smoking Status")
}

pub(crate) fn get_tar_causaloid() -> BaseCausaloid<EffectValue, EffectValue> {
    fn causal_fn(effect: EffectValue) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
        let mut log = CausalEffectLog::new();
        let is_smoking = effect.as_bool().unwrap_or(false);
        log.add_entry(&format!("Is smoking {}", is_smoking));
        Ok(CausalFnOutput::new(EffectValue::Boolean(is_smoking), log))
    }

    Causaloid::new(2, causal_fn, "Tar in Lungs")
}

pub(crate) fn get_cancer_risk_causaloid() -> BaseCausaloid<EffectValue, EffectValue> {
    fn causal_fn(effect: EffectValue) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
        let mut log = CausalEffectLog::new();
        let has_tar = effect.as_bool().unwrap_or(false);
        log.add_entry(&format!("Has tar in lung {}", has_tar));
        // Cancer risk is high if tar is present.
        Ok(CausalFnOutput::new(EffectValue::Boolean(has_tar), log))
    }

    Causaloid::new(3, causal_fn, "Cancer Risk")
}

/// A contextual causal function that determines cancer risk.
/// It prioritizes checking for tar, then for smoking.
pub(crate) fn contextual_cancer_risk_logic(
    _effect: EffectValue,
    context: &Arc<RwLock<BaseContext>>,
) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
    let mut log = CausalEffectLog::new();
    let mut tar_level = 0.0;
    let mut nicotine_level = 0.0;

    let ctx = context.read().unwrap();

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
        log.add_entry(&format!(
            "Tar level elevated {} AND Nicotine level elevated {}  | Highest risk of cancer",
            tar_level, nicotine_level
        ));
        return Ok(CausalFnOutput::new(EffectValue::Boolean(true), log));
    }

    // Causal Logic: High tar is a direct cause of cancer risk, regardless of smoking.
    if tar_level > 0.6 {
        log.add_entry(&format!(
            "Tar level elevated {} | Higher risk of cancer",
            tar_level
        ));
        return Ok(CausalFnOutput::new(EffectValue::Boolean(true), log));
    }
    // If tar is low, then smoking becomes the relevant factor.
    if nicotine_level > 0.6 {
        log.add_entry(&format!(
            "Nicotine level elevated {} | Higher risk of cancer",
            tar_level
        ));
        return Ok(CausalFnOutput::new(EffectValue::Boolean(true), log));
    }

    log.add_entry("Low Tar and Nicotine levels. | Low risk of cancer");
    Ok(CausalFnOutput::new(EffectValue::Boolean(false), log))
}
