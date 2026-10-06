/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Context node types
//!
//! The data, space, time and spacetime types a `Context` holds, grouped by the geometry each
//! models. Every spacetime indexes its coordinates `0 => t, 1 => x, 2 => y, 3 => z`.
//!
//! | Geometry | Space | Time | Spacetime |
//! |---|---|---|---|
//! | Euclidean space, Newtonian spacetime with a rest frame | `EuclideanSpace`, `EcefSpace`, `NedSpace`, `GeoSpace` | `NewtonianTime`, `DiscreteTime` | `NewtonianSpacetime` |
//! | Galilean spacetime, no rest frame | — | `NewtonianTime` | `GalileanSpacetime` |
//! | Minkowski spacetime, inertial frame | — | `MinkowskiTime` | `MinkowskiSpacetime` |
//! | Relativistic spacetime, metric at one event | — | — | `TangentSpacetime` |
//! | Counted steps | — | `DiscreteTime`, `EntropicTime` | — |
//! | Nothing of that kind | `NoSpace` | `NoTime` | `NoSpaceTime` |
//!
//! `SpaceKind`, `TimeKind` and `SpaceTimeKind` hold any variant of their slot, so one context can
//! carry nodes of several geometries; each spacetime node reports its own signature through
//! `MetricSignature`. `CausalSetSpacetime` and `ConformalSpacetime` record a causal order between
//! identifiers and are not context node types.
//!
//! Each type documents its definition and the textbook or standard it follows.

pub mod data;
pub mod data_uncertain;
pub mod root;
pub mod space;
pub mod space_time;
pub mod time;
