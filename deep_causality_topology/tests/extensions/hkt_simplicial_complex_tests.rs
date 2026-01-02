use deep_causality_haft::Adjunction;
use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::{Chain, ChainWitness, Simplex, SimplicialComplex, Skeleton};
use std::sync::Arc;

fn create_simple_complex() -> Arc<SimplicialComplex> {
    // Single triangle: {0, 1, 2}
    let vertices = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
    ];
    let skeleton_0 = Skeleton::new(0, vertices);

    // We only need 0-skeleton for the current HKT implementation of unit/left_adjunct
    // as it defaults to 0-skeleton in the code I read.
    Arc::new(SimplicialComplex::new(
        vec![skeleton_0],
        vec![],
        vec![],
        vec![],
    ))
}

#[test]
fn test_simplicial_complex_unit() {
    let complex = create_simple_complex();
    let val = 42.0;

    // Unit: Embed scalar into Chain<Chain<A>>
    let chain_of_chains: Chain<Chain<f64>> = ChainWitness::unit(&(complex, 0), val);

    // Verify outer chain
    assert_eq!(chain_of_chains.grade(), 0);
    // Outer chain contains 1 element (the inner chain) at index 0 (unit impl details).
    let outer_w = chain_of_chains.weights();
    // Pure creates 1x1 matrix with value at (0,0).
    // Check first element of outer chain
    if let Some(inner_chain) = outer_w.values().iter().next() {
        let w: &CsrMatrix<f64> = inner_chain.weights();
        assert_eq!(w.get_value_at(0, 0), 42.0);
    } else {
        panic!("Unit resulted in empty outer chain");
    }
}

#[test]
fn test_simplicial_complex_left_adjunct() {
    let complex = create_simple_complex();

    // Left Adjunct: (Chain<A> -> B) -> (A -> Chain<B>)
    // f: Chain<f64> -> f64
    let f = |c: Chain<f64>| c.weights().values().iter().sum::<f64>();

    let chain: Chain<f64> = ChainWitness::left_adjunct(&(complex, 0), 0.0, f);

    // Expect Chain<f64> containing f(unit(0.0)).
    // unit(0.0) -> Chain<Chain<f64>> with inner value 0.0.
    // f(inner) -> sum(0.0) -> 0.0.
    // So distinct chain with single value 0.0.

    assert_eq!(chain.weights().get_value_at(0, 0), 0.0);
}

#[test]
#[allow(unused_variables)]
fn test_simplicial_complex_counit() {
    let complex = create_simple_complex();

    // Counit: Chain<Chain<B>> -> B
    // We construct a nested chain manually or via unit.
    let inner_val = 100.0;
    // We can use unit to create Chain<Chain<f64>> easily.
    let chain_chain = ChainWitness::unit(&(complex.clone(), 0), inner_val);

    let result = ChainWitness::counit(&(complex, 0), chain_chain);
    assert_eq!(result, 100.0);
}

#[test]
fn test_simplicial_complex_right_adjunct() {
    let complex = create_simple_complex();

    // Right Adjunct: (A -> Chain<B>) -> (Chain<A> -> B)
    // Chain<A> with weights at 0 and 2.
    let size = 3;
    let weights =
        CsrMatrix::from_triplets(1, size, &[(0, 0, 2.0), (0, 2, 3.0)]).expect("Matrix failed");

    let chain = Chain::new(complex.clone(), 0, weights);

    // f: f64 -> Chain<f64>
    // f(w) -> Chain with weight w*10 at index 0.
    let f = |w: f64| -> Chain<f64> {
        let val = w * 10.0;
        let w_matrix = CsrMatrix::from_triplets(1, 1, &[(0, 0, val)]).unwrap();
        Chain::new(complex.clone(), 0, w_matrix)
    };

    // Execution:
    // fmap(chain, f) -> Chain<Chain<f64>>.    // Execution:
    // right_adjunct expects context: &(Arc<SimplicialComplex>, usize).
    // We clone complex for the context tuple to avoid move errors since closure borrows it.
    let ctx_complex = complex.clone();
    let result = ChainWitness::right_adjunct(&(ctx_complex, 0), chain, f);

    assert_eq!(result, 20.0);
}
