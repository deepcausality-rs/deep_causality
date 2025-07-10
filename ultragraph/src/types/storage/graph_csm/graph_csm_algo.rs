use crate::{CsmGraph, GraphAlgorithms};

impl<N, W> GraphAlgorithms<N, W> for CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default,
{
}
