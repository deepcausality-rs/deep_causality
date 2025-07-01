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
fn test_update_context_with_mismatched_id() {
    // 1. SETUP: Create a processor and place a context with ID 1 in its destination.
    let mut processor = TestProcessorAlias::new();
    processor.context_dest = Some(TestContext::with_capacity(1, "I exist", 10));

    // 2. ACTION: Attempt to process an `UpdateContext` command that targets a *different* ID (e.g., 99).
    // This will call `get_and_verify_context` with `target_id = 99`.
    // The function will find the context with ID 1, see that `1 != 99`, and return the target error.
    let update_with_mismatched_id: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::UpdateContext {
        id: 99, // The ID we are looking for, which doesn't match the one in the processor.
        new_name: Some("New Name".to_string()),
    };

    let result = processor.process_output(update_with_mismatched_id);

    // 3. ASSERT: Verify that the specific error for a mismatched ID was returned.
    assert!(matches!(
        result,
        Err(ModelValidationError::TargetContextNotFound { id: 99 })
    ));
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

#[test]
fn test_generate_update_causaloid() {
    struct UpdateCausaloidGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            UpdateCausaloidGenerator,
        > for UpdateCausaloidGenerator
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
                UpdateCausaloidGenerator,
            >,
            ModelGenerativeError,
        > {
            let create_causaloid = GenerativeOutput::CreateCausaloid(
                1,
                TestCausaloid::new(1, |_| Ok(false), "Initial Causaloid"),
            );

            let updated_causaloid = TestCausaloid::new(1, |_| Ok(false), "Updated Causaloid");
            let update_causaloid = GenerativeOutput::UpdateCausaloid(1, updated_causaloid);

            Ok(GenerativeOutput::Composite(vec![
                create_causaloid,
                update_causaloid,
            ]))
        }
    }

    let mut generator = UpdateCausaloidGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_ok());
    assert!(processor.causaloid_dest.is_some());
    assert_eq!(
        processor.causaloid_dest.as_ref().unwrap().description(),
        "Updated Causaloid"
    );
}

#[test]
fn test_generate_delete_causaloid() {
    struct DeleteCausaloidGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            DeleteCausaloidGenerator,
        > for DeleteCausaloidGenerator
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
                DeleteCausaloidGenerator,
            >,
            ModelGenerativeError,
        > {
            let create_causaloid = GenerativeOutput::CreateCausaloid(
                1,
                TestCausaloid::new(1, |_| Ok(false), "Initial Causaloid"),
            );
            let delete_causaloid = GenerativeOutput::DeleteCausaloid(1);
            Ok(GenerativeOutput::Composite(vec![
                create_causaloid,
                delete_causaloid,
            ]))
        }
    }

    let mut generator = DeleteCausaloidGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_ok());
    assert!(processor.causaloid_dest.is_none());
}

#[test]
fn test_generate_delete_context() {
    struct DeleteContextGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            DeleteContextGenerator,
        > for DeleteContextGenerator
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
                DeleteContextGenerator,
            >,
            ModelGenerativeError,
        > {
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 42,
                name: "Initial Name".to_string(),
                capacity: 10,
            };
            let delete_context = GenerativeOutput::DeleteContext { id: 42 };
            Ok(GenerativeOutput::Composite(vec![
                create_context,
                delete_context,
            ]))
        }
    }

    let mut generator = DeleteContextGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_ok());
    assert!(processor.context_dest.is_none());
}

#[test]
fn test_generate_create_extra_context() {
    struct ExtraContextGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            ExtraContextGenerator,
        > for ExtraContextGenerator
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
                ExtraContextGenerator,
            >,
            ModelGenerativeError,
        > {
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 1,
                name: "Base Context".to_string(),
                capacity: 5,
            };
            let create_extra = GenerativeOutput::CreateExtraContext {
                extra_context_id: 2,
                capacity: 3,
            };
            Ok(GenerativeOutput::Composite(vec![
                create_context,
                create_extra,
            ]))
        }
    }

    let mut generator = ExtraContextGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_ok());
    assert!(processor.context_dest.is_some());
}

#[test]
fn test_generate_update_contextoid() {
    struct UpdateContextoidGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            UpdateContextoidGenerator,
        > for UpdateContextoidGenerator
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
                UpdateContextoidGenerator,
            >,
            ModelGenerativeError,
        > {
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 10,
                name: "Test Context".to_string(),
                capacity: 5,
            };

            let data1 = MockData { id: 100, data: 1 };
            let contextoid1 = TestContextoid::new(100, ContextoidType::Datoid(data1));
            let add_contextoid = GenerativeOutput::AddContextoidToContext {
                context_id: 10,
                contextoid: contextoid1,
            };

            let data2 = MockData { id: 100, data: 2 };
            let contextoid2 = TestContextoid::new(100, ContextoidType::Datoid(data2));
            let update_contextoid = GenerativeOutput::UpdateContextoidInContext {
                context_id: 10,
                existing_contextoid: 100,
                new_contextoid: contextoid2,
            };

            Ok(GenerativeOutput::Composite(vec![
                create_context,
                add_contextoid,
                update_contextoid,
            ]))
        }
    }

    let mut generator = UpdateContextoidGenerator;
    let empty_context = TestContext::with_capacity(0, "Empty", 50);
    let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
    let generated_output = generator.generate(&trigger, &empty_context).unwrap();

    let mut processor = TestProcessorAlias::new();
    let result = processor.process_output(generated_output);

    assert!(result.is_ok());
    let context = processor.context_dest.as_ref().unwrap();
    // The add_node method returns the index, which is needed for get_node.
    // Assuming the first node added gets index 0.
    let node = context.get_node(0).unwrap();
    if let ContextoidType::Datoid(d) = node.vertex_type() {
        assert_eq!(d.get_data(), 2);
    } else {
        panic!("Wrong contextoid type");
    }
}

// Define a dummy generator for testing standalone outputs.
struct DummyGenerator;
impl
    Generatable<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > for DummyGenerator
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
            DummyGenerator,
        >,
        ModelGenerativeError,
    > {
        Ok(GenerativeOutput::NoOp)
    }
}

#[test]
fn test_no_op() {
    let mut processor = TestProcessorAlias::new();
    let output: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::NoOp;
    let result = processor.process_output(output);
    assert!(result.is_ok());
    assert!(processor.causaloid_dest.is_none());
    assert!(processor.context_dest.is_none());
}

#[test]
fn test_processing_failures() {
    // 1. Update non-existent causaloid
    let mut processor = TestProcessorAlias::new();
    let update_non_existent_causaloid: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::UpdateCausaloid(99, TestCausaloid::new(99, |_| Ok(false), ""));
    let res = processor.process_output(update_non_existent_causaloid);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetCausaloidNotFound { id: 99 })
    ));

    // 2. Delete non-existent causaloid
    let mut processor = TestProcessorAlias::new();
    let delete_non_existent_causaloid: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::DeleteCausaloid(99);
    let res = processor.process_output(delete_non_existent_causaloid);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetCausaloidNotFound { id: 99 })
    ));

    // 3. Create causaloid that already exists
    let mut processor = TestProcessorAlias::new();
    let create_causaloid: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateCausaloid(1, TestCausaloid::new(1, |_| Ok(false), ""));
    processor.process_output(create_causaloid).unwrap();
    let create_duplicate_causaloid: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateCausaloid(1, TestCausaloid::new(1, |_| Ok(false), ""));
    let res = processor.process_output(create_duplicate_causaloid);
    assert!(matches!(
        res,
        Err(ModelValidationError::DuplicateCausaloidID { id: 1 })
    ));

    // 4. Create context that already exists
    let mut processor = TestProcessorAlias::new();
    let create_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateBaseContext {
        id: 1,
        name: "".to_string(),
        capacity: 1,
    };
    processor.process_output(create_context).unwrap();
    let create_duplicate_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateBaseContext {
        id: 1,
        name: "".to_string(),
        capacity: 1,
    };
    let res = processor.process_output(create_duplicate_context);
    assert!(matches!(
        res,
        Err(ModelValidationError::DuplicateContextId { id: 1 })
    ));

    // 5. Update non-existent context
    let mut processor = TestProcessorAlias::new();
    let update_non_existent_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::UpdateContext {
        id: 99,
        new_name: None,
    };
    let res = processor.process_output(update_non_existent_context);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetContextNotFound { id: 99 })
    ));

    // 6. Delete non-existent context
    let mut processor = TestProcessorAlias::new();
    let delete_non_existent_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::DeleteContext { id: 99 };
    let res = processor.process_output(delete_non_existent_context);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetContextNotFound { id: 99 })
    ));

    // 7. Create extra context without base context
    let mut processor = TestProcessorAlias::new();
    let create_extra_no_base: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateExtraContext {
        extra_context_id: 99,
        capacity: 1,
    };
    let res = processor.process_output(create_extra_no_base);
    assert!(matches!(
        res,
        Err(ModelValidationError::BaseContextNotFound)
    ));

    // 8. Create extra context with duplicate ID
    let mut processor = TestProcessorAlias::new();
    let create_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateBaseContext {
        id: 1,
        name: "".to_string(),
        capacity: 1,
    };
    processor.process_output(create_context).unwrap();
    let create_extra: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateExtraContext {
        extra_context_id: 2,
        capacity: 1,
    };
    processor.process_output(create_extra).unwrap();
    let create_duplicate_extra: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateExtraContext {
        extra_context_id: 2,
        capacity: 1,
    };
    let res = processor.process_output(create_duplicate_extra);
    assert!(matches!(
        res,
        Err(ModelValidationError::DuplicateExtraContextId { id: 2 })
    ));

    // 9. Add contextoid to non-existent context
    let mut processor = TestProcessorAlias::new();
    let add_to_non_existent_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::AddContextoidToContext {
        context_id: 99,
        contextoid: TestContextoid::new(1, ContextoidType::Datoid(MockData::default())),
    };
    let res = processor.process_output(add_to_non_existent_context);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetContextNotFound { id: 99 })
    ));

    // 10. Update contextoid in non-existent context
    let mut processor = TestProcessorAlias::new();
    let update_in_non_existent_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::UpdateContextoidInContext {
        context_id: 99,
        existing_contextoid: 1,
        new_contextoid: TestContextoid::new(1, ContextoidType::Datoid(MockData::default())),
    };
    let res = processor.process_output(update_in_non_existent_context);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetContextNotFound { id: 99 })
    ));

    // 11. Delete contextoid from non-existent context
    let mut processor = TestProcessorAlias::new();
    let delete_from_non_existent_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::DeleteContextoidFromContext {
        context_id: 99,
        contextoid_id: 1,
    };
    let res = processor.process_output(delete_from_non_existent_context);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetContextNotFound { id: 99 })
    ));

    // 12. Update non-existent contextoid
    let mut processor = TestProcessorAlias::new();
    let create_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateBaseContext {
        id: 1,
        name: "".to_string(),
        capacity: 1,
    };
    processor.process_output(create_context).unwrap();
    let update_non_existent_contextoid: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::UpdateContextoidInContext {
        context_id: 1,
        existing_contextoid: 99,
        new_contextoid: TestContextoid::new(99, ContextoidType::Datoid(MockData::default())),
    };
    let res = processor.process_output(update_non_existent_contextoid);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetContextoidNotFound { id: 99 })
    ));

    // 13. Delete non-existent contextoid
    let mut processor = TestProcessorAlias::new();
    let create_context: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::CreateBaseContext {
        id: 1,
        name: "".to_string(),
        capacity: 1,
    };
    processor.process_output(create_context).unwrap();
    let delete_non_existent_contextoid: GenerativeOutput<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > = GenerativeOutput::DeleteContextoidFromContext {
        context_id: 1,
        contextoid_id: 99,
    };
    let res = processor.process_output(delete_non_existent_contextoid);
    assert!(matches!(
        res,
        Err(ModelValidationError::TargetContextoidNotFound { id: 99 })
    ));
}
