/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the cup product on cochains.
//!
//! Every formula and every numeric expectation asserted here was verified
//! against this crate's own boundary and coboundary operators before the test
//! was written.
//!
//! Reference: Chen, Y.-A. & Tata, S., *Higher cup products on hypercubic
//! lattices: application to lattice models of topological phases*,
//! arXiv:2106.05274, J. Math. Phys. **64**, 091902 (2023).
//!
//! - Eq. (5): the simplicial (Alexander–Whitney) cup product.
//! - Fig. 1: the two-dimensional cubical cup product.
//! - Prop. 3: the Leibniz rule over `ℤ`, `δ(αⁿ ∪ βᵐ) = δαⁿ ∪ βᵐ + (−1)ⁿ αⁿ ∪ δβᵐ`.
//!
//! Two expectations are ground truth from topology rather than from a formula:
//! the `H¹ × H¹ → H²` pairing on a 2-torus is the intersection number of the two
//! fundamental cycles, and the triple product of the three direction generators
//! on a 3-torus is the pairing of the three fundamental cycles.

use deep_causality_topology::utils_tests::{
    delta, direction_cochain, lattice_index, max_abs_diff, pseudo_cochain, simplex_index,
    tetrahedron,
};
use deep_causality_topology::{
    CellularComplex, ChainComplex, LatticeCell, LatticeComplex, Simplex, SplittableCell,
    cup_product, cup_product_n,
};

const TOL: f64 = 1e-9;

/// The Leibniz residual for one degree pair on any splittable complex:
/// `|δ(α ∪ β) − (δα ∪ β + (−1)^p α ∪ δβ)|`, Chen & Tata Prop. 3.
fn leibniz_residual<K: CellularComplex>(c: &K, p: usize, q: usize) -> f64
where
    K::CellType: SplittableCell,
{
    let a = pseudo_cochain(c.num_cells(p), 11 + p as u64 * 7);
    let b = pseudo_cochain(c.num_cells(q), 23 + q as u64 * 13);
    let lhs = delta(c, p + q, &cup_product(c, &a, p, &b, q).expect("cup"));
    let t1 = cup_product(c, &delta(c, p, &a), p + 1, &b, q).expect("cup da b");
    let t2 = cup_product(c, &a, p, &delta(c, q, &b), q + 1).expect("cup a db");
    let sgn = if p.is_multiple_of(2) { 1.0 } else { -1.0 };
    let rhs: Vec<f64> = t1.iter().zip(&t2).map(|(x, y)| x + sgn * y).collect();
    max_abs_diff(&lhs, &rhs)
}

fn assert_leibniz<K: CellularComplex>(c: &K, p: usize, q: usize)
where
    K::CellType: SplittableCell,
{
    let r = leibniz_residual(c, p, q);
    assert!(
        r < TOL,
        "Leibniz failed at (p,q)=({p},{q}); residual {r:.3e}"
    );
}

// --------------------------------------------------------------------------
// The defining formulas
// --------------------------------------------------------------------------

#[test]
fn simplicial_cup_reproduces_alexander_whitney() {
    // Chen & Tata Eq. (5): (a cup b)([v0,v1,v2]) = a([v0,v1]) * b([v1,v2]).
    let c = tetrahedron();
    let e = simplex_index(&c, 1);
    let f = simplex_index(&c, 2);

    let mut a = vec![0.0; c.num_cells(1)];
    let mut b = vec![0.0; c.num_cells(1)];
    a[e[&Simplex::new(vec![0, 1])]] = 3.0;
    b[e[&Simplex::new(vec![1, 2])]] = 5.0;

    let out = cup_product(&c, &a, 1, &b, 1).expect("cup");
    assert_eq!(
        out[f[&Simplex::new(vec![0, 1, 2])]],
        15.0,
        "a([0,1]) * b([1,2]) = 3 * 5"
    );
}

#[test]
fn simplicial_cup_is_zero_where_the_split_misses() {
    // The face [0,1,3] splits as ([0,1],[1,3]); b is supported on [1,2] only.
    let c = tetrahedron();
    let e = simplex_index(&c, 1);
    let f = simplex_index(&c, 2);
    let mut a = vec![0.0; c.num_cells(1)];
    let mut b = vec![0.0; c.num_cells(1)];
    a[e[&Simplex::new(vec![0, 1])]] = 3.0;
    b[e[&Simplex::new(vec![1, 2])]] = 5.0;
    let out = cup_product(&c, &a, 1, &b, 1).expect("cup");
    assert_eq!(out[f[&Simplex::new(vec![0, 1, 3])]], 0.0);
}

#[test]
fn cubical_2d_cup_matches_the_published_formula() {
    // Geometry and signs from Chen & Tata Fig. 1: on the unit square the product
    // is a(bottom x-edge)*b(right y-edge) - a(left y-edge)*b(top x-edge).
    let l = 4;
    let c = LatticeComplex::<2, f64>::square_torus(l);
    let e = lattice_index(&c, 1);
    let f = lattice_index(&c, 2);

    let bottom = LatticeCell::<2>::new([0, 0], 0b01);
    let right = LatticeCell::<2>::new([1, 0], 0b10);
    let left = LatticeCell::<2>::new([0, 0], 0b10);
    let top = LatticeCell::<2>::new([0, 1], 0b01);

    // First term only.
    let mut a = vec![0.0; c.num_cells(1)];
    let mut b = vec![0.0; c.num_cells(1)];
    a[e[&bottom]] = 2.0;
    b[e[&right]] = 7.0;
    let out = cup_product(&c, &a, 1, &b, 1).expect("cup");
    let face = f[&LatticeCell::<2>::new([0, 0], 0b11)];
    assert_eq!(out[face], 14.0, "+a(bottom)*b(right)");

    // Second term only, which carries the minus sign.
    let mut a2 = vec![0.0; c.num_cells(1)];
    let mut b2 = vec![0.0; c.num_cells(1)];
    a2[e[&left]] = 2.0;
    b2[e[&top]] = 7.0;
    let out2 = cup_product(&c, &a2, 1, &b2, 1).expect("cup");
    assert_eq!(out2[face], -14.0, "-a(left)*b(top)");
}

#[test]
fn cubical_cup_output_has_the_summed_degree() {
    let c = LatticeComplex::<3, f64>::cubic_torus(3);
    let a = pseudo_cochain(c.num_cells(1), 1);
    let b = pseudo_cochain(c.num_cells(1), 2);
    let out = cup_product(&c, &a, 1, &b, 1).expect("cup");
    assert_eq!(out.len(), c.num_cells(2));
}

// --------------------------------------------------------------------------
// Error paths
// --------------------------------------------------------------------------

#[test]
fn wrong_length_left_cochain_is_rejected() {
    let c = tetrahedron();
    let a = vec![0.0; c.num_cells(1) + 1];
    let b = pseudo_cochain(c.num_cells(1), 2);
    assert!(cup_product(&c, &a, 1, &b, 1).is_err());
}

#[test]
fn wrong_length_right_cochain_is_rejected() {
    let c = tetrahedron();
    let a = pseudo_cochain(c.num_cells(1), 1);
    let b = vec![0.0; c.num_cells(1) - 1];
    assert!(cup_product(&c, &a, 1, &b, 1).is_err());
}

#[test]
fn degree_sum_exceeding_the_complex_is_rejected() {
    // A 2-torus has no 3-cells, so a 1-cochain cupped with a 2-cochain has
    // nowhere to land.
    let c = LatticeComplex::<2, f64>::square_torus(3);
    let a = pseudo_cochain(c.num_cells(1), 1);
    let b = pseudo_cochain(c.num_cells(2), 2);
    assert!(cup_product(&c, &a, 1, &b, 2).is_err());
}

#[test]
fn n_fold_rejects_an_empty_factor_list() {
    let c = tetrahedron();
    let factors: [(&[f64], usize); 0] = [];
    assert!(cup_product_n(&c, &factors).is_err());
}

// --------------------------------------------------------------------------
// Leibniz: the acceptance gate (Chen & Tata Prop. 3)
// --------------------------------------------------------------------------

#[test]
fn leibniz_holds_on_a_hand_built_simplicial_complex() {
    let c = tetrahedron();
    for (p, q) in [(0, 0), (0, 1), (1, 0), (1, 1), (0, 2), (2, 0)] {
        assert_leibniz(&c, p, q);
    }
}

#[test]
fn leibniz_holds_on_a_square_torus() {
    for l in [3, 4] {
        let c = LatticeComplex::<2, f64>::square_torus(l);
        for (p, q) in [(0, 0), (0, 1), (1, 0)] {
            assert_leibniz(&c, p, q);
        }
    }
}

#[test]
fn leibniz_holds_on_a_cubic_torus() {
    let c = LatticeComplex::<3, f64>::cubic_torus(3);
    for (p, q) in [(0, 0), (0, 1), (1, 0), (1, 1), (0, 2), (2, 0)] {
        assert_leibniz(&c, p, q);
    }
}

// --------------------------------------------------------------------------
// Associativity and the n-fold form
// --------------------------------------------------------------------------

#[test]
fn associativity_holds_on_a_simplicial_complex() {
    let c = tetrahedron();
    let n = c.num_cells(1);
    let (x, y, z) = (
        pseudo_cochain(n, 3),
        pseudo_cochain(n, 5),
        pseudo_cochain(n, 7),
    );
    let l = cup_product(&c, &cup_product(&c, &x, 1, &y, 1).unwrap(), 2, &z, 1).unwrap();
    let r = cup_product(&c, &x, 1, &cup_product(&c, &y, 1, &z, 1).unwrap(), 2).unwrap();
    assert!(
        max_abs_diff(&l, &r) < TOL,
        "residual {:.3e}",
        max_abs_diff(&l, &r)
    );
}

#[test]
fn associativity_holds_on_a_cubic_torus() {
    let c = LatticeComplex::<3, f64>::cubic_torus(3);
    let n = c.num_cells(1);
    let (x, y, z) = (
        pseudo_cochain(n, 3),
        pseudo_cochain(n, 5),
        pseudo_cochain(n, 7),
    );
    let l = cup_product(&c, &cup_product(&c, &x, 1, &y, 1).unwrap(), 2, &z, 1).unwrap();
    let r = cup_product(&c, &x, 1, &cup_product(&c, &y, 1, &z, 1).unwrap(), 2).unwrap();
    assert!(
        max_abs_diff(&l, &r) < TOL,
        "residual {:.3e}",
        max_abs_diff(&l, &r)
    );
}

#[test]
fn n_fold_agrees_with_the_left_fold() {
    let c = LatticeComplex::<3, f64>::cubic_torus(3);
    let n = c.num_cells(1);
    let (x, y, z) = (
        pseudo_cochain(n, 3),
        pseudo_cochain(n, 5),
        pseudo_cochain(n, 7),
    );
    let folded = cup_product(&c, &cup_product(&c, &x, 1, &y, 1).unwrap(), 2, &z, 1).unwrap();
    let nfold = cup_product_n(
        &c,
        &[(x.as_slice(), 1), (y.as_slice(), 1), (z.as_slice(), 1)],
    )
    .expect("n-fold");
    assert!(max_abs_diff(&folded, &nfold) < TOL);
}

#[test]
fn n_fold_of_a_single_factor_returns_it_unchanged() {
    let c = tetrahedron();
    let x = pseudo_cochain(c.num_cells(1), 3);
    let out = cup_product_n(&c, &[(x.as_slice(), 1)]).expect("n-fold");
    assert_eq!(out, x);
}

#[test]
fn triple_product_on_a_two_dimensional_complex_is_rejected() {
    let c = LatticeComplex::<2, f64>::square_torus(3);
    let n = c.num_cells(1);
    let (x, y, z) = (
        pseudo_cochain(n, 3),
        pseudo_cochain(n, 5),
        pseudo_cochain(n, 7),
    );
    assert!(
        cup_product_n(
            &c,
            &[(x.as_slice(), 1), (y.as_slice(), 1), (z.as_slice(), 1)]
        )
        .is_err(),
        "degree 3 does not exist on a surface"
    );
}

// --------------------------------------------------------------------------
// Topological ground truth
// --------------------------------------------------------------------------

#[test]
fn torus_direction_cochains_are_cocycles() {
    // The pairing below is a statement about cohomology classes, so the inputs
    // must be cocycles first.
    for l in [3, 4, 5] {
        let c = LatticeComplex::<2, f64>::square_torus(l);
        for dir in 0..2 {
            let a = direction_cochain(&c, dir);
            let d = delta(&c, 1, &a);
            assert!(
                d.iter().all(|v| v.abs() < 1e-12),
                "direction {dir} cochain is not a cocycle at L={l}"
            );
        }
    }
}

#[test]
fn torus_pairing_is_the_intersection_number() {
    // <a_x, a_y> = +L^2 and <a_y, a_x> = -L^2: the intersection number of the
    // two fundamental cycles, with the antisymmetry graded commutativity forces
    // at p = q = 1.
    for l in [3, 4, 5] {
        let c = LatticeComplex::<2, f64>::square_torus(l);
        let (ax, ay) = (direction_cochain(&c, 0), direction_cochain(&c, 1));
        let xy: f64 = cup_product(&c, &ax, 1, &ay, 1).unwrap().iter().sum();
        let yx: f64 = cup_product(&c, &ay, 1, &ax, 1).unwrap().iter().sum();
        let l2 = (l * l) as f64;
        assert!((xy - l2).abs() < TOL, "L={l}: <x,y> = {xy}, expected {l2}");
        assert!(
            (yx + l2).abs() < TOL,
            "L={l}: <y,x> = {yx}, expected {}",
            -l2
        );
    }
}

#[test]
fn a_torus_generator_has_no_self_intersection() {
    for l in [3, 4, 5] {
        let c = LatticeComplex::<2, f64>::square_torus(l);
        let ax = direction_cochain(&c, 0);
        let xx: f64 = cup_product(&c, &ax, 1, &ax, 1).unwrap().iter().sum();
        assert!(xx.abs() < TOL, "L={l}: <x,x> = {xx}, expected 0");
    }
}

#[test]
fn graded_commutativity_holds_on_cohomology() {
    // For cocycles of degree 1, a cup b and (-1)^{1*1} b cup a differ by a
    // coboundary. On a 2-torus a 2-cochain is a coboundary exactly when it sums
    // to zero, so the sum of the difference must vanish.
    let c = LatticeComplex::<2, f64>::square_torus(4);
    let (ax, ay) = (direction_cochain(&c, 0), direction_cochain(&c, 1));
    let xy: f64 = cup_product(&c, &ax, 1, &ay, 1).unwrap().iter().sum();
    let yx: f64 = cup_product(&c, &ay, 1, &ax, 1).unwrap().iter().sum();
    assert!((xy + yx).abs() < TOL, "difference is not a coboundary");
}

#[test]
fn arbitrary_cochains_need_not_commute() {
    // The cochain-level product is genuinely non-commutative; only the
    // cohomology statement holds. A test asserting equality here would be wrong.
    let c = LatticeComplex::<3, f64>::cubic_torus(3);
    let n = c.num_cells(1);
    let (x, y) = (pseudo_cochain(n, 31), pseudo_cochain(n, 37));
    let xy = cup_product(&c, &x, 1, &y, 1).unwrap();
    let yx = cup_product(&c, &y, 1, &x, 1).unwrap();
    assert!(
        max_abs_diff(&xy, &yx) > TOL,
        "expected the two orders to differ"
    );
}

#[test]
fn triple_product_of_the_generators_is_l_cubed() {
    // The pairing of the three fundamental cycles of the 3-torus.
    for l in [2, 3, 4] {
        let c = LatticeComplex::<3, f64>::cubic_torus(l);
        let (e0, e1, e2) = (
            direction_cochain(&c, 0),
            direction_cochain(&c, 1),
            direction_cochain(&c, 2),
        );
        let t: f64 = cup_product_n(
            &c,
            &[(e0.as_slice(), 1), (e1.as_slice(), 1), (e2.as_slice(), 1)],
        )
        .unwrap()
        .iter()
        .sum();
        let l3 = (l * l * l) as f64;
        assert!((t - l3).abs() < TOL, "L={l}: got {t}, expected {l3}");
    }
}

#[test]
fn triple_product_class_is_invariant_under_a_coboundary_shift() {
    // Replacing an input by alpha + delta f changes the product by a coboundary,
    // so its class, detected on the 3-torus by the total sum, is unchanged. This
    // is the cochain-level form of the invariance a logical gate needs.
    let l = 3;
    let c = LatticeComplex::<3, f64>::cubic_torus(l);
    let (e0, e1, e2) = (
        direction_cochain(&c, 0),
        direction_cochain(&c, 1),
        direction_cochain(&c, 2),
    );
    let total = |a: &[f64]| -> f64 {
        cup_product_n(&c, &[(a, 1), (e1.as_slice(), 1), (e2.as_slice(), 1)])
            .unwrap()
            .iter()
            .sum()
    };
    let before = total(&e0);
    let df = delta(&c, 0, &pseudo_cochain(c.num_cells(0), 4242));
    let shifted: Vec<f64> = e0.iter().zip(&df).map(|(a, b)| a + b).collect();
    assert!(
        (before - total(&shifted)).abs() < TOL,
        "class changed: {before} -> {}",
        total(&shifted)
    );
}

// --------------------------------------------------------------------------
// Genericity
// --------------------------------------------------------------------------

#[test]
fn both_complex_families_resolve_through_the_same_generic_path() {
    // One generic function, bounded only on ChainComplex plus the splitting
    // trait, serves a hand-built simplicial complex and a lattice complex alike.
    fn total_leibniz_residual<K: CellularComplex>(c: &K) -> f64
    where
        K::CellType: SplittableCell,
    {
        let a = pseudo_cochain(c.num_cells(1), 91);
        let b = pseudo_cochain(c.num_cells(1), 93);
        let lhs = delta(c, 2, &cup_product(c, &a, 1, &b, 1).unwrap());
        let t1 = cup_product(c, &delta(c, 1, &a), 2, &b, 1).unwrap();
        let t2 = cup_product(c, &a, 1, &delta(c, 1, &b), 2).unwrap();
        let rhs: Vec<f64> = t1.iter().zip(&t2).map(|(x, y)| x - y).collect();
        max_abs_diff(&lhs, &rhs)
    }
    assert!(total_leibniz_residual(&tetrahedron()) < TOL);
    assert!(total_leibniz_residual(&LatticeComplex::<3, f64>::cubic_torus(3)) < TOL);
}

// --------------------------------------------------------------------------
// Corner cases
//
// Every fixture below satisfies the Leibniz identity exactly under an
// independent Python reference using its own boundary operators and exact
// rational arithmetic, so these expectations are known-correct before any
// implementation exists to satisfy them.
// --------------------------------------------------------------------------

#[test]
fn leibniz_holds_on_a_non_square_torus() {
    // Extents 3 and 5: a wrapping bug reusing one axis extent for all axes
    // survives every square-lattice test and dies here.
    let c = LatticeComplex::<2, f64>::new([3, 5], [true, true]);
    for (p, q) in [(0, 0), (0, 1), (1, 0)] {
        assert_leibniz(&c, p, q);
    }
}

#[test]
fn leibniz_holds_on_a_cylinder() {
    // Mixed periodicity: x wraps, y does not. Split terms that fall outside the
    // complex contribute nothing, and the identity still holds.
    let c = LatticeComplex::<2, f64>::new([3, 3], [true, false]);
    for (p, q) in [(0, 0), (0, 1), (1, 0)] {
        assert_leibniz(&c, p, q);
    }
}

#[test]
fn leibniz_holds_on_a_fully_open_box() {
    // No periodicity at all, so every boundary cell has missing partners.
    let c = LatticeComplex::<2, f64>::new([3, 3], [false, false]);
    for (p, q) in [(0, 0), (0, 1), (1, 0)] {
        assert_leibniz(&c, p, q);
    }
}

#[test]
fn leibniz_holds_on_a_four_torus() {
    // Degree pairs unavailable in three dimensions.
    let c = LatticeComplex::<4, f64>::new([2, 2, 2, 2], [true; 4]);
    for (p, q) in [(1, 1), (1, 2)] {
        assert_leibniz(&c, p, q);
    }
}

#[test]
fn leibniz_holds_on_a_circle() {
    let c = LatticeComplex::<1, f64>::new([4], [true]);
    assert_leibniz(&c, 0, 0);
}

#[test]
fn n_fold_of_two_factors_equals_the_binary_product() {
    // An implementation that special-cases small arities would slip past a
    // three-factor test alone.
    let c = LatticeComplex::<3, f64>::cubic_torus(3);
    let n = c.num_cells(1);
    let (x, y) = (pseudo_cochain(n, 3), pseudo_cochain(n, 5));
    let binary = cup_product(&c, &x, 1, &y, 1).expect("binary");
    let nfold = cup_product_n(&c, &[(x.as_slice(), 1), (y.as_slice(), 1)]).expect("n-fold");
    assert!(max_abs_diff(&binary, &nfold) < TOL);
}

#[test]
fn n_fold_validates_every_factor_length() {
    let c = tetrahedron();
    let good = pseudo_cochain(c.num_cells(1), 3);
    let bad = vec![0.0; c.num_cells(1) + 1];
    assert!(
        cup_product_n(&c, &[(good.as_slice(), 1), (bad.as_slice(), 1)]).is_err(),
        "a wrong-length factor must be rejected wherever it appears"
    );
    assert!(
        cup_product_n(&c, &[(bad.as_slice(), 1), (good.as_slice(), 1)]).is_err(),
        "including in first position"
    );
}

#[test]
fn cubic_torus_direction_cochains_are_cocycles() {
    // Precondition for the triple-product tests: the pairing is a statement
    // about cohomology classes only if the inputs are cocycles.
    let c = LatticeComplex::<3, f64>::cubic_torus(3);
    for dir in 0..3 {
        let e = direction_cochain(&c, dir);
        let d = delta(&c, 1, &e);
        assert!(
            d.iter().all(|v| v.abs() < 1e-12),
            "direction {dir} cochain is not a cocycle"
        );
    }
}

#[test]
fn binary_product_class_is_independent_of_the_representative() {
    // The two-dimensional counterpart of the triple-product invariance: on a
    // 2-torus the class of a 2-cochain is its total sum.
    let l = 4;
    let c = LatticeComplex::<2, f64>::square_torus(l);
    let (ax, ay) = (direction_cochain(&c, 0), direction_cochain(&c, 1));
    let total = |a: &[f64]| -> f64 { cup_product(&c, a, 1, &ay, 1).unwrap().iter().sum() };
    let before = total(&ax);
    let df = delta(&c, 0, &pseudo_cochain(c.num_cells(0), 77));
    let shifted: Vec<f64> = ax.iter().zip(&df).map(|(a, b)| a + b).collect();
    assert!(
        (before - total(&shifted)).abs() < TOL,
        "class changed: {before} -> {}",
        total(&shifted)
    );
}

#[test]
fn a_zero_cochain_annihilates_the_product() {
    let c = tetrahedron();
    let zeros = vec![0.0; c.num_cells(1)];
    let x = pseudo_cochain(c.num_cells(1), 5);
    let out = cup_product(&c, &zeros, 1, &x, 1).expect("cup");
    assert!(out.iter().all(|v| v.abs() < TOL));
}

#[test]
fn degree_zero_cup_acts_by_the_leading_vertex() {
    // For AW at p = 0 the left cell is the simplex's first vertex, so a
    // 0-cochain reweights each simplex by its leading vertex's value.
    let c = tetrahedron();
    let v = simplex_index(&c, 0);
    let e = simplex_index(&c, 1);
    let mut f = vec![0.0; c.num_cells(0)];
    f[v[&Simplex::new(vec![0])]] = 4.0;
    let mut b = vec![0.0; c.num_cells(1)];
    b[e[&Simplex::new(vec![0, 1])]] = 6.0;
    let out = cup_product(&c, &f, 0, &b, 1).expect("cup");
    assert_eq!(out[e[&Simplex::new(vec![0, 1])]], 24.0, "f([0]) * b([0,1])");
}

// --------------------------------------------------------------------------
// Grade-contract edge cases
// --------------------------------------------------------------------------

#[test]
fn degree_sum_that_overflows_is_rejected() {
    // The degrees are caller-supplied. An overflowing sum would panic in debug
    // and wrap in release, the wrapped value then passing the dimension check.
    let c = tetrahedron();
    let a = pseudo_cochain(c.num_cells(1), 1);
    let b = pseudo_cochain(c.num_cells(1), 2);
    assert!(cup_product(&c, &a, usize::MAX, &b, 1).is_err());
    assert!(cup_product(&c, &a, 1, &b, usize::MAX).is_err());
}

#[test]
fn n_fold_rejects_a_first_factor_above_the_maximum_dimension() {
    // A degree above the complex's dimension has zero cells, so an empty
    // cochain there passes the length check. With a single factor the binary
    // path is never reached, so the grade contract has to be enforced here too,
    // or the two APIs would disagree on the same request.
    let c = LatticeComplex::<2, f64>::square_torus(3);
    let empty: Vec<f64> = Vec::new();
    assert_eq!(c.num_cells(99), 0, "the degree really does have no cells");
    assert!(
        cup_product_n(&c, &[(empty.as_slice(), 99)]).is_err(),
        "single factor above max_dim must be rejected"
    );
}

#[test]
fn the_two_apis_agree_on_an_out_of_range_degree() {
    // Whatever the answer is, `cup_product` and `cup_product_n` must give it.
    let c = LatticeComplex::<2, f64>::square_torus(3);
    let empty: Vec<f64> = Vec::new();
    let a = pseudo_cochain(c.num_cells(0), 5);
    let binary = cup_product(&c, &empty, 99, &a, 0).is_err();
    let nfold = cup_product_n(&c, &[(empty.as_slice(), 99), (a.as_slice(), 0)]).is_err();
    assert_eq!(
        binary, nfold,
        "binary and n-fold disagree on the grade contract"
    );
    assert!(binary);
}

#[test]
fn simplicial_cup_skips_a_split_whose_partner_is_absent_from_the_complex() {
    use deep_causality_topology::{SimplicialComplex, Skeleton};

    // Alexander–Whitney splits [0,1,2] as ([0,1], [1,2]). This complex is not
    // closed under faces: its 1-skeleton holds [0,1] alone, so the right factor
    // has no cell to be evaluated on. The term contributes nothing rather than
    // failing, and the product on that face is zero.
    let skeletons = vec![
        Skeleton::new(
            0,
            vec![
                Simplex::new(vec![0]),
                Simplex::new(vec![1]),
                Simplex::new(vec![2]),
            ],
        ),
        Skeleton::new(1, vec![Simplex::new(vec![0, 1])]),
        Skeleton::new(2, vec![Simplex::new(vec![0, 1, 2])]),
    ];
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(skeletons, Vec::new(), Vec::new(), Vec::new());

    let alpha = vec![3.0];
    let beta = vec![5.0];
    let out = cup_product(&complex, &alpha, 1, &beta, 1).expect("cup");

    assert_eq!(out, vec![0.0]);
}
