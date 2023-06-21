/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
use std::collections::HashMap;
use std::fmt::Debug;

use crate::prelude::{Adjustable, CausalFn, CausalityError, Causaloid, CausaloidGraph, DescriptionValue, Identifiable, IdentificationValue, NumericalValue};

pub trait Causable: Debug + Identifiable + Adjustable {
    fn causal_function(&self) -> CausalFn;
    fn causal_collection(&self) -> Option<Vec<Causaloid>>;
    fn causal_graph(&self) -> Option<CausaloidGraph<Causaloid>>;
    fn description(&self) -> DescriptionValue;
    fn data_set_id(&self) -> DescriptionValue;
    fn explain(&self) -> Result<String, CausalityError>;
    fn is_active(&self) -> bool;
    fn is_singleton(&self) -> bool;

    fn verify_single_cause(
        &self,
        obs: &NumericalValue,
    )
        -> Result<bool, CausalityError>;

    fn verify_all_causes(
        &self,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityError>;
}
