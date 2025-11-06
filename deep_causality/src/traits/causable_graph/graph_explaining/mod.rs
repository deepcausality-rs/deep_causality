use crate::{CausableGraph, CausalMonad, MonadicCausable};

#[allow(clippy::type_complexity)]
pub trait CausableGraphExplaining<T>: CausableGraph<T>
where
    T: MonadicCausable<CausalMonad> + PartialEq + Clone,
{
}
