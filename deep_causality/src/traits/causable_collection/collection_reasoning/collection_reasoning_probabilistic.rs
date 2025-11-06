use crate::{AggregateLogic, CausalMonad, CausalityError, MonadicCausable, NumericalValue, PropagatingEffect};

pub(in crate::traits) fn _evaluate_probabilistic_logic<T: MonadicCausable<CausalMonad>>(
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

    let mut probabilities = Vec::new();
    let num_samples = 100; // Number of samples for estimation

    for cause in items {
        let evaluated_effect = cause.evaluate_monadic(effect.clone());

        if let Some(err) = evaluated_effect.error {
            return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(err),
                logs: evaluated_effect.logs,
            };
        }

        let prob = match evaluated_effect.value {
            crate::EffectValue::Deterministic(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            crate::EffectValue::Probabilistic(p) => p,
            crate::EffectValue::UncertainFloat(u) => {
                match u.estimate_probability_exceeds(threshold, num_samples) {
                    Ok(p) => p,
                    Err(e) => return PropagatingEffect {
                        value: crate::EffectValue::None,
                        error: Some(e),
                        logs: evaluated_effect.logs,
                    },
                }
            }
            crate::EffectValue::UncertainBool(u) => {
                match u.estimate_probability(num_samples) {
                    Ok(p) => p,
                    Err(e) => return PropagatingEffect {
                        value: crate::EffectValue::None,
                        error: Some(e),
                        logs: evaluated_effect.logs,
                    },
                }
            },
            _ => return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(CausalityError::new("Invalid effect type".to_string())),
                logs: evaluated_effect.logs,
            },
        };
        probabilities.push(prob);
    }

    let final_prob = match logic {
        AggregateLogic::All => probabilities.iter().product(),
        AggregateLogic::Any => 1.0 - probabilities.iter().map(|p| 1.0 - p).product::<f64>(),
        AggregateLogic::None => {
            1.0 - (1.0 - probabilities.iter().map(|p| 1.0 - p).product::<f64>())
        }
        AggregateLogic::Some(k) => {
            // This is a simplification. A full implementation would use binomial distribution.
            let count = probabilities.iter().filter(|&&p| p > 0.5).count();
            if count >= *k { 1.0 } else { 0.0 }
        }
    };

    PropagatingEffect::probabilistic(final_prob, effect.logs.clone())
}
