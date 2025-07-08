/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_generator::*;
use deep_causality::*;

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
