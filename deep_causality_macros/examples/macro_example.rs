/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_macros::Getters;

#[derive(Getters, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data<T> {
    #[getter(name = data_id)] // Rename getter methods as you wish
    id: u64,
    data: T,
    filled: bool,
}

impl<T> Data<T> {
    pub fn new(id: u64, data: T, filled: bool) -> Self {
        Self { id, data, filled }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Scale {
    Small,
    Big,
}

pub fn main() {
    let d = Data::new(0, 42, true);

    assert_eq!(*d.data_id(), 0);
    assert_eq!(*d.data(), 42);
    assert!(*d.filled());

    let big = Scale::Big;
    assert_eq!(big, Scale::Big);

    let small = Scale::Small;
    assert_eq!(small, Scale::Small)
}
