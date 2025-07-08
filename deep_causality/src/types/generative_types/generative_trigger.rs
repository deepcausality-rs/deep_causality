/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Data, TimeKind};
use std::fmt::{Debug, Display};
use std::hash::Hash;

/// A concrete, type-safe definition of all possible triggers.
#[derive(Debug, Clone, PartialEq)]
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
    D: Default + Copy + Clone + Hash + Eq + PartialEq,
{
    /// Returns a reference to the `TimeKind` if the trigger is `TimeElapsed`.
    ///
    /// This function checks if the `GenerativeTrigger` enum instance is of the
    /// `TimeElapsed` variant. If it is, it returns `Some` containing a reference
    /// to the inner `TimeKind`. Otherwise, it returns `None`.
    ///
    pub fn time_elapsed(&self) -> Option<&TimeKind> {
        if let GenerativeTrigger::TimeElapsed(time_kind) = self {
            Some(time_kind)
        } else {
            None
        }
    }

    /// Returns a reference to the `Data<D>` if the trigger is `DataReceived`.
    ///
    /// This function provides a convenient way to check if a `GenerativeTrigger`
    /// is the `DataReceived` variant and, if so, to get a reference to the
    /// contained data. It avoids the need for a full `match` statement when
    /// you are only interested in this specific case.
    ///
    /// # Returns
    ///
    /// * `Some(&Data<D>)` if `self` is `GenerativeTrigger::DataReceived`.
    /// * `None` if `self` is any other variant, such as `TimeElapsed` or `ManualIntervention`.
    ///
    pub fn data_received(&self) -> Option<&Data<D>> {
        if let GenerativeTrigger::DataReceived(data) = self {
            Some(data)
        } else {
            None
        }
    }

    /// Returns a reference to the `String` if the trigger is `ManualIntervention`.
    ///
    /// This function provides a convenient way to check if a `GenerativeTrigger`
    /// is the `ManualIntervention` variant and, if so, to get a reference to the
    /// contained message. It avoids the need for a full `match` statement when
    /// you are only interested in this specific case.
    ///
    /// # Returns
    ///
    /// * `Some(&String)` if `self` is `GenerativeTrigger::ManualIntervention`.
    /// * `None` if `self` is any other variant, such as `TimeElapsed` or `DataReceived`.
    ///
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
