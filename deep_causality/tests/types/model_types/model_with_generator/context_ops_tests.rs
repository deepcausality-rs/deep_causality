/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;
use deep_causality::utils_test::test_utils_generator::*;

#[test]
fn test_updates_context_name() {
    struct UpdateContextNameGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            UpdateContextNameGenerator,
        > for UpdateContextNameGenerator
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
                UpdateContextNameGenerator,
            >,
            ModelGenerativeError,
        > {
            let causaloid = TestCausaloid::new(1, |_| Ok(false), "causaloid");
            let create_causaloid = GenerativeOutput::CreateCausaloid(1, causaloid);
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 10,
                name: "Initial Name".to_string(),
                capacity: 5,
            };
            let update_context = GenerativeOutput::UpdateContext {
                id: 10,
                new_name: Some("Updated Name".to_string()),
            };
            Ok(GenerativeOutput::Composite(vec![
                create_causaloid,
                create_context,
                update_context,
            ]))
        }
    }

    let model_result = TestModel::with_generator(
        1,
        "author",
        "desc",
        None,
        UpdateContextNameGenerator,
        &GenerativeTrigger::ManualIntervention("trigger".to_string()),
    );

    assert!(model_result.is_ok());
    let model = model_result.unwrap();
    let context = model.context().as_ref().unwrap();
    assert_eq!(context.id(), 10);
    assert_eq!(context.name(), "Updated Name");
}

#[test]
fn test_deletes_context() {
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
            let causaloid = TestCausaloid::new(1, |_| Ok(false), "causaloid");
            let create_causaloid = GenerativeOutput::CreateCausaloid(1, causaloid);
            let create_context = GenerativeOutput::CreateBaseContext {
                id: 10,
                name: "To Be Deleted".to_string(),
                capacity: 5,
            };
            let delete_context = GenerativeOutput::DeleteContext { id: 10 };
            Ok(GenerativeOutput::Composite(vec![
                create_causaloid,
                create_context,
                delete_context,
            ]))
        }
    }

    let model_result = TestModel::with_generator(
        1,
        "author",
        "desc",
        None,
        DeleteContextGenerator,
        &GenerativeTrigger::ManualIntervention("trigger".to_string()),
    );

    assert!(model_result.is_ok());
    let model = model_result.unwrap();
    assert!(model.context().is_none());
}
