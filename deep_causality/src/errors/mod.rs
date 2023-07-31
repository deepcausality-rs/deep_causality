// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct BuildError(pub String);

impl Error for BuildError {}

impl fmt::Display for BuildError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BuildError: {}", self.0)
    }
}


#[derive(Debug)]
pub struct CausalityGraphError(pub String);

impl Error for CausalityGraphError {}

impl fmt::Display for CausalityGraphError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CausalityGraphError: {}", self.0)
    }
}

#[derive(Debug)]
pub struct ContextIndexError(pub String);

impl Error for ContextIndexError {}

impl fmt::Display for ContextIndexError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ContextIndexError: {}", self.0)
    }
}

#[derive(Debug)]
pub struct CausalGraphIndexError(pub String);

impl Error for CausalGraphIndexError {}

impl fmt::Display for CausalGraphIndexError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CausalGraphIndexError: {}", self.0)
    }
}

#[derive(Debug)]
pub struct CausalityError(pub String);

impl Error for CausalityError {}

impl fmt::Display for CausalityError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CausalityError: {}", self.0)
    }
}


#[derive(Debug)]
pub struct AdjustmentError(pub String);

impl Error for AdjustmentError {}

impl fmt::Display for AdjustmentError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AdjustmentError: {}", self.0)
    }
}


#[derive(Debug)]
pub struct PropagateError(pub String);

impl Error for PropagateError {}

impl fmt::Display for PropagateError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PropagateError: {}", self.0)
    }
}


#[derive(Debug)]
pub struct UpdateError(pub String);

impl Error for UpdateError {}

impl fmt::Display for UpdateError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UpdateError: {}", self.0)
    }
}


#[derive(Debug)]
pub struct ActionError(pub String);

impl Error for ActionError {}

impl fmt::Display for ActionError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ActionError: {}", self.0)
    }
}
