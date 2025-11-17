/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

fn main() {
    println!();
    // The graph must be mutable at first to build it.
    let g = get_multi_cause_graph();

    println!("Full reasoning over the entire graph");
    // The new reasoning API uses a unified `PropagatingEffect` type.
    let evidence = PropagatingEffect::from_numerical(0.99);
    let root_index = g.get_root_index().expect("Graph has no root");

    // Call the new reasoning method from the `CausableGraphReasoning` trait.
    let res = g.evaluate_subgraph_from_cause(root_index, &evidence);
    assert!(res.is_ok());

    assert_eq!(res.value, EffectValue::Numerical(1.0));
    println!("Explain subgraph reasoning");
    println!("{}", res.explain());

    println!("Partial reasoning over shortest path through the graph");
    let start_index = 2;
    let stop_index = 3;

    // Call the new shortest path reasoning method.
    let res = g.evaluate_shortest_path_between_causes(start_index, stop_index, &evidence);
    assert!(res.is_ok());

    assert_eq!(res.value, EffectValue::Numerical(1.0));
    println!();

    println!("Explain partial reasoning");
    println!("{}", res.explain());
}

pub fn get_test_causaloid(
    id: IdentificationValue,
) -> BaseCausaloid<NumericalValue, NumericalValue> {
    let description = "tests whether data exceeds threshold of 0.75";

    // The signature: CausalFn<I: IntoEffectValue, O: IntoEffectValue> = fn(value: I) -> Result<CausalFnOutput<O>, CausalityError>
    // IntoEffectValue is implemented by default for all primitive types and by all complex types supported
    // by PropagatingEffect. Notice, when you call causaloid.evaluate(&PropagatingEffect), the PropagatingEffect
    // converts automatically into the matching NumericalValue via the IntoEffectValue default implementation.
    fn causal_fn(obs: NumericalValue) -> Result<CausalFnOutput<NumericalValue>, CausalityError> {
        // the log is part of the CausalFnOutput.
        // When multiple causaloid are called in sequence, the logs are appended to the resulting
        // propagating effect meaning the final result carries a full immutable history how it was produced.
        let mut log = CausalEffectLog::new();
        if obs.is_sign_negative() {
            // At any point, you can short circuit and return an error,
            return Err(CausalityError("Observation is negative".into()));
        }

        // Logic can be arbitrary as long as it produces the annotated return type.
        let threshold: NumericalValue = 0.75;
        // The return type is NumericalValue, an alias for f64, so we have to encode the result as float
        let is_active = if obs.ge(&threshold) { 1.0f64 } else { 0.0f64 };

        log.add_entry(&format!(
            "Observation {} is larger than threshold {}: {}",
            obs, threshold, is_active
        ));

        // Log each relevant step
        log.add_entry("Causal function executed successfully");
        // Return the final result and its log.
        Ok(CausalFnOutput::new(is_active, log))
    }

    Causaloid::new(id, causal_fn, description)
}

fn get_multi_cause_graph() -> StarterCausalGraph {
    // Builds a multi cause graph:
    //  root
    //  / \
    //  A B
    //  \ /
    //   C

    // The CausaloidGraph constructor now requires a unique ID.
    let mut g = CausaloidGraph::new(0);

    // Add root causaloid
    let root_causaloid = get_test_causaloid(0);
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root");

    // Add causaloid A
    let causaloid = get_test_causaloid(1);
    let idx_a = g
        .add_causaloid(causaloid)
        .expect("Failed to add causaloid A");

    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a)
        .expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = get_test_causaloid(2);
    let idx_b = g
        .add_causaloid(causaloid)
        .expect("Failed to add causaloid B");

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b)
        .expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = get_test_causaloid(3);
    let idx_c = g
        .add_causaloid(causaloid)
        .expect("Failed to add causaloid C");

    // Link causaloid C to A
    g.add_edge(idx_a, idx_c)
        .expect("Failed to add edge between A and C");

    // Link causaloid C to B
    g.add_edge(idx_b, idx_c)
        .expect("Failed to add edge between C and B");

    // Freeze the graph before reasoning or explaining.
    // This ensures high-performance graph traversal algorithms.
    g.freeze();

    g
}

// # Causaloid Type Parameters
// - `I`: The type of the input effect value, must implement `IntoEffectValue`.
// - `O`: The type of the output effect value, must implement `IntoEffectValue`.
// -- These are only relevant when using context.
// - `D`: The type for data context, must implement `Datable` and `Clone`.
// - `S`: The type for spatial context, must implement `Spatial<VS>` and `Clone`.
// - `T`: The type for temporal context, must implement `Temporal<VT>` and `Clone`.
// - `ST`: The type for spatiotemporal context, must implement `SpaceTemporal<VS, VT>` and `Clone`.
// - `SYM`: The type for symbolic context, must implement `Symbolic` and `Clone`.
// - `VS`: The value type for spatial data, must implement `Clone`.
// - `VT`: The value type for temporal data, must implement `Clone`.
pub type StarterCausalGraph = CausaloidGraph<
    Causaloid<
        // For a graph with heterogeneous nodes, set the input and output types to "EffectValue"
        // to allow for diverse input types. Note, these are only runtime checked.
        //
        // The input type of the causaloid.
        NumericalValue,
        // The output type of the causaloid
        NumericalValue,
        // Context type parameters. Unused in this example and thus set to some defaults.
        Data<NumericalValue>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;
