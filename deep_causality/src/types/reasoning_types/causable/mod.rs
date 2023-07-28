// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;

use crate::prelude::{BuildError, Causaloid, CausaloidGraph, Inference};
use crate::protocols::inferable::InferableReasoning;
use crate::types::alias_types::{CausalFn, IdentificationValue};

pub mod causaloid;
pub mod causaloid_graph;

/// Create a new causaloid from a causal vector.
/// Encapsulates a linear causal collection into one single causaloid
/// that can be used individually, as part of another causal collection,
/// or embedded into a causal graph.
///
/// Verifies that description, data_set_id, and causal_coll are non-empty.
pub fn build_causaloid_from_vec<'l>(
    id: IdentificationValue,
    causal_vec: &'l Vec<Causaloid>,
    description: &'l str,
)
    -> Result<Causaloid<'l>, Box<dyn Error>>
{
    // check description
    if description.is_empty() {
        return Err(Box::new(BuildError("Description empty".into())));
    }

    // check causal collection
    if causal_vec.is_empty() {
        return Err(Box::new(BuildError("Causal collection empty".into())));
    }

    Ok(
        Causaloid::from_causal_collection(
            id,
            causal_vec,
            description,
        )
    )
}

/// Create a new causaloid from a causal graph.
/// Encapsulates a complex causal graph into one single causaloid
/// that can be used individually, as part of causal collection,
/// or embedded into another causal graph.
///
/// Verifies that description, data_set_id, and causal_graph are non-empty.
pub fn build_causaloid_from_graph<'l>(
    id: IdentificationValue,
    causal_graph: &'l CausaloidGraph<Causaloid>,
    description: &'l str,
)
    -> Result<Causaloid<'l>, Box<dyn Error>>
{
    // check description
    if description.is_empty() {
        return Err(Box::new(BuildError("Description empty".into())));
    }

    Ok(
        Causaloid::from_causal_graph(
            id,
            causal_graph,
            description,
        )
    )
}


/// Builds a new singleton Causaloid.
///
/// Verifies that causal function is valid,
/// by checking the underlying inference collections.
pub fn build_causaloid<'l>(
    id: IdentificationValue,
    causal_fn: CausalFn,
    description: &'l str,
    inferable_coll: &'l Vec<Inference>,
    inverse_inferable_coll: &'l Vec<Inference>,
)
    -> Result<Causaloid<'l>, Box<dyn Error>>
{

    // check description
    if description.is_empty() {
        return Err(Box::new(BuildError("Description empty".into())));
    }

    // check inferable collection
    if inferable_coll.is_empty() {
        return Err(Box::new(BuildError("Inferable collection empty".into())));
    }

    if inferable_coll.all_inverse_inferable() {
        return Err(Box::new(BuildError("Inferable collection actually inverse".into())));
    }

    if inferable_coll.all_non_inferable() {
        return Err(Box::new(BuildError("Inferable collection is non-inferable".into())));
    }

    // check inverse inferable collection
    if inverse_inferable_coll.is_empty() {
        return Err(Box::new(BuildError("Inverse inferable collection empty".into())));
    }

    if inverse_inferable_coll.all_inferable() {
        return Err(Box::new(BuildError("Inverse inferable collection is NON inverse".into())));
    }

    if inverse_inferable_coll.all_non_inferable() {
        return Err(Box::new(BuildError("Inverse inferable collection is non-inferable".into())));
    }

    Ok(Causaloid::new(id, causal_fn, description,)
    )
}
