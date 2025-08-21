/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    Datable, EffectEthos, SpaceTemporal, Spatial, Symbolic, Teloid, TeloidID, TeloidStorable,
    Temporal,
};
use ultragraph::GraphMut;

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// A facade method to add a new norm to the ethos.
    /// This is the primary way to build the norm base, ensuring consistency.
    /// It adds the Teloid to the store, updates the tag index, and adds the
    /// TeloidID to the graph, invalidating the verification status.
    pub fn add_teloid(&mut self, teloid: Teloid<D, S, T, ST, SYM, VS, VT>) {
        let id = teloid.id();
        let tags = teloid.tags().clone();

        self.teloid_store.insert(teloid);
        for tag in tags {
            self.tag_index.add(tag, id);
        }
        self.teloid_graph
            .graph
            .add_node(id)
            .expect("Failed to add node");
        self.is_verified = false; // A modification invalidates prior verification.
    }

    pub fn get_teloid(&mut self, id: TeloidID) -> Option<Teloid<D, S, T, ST, SYM, VS, VT>> {
        self.teloid_store.get(&id).cloned()
    }
}
