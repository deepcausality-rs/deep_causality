use crate::DynamicGraph;

pub trait Unfreezable<N, W> {
    fn unfreeze(self) -> DynamicGraph<N, W>;
}
