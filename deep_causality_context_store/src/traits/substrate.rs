/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, DataRecord, SubstrateRef};

/// A store that holds values and hands back references to them.
///
/// A store that holds structure and not values accepts a data node only as a
/// [`DataRecord::Reference`]. The substrate is where the value then lives: `deposit` stores a
/// record under the node that carries it and returns where it now lives, and `resolve` returns
/// the record a reference names.
///
/// `deposit` is given any record other than a `Reference`; a substrate refuses a `Reference` with
/// its own error, because a reference to a reference names nothing. `resolve` refuses a reference
/// it does not hold.
pub trait Substrate {
    type Error: core::fmt::Debug + core::fmt::Display;

    /// Stores a value under the node that carries it and returns where it now lives.
    fn deposit(
        &self,
        node: ContextoidId,
        value: &DataRecord,
    ) -> impl Future<Output = Result<SubstrateRef, Self::Error>> + Send;

    /// The record a reference names.
    fn resolve(
        &self,
        reference: &SubstrateRef,
    ) -> impl Future<Output = Result<DataRecord, Self::Error>> + Send;
}
