mod model;
mod types;

use deep_causality::*;
use std::sync::{Arc, RwLock};

fn main() {
    // Create a context and wrap it in an Arc for shared ownership
    let context = Arc::new(RwLock::new(model::get_base_context()));

    // 1. Build a CausaloidGraph
    let causaloid = model::get_test_causaloid(Arc::clone(&context));
    let state = CausalState::new(
        1,
        1,
        PropagatingEffect::from_numerical(0.0),
        causaloid,
        None,
    );
    let action = model::get_alert_action();

    // 2. Build an EffectEthos
    let ethos = model::get_effect_ethos();

    // 3. Initialize CSM with and without EffectEthos
    let csm_no_ethos = CSM::new(&[(&state, &action)], None);
    let csm_with_ethos = CSM::new(&[(&state, &action)], Some((ethos, &["temperature"])));

    // 4. Run the CSMs and show the difference
    println!("--- Running CSM without EffectEthos ---");
    let data = PropagatingEffect::from_numerical(0.6);
    let res_no_ethos = csm_no_ethos.eval_single_state(1, &data);
    println!();
    println!("Result without ethos: {:?}", res_no_ethos);

    println!("\n--- Running CSM with EffectEthos ---");
    let res_with_ethos = csm_with_ethos.eval_single_state(1, &data);
    println!();
    println!("Result with ethos: Forbidden",);
    println!("Explanation: {}", res_with_ethos.unwrap_err());
}
