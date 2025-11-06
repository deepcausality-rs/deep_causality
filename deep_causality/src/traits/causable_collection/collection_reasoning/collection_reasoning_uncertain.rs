use crate::{AggregateLogic, CausalMonad, CausalityError, MonadicCausable, NumericalValue, PropagatingEffect};
use deep_causality_uncertain::Uncertain;

pub(in crate::traits) fn _evaluate_uncertain_logic<T: MonadicCausable<CausalMonad>>(
    items: Vec<&T>,
    effect: &PropagatingEffect,
    logic: &AggregateLogic,
    threshold: NumericalValue,
) -> PropagatingEffect {
    if items.is_empty() {
        return PropagatingEffect {
            value: crate::EffectValue::None,
            error: Some(CausalityError(
                "No Causaloids found to evaluate".to_string(),
            )),
            logs: effect.logs.clone(),
        };
    }

    let mut uncertain_bools: Vec<Uncertain<bool>> = Vec::new();

    for cause in items {
        let evaluated_effect = cause.evaluate_monadic(effect.clone());

        if let Some(err) = evaluated_effect.error {
            return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(err),
                logs: evaluated_effect.logs,
            };
        }

        match evaluated_effect.value {
            crate::EffectValue::UncertainBool(u) => {
                uncertain_bools.push(u);
            }
            crate::EffectValue::UncertainFloat(u) => {
                uncertain_bools.push(u.greater_than(threshold));
            }
            _ => {
                return PropagatingEffect {
                    value: crate::EffectValue::None,
                    error: Some(CausalityError::new(format!(
                        "Invalid effect type for uncertain evaluation: {:?}",
                        evaluated_effect.value
                    ))),
                    logs: evaluated_effect.logs,
                };
            }
        }
    }

    if uncertain_bools.is_empty() {
        // This case might be hit if all effects were of ignored types.
        // Returning an error might be better, but for now, let's stick to the original logic.
        return PropagatingEffect {
            value: crate::EffectValue::None,
            error: Some(CausalityError::new(
                "No uncertain-compatible effects found in the collection".to_string(),
            )),
            logs: effect.logs.clone(),
        };
    }

    let final_uncertain_bool = match logic {
        AggregateLogic::All => {
            let mut result = uncertain_bools.remove(0);
            for u in uncertain_bools {
                result = result & u;
            }
            result
        }
        AggregateLogic::Any => {
            let mut result = uncertain_bools.remove(0);
            for u in uncertain_bools {
                result = result | u;
            }
            result
        }
        AggregateLogic::None => {
            let mut any_result = uncertain_bools.remove(0);
            for u in uncertain_bools {
                any_result = any_result | u;
            }
            !any_result
        }
        AggregateLogic::Some(k) => {
            let confidence = 0.95; // Default confidence for hypothesis testing
            let epsilon = 0.05; // The indifference region.
            let max_samples = 1000; // The maximum number of samples to take.

            let bool_results: Vec<bool> = uncertain_bools
                .into_iter()
                .map(|u| {
                    match u.to_bool(threshold, confidence, epsilon, max_samples) {
                        Ok(b) => b,
                        Err(e) => panic!("Error converting uncertain bool to bool: {}", e), // This should be handled better
                    }
                })
                .collect();

            let true_count = bool_results.iter().filter(|&&b| b).count();
            Uncertain::<bool>::point(true_count >= *k)
        }
    };

    PropagatingEffect::uncertain_bool(final_uncertain_bool, effect.logs.clone())
}
