/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `ContextStore::subscribe`: a context kept current by draining its stream into
//! `Context::apply`, and the fixed scope of a subscription seen end to end. Expected values are
//! the records written through the backend, compared through canonical snapshots.
//!
//! Corner cases (rows A to K): A a stream with nothing to deliver,
//! `test_a_subscribed_context_follows_the_store`; C an attached container arriving after the
//! subscription, `test_an_attachment_is_materialised_by_resubscribing`; a container the host itself
//! stores while subscribed, `test_a_new_container_is_outside_an_existing_subscription`; every other
//! row n/a.

use deep_causality_context::{
    ContextStore, Contextoid, ContextoidType, ContextuableGraph, Data, ExtendableContextuableGraph,
    StoreErrorEnum, UniformContext,
};
use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{
    ContextEvent, ContextEvents, ContextStorage, ContextStorageStream, ContextoidId,
    ContextoidRecord, DataRecord, MemoryStorageError, NodeRecord,
};

fn count(id: ContextoidId, value: u64) -> ContextoidRecord {
    ContextoidRecord::new(id, NodeRecord::Data(DataRecord::Count(value)))
}

fn drain_into<E: ContextEvents>(ctx: &mut UniformContext, events: &mut E) -> Vec<ContextEvent> {
    let mut seen = Vec::new();
    while let Some(item) = block_on(events.next()) {
        let (_, event) = item.unwrap();
        ctx.apply(&event).unwrap();
        seen.push(event);
    }
    seen
}

#[test]
fn test_a_subscribed_context_follows_the_store() {
    let storage = MemoryStorage::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(storage.reserve(2)).unwrap().collect();
    block_on(storage.create_node(&[count(n[0], 1)])).unwrap();
    let base = block_on(storage.create_context("base")).unwrap();
    block_on(storage.link(base, &n[..1])).unwrap();
    let (mut ctx, mut events) = block_on(store.subscribe::<_, _, _, _>(&base, None)).unwrap();
    let ctx: &mut UniformContext = &mut ctx;
    assert_eq!(drain_into(ctx, &mut events), vec![]);
    block_on(storage.create_node(&[count(n[1], 2)])).unwrap();
    block_on(storage.link(base, &n[1..])).unwrap();
    block_on(storage.unlink(base, &n[..1])).unwrap();
    let seen = drain_into(ctx, &mut events);
    assert_eq!(seen.len(), 3);
    assert_eq!(
        ctx.snapshot().unwrap(),
        block_on(storage.hydrate(&base)).unwrap()
    );
}

#[test]
fn test_an_attachment_is_materialised_by_resubscribing() {
    let storage = MemoryStorage::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(storage.reserve(1)).unwrap().collect();
    block_on(storage.create_node(&[count(n[0], 1)])).unwrap();
    let base = block_on(storage.create_context("base")).unwrap();
    let weather = block_on(storage.create_context("weather")).unwrap();
    block_on(storage.link(weather, &n)).unwrap();
    let (mut ctx, mut events) = block_on(store.subscribe::<_, _, _, _>(&base, None)).unwrap();
    let ctx: &mut UniformContext = &mut ctx;
    let cursor = block_on(storage.attach(base, weather))
        .map(|_| block_on(storage.apply_batch(&[])).unwrap())
        .unwrap();
    let seen = drain_into(ctx, &mut events);
    assert_eq!(seen.len(), 1);
    assert!(matches!(seen[0], ContextEvent::ContextAttached { .. }));
    assert_eq!(ctx.extra_ctx_get_name(weather), Some("weather"));
    ctx.extra_ctx_set_current_id(weather).unwrap();
    assert_eq!(
        ctx.extra_ctx_node_count().unwrap(),
        0,
        "empty until resubscribed"
    );
    let (wider, _) = block_on(store.subscribe::<_, _, _, _>(&base, Some(cursor))).unwrap();
    let mut wider: UniformContext = wider;
    wider.extra_ctx_set_current_id(weather).unwrap();
    assert_eq!(wider.extra_ctx_node_count().unwrap(), 1);
    assert_eq!(
        wider.snapshot().unwrap(),
        block_on(storage.hydrate(&base)).unwrap()
    );
}

#[test]
fn test_an_unknown_slice_is_the_backends_refusal() {
    let store = ContextStore::new(MemoryStorage::new());
    let result =
        block_on(store.subscribe::<_, _, _, _>(&5, None)).map(|(_, _): (UniformContext, _)| ());
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Storage(MemoryStorageError::UnknownContext(
            5
        )))
    );
}

#[test]
fn test_a_new_container_is_outside_an_existing_subscription() {
    let storage = MemoryStorage::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(storage.reserve(1)).unwrap().collect();
    block_on(storage.create_node(&[count(n[0], 1)])).unwrap();
    let a = block_on(storage.create_context("a")).unwrap();
    block_on(storage.link(a, &n)).unwrap();
    let (mut ctx, mut events) = block_on(store.subscribe::<_, _, _, _>(&a, None)).unwrap();
    let ctx: &mut UniformContext = &mut ctx;

    // The host explores a branch of A and stores it as B, with an extra of its own.
    let mut branch = ctx.clone();
    let fresh: Vec<ContextoidId> = block_on(store.reserve(2)).unwrap().collect();
    branch
        .add_node(Contextoid::new(
            fresh[0],
            ContextoidType::Datoid(Data::new(fresh[0], 2)),
        ))
        .unwrap();
    branch.extra_ctx_add_new("weather", 1, true);
    branch
        .extra_ctx_add_node(Contextoid::new(
            fresh[1],
            ContextoidType::Datoid(Data::new(fresh[1], 3)),
        ))
        .unwrap();
    let b = block_on(store.store_branch("b", &branch)).unwrap();

    let before = ctx.snapshot().unwrap();
    let seen = drain_into(ctx, &mut events);
    assert!(
        seen.iter().all(|event| !matches!(
            event,
            ContextEvent::NodeLinked { context, .. }
                | ContextEvent::ContextAttached { context, .. } if *context == b
        )),
        "no membership event of B arrives: {seen:?}"
    );
    assert_eq!(
        ctx.snapshot().unwrap(),
        before,
        "A is unchanged by storing B"
    );

    let cursor = block_on(storage.apply_batch(&[])).unwrap();
    let (wider, _) = block_on(store.subscribe::<_, _, _, _>(&b, Some(cursor))).unwrap();
    let wider: UniformContext = wider;
    assert_eq!(
        wider.snapshot().unwrap(),
        block_on(storage.hydrate(&b)).unwrap()
    );
    assert_eq!(
        wider.snapshot().unwrap().nodes(),
        branch.snapshot().unwrap().nodes()
    );
    assert_eq!(wider.snapshot().unwrap().extras()[0].name(), "weather");
}

#[test]
fn test_a_local_extra_never_takes_a_store_container() {
    // The store and the context both count from 1: the context's second local extra and the
    // store's next container receive the same identifier.
    let storage = MemoryStorage::new();
    let store = ContextStore::new(storage.clone());
    let base = block_on(storage.create_context("base")).unwrap();
    let (mut ctx, mut events) = block_on(store.subscribe::<_, _, _, _>(&base, None)).unwrap();
    let ctx: &mut UniformContext = &mut ctx;
    ctx.extra_ctx_add_new("scratch", 1, false);
    let local = ctx.extra_ctx_add_new("local", 1, false);
    let sea = block_on(storage.create_context("sea")).unwrap();
    assert_eq!(sea, local, "the collision this test reproduces");
    block_on(storage.attach(base, sea)).unwrap();
    let (_, event) = block_on(events.next()).unwrap().unwrap();
    assert!(ctx.apply(&event).is_err());
    assert_eq!(ctx.extra_ctx_get_name(local), Some("local"));
    // A fresh subscription holds the store's container under the store's identifier.
    let (fresh, _) = block_on(store.subscribe::<_, _, _, _>(&base, None)).unwrap();
    let fresh: UniformContext = fresh;
    assert_eq!(fresh.extra_ctx_get_name(sea), Some("sea"));
}

#[test]
fn test_a_store_extra_keeps_its_identifier() {
    let storage = MemoryStorage::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(storage.reserve(1)).unwrap().collect();
    block_on(storage.create_node(&[count(n[0], 1)])).unwrap();
    let base = block_on(storage.create_context("base")).unwrap();
    let weather = block_on(storage.create_context("weather")).unwrap();
    block_on(storage.link(weather, &n)).unwrap();
    block_on(storage.attach(base, weather)).unwrap();

    // hydrate, snapshot and restore keep the store's identifier.
    let ctx: UniformContext = block_on(store.hydrate(&base)).unwrap();
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(snapshot, block_on(storage.hydrate(&base)).unwrap());
    assert_eq!(snapshot.extras()[0].id(), weather);
    let mut restored = UniformContext::restore(snapshot.clone()).unwrap();
    assert_eq!(restored.snapshot().unwrap(), snapshot);
    // The restored extra is the store's: an echo of the attachment is harmless and a link lands.
    restored
        .apply(&ContextEvent::ContextAttached {
            context: base,
            extra: deep_causality_context_store::ContextRecord::new(weather, "weather".to_string()),
        })
        .unwrap();
    assert_eq!(restored.snapshot().unwrap(), snapshot);
    restored
        .apply(&ContextEvent::NodeUnlinked {
            context: weather,
            node: n[0],
        })
        .unwrap();
    assert!(restored.snapshot().unwrap().extras()[0].nodes().is_empty());

    // store_branch stores each extra as a container of its own; the store assigns its identifier.
    let b = block_on(store.store_branch("b", &ctx)).unwrap();
    let stored = block_on(storage.hydrate(&b)).unwrap();
    let copy = &stored.extras()[0];
    assert!(copy.id() != weather && copy.id() != b);
    assert_eq!(copy.name(), "weather");
    assert_eq!(copy.nodes(), snapshot.extras()[0].nodes());
}
