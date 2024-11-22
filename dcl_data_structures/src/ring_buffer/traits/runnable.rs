// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

pub trait Runnable: Send {
    fn run(self: Box<Self>);
}
