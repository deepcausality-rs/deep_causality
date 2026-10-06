/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Context node types
//!
//! The data, space, time and spacetime types a `Context` holds, grouped by the geometry each
//! models. Every four-dimensional spacetime indexes its coordinates
//! `0 => t, 1 => x, 2 => y, 3 => z`; `NoSpaceTime` has zero coordinates.
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
//! `SpaceKind` holds a `EuclideanSpace`, `EcefSpace`, `NedSpace` or `GeoSpace`; `TimeKind` a
//! `NewtonianTime`, `MinkowskiTime`, `DiscreteTime` or `EntropicTime`; `SpaceTimeKind` a
//! `GalileanSpacetime`, `NewtonianSpacetime`, `MinkowskiSpacetime` or `TangentSpacetime`. A
//! context over these enums carries nodes of several geometries, and each spacetime node reports
//! its own signature through `MetricSignature`. `CausalSetSpacetime` records a causal order
//! between identifiers and is not a context node type.
//!
//! Each type documents its definition and the textbook or standard it follows.

pub mod data;
pub mod data_uncertain;
pub mod root;
pub mod space;
pub mod space_time;
pub mod time;
