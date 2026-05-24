/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Network topology for the cascade demo.
//!
//! ```text
//!                    source (0)
//!                   /    |    \
//!             trunk1(1) trunk2(2) trunk3(3)
//!                   \    |    /
//!                     sink (4)
//! ```
//!
//! The main trunk (e0/e3) is high-capacity; the two alternates (e1/e4 and
//! e2/e5) are low-capacity. The network is designed so that:
//!
//!   * Failing the main trunk (e0) overwhelms both alternates. The cascade
//!     fails them in turn, ending in total collapse after two steps.
//!   * Failing an alternate trunk (e1) is absorbed by the remaining
//!     capacity. No overloads, no cascade, full supply still delivered.
//!
//! Same network, two interventions, qualitatively different outcomes.

use crate::model_types::{Edge, NetworkConfig};

pub fn build_network() -> NetworkConfig {
    let edges = vec![
        Edge {
            id: 0,
            from: 0,
            to: 1,
            capacity: 6.0,
        }, // source -> trunk1 (main)
        Edge {
            id: 1,
            from: 0,
            to: 2,
            capacity: 2.0,
        }, // source -> trunk2 (alt)
        Edge {
            id: 2,
            from: 0,
            to: 3,
            capacity: 2.0,
        }, // source -> trunk3 (alt)
        Edge {
            id: 3,
            from: 1,
            to: 4,
            capacity: 6.0,
        }, // trunk1 -> sink
        Edge {
            id: 4,
            from: 2,
            to: 4,
            capacity: 2.0,
        }, // trunk2 -> sink
        Edge {
            id: 5,
            from: 3,
            to: 4,
            capacity: 2.0,
        }, // trunk3 -> sink
    ];
    NetworkConfig {
        n_nodes: 5,
        edges,
        source: 0,
        sink: 4,
        source_supply: 6.0,
    }
}
