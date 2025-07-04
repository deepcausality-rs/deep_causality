use crate::CsmGraph;

pub trait Freezable<N, W>
where
    W: Default,
{
    fn freeze(self) -> CsmGraph<N, W>;
}
