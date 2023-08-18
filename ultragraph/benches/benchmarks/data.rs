// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    pub a: u8,
    pub b: u8,
    pub z: bool,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            a: 0,
            b: 1,
            z: false,
        }
    }
}
