/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::lift_scalar::lift_scalar;
use crate::{MetricTensor4D, SpaceTemporal, TangentSpacetime};
use alloc::string::ToString;
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceTimeRecord};
use deep_causality_num::FromPrimitive;

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceTimeRecord> for TangentSpacetime<R> {
    fn to_record(&self) -> Result<SpaceTimeRecord, ProjectionError> {
        let [dx, dy, dz] = self.velocity_vector();
        let metric = self.metric_tensor().map(|row| row.map(Into::into));
        Ok(SpaceTimeRecord::Tangent {
            t: (*self.t()).into(),
            x: self.x().into(),
            y: self.y().into(),
            z: self.z().into(),
            dt: self.time_velocity().into(),
            dx: dx.into(),
            dy: dy.into(),
            dz: dz.into(),
            metric,
        })
    }

    fn from_record(id: ContextoidId, record: SpaceTimeRecord) -> Result<Self, ProjectionError> {
        match record {
            SpaceTimeRecord::Tangent {
                t,
                x,
                y,
                z,
                dt,
                dx,
                dy,
                dz,
                metric,
            } => {
                let mut node = TangentSpacetime::new(
                    id,
                    lift_scalar(id, x)?,
                    lift_scalar(id, y)?,
                    lift_scalar(id, z)?,
                    lift_scalar(id, t)?,
                    lift_scalar(id, dt)?,
                    lift_scalar(id, dx)?,
                    lift_scalar(id, dy)?,
                    lift_scalar(id, dz)?,
                );
                let mut lifted = [[R::zero(); 4]; 4];
                for (row, values) in lifted.iter_mut().zip(metric) {
                    for (cell, value) in row.iter_mut().zip(values) {
                        *cell = lift_scalar(id, value)?;
                    }
                }
                node.update_metric_tensor(lifted)
                    .map_err(|e| ProjectionError::Rejected(id, e.to_string()))?;
                Ok(node)
            }
            other => Err(ProjectionError::WrongVariant(
                id,
                "Tangent",
                other.kind_name(),
            )),
        }
    }
}
