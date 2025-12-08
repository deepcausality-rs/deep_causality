/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    BaseContext, CausalAction, CausalityError, CausalityErrorEnum, Causaloid, Context, Contextoid,
    ContextoidType, ContextuableGraph, EffectValue, IdentificationValue, NumericalValue,
    PropagatingProcess, Root,
};
use deep_causality_ethos::{EffectEthos, TeloidModal};
use std::sync::{Arc, RwLock};

// Type aliases for manageable generics
pub type CsmCausaloid = Causaloid<f64, bool, (), Arc<RwLock<BaseContext>>>;

pub type CsmEthos = EffectEthos<
    deep_causality::Data<NumericalValue>,
    deep_causality::EuclideanSpace,
    deep_causality::EuclideanTime,
    deep_causality::EuclideanSpacetime,
    deep_causality::BaseSymbol,
    deep_causality::FloatType,
    deep_causality::FloatType,
>;

pub(crate) fn get_effect_ethos() -> CsmEthos {
    let mut ethos = EffectEthos::new()
        .add_deterministic_norm(
            1,
            "high_temp_alert",
            &["temperature"],
            |_context, _action| true, // Always active for demo
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

    // New API: fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
    fn context_causal_fn(
        effect: EffectValue<f64>,
        _state: (),
        _context: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        let obs = effect.into_value().unwrap_or(0.0);

        if obs.is_sign_negative() {
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Observation is negative".into(),
            )));
        }

        let threshold: NumericalValue = 0.55;
        let is_active = obs.ge(&threshold);

        PropagatingProcess::pure(is_active)
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
