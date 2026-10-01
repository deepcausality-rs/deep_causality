/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What a campaign flies, records and counts.

use dynamic_drone_failsafe::{FloatType, Mission, Outcome, Terrain};

/// The controllers under test, each the unchanged controller of one part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Controller {
    /// Part 1: the textbook fail-safe ladder, with no context.
    Textbook,
    /// Part 3: the ground as context and a fail-safe machine that acts on it.
    ContextAndAction,
    /// Part 4: part 3 with the Effect Ethos approving every landing.
    EffectEthos,
}

impl Controller {
    pub const ALL: [Controller; 3] = [
        Controller::Textbook,
        Controller::ContextAndAction,
        Controller::EffectEthos,
    ];
}

/// One scenario: its number, the mission the drone flies, the ground below it, and how many people
/// stand on it.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub id: usize,
    pub mission: Mission,
    pub terrain: Terrain,
    pub crew: usize,
}

impl Scenario {
    /// Whether a cell fails in this scenario.
    pub fn cell_fails(&self) -> bool {
        self.mission.faults().battery_fault_at_s().is_some()
    }
}

/// A scenario with each controller's flight, in the order of [`Controller::ALL`].
pub type Flown = (Scenario, Vec<Flight>);

/// How one flight ended: what became of the drone, when, how far from the nearest person, and
/// the controller's log of faults, fail-safes and maneuvers.
#[derive(Debug, Clone)]
pub struct Flight {
    pub ending: Ending,
    pub time_s: usize,
    pub nearest_person_m: FloatType,
    pub log: Vec<String>,
}

/// How a flight ended, as the campaign counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// The fix and the link came back and the drone resumed its mission.
    Resumed,
    /// Landed upright, clear of people.
    Safe,
    /// Came down within the clearance of a person, by landing or by falling.
    NearPerson,
    /// Came down in the creek.
    Ditched,
    /// Came down into the ravine.
    IntoRavine,
    /// Tipped over on steep ground and tumbled.
    TippedAndRolled,
    /// Flew or descended into the trees, clear of people.
    HitTrees,
    /// The battery died in the air and the drone fell, clear of people.
    Fell,
}

impl Ending {
    pub const ALL: [Ending; 8] = [
        Ending::Resumed,
        Ending::Safe,
        Ending::NearPerson,
        Ending::Ditched,
        Ending::IntoRavine,
        Ending::TippedAndRolled,
        Ending::HitTrees,
        Ending::Fell,
    ];

    /// Whether the drone was lost.
    pub fn drone_lost(self) -> bool {
        !matches!(self, Ending::Resumed | Ending::Safe | Ending::NearPerson)
    }

    pub fn of(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Safe => Ending::Safe,
            Outcome::AmongPeople => Ending::NearPerson,
            Outcome::Ditched => Ending::Ditched,
            Outcome::IntoRavine => Ending::IntoRavine,
            Outcome::TippedAndRolled { .. } => Ending::TippedAndRolled,
            Outcome::Fell => Ending::Fell,
            Outcome::HitTrees => Ending::HitTrees,
        }
    }
}
