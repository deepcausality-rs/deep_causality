use crate::{CausableCollectionAccessor, CausalMonad, MonadicCausable};

pub trait CausableCollectionExplaining<T>: CausableCollectionAccessor<T>
where
    T: MonadicCausable<CausalMonad>,
{
}
