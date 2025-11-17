/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::{CsmCausaloid, CsmEthos};
use deep_causality::{
    BaseContext, CausalAction, CausalEffectLog, CausalFnOutput, CausalityError, Causaloid, Context,
    Contextoid, ContextoidType, ContextuableGraph, EffectEthos, IdentificationValue,
    NumericalValue, Root, TeloidModal,
};
use std::sync::{Arc, RwLock};

pub(crate) fn get_effect_ethos() -> CsmEthos {
    let mut ethos = EffectEthos::new()
        .add_deterministic_norm(
            1,
            "High temp alert",
            &["temperature"],
            |_context, _action| true,
            TeloidModal::Impermissible,
            1,
            1,
            1,
        )
        .unwrap();
    ethos.verify_graph().unwrap();
    ethos
}

pub(crate) fn get_test_causaloid(context: Arc<RwLock<BaseContext>>) -> CsmCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    fn context_causal_fn(
        effect: NumericalValue,
        _context: &Arc<RwLock<BaseContext>>,
    ) -> Result<CausalFnOutput<bool>, CausalityError> {
        let mut log = CausalEffectLog::new();

        if effect.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;

        let is_active = effect.ge(&threshold);

        log.add_entry(&format!(
            "Observation {} is larger than threshold {}: {}",
            effect, threshold, is_active
        ));

        // Log each relevant step
        log.add_entry("Causal function executed successfully");
        // Return the final result and its log.
        Ok(CausalFnOutput::new(is_active, log))
    }

    Causaloid::new_with_context(id, context_causal_fn, context, description)
}

pub(crate) fn get_alert_action() -> CausalAction {
    let func = || {
        println!("Alert! High temperature detected!");
        Ok(())
    };
    let descr = "Action that triggers an alert";
    let version = 1;
    CausalAction::new(func, descr, version)
}

pub(crate) fn get_base_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    let mut context = Context::with_capacity(id, name, 1);
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    let idx = context.add_node(contextoid).expect("Failed to add node");
    assert_eq!(idx, 0);
    assert_eq!(context.size(), 1);

    context
}
