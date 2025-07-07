/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_generator::*;
use deep_causality::*;

#[test]
fn test_updates_contextoid() {
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
            let causaloid = TestCausaloid::new(1, |_| Ok(false), "causaloid");
            let create_causaloid = GenerativeOutput::CreateCausaloid(1, causaloid);
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 10,
                name: "Context".to_string(),
                capacity: 5,
            };

            let original_data = MockData { id: 100, data: 1 };
            let original_contextoid =
                TestContextoid::new(100, ContextoidType::Datoid(original_data));

            let add_contextoid = GenerativeOutput::AddContextoidToContext {
                context_id: 10,
                contextoid: original_contextoid,
            };

            let updated_data = MockData { id: 100, data: 99 };
            let updated_contextoid = TestContextoid::new(100, ContextoidType::Datoid(updated_data));

            let update_contextoid = GenerativeOutput::UpdateContextoidInContext {
                context_id: 10,
                existing_contextoid: 100,
                new_contextoid: updated_contextoid,
            };

            Ok(GenerativeOutput::Composite(vec![
                create_causaloid,
                create_context,
                add_contextoid,
                update_contextoid,
            ]))
        }
    }

    let model_result = TestModel::with_generator(
        1,
        "author",
        "desc",
        None,
        UpdateContextoidGenerator,
        &GenerativeTrigger::ManualIntervention("trigger".to_string()),
    );

    // The main goal is to ensure this complex operation succeeds without error.
    // Direct verification of the updated data is difficult without changing the public API,
    // but success implies the processor correctly routed the command.
    assert!(
        model_result.is_ok(),
        "Model generation with contextoid update failed: {:?}",
        model_result.err()
    );
}

#[test]
fn test_deletes_contextoid() {
    struct DeleteContextoidGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            DeleteContextoidGenerator,
        > for DeleteContextoidGenerator
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
                DeleteContextoidGenerator,
            >,
            ModelGenerativeError,
        > {
            let causaloid = TestCausaloid::new(1, |_| Ok(false), "causaloid");
            let create_causaloid = GenerativeOutput::CreateCausaloid(1, causaloid);
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 10,
                name: "Context".to_string(),
                capacity: 5,
            };

            let data1 = MockData { id: 100, data: 1 };
            let contextoid1 = TestContextoid::new(100, ContextoidType::Datoid(data1));
            let add_contextoid1 = GenerativeOutput::AddContextoidToContext {
                context_id: 10,
                contextoid: contextoid1,
            };

            let data2 = MockData { id: 200, data: 2 };
            let contextoid2 = TestContextoid::new(200, ContextoidType::Datoid(data2));
            let add_contextoid2 = GenerativeOutput::AddContextoidToContext {
                context_id: 10,
                contextoid: contextoid2,
            };

            let delete_contextoid = GenerativeOutput::DeleteContextoidFromContext {
                context_id: 10,
                contextoid_id: 100, // Delete the first one
            };

            Ok(GenerativeOutput::Composite(vec![
                create_causaloid,
                create_context,
                add_contextoid1,
                add_contextoid2,
                delete_contextoid,
            ]))
        }
    }

    let model_result = TestModel::with_generator(
        1,
        "author",
        "desc",
        None,
        DeleteContextoidGenerator,
        &GenerativeTrigger::ManualIntervention("trigger".to_string()),
    );

    assert!(
        model_result.is_ok(),
        "Model generation with contextoid delete failed: {:?}",
        model_result.err()
    );
    let model = model_result.unwrap();
    let context = model.context().as_ref().unwrap();

    // Started with 0, added 2, deleted 1. Should be 1 left.
    assert_eq!(context.number_of_nodes(), 1);
}
