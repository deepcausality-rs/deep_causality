// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::Adjustable;
use deep_causality_macros::Constructor;

use crate::types::context_types::adjustable::utils;

// Tests the (empty) default implementation of the node_types_adjustable protocol.

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data<T>
where
    T: Copy + Default,
{
    data: T,
}

impl<T> Adjustable<T> for Data<T> where T: Copy + Default {}

#[test]
fn test_update() {
    let mut d = Data::new(32);
    let array_grid = utils::get_1d_array_grid(42);

    // Default implementation does nothing
    let res = d.update(&array_grid);
    assert!(res.is_ok());
}

#[test]
fn test_adjustment() {
    let mut d = Data::new(32);
    let array_grid = utils::get_1d_array_grid(42);

    // Default implementation does nothing
    let res = d.adjust(&array_grid);
    assert!(res.is_ok());
}
