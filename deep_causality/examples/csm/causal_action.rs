// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use std::fmt;

type CausalFn = fn() -> Result<(), ActionError>;

#[derive(Clone)]
pub struct CausalAction {
    action: fn() -> Result<(), ActionError>,
}

impl CausalAction
{
    pub fn new(action: fn() -> Result<(), ActionError>) -> Self {
        Self { action }
    }
}

impl CausalAction
{
    pub fn fire(&self) -> Result<(), ActionError>
    {
        (self.action)()
    }
}

#[derive(Debug, Clone)]
pub struct ActionError(pub String);

impl Error for ActionError {}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CausalityError: {}", self.0)
    }
}
