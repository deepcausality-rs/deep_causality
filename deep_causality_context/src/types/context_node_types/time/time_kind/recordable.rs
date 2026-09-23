/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DiscreteTime, EntropicTime, EuclideanTime, LorentzianTime, TimeKind};
use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable, TimeRecord};
use deep_causality_num::FromPrimitive;

/// Total: four variants, four arms, each delegating to the type it wraps.
impl<R: RealField + Into<f64> + FromPrimitive> Recordable<TimeRecord> for TimeKind<R> {
    fn to_record(&self) -> Result<TimeRecord, ProjectionError> {
        match self {
            TimeKind::Euclidean(time) => time.to_record(),
            TimeKind::Entropic(time) => time.to_record(),
            TimeKind::Discrete(time) => time.to_record(),
            TimeKind::Lorentzian(time) => time.to_record(),
        }
    }

    fn from_record(id: ContextoidId, record: TimeRecord) -> Result<Self, ProjectionError> {
        match record {
            TimeRecord::Euclidean { .. } => {
                EuclideanTime::from_record(id, record).map(TimeKind::Euclidean)
            }
            TimeRecord::Lorentzian { .. } => {
                LorentzianTime::from_record(id, record).map(TimeKind::Lorentzian)
            }
            TimeRecord::Discrete { .. } => {
                DiscreteTime::from_record(id, record).map(TimeKind::Discrete)
            }
            TimeRecord::Entropic { .. } => {
                EntropicTime::from_record(id, record).map(TimeKind::Entropic)
            }
        }
    }
}
