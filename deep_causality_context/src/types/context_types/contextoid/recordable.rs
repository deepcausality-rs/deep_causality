/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Contextoid, ContextoidType, Datable, Root, SpaceTemporal, Spatial, Temporal};
use deep_causality_context_store::{
    ContextoidId, DataRecord, NodeRecord, ProjectionError, Recordable, SpaceRecord,
    SpaceTimeRecord, TimeRecord,
};
use deep_causality_core::Identifiable;

/// A contextoid dispatches to the record of the node it holds. A root is `NodeRecord::Root` in
/// both directions; the phantom arm has no record.
impl<D, S, T, ST> Recordable<NodeRecord> for Contextoid<D, S, T, ST>
where
    D: Datable + Clone + Recordable<DataRecord>,
    S: Spatial + Clone + Recordable<SpaceRecord>,
    T: Temporal + Clone + Recordable<TimeRecord>,
    ST: SpaceTemporal + Clone + Recordable<SpaceTimeRecord>,
{
    fn to_record(&self) -> Result<NodeRecord, ProjectionError> {
        match self.vertex_type() {
            ContextoidType::Datoid(data) => data.to_record().map(NodeRecord::Data),
            ContextoidType::Tempoid(time) => time.to_record().map(NodeRecord::Time),
            ContextoidType::Root(_) => Ok(NodeRecord::Root),
            ContextoidType::Spaceoid(space) => space.to_record().map(NodeRecord::Space),
            ContextoidType::SpaceTempoid(spacetime) => {
                spacetime.to_record().map(NodeRecord::SpaceTime)
            }
            ContextoidType::_Marker(_) => Err(ProjectionError::Unrecordable(self.id(), "Marker")),
        }
    }

    fn from_record(id: ContextoidId, record: NodeRecord) -> Result<Self, ProjectionError> {
        let vertex = match record {
            NodeRecord::Root => ContextoidType::Root(Root::new(id)),
            NodeRecord::Data(data) => ContextoidType::Datoid(D::from_record(id, data)?),
            NodeRecord::Time(time) => ContextoidType::Tempoid(T::from_record(id, time)?),
            NodeRecord::Space(space) => ContextoidType::Spaceoid(S::from_record(id, space)?),
            NodeRecord::SpaceTime(spacetime) => {
                ContextoidType::SpaceTempoid(ST::from_record(id, spacetime)?)
            }
        };
        Ok(Contextoid::new(id, vertex))
    }
}
