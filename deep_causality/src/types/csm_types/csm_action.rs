// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::ActionError;

#[derive(Clone, Debug)]
pub struct CausalAction {
    action: fn() -> Result<(), ActionError>,
    descr: &'static str,
    version: usize,
}

impl CausalAction
{
    pub fn new(action: fn() -> Result<(), ActionError>, descr: &'static str, version: usize) -> Self {
        Self { action, descr, version }
    }

    pub fn descr(&self) -> &'static str {
        self.descr
    }

    pub fn version(&self) -> usize {
        self.version
    }
}

impl CausalAction
{
    pub fn fire(&self) -> Result<(), ActionError>
    {
        (self.action)()
    }
}
