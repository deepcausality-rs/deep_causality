// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

pub trait Identifiable:
{
    fn id(&self) -> u64;
}
