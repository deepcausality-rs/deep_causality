/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Flow-DSL zone conveniences.
//!
//! Boundary conditions reuse the migrated `BoundaryZone` terms (`Inflow`,
//! `Outflow`, `SlipWall`, `MovingWall`, …) composed as static tuples. A prescribed
//! moving wall is carried on the marching case as a raw `(axis, max_side, velocity)`
//! directive and applied through the solver's `with_moving_wall` builder, so no name
//! collides with the `MovingWall` boundary zone. Named-axis conveniences
//! (`Inflow::west`, `SlipWall::top`, …) and the open-zone tuple are added here as the
//! wall-bounded and cut-cell cases land.
