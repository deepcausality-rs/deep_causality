/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PropagatingEffect;
use std::fmt::{Debug, Formatter};
use ultragraph::GraphView;

impl Debug for PropagatingEffect {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PropagatingEffect::None => write!(_f, "PropagatingEffect::None"),
            PropagatingEffect::Deterministic(val) => {
                write!(_f, "PropagatingEffect::Deterministic({val})")
            }
            PropagatingEffect::Numerical(val) => {
                write!(_f, "PropagatingEffect::Numerical({val:?})")
            }
            PropagatingEffect::Probabilistic(val) => {
                write!(_f, "PropagatingEffect::Probabilistic({val:?})")
            }
            PropagatingEffect::ContextualLink(id, val) => {
                write!(_f, "PropagatingEffect::ContextualLink({id}, {val})")
            }
            PropagatingEffect::Map(map) => write!(_f, "PropagatingEffect::Map({map:?})"),
            PropagatingEffect::Graph(g) => write!(
                _f,
                "PropagatingEffect::Graph(nodes: {}, edges: {})",
                g.number_nodes(),
                g.number_edges()
            ),
            PropagatingEffect::RelayTo(idx, effect) => {
                write!(_f, "PropagatingEffect::RelayTo({idx}, {effect:?})")
            }
            PropagatingEffect::UncertainBool(val) => {
                write!(_f, "PropagatingEffect::UncertainBool({val:?})")
            }
            PropagatingEffect::UncertainFloat(val) => {
                write!(_f, "PropagatingEffect::UncertainFloat({val:?})")
            }
        }
    }
}
