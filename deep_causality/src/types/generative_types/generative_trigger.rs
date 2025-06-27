/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::prelude::{Data, TimeKind};
use std::fmt::{Debug, Display};
use std::hash::Hash;

/// A concrete, type-safe definition of all possible triggers.
pub enum GenerativeTrigger<D>
where
    D: Default + Copy + Clone + Hash + Eq + PartialEq,
{
    TimeElapsed(TimeKind),
    DataReceived(Data<D>),
    ManualIntervention(String),
}

impl<D> GenerativeTrigger<D>
where
    D: Debug + Default + Copy + Clone + Hash + Eq + PartialEq,
{
    pub fn time_elapsed(&self) -> Option<&TimeKind> {
        if let GenerativeTrigger::TimeElapsed(time_kind) = self {
            Some(time_kind)
        } else {
            None
        }
    }

    pub fn data_received(&self) -> Option<&Data<D>> {
        if let GenerativeTrigger::DataReceived(data) = self {
            Some(data)
        } else {
            None
        }
    }

    pub fn manual_intervention(&self) -> Option<&String> {
        if let GenerativeTrigger::ManualIntervention(message) = self {
            Some(message)
        } else {
            None
        }
    }
}

impl<D> Display for GenerativeTrigger<D>
where
    D: Debug + Default + Copy + Clone + Hash + Eq + PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerativeTrigger::TimeElapsed(time_kind) => write!(f, "{time_kind}"),
            GenerativeTrigger::DataReceived(data) => write!(f, "{data}"),
            GenerativeTrigger::ManualIntervention(message) => write!(f, "{message}"),
        }
    }
}
