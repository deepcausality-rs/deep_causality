use crate::{CausalityError, Causaloid};
use deep_causality_core::{EffectValue, PropagatingEffect};
use std::fmt::Debug;

/// Represents the execution step in a `Causaloid::Singleton` monadic chain.
///
/// This function executes the core causal function (`causal_fn` or `context_causal_fn`)
/// of the causaloid.
///
/// # Arguments
/// * `input`: The specific input value of type `I`.
/// * `causaloid`: A reference to the causaloid containing the function to execute.
///
/// # Returns
/// A `PropagatingEffect` containing either the resulting output
/// value of type `O` or an error.
pub(super) fn execute_causal_logic<I, O, PS, C>(
    input: I,
    causaloid: &Causaloid<I, O, PS, C>,
) -> PropagatingEffect<O>
where
    I: Default + Clone,
    O: Default + Clone + Debug,
    PS: Default + Clone,
    C: Clone,
{
    if let Some(context_fn) = &causaloid.context_causal_fn {
        if let Some(context) = causaloid.context.as_ref() {
            // context_fn signature: fn(EffectValue<I>, PS, Option<C>) -> PropagatingProcess<O, PS, C>
            // We invoke it with default state and the context.
            // The result is PropagatingProcess<O, PS, C>.
            // We need to convert it to PropagatingEffect<O>.

            let ev = EffectValue::from(input);
            let process = context_fn(ev, PS::default(), Some(context.clone()));

            // PropagatingProcess is CausalEffectPropagationProcess<O, PS, C>.
            // We extract the value and return it as PropagatingEffect (stateless).
            // This effectively discards the state and context updates for this trait method,
            // which aligns with MonadicCausable<I, O> signature.

            // Safely extract value potentially containing error
            match process.value.into_value() {
                Some(val) => PropagatingEffect::pure(val),
                None => PropagatingEffect::from_error(CausalityError(
                    deep_causality_core::CausalityErrorEnum::Custom(
                        "execute_causal_logic: context_fn returned None value".into(),
                    ),
                )),
            }
        } else {
            PropagatingEffect::from_error(CausalityError(
                deep_causality_core::CausalityErrorEnum::Custom(
                    "Causaloid::evaluate: context is None".into(),
                ),
            ))
        }
    } else if let Some(causal_fn) = &causaloid.causal_fn {
        causal_fn(input)
    } else {
        let err_msg = format!(
            "Causaloid {} is missing both causal_fn and context_causal_fn",
            causaloid.id
        );
        PropagatingEffect::from_error(CausalityError(
            deep_causality_core::CausalityErrorEnum::Custom(err_msg),
        ))
    }
}
