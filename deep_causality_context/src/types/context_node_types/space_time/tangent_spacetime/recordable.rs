/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MetricTensor4D, SpaceTemporal, TangentSpacetime};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, SpaceTimeRecord};
use deep_causality_num::FromPrimitive;

fn lift<R: FromPrimitive>(id: ContextoidId, value: f64) -> Result<R, ProjectionError> {
    R::from_f64(value).ok_or(ProjectionError::Scalar(id, value))
}

impl<R: RealField + Into<f64> + FromPrimitive> Recordable<SpaceTimeRecord> for TangentSpacetime<R> {
    fn to_record(&self) -> Result<SpaceTimeRecord, ProjectionError> {
        let [dx, dy, dz] = self.velocity_vector();
        let metric = self.metric_tensor().map(|row| row.map(Into::into));
        Ok(SpaceTimeRecord::Tangent {
            x: self.x().into(),
            y: self.y().into(),
            z: self.z().into(),
            t: (*self.t()).into(),
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
                x,
                y,
                z,
                t,
                dt,
                dx,
                dy,
                dz,
                metric,
            } => {
                let mut node = TangentSpacetime::new(
                    id,
                    lift(id, x)?,
                    lift(id, y)?,
                    lift(id, z)?,
                    lift(id, t)?,
                    lift(id, dt)?,
                    lift(id, dx)?,
                    lift(id, dy)?,
                    lift(id, dz)?,
                );
                let mut lifted = [[R::zero(); 4]; 4];
                for (row, values) in lifted.iter_mut().zip(metric) {
                    for (cell, value) in row.iter_mut().zip(values) {
                        *cell = lift(id, value)?;
                    }
                }
                node.update_metric_tensor(lifted);
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
