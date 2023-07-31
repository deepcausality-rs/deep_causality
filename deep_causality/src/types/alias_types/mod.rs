// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{CausalityError, Causaloid, Dataoid, Spaceoid, SpaceTempoid, Tempoid};

// Type aliases
pub type IdentificationValue = u64;
pub type NumericalValue = f64;
pub type DescriptionValue = String;

// Fn aliases for assumable, assumption, & assumption collection
pub type EvalFn = fn(&[NumericalValue]) -> bool;
pub type CausalFn = fn(NumericalValue) -> Result<bool, CausalityError>;

// Default type aliases for basic causaloids
pub type BaseCausaloidVec<'l> = Vec<Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>>;
pub type BaseCausaloid<'l> = Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>;