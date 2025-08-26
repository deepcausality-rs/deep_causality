/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ActionError, CausalityError, DeonticError};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CsmError {
    Action(ActionError),
    Causal(CausalityError),
    Deontic(DeonticError),
    Forbidden(String),
}

impl Error for CsmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CsmError::Action(e) => Some(e),
            CsmError::Deontic(e) => Some(e),
            CsmError::Causal(e) => Some(e),
            CsmError::Forbidden(_) => None,
        }
    }
}

impl Display for CsmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CsmError::Action(e) => write!(f, "CSM Action Error: {e}"),
            CsmError::Deontic(e) => write!(f, "CSM Deontic Error: {e}"),
            CsmError::Causal(e) => write!(f, "CSM Causal Error: {e}"),
            CsmError::Forbidden(s) => write!(f, "{s}"),
        }
    }
}

impl From<ActionError> for CsmError {
    fn from(err: ActionError) -> Self {
        CsmError::Action(err)
    }
}

impl From<DeonticError> for CsmError {
    fn from(err: DeonticError) -> Self {
        CsmError::Deontic(err)
    }
}

impl From<CausalityError> for CsmError {
    fn from(err: CausalityError) -> Self {
        CsmError::Causal(err)
    }
}
