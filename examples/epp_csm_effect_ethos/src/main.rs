use deep_causality::*;
use std::sync::Arc;

fn main() {
    // Create a context and wrap it in an Arc for shared ownership
    let context = Arc::new(get_base_context());

    // 1. Build a CausaloidGraph
    let causaloid = get_test_causaloid(Arc::clone(&context));
    let state = CausalState::new(1, 1, PropagatingEffect::Numerical(0.0), causaloid);
    let action = get_alert_action();

    // 2. Build an EffectEthos
    let ethos = get_effect_ethos();

    // 3. Initialize CSM with and without EffectEthos
    let csm_no_ethos = CSM::new(&[(&state, &action)], None);
    let csm_with_ethos = CSM::new(&[(&state, &action)], Some((ethos, &["temperature"])));

    // 4. Run the CSMs and show the difference
    println!("--- Running CSM without EffectEthos ---");
    let data = PropagatingEffect::Numerical(0.6);
    let res_no_ethos = csm_no_ethos.eval_single_state(1, &data);
    println!();
    println!("Result without ethos: {:?}", res_no_ethos);

    println!("\n--- Running CSM with EffectEthos ---");
    let res_with_ethos = csm_with_ethos.eval_single_state(1, &data);
    println!();
    println!("Result with ethos: {:?}", res_with_ethos);
}

fn get_effect_ethos() -> EffectEthos<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
> {
    let mut ethos = EffectEthos::new()
        .add_norm(
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

fn get_test_causaloid(context: Arc<BaseContext>) -> BaseCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    fn context_causal_fn(
        effect: &PropagatingEffect,
        _context: &Arc<BaseContext>,
    ) -> Result<PropagatingEffect, CausalityError> {
        let obs = match effect {
            PropagatingEffect::Numerical(val) => *val,
            _ => 0.0,
        };
        let threshold: NumericalValue = 0.55;
        if obs > threshold {
            Ok(PropagatingEffect::Deterministic(true))
        } else {
            Ok(PropagatingEffect::Deterministic(false))
        }
    }

    Causaloid::new_with_context(id, context_causal_fn, context, description)
}

fn get_alert_action() -> CausalAction {
    let func = || {
        println!("Alert! High temperature detected!");
        Ok(())
    };
    let descr = "Action that triggers an alert";
    let version = 1;
    CausalAction::new(func, descr, version)
}

fn get_base_context() -> BaseContext {
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
