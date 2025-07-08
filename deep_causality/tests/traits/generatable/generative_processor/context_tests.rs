/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_generator::*;
use deep_causality::*;

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
