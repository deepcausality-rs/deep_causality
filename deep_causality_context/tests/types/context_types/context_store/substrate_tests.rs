/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `create_node_via` and `hydrate_via`: values externalised into a substrate on the way in and
//! internalised on the way out, so a value-typed context reaches a store that holds references.
//! Expected values are the payloads written.
//!
//! Corner cases (rows A to K): A an empty node slice, `test_an_empty_slice_deposits_nothing`; C a
//! payload already a reference is passed through untouched, same test; a payload the type cannot
//! hold, `test_a_payload_the_type_cannot_hold_is_loud`; a container referencing another whose
//! nodes are references beside a node that is not, `test_extras_are_resolved_too`; every other
//! row n/a.

use deep_causality_context::{
    BaseContext, Context, ContextStore, Contextoid, ContextoidType, ContextuableGraph, Data,
    EuclideanSpace, EuclideanSpacetime, EuclideanTime, ExtendableContextuableGraph, Storable,
    StoreErrorEnum, SubstrateRef,
};
use deep_causality_context_store::utils_test::{MemoryStorage, MemorySubstrate, block_on};
use deep_causality_context_store::{
    ContextId, ContextStorage, ContextoidId, ContextoidRecord, DataRecord, MemoryStorageError,
    MemorySubstrateError, NodeRecord, ProjectionError, SpaceRecord, Substrate,
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;

fn number(
    id: ContextoidId,
    value: f64,
) -> Contextoid<Data<f64>, EuclideanSpace<f64>, EuclideanTime<f64>, EuclideanSpacetime<f64>> {
    Contextoid::new(id, ContextoidType::Datoid(Data::new(id, value)))
}

/// Deposits a context's nodes through the substrate and links them into a new container.
fn store_via(
    store: &ContextStore<MemoryStorage>,
    substrate: &MemorySubstrate,
    nodes: &[ContextoidRecord],
    name: &str,
) -> ContextId {
    block_on(store.create_node_via(substrate, nodes)).unwrap();
    let ids: Vec<ContextoidId> = nodes.iter().map(|r| r.id()).collect();
    let container = block_on(store.storage().create_context(name)).unwrap();
    block_on(store.storage().link(container, &ids)).unwrap();
    container
}

#[test]
fn test_a_value_typed_context_reaches_a_reference_holding_store() {
    let storage = MemoryStorage::new();
    let substrate = MemorySubstrate::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(store.reserve(2)).unwrap().collect();
    let mut ctx: BaseContext = Context::with_capacity(1, "values", 2);
    ctx.add_node(number(n[0], 2.5)).unwrap();
    ctx.add_node(number(n[1], -1.0)).unwrap();
    let snapshot = ctx.snapshot().unwrap();
    let container = store_via(&store, &substrate, snapshot.nodes(), "values");

    let held = block_on(storage.lookup(&n)).unwrap();
    for (record, id) in held.iter().zip(&n) {
        let Some(record) = record else { panic!("held") };
        let NodeRecord::Data(DataRecord::Reference(reference)) = record.node() else {
            panic!("a reference in the graph, not a value");
        };
        assert_eq!(reference.source(), MemorySubstrate::SOURCE);
        assert_eq!(reference.key(), id.to_string());
    }
    let back: BaseContext = block_on(store.hydrate_via(&substrate, &container)).unwrap();
    assert_eq!(back.snapshot().unwrap().nodes(), snapshot.nodes());
    let index = back.get_node_index_by_id(n[1]).unwrap();
    assert_eq!(
        back.get_node(index).unwrap().vertex_type().dataoid(),
        Some(&Data::new(n[1], -1.0))
    );
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Reading {
    temperature: f64,
    samples: u64,
}

impl Storable for Reading {
    fn to_record(&self) -> DataRecord {
        DataRecord::Fields(vec![
            ("temperature".to_string(), self.temperature.to_record()),
            ("samples".to_string(), self.samples.to_record()),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let DataRecord::Fields(entries) = record else {
            return Err(ProjectionError::WrongPayload(
                id,
                "Fields",
                record.kind_name(),
            ));
        };
        let field = |name: &'static str| {
            entries
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
                .ok_or(ProjectionError::MissingField(id, name))
        };
        Ok(Self {
            temperature: f64::from_record(id, field("temperature")?)?,
            samples: u64::from_record(id, field("samples")?)?,
        })
    }
}

type ReadingContext =
    Context<Data<Reading>, EuclideanSpace<f64>, EuclideanTime<f64>, EuclideanSpacetime<f64>>;

#[test]
fn test_a_struct_payload_reaches_a_reference_holding_store() {
    let storage = MemoryStorage::new();
    let substrate = MemorySubstrate::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(store.reserve(1)).unwrap().collect();
    let reading = Reading {
        temperature: 21.5,
        samples: 3,
    };
    let mut ctx: ReadingContext = Context::with_capacity(1, "readings", 1);
    ctx.add_node(Contextoid::new(
        n[0],
        ContextoidType::Datoid(Data::new(n[0], reading.clone())),
    ))
    .unwrap();
    let snapshot = ctx.snapshot().unwrap();
    let container = store_via(&store, &substrate, snapshot.nodes(), "readings");
    let held = block_on(storage.lookup(&n))
        .unwrap()
        .swap_remove(0)
        .unwrap();
    assert!(matches!(
        held.node(),
        NodeRecord::Data(DataRecord::Reference(_))
    ));
    let back: ReadingContext = block_on(store.hydrate_via(&substrate, &container)).unwrap();
    let index = back.get_node_index_by_id(n[0]).unwrap();
    assert_eq!(
        back.get_node(index).unwrap().vertex_type().dataoid(),
        Some(&Data::new(n[0], reading))
    );
}

#[test]
fn test_a_payload_the_type_cannot_hold_is_loud() {
    let storage = MemoryStorage::new();
    let substrate = MemorySubstrate::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(store.reserve(1)).unwrap().collect();
    let reference = block_on(substrate.deposit(n[0], &DataRecord::Count(1))).unwrap();
    block_on(storage.create_node(&[ContextoidRecord::new(
        n[0],
        NodeRecord::Data(DataRecord::Reference(reference)),
    )]))
    .unwrap();
    let container = block_on(storage.create_context("counts")).unwrap();
    block_on(storage.link(container, &n)).unwrap();
    let result = block_on(store.hydrate_via(&substrate, &container)).map(|_: BaseContext| ());
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Projection(ProjectionError::WrongPayload(
            n[0], "Number", "Count"
        )))
    );
}

#[test]
fn test_an_empty_slice_deposits_nothing() {
    let storage = MemoryStorage::new();
    let substrate = MemorySubstrate::new();
    let store = ContextStore::new(storage.clone());
    assert_eq!(block_on(store.create_node_via(&substrate, &[])), Ok(()));
    assert!(substrate.is_empty());
    // A payload that is already a reference passes through untouched.
    let n: Vec<ContextoidId> = block_on(store.reserve(1)).unwrap().collect();
    let reference = SubstrateRef::new("elsewhere".to_string(), "k".to_string());
    let node = ContextoidRecord::new(
        n[0],
        NodeRecord::Data(DataRecord::Reference(reference.clone())),
    );
    block_on(store.create_node_via(&substrate, std::slice::from_ref(&node))).unwrap();
    assert!(substrate.is_empty());
    assert_eq!(block_on(storage.lookup(&n)).unwrap(), vec![Some(node)]);
}

#[test]
fn test_each_side_of_the_error_is_named() {
    let storage = MemoryStorage::new();
    let substrate = MemorySubstrate::new();
    let store = ContextStore::new(storage.clone());
    // The backend refuses an identifier no reserve handed out.
    let stranger = ContextoidRecord::new(1_000, NodeRecord::Data(DataRecord::Number(1.0)));
    let result = block_on(store.create_node_via(&substrate, &[stranger]));
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Storage(
            MemoryStorageError::IdentityNotReserved(1_000)
        ))
    );
    // The substrate refuses a reference it does not hold.
    let n: Vec<ContextoidId> = block_on(store.reserve(1)).unwrap().collect();
    let unknown = SubstrateRef::new(MemorySubstrate::SOURCE.to_string(), "gone".to_string());
    block_on(storage.create_node(&[ContextoidRecord::new(
        n[0],
        NodeRecord::Data(DataRecord::Reference(unknown.clone())),
    )]))
    .unwrap();
    let container = block_on(storage.create_context("gone")).unwrap();
    block_on(storage.link(container, &n)).unwrap();
    let result = block_on(store.hydrate_via(&substrate, &container)).map(|_: BaseContext| ());
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Substrate(
            MemorySubstrateError::UnknownReference(unknown)
        ))
    );
}

#[test]
fn test_extras_are_resolved_too() {
    let storage = MemoryStorage::new();
    let substrate = MemorySubstrate::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(store.reserve(3)).unwrap().collect();
    // The base holds a value and a space node; the referenced container holds a value.
    let mut base: BaseContext = Context::with_capacity(1, "base", 2);
    base.add_node(number(n[0], 0.5)).unwrap();
    base.add_node(Contextoid::new(
        n[1],
        ContextoidType::Spaceoid(EuclideanSpace::new(n[1], 1.0, 2.0, 3.0)),
    ))
    .unwrap();
    let mut extra: BaseContext = Context::with_capacity(2, "weather", 1);
    extra.add_node(number(n[2], 7.0)).unwrap();
    let base_id = store_via(&store, &substrate, base.snapshot().unwrap().nodes(), "base");
    let extra_id = store_via(
        &store,
        &substrate,
        extra.snapshot().unwrap().nodes(),
        "weather",
    );
    block_on(storage.attach(base_id, extra_id)).unwrap();
    assert_eq!(substrate.len(), 2, "the space node was not deposited");
    let held = block_on(storage.lookup(&n[1..2]))
        .unwrap()
        .swap_remove(0)
        .unwrap();
    assert_eq!(
        held.node(),
        &NodeRecord::Space(SpaceRecord::Euclidean {
            x: 1.0,
            y: 2.0,
            z: 3.0
        })
    );

    let back: BaseContext = block_on(store.hydrate_via(&substrate, &base_id)).unwrap();
    assert_eq!(
        back.snapshot().unwrap().nodes(),
        base.snapshot().unwrap().nodes()
    );
    assert_eq!(back.extra_ctx_get_name(extra_id), Some("weather"));
    let snapshot = back.snapshot().unwrap();
    assert_eq!(snapshot.extras().len(), 1);
    assert_eq!(
        snapshot.extras()[0].nodes(),
        extra.snapshot().unwrap().nodes()
    );
}

/// A substrate under the `Substrate` contract, keyed by node, that also counts its deposits.
struct CountingSubstrate {
    values: RefCell<HashMap<String, DataRecord>>,
    deposits: Cell<usize>,
}

impl CountingSubstrate {
    fn new() -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
            deposits: Cell::new(0),
        }
    }
}

impl Substrate for CountingSubstrate {
    type Error = MemorySubstrateError;

    fn deposit(
        &self,
        node: ContextoidId,
        value: &DataRecord,
    ) -> impl Future<Output = Result<SubstrateRef, Self::Error>> + Send {
        self.deposits.set(self.deposits.get() + 1);
        let key = node.to_string();
        self.values.borrow_mut().insert(key.clone(), value.clone());
        std::future::ready(Ok(SubstrateRef::new(
            MemorySubstrate::SOURCE.to_string(),
            key,
        )))
    }

    fn resolve(
        &self,
        reference: &SubstrateRef,
    ) -> impl Future<Output = Result<DataRecord, Self::Error>> + Send {
        let held = self.values.borrow().get(reference.key()).cloned();
        std::future::ready(
            held.ok_or_else(|| MemorySubstrateError::UnknownReference(reference.clone())),
        )
    }
}

#[test]
fn test_create_node_via_is_idempotent() {
    // A retry must not deposit again and must hand create_node the record it accepted before.
    let storage = MemoryStorage::new();
    let substrate = CountingSubstrate::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(store.reserve(2)).unwrap().collect();
    let nodes = [
        ContextoidRecord::new(n[0], NodeRecord::Data(DataRecord::Number(1.5))),
        ContextoidRecord::new(n[1], NodeRecord::Data(DataRecord::Number(2.5))),
    ];
    block_on(store.create_node_via(&substrate, &nodes)).unwrap();
    let first = block_on(storage.lookup(&n)).unwrap();
    assert_eq!(substrate.deposits.get(), 2);
    assert_eq!(block_on(store.create_node_via(&substrate, &nodes)), Ok(()));
    assert_eq!(substrate.deposits.get(), 2, "nothing deposited twice");
    assert_eq!(block_on(storage.lookup(&n)).unwrap(), first);
    // A partial retry deposits only the node the store does not hold yet.
    let more: Vec<ContextoidId> = block_on(store.reserve(1)).unwrap().collect();
    let mixed = [
        nodes[0].clone(),
        ContextoidRecord::new(more[0], NodeRecord::Data(DataRecord::Number(3.5))),
    ];
    block_on(store.create_node_via(&substrate, &mixed)).unwrap();
    assert_eq!(substrate.deposits.get(), 3);
}

#[test]
fn test_create_node_via_refuses_a_changed_value_under_a_held_identifier() {
    // The store holds n under a reference to 1.5; asking for 9.5 under n must be refused as a
    // conflict, deposit nothing, and leave the value the reference names unchanged.
    let storage = MemoryStorage::new();
    let substrate = CountingSubstrate::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(store.reserve(1)).unwrap().collect();
    let first = [ContextoidRecord::new(
        n[0],
        NodeRecord::Data(DataRecord::Number(1.5)),
    )];
    block_on(store.create_node_via(&substrate, &first)).unwrap();
    let held = block_on(storage.lookup(&n)).unwrap();

    let changed = [ContextoidRecord::new(
        n[0],
        NodeRecord::Data(DataRecord::Number(9.5)),
    )];
    let refused = block_on(store.create_node_via(&substrate, &changed)).unwrap_err();
    assert_eq!(
        refused.kind(),
        &StoreErrorEnum::Storage(MemoryStorageError::NodeConflict(n[0]))
    );
    assert_eq!(
        substrate.deposits.get(),
        1,
        "nothing deposited for the refusal"
    );
    assert_eq!(block_on(storage.lookup(&n)).unwrap(), held);
    let Some(Some(record)) = held.first() else {
        panic!("held")
    };
    let NodeRecord::Data(DataRecord::Reference(reference)) = record.node() else {
        panic!("a reference")
    };
    assert_eq!(
        block_on(substrate.resolve(reference)),
        Ok(DataRecord::Number(1.5))
    );
}

#[test]
fn test_a_refused_create_leaves_one_value_per_node() {
    // An identifier the store never reserved: every attempt deposits and is refused, and the
    // substrate still holds one value for the node, the last one deposited.
    let storage = MemoryStorage::new();
    let substrate = CountingSubstrate::new();
    let store = ContextStore::new(storage.clone());
    let unreserved: ContextoidId = 1_000;
    for value in [1.5, 2.5] {
        let nodes = [ContextoidRecord::new(
            unreserved,
            NodeRecord::Data(DataRecord::Number(value)),
        )];
        assert_eq!(
            block_on(store.create_node_via(&substrate, &nodes)).map_err(|e| e.0),
            Err(StoreErrorEnum::Storage(
                MemoryStorageError::IdentityNotReserved(unreserved)
            ))
        );
    }
    assert_eq!(substrate.deposits.get(), 2);
    assert_eq!(substrate.values.borrow().len(), 1);
    let reference = SubstrateRef::new(MemorySubstrate::SOURCE.to_string(), unreserved.to_string());
    assert_eq!(
        block_on(substrate.resolve(&reference)),
        Ok(DataRecord::Number(2.5))
    );
}
