/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Spatial context node types.
//!
//! # Pairing a space with a spacetime
//!
//! Every spacetime in this crate — [`EuclideanSpacetime`](crate::EuclideanSpacetime),
//! [`LorentzianSpacetime`](crate::LorentzianSpacetime) and
//! [`TangentSpacetime`](crate::TangentSpacetime) — stores Cartesian `x`, `y`, `z` beside a time
//! coordinate. A spatial type therefore pairs with a spacetime directly only when its three axes
//! are already Cartesian and carry no further reference the spacetime cannot hold.
//!
//! | Spatial type | Pairs directly | What a conversion has to supply |
//! |---|---|---|
//! | [`EuclideanSpace`](crate::EuclideanSpace) | yes | nothing |
//! | [`EcefSpace`](crate::EcefSpace) | numerically | the spacetime has no field recording that the axes are earth-fixed, so the frame is carried by the caller |
//! | [`NedSpace`](crate::NedSpace) | no | the local origin, which neither type stores, plus the axis convention: `down` is positive downward, so it is not a `z` |
//! | [`GeoSpace`](crate::GeoSpace) | no | a geodetic-to-Cartesian conversion, and a decision about the datum |
//!
//! # The datum is lost on the way out of a geodetic position
//!
//! [`GeoSpace`](crate::GeoSpace) carries a [`VerticalDatum`](crate::VerticalDatum) that says what
//! its altitude is measured against. No Cartesian spatial type and no spacetime has such a field,
//! so converting a geodetic position into any of them drops it, and the resulting coordinates
//! cannot be converted back without the datum being supplied again from somewhere else.
//!
//! This is recorded rather than solved: closing it means either a datum on the Cartesian types,
//! which is meaningless for most of them, or a conversion that refuses every datum but one.

pub mod ecef_space;
pub mod euclidean_space;
pub mod geo_space;
pub mod ned_space;
pub mod space_kind;
