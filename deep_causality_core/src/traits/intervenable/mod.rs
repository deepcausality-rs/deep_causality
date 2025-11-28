use deep_causality_haft::{Effect5, HKT5};

/// Defines the `intervene` operation for a monadic effect system.
/// This trait is intended for causal reasoning systems where counterfactuals
/// are modeled by forcing a value at a specific point in a computation chain.
#[allow(clippy::type_complexity)]
pub trait Intervenable<E: Effect5>
where
    E::HktWitness: Sized,
{
    /// Overrides the value within an effectful computation.
    ///
    /// This function takes an existing `effect` and a `new_value`. It returns a new
    /// effect where the original value is discarded and replaced by `new_value`.
    ///
    /// Crucially, it should preserve the context of the computation:
    /// - **Error State**: If the incoming `effect` was already in an error state,
    ///   that error is propagated. An intervention cannot fix a previously broken chain.
    /// - **Log History**: The logs from the incoming `effect` are preserved, and a
    ///   new entry is added to signify that an intervention occurred.
    fn intervene<T>(
        effect: <E::HktWitness as HKT5<E::Fixed1, E::Fixed2, E::Fixed3, E::Fixed4>>::Type<T>,
        new_value: T,
    ) -> <E::HktWitness as HKT5<E::Fixed1, E::Fixed2, E::Fixed3, E::Fixed4>>::Type<T>
    where
        T: std::fmt::Debug;
}
