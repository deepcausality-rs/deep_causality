/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Context, ContextStore, Datable, SpaceTemporal, Spatial, StoreError, Temporal};
use deep_causality_context_store::{
    ContextStorageStream, DataRecord, Recordable, SpaceRecord, SpaceTimeRecord, TimeRecord,
};

/// The streaming block, bounded on the streaming trait alone: a `ContextStore` over a backend
/// that implements `ContextStorage` and not `ContextStorageStream` has `hydrate`, `reserve` and
/// `store_branch` and lacks `subscribe` at compile time.
///
/// ```compile_fail
/// use core::future::{Future, ready};
/// use deep_causality_context::{ContextStore, UniformContext};
/// use deep_causality_context_store::*;
///
/// struct StoreOnly;
///
/// impl ContextStorage for StoreOnly {
///     type Error = ProjectionError;
///     type Slice = ContextId;
///     fn reserve(&self, _: usize) -> impl Future<Output = Result<IdReserve, Self::Error>> + Send { ready(Ok(IdReserve::new(vec![]))) }
///     fn create_context(&self, _: &str) -> impl Future<Output = Result<ContextId, Self::Error>> + Send { ready(Ok(1)) }
///     fn retract_context(&self, _: ContextId) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn create_node(&self, _: &[ContextoidRecord]) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn retract_node(&self, _: ContextoidId) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn create_edge(&self, _: &[RelationRecord]) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn retract_edge(&self, _: ContextoidId, _: ContextoidId) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn link(&self, _: ContextId, _: &[ContextoidId]) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn unlink(&self, _: ContextId, _: &[ContextoidId]) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn attach(&self, _: ContextId, _: ContextId) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn detach(&self, _: ContextId, _: ContextId) -> impl Future<Output = Result<(), Self::Error>> + Send { ready(Ok(())) }
///     fn commit(&self, _: &[ContextWrite]) -> impl Future<Output = Result<Vec<ContextId>, Self::Error>> + Send { ready(Ok(vec![])) }
///     fn lookup(&self, _: &[ContextoidId]) -> impl Future<Output = Result<Vec<Option<ContextoidRecord>>, Self::Error>> + Send { ready(Ok(vec![])) }
///     fn hydrate(&self, spec: &Self::Slice) -> impl Future<Output = Result<ContextSnapshot, Self::Error>> + Send {
///         ready(Ok(ContextSnapshot::new(ContextRecord::new(*spec, String::new()), vec![], vec![], vec![])))
///     }
/// }
///
/// let store = ContextStore::new(StoreOnly);
/// let _ = store.subscribe::<_, _, _, _>(&1, None);
/// ```
impl<S: ContextStorageStream> ContextStore<S> {
    /// A context kept current: the context the slice names as of a consistent point, and the
    /// stream of every change to it from that point on. The host applies the stream between
    /// evaluations through `Context::apply`.
    #[allow(clippy::type_complexity)]
    pub async fn subscribe<D, S_, T, ST>(
        &self,
        spec: &S::Slice,
        from: Option<S::Cursor>,
    ) -> Result<(Context<D, S_, T, ST>, S::Events), StoreError<S::Error>>
    where
        D: Datable + Clone + Recordable<DataRecord>,
        S_: Spatial + Clone + Recordable<SpaceRecord>,
        T: Temporal + Clone + Recordable<TimeRecord>,
        ST: SpaceTemporal + Clone + Recordable<SpaceTimeRecord>,
    {
        let (snapshot, events) = self
            .storage
            .subscribe(spec, from)
            .await
            .map_err(StoreError::Storage)?;
        Ok((Context::restore(snapshot)?, events))
    }
}
