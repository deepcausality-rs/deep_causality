/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Surface-level conformance tests for the `HasHodgeStar<R>` capability trait.
//!
//! Concrete implementations live in later R4 sub-blocks:
//! - `HasHodgeStar<R> for ReggeGeometry<R>` (simplicial) — R4.2.
//! - `HasHodgeStar<R> for CubicalReggeGeometry<D, R, S>` (cubical) — R4.3 / R4.4.
//!
//! These stub tests only verify the trait's public re-export and that the
//! associated-type + `Cow`-returning shape compiles end-to-end through a
//! minimal dummy implementor. They guard against regressions to the trait's
//! public surface independently of the two production impls.

use deep_causality_topology::{CellComplex, ChainComplex, HasHodgeStar, TopologyError};
use std::borrow::Cow;

use deep_causality_sparse::CsrMatrix;

/// Minimal unit-struct implementor used purely to exercise the trait's shape.
/// Pairs with `CellComplex<DummyCell>` (the simplest `ChainComplex` in the
/// crate that takes no metric of its own).
struct DummyMetric;

impl HasHodgeStar<f64> for DummyMetric {
    type Complex = CellComplex<DummyCell>;

    fn hodge_star_matrix<'a>(
        &'a self,
        _complex: &'a Self::Complex,
        _k: usize,
    ) -> Result<Cow<'a, CsrMatrix<f64>>, TopologyError> {
        Ok(Cow::Owned(CsrMatrix::new()))
    }
}

/// A trivial `Cell` impl required by `CellComplex<DummyCell>`. We don't
/// exercise its semantics here; we only need the type to compile.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DummyCell;

impl deep_causality_topology::Cell for DummyCell {
    fn dim(&self) -> usize {
        0
    }
    fn boundary(&self) -> Vec<(Self, i8)> {
        Vec::new()
    }
}

#[test]
fn has_hodge_star_is_publicly_re_exported_from_crate_root() {
    // Compile-pass: if the trait is not re-exported from the crate root, the
    // `use deep_causality_topology::HasHodgeStar;` at the top of this file
    // fails. Reaching this test body means the re-export is intact.
    fn _accepts_impl<R, M>(_m: &M)
    where
        R: deep_causality_algebra::RealField,
        M: HasHodgeStar<R>,
    {
    }
    let m = DummyMetric;
    _accepts_impl::<f64, _>(&m);
}

#[test]
fn associated_complex_type_compiles_through_generic_bound() {
    // Exercises the `Self::Complex: ChainComplex` bound at the call site.
    fn _accepts_paired<R, M>(_m: &M)
    where
        R: deep_causality_algebra::RealField,
        M: HasHodgeStar<R>,
        M::Complex: ChainComplex,
    {
    }
    let m = DummyMetric;
    _accepts_paired::<f64, _>(&m);
}

#[test]
fn dummy_metric_returns_owned_cow_for_compute_on_demand_backends() {
    // Sanity: the `Cow::Owned` path used by compute-on-demand implementors
    // round-trips through the trait method. This is the shape that the
    // forthcoming cubical impl (R4.3) will use.
    let complex: CellComplex<DummyCell> = CellComplex::from_cells(Vec::new());
    let metric = DummyMetric;
    let star = metric.hodge_star_matrix(&complex, 0);
    assert!(matches!(star, Ok(Cow::Owned(_))));
}

#[test]
fn uniform_axis_spacings_defaults_to_none() {
    // `DummyMetric` does not override `uniform_axis_spacings`, so the default
    // trait-method body (returns `None`) is exercised here.
    let metric = DummyMetric;
    let spacings: Option<Vec<f64>> = metric.uniform_axis_spacings();
    assert!(spacings.is_none());
}
