use crate::{CausalityError, Causaloid, Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
use deep_causality_core::PropagatingEffect;

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
pub(super) fn execute_causal_logic<I, O, D, S, T, ST, SYM, VS, VT>(
    input: I,
    causaloid: &Causaloid<I, O, D, S, T, ST, SYM, VS, VT>,
) -> PropagatingEffect<O>
where
    I: Default,
    O: Default + Clone + std::fmt::Debug,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    if let Some(context_fn) = &causaloid.context_causal_fn {
        if let Some(context) = causaloid.context.as_ref() {
            context_fn(input, context)
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
