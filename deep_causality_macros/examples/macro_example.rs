// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::{Constructor, Getters};

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data<T> {
    #[getter(name = data_id)] // Rename getter methods as you wish
    id: u64,
    data: T,
    filled: bool,
}

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Scale {
    Small,
    Big,
}

pub fn main() {
    let d = Data::new(0, 42, true);

    assert_eq!(*d.data_id(), 0);
    assert_eq!(*d.data(), 42);
    assert!(*d.filled());

    let big = Scale::new_big();
    assert_eq!(big, Scale::Big);

    let small = Scale::new_small();
    assert_eq!(small, Scale::Small)
}
