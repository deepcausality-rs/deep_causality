/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Functor laws for `ChainWitness<f64>` on a complex that carries geometry.
//!
//! This file held the adjunction laws for `ChainWitness`. Writing them is what showed the
//! `Chain ⊣ Chain` self-adjunction did not exist: `unit` receives one value and must produce a
//! chain over the whole complex, so `right_adjunct` after `left_adjunct` rebuilds from a single
//! stored weight and agrees with the original only on one-entry chains. The impl is gone, and so
//! are those tests; see the note where it stood in
//! `src/extensions/hkt_simplicial_complex/mod.rs`.
//!
//! What remains is the functor side, which is lawful and which these fixtures cover well: a
//! complex built from real coordinates, so the Hodge ⋆ operators are populated and `fmap` has
//! something to preserve beyond the weights.

use deep_causality_linear::CsrMatrix;
use deep_causality_topology::{Chain, ChainWitness, SimplicialComplex};
use std::sync::Arc;

// ----------------------------------------------------------------------------
// Functor identity on a complex that carries geometry
// ----------------------------------------------------------------------------

/// A line complex built from real coordinates, so its Hodge ⋆ operators are available.
fn geometric_line() -> SimplicialComplex<f64> {
    use deep_causality_topology::{Simplex, Skeleton};
    let v = Skeleton::new(0, vec![Simplex::new(vec![0]), Simplex::new(vec![1])]);
    let e = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    let d1 = CsrMatrix::from_triplets(2, 1, &[(0, 0, -1i8), (1, 0, 1i8)]).expect("boundary");
    SimplicialComplex::with_geometry(vec![v, e], vec![d1], vec![], vec![0.0, 0.0, 1.0, 0.0], 2)
}

#[test]
fn fmap_preserves_the_complex_geometry() {
    // `Chain` used to hold one parameter for both the complex's precision and the coefficient
    // group, so `fmap` had to rebuild the complex and dropped its Hodge ⋆ operators. With the two
    // separated the complex is carried across, and this is the regression guard for that.
    use deep_causality_haft::Functor;

    let source = geometric_line();
    assert!(
        source.hodge_star_operators().is_ok(),
        "fixture is wrong: the source complex must carry geometry, or this test proves nothing"
    );

    let weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1.5f64), (0, 1, 2.5)]).expect("weights");
    let chain: Chain<f64, f64> = Chain::new(Arc::new(source), 0, weights);

    let mapped = <ChainWitness<f64> as Functor<ChainWitness<f64>>>::fmap(chain, |x| x * 2.0);
    assert!(
        mapped.complex().hodge_star_operators().is_ok(),
        "fmap dropped the Hodge star operators"
    );
}

#[test]
fn functor_identity_holds_on_a_geometric_complex() {
    // The law the geometry drop broke: `fmap(id, c) == c`, on a complex carrying a metric.
    use deep_causality_haft::Functor;

    let weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, 3.0f64), (0, 1, 4.0)]).expect("weights");
    let chain: Chain<f64, f64> = Chain::new(Arc::new(geometric_line()), 0, weights);

    let mapped = <ChainWitness<f64> as Functor<ChainWitness<f64>>>::fmap(chain.clone(), |x| x);
    assert_eq!(
        mapped, chain,
        "fmap(id, c) != c on a geometry-carrying complex"
    );
}

#[test]
fn mapping_coefficients_leaves_the_precision_alone() {
    // The type-level statement of the same fact: mapping f64 coefficients to i32 yields
    // `Chain<f64, i32>`, not `Chain<i32, i32>`. The complex keeps its precision.
    use deep_causality_haft::Functor;

    let weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1.9f64), (0, 1, 2.9)]).expect("weights");
    let chain: Chain<f64, f64> = Chain::new(Arc::new(geometric_line()), 0, weights);

    let ints: Chain<f64, i32> =
        <ChainWitness<f64> as Functor<ChainWitness<f64>>>::fmap(chain, |x| x as i32);
    assert_eq!(ints.weights().values(), &vec![1, 2]);
    assert!(
        ints.complex().hodge_star_operators().is_ok(),
        "changing the coefficient type must not disturb the complex"
    );
}

// ---------------------------------------------------------------------------
// The partial operations report rather than panic
// ---------------------------------------------------------------------------
