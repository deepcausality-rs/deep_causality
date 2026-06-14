/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cut-cell geometry — CFD Stage 4, Group A (`add-cut-cells-and-immersed-boundaries`).
//!
//! A cut cell is the **fractional-aperture generalisation of the Stage-3 axis-aligned wall
//! clip**: an immersed surface replaces the integer `2^{-b}` boundary clip with a continuous
//! wetted fraction and a clipped cell volume. The lattice and its connectivity are unchanged
//! — a cut is a per-cell *geometric overlay* keyed by lattice cell id, so the existing
//! DEC operator stack (volumes → Hodge star → operators) consumes it without an API break
//! (design D1/D2).
//!
//! ## Contents
//!
//! - [`CellClass`] — `Fluid | Cut | Solid`.
//! - [`SourceGeometry`] — which immersed surface a fragment came from.
//! - [`CutFaceFragment`] — a single piece of immersed surface inside a cell (area, outward
//!   normal, source tag).
//! - [`CutCell`] — the per-cell overlay (clipped volume, per-face apertures, fragments,
//!   class).
//! - [`CutCellRegistry`] — the sparse `cell-id → CutCell` map plus the cut-aware volume /
//!   dual-clip accessors that reproduce the Stage-3 integer clip on an axis-aligned cut.
//! - [`Primitive`] — the analytic immersed surfaces (`Halfspace`, `Cylinder`, `Ball`) a
//!   registry is built from, via exact closed-form clipping.
//!
//! ## Cochain convention (load-bearing — carried from `graded-metrics`)
//!
//! Every clipped volume and aperture is a **measure** (an integral over a cell), produced and
//! consumed as the measure the Hodge-star dispatch expects — never a pointwise field value.
//! This is the same discipline that resolved the false graded "order-loss": a cut aperture is
//! a fractional cell measure, and the exactness tests compare it against closed-form measures.
//!
//! ## Scope (Group A)
//!
//! Analytic primitives only — STL / triangle meshes are postponed (design D3). Closed-form
//! exact cuts: half-space (all `D`), axis-aligned cylinder (`D = 3`, the cylinder-validation
//! path), and disk (`D = 2`). The 3D ball closed form is deferred (off the validation path).

mod carrier;
mod cell_class;
mod cut_face_fragment;
mod geometry;
mod intersection;
mod primitive;
mod registry;
mod source_geometry;

pub use carrier::CutCell;
pub use cell_class::CellClass;
pub use cut_face_fragment::CutFaceFragment;
pub use primitive::Primitive;
pub use registry::CutCellRegistry;
pub use source_geometry::SourceGeometry;
