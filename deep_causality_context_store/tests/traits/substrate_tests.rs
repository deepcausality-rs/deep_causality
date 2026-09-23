/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! An echo substrate pins the trait's shape and its one stated refusal. Expected values are the
//! literals the echo returns.
//!
//! Corner cases (rows A to K): A an empty text record deposited in `test_empty_value_is_a_value`;
//! F node identifier 0 in the same test; every other row n/a.
use core::future::{Future, ready};
use deep_causality_context_store::utils_test::block_on;
use deep_causality_context_store::{ContextoidId, DataRecord, Substrate, SubstrateRef};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
struct EchoError;

impl Display for EchoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "EchoError")
    }
}

/// A substrate that keys every value by the node that carries it and answers with the key.
struct EchoSubstrate;

impl Substrate for EchoSubstrate {
    type Error = EchoError;

    fn deposit(
        &self,
        node: ContextoidId,
        value: &DataRecord,
    ) -> impl Future<Output = Result<SubstrateRef, Self::Error>> + Send {
        let answer = if matches!(value, DataRecord::Reference(_)) {
            Err(EchoError)
        } else {
            Ok(SubstrateRef::new("echo".to_string(), node.to_string()))
        };
        ready(answer)
    }

    fn resolve(
        &self,
        reference: &SubstrateRef,
    ) -> impl Future<Output = Result<DataRecord, Self::Error>> + Send {
        ready(Ok(DataRecord::Text(reference.key().to_string())))
    }
}

fn assert_send<F: Send>(_: &F) {}

#[test]
fn test_deposit_and_resolve() {
    let substrate = EchoSubstrate;
    let reference = block_on(substrate.deposit(7, &DataRecord::Number(2.5))).unwrap();
    assert_eq!(reference.source(), "echo");
    assert_eq!(reference.key(), "7");
    assert_eq!(
        block_on(substrate.resolve(&reference)),
        Ok(DataRecord::Text("7".to_string()))
    );
}

#[test]
fn test_a_reference_is_refused() {
    let substrate = EchoSubstrate;
    let inner = SubstrateRef::new("s".to_string(), "k".to_string());
    assert_eq!(
        block_on(substrate.deposit(1, &DataRecord::Reference(inner))),
        Err(EchoError)
    );
    assert_eq!(EchoError.to_string(), "EchoError");
}

#[test]
fn test_the_futures_are_send() {
    let substrate = EchoSubstrate;
    let reference = SubstrateRef::default();
    assert_send(&substrate.deposit(1, &DataRecord::Flag(true)));
    assert_send(&substrate.resolve(&reference));
}

#[test]
fn test_empty_value_is_a_value() {
    let substrate = EchoSubstrate;
    let reference = block_on(substrate.deposit(0, &DataRecord::Text(String::new()))).unwrap();
    assert_eq!(reference.key(), "0");
}
