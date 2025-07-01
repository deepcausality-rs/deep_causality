/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;
use deep_causality::utils_test::test_utils_generator::*;
use std::hash::Hash;

// A test processor to act as a destination for the generative output.
struct TestProcessor<D, S, T, ST, SYM, VS, VT>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    causaloid_dest: Option<Causaloid<D, S, T, ST, SYM, VS, VT>>,
    context_dest: Option<Context<D, S, T, ST, SYM, VS, VT>>,
}

impl<D, S, T, ST, SYM, VS, VT> TestProcessor<D, S, T, ST, SYM, VS, VT>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn new() -> Self {
        Self {
            causaloid_dest: None,
            context_dest: None,
        }
    }
}

// Implement the processor trait so it can be used to test generators.
impl<D, S, T, ST, SYM, VS, VT, G> GenerativeProcessor<D, S, T, ST, SYM, VS, VT, G>
    for TestProcessor<D, S, T, ST, SYM, VS, VT>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<D, S, T, ST, SYM, VS, VT, G>,
{
    fn get_causaloid_dest(&mut self) -> &mut Option<Causaloid<D, S, T, ST, SYM, VS, VT>> {
        &mut self.causaloid_dest
    }

    fn get_context_dest(&mut self) -> &mut Option<Context<D, S, T, ST, SYM, VS, VT>> {
        &mut self.context_dest
    }
}

// Type alias for brevity in tests
type TestProcessorAlias = TestProcessor<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

#[test]
fn test_generate_create_causaloid() {
    struct CreateCausaloidGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            CreateCausaloidGenerator,
        > for CreateCausaloidGenerator
    {
        fn generate(
            &mut self,
            _trigger: &GenerativeTrigger<MockData>,
            _context: &Context<
                MockData,
                EuclideanSpace,
                EuclideanTime,
                EuclideanSpacetime,
                BaseSymbol,
                FloatType,
                FloatType,
            >,
        ) -> Result<
            GenerativeOutput<
                MockData,
                EuclideanSpace,
                EuclideanTime,
                EuclideanSpacetime,
                BaseSymbol,
                FloatType,
                FloatType,
                CreateCausaloidGenerator,
            >,
            ModelGenerativeError,
        > {
            let causaloid = TestCausaloid::new(1, |_| Ok(false), "Test Causaloid");
            Ok(GenerativeOutput::CreateCausaloid(1, causaloid))
        }
    }

    let mut generator = CreateCausaloidGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());

    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_ok());
    assert!(processor.causaloid_dest.is_some());
    assert_eq!(processor.causaloid_dest.unwrap().id(), 1);
    assert!(processor.context_dest.is_none());
}

#[test]
fn test_generate_create_and_update_context() {
    struct UpdateContextGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            UpdateContextGenerator,
        > for UpdateContextGenerator
    {
        fn generate(
            &mut self,
            _trigger: &GenerativeTrigger<MockData>,
            _context: &Context<
                MockData,
                EuclideanSpace,
                EuclideanTime,
                EuclideanSpacetime,
                BaseSymbol,
                FloatType,
                FloatType,
            >,
        ) -> Result<
            GenerativeOutput<
                MockData,
                EuclideanSpace,
                EuclideanTime,
                EuclideanSpacetime,
                BaseSymbol,
                FloatType,
                FloatType,
                UpdateContextGenerator,
            >,
            ModelGenerativeError,
        > {
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 42,
                name: "Initial Name".to_string(),
                capacity: 10,
            };
            let update_context = GenerativeOutput::UpdateContext {
                id: 42,
                new_name: Some("Updated Name".to_string()),
            };
            Ok(GenerativeOutput::Composite(vec![
                create_context,
                update_context,
            ]))
        }
    }

    let mut generator = UpdateContextGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_ok());
    assert!(processor.context_dest.is_some());
    let context = processor.context_dest.unwrap();
    assert_eq!(context.id(), 42);
    assert_eq!(context.name(), "Updated Name");
}

#[test]
fn test_generate_add_and_delete_contextoid() {
    struct ContextoidLifecycleGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            ContextoidLifecycleGenerator,
        > for ContextoidLifecycleGenerator
    {
        fn generate(
            &mut self,
            _trigger: &GenerativeTrigger<MockData>,
            _context: &Context<
                MockData,
                EuclideanSpace,
                EuclideanTime,
                EuclideanSpacetime,
                BaseSymbol,
                FloatType,
                FloatType,
            >,
        ) -> Result<
            GenerativeOutput<
                MockData,
                EuclideanSpace,
                EuclideanTime,
                EuclideanSpacetime,
                BaseSymbol,
                FloatType,
                FloatType,
                ContextoidLifecycleGenerator,
            >,
            ModelGenerativeError,
        > {
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 10,
                name: "Test Context".to_string(),
                capacity: 5,
            };

            let data = MockData { id: 100, data: 1 };
            let contextoid = TestContextoid::new(100, ContextoidType::Datoid(data));
            let add_contextoid = GenerativeOutput::AddContextoidToContext {
                context_id: 10,
                contextoid,
            };

            let delete_contextoid = GenerativeOutput::DeleteContextoidFromContext {
                context_id: 10,
                contextoid_id: 100,
            };

            Ok(GenerativeOutput::Composite(vec![
                create_context,
                add_contextoid,
                delete_contextoid,
            ]))
        }
    }

    let mut generator = ContextoidLifecycleGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_ok(), "Processing failed: {:?}", result.err());
    assert!(processor.context_dest.is_some());
    let context = processor.context_dest.as_ref().unwrap();
    assert_eq!(context.number_of_nodes(), 0);
}

#[test]
fn test_generate_evolve_fails_in_default_processor() {
    // The default processor does not support Evolve. This test verifies that.
    struct EvolveGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            EvolveGenerator,
        > for EvolveGenerator
    {
        fn generate(
            &mut self,
            _trigger: &GenerativeTrigger<MockData>,
            _context: &Context<
                MockData,
                EuclideanSpace,
                EuclideanTime,
                EuclideanSpacetime,
                BaseSymbol,
                FloatType,
                FloatType,
            >,
        ) -> Result<
            GenerativeOutput<
                MockData,
                EuclideanSpace,
                EuclideanTime,
                EuclideanSpacetime,
                BaseSymbol,
                FloatType,
                FloatType,
                EvolveGenerator,
            >,
            ModelGenerativeError,
        > {
            Ok(GenerativeOutput::Evolve(EvolveGenerator))
        }
    }

    let mut generator = EvolveGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_err());
    if let Err(ModelValidationError::UnsupportedOperation { .. }) = result {
        // Correct error type
    } else {
        panic!("Expected UnsupportedOperation error, got {result:?}");
    }
}
