/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;
use deep_causality::utils_test::test_utils_generator::*;

#[test]
fn test_fails_on_add_contextoid_to_nonexistent_context() {
    struct BadContextoidGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            BadContextoidGenerator,
        > for BadContextoidGenerator
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
                BadContextoidGenerator,
            >,
            ModelGenerativeError,
        > {
            let causaloid = TestCausaloid::new(1000, |_| Ok(false), "causaloid");
            let bad_contextoid = TestContextoid::new(1001, ContextoidType::Root(Root::new(101)));
            let add_contextoid_output = GenerativeOutput::AddContextoidToContext {
                context_id: 999, // Non-existent context ID
                contextoid: bad_contextoid,
            };

            Ok(GenerativeOutput::Composite(vec![
                GenerativeOutput::CreateCausaloid(1000, causaloid),
                add_contextoid_output,
            ]))
        }
    }

    let model_result = TestModel::with_generator(
        6,
        "author",
        "desc",
        None,
        BadContextoidGenerator,
        &GenerativeTrigger::DataReceived(Data::new(60, MockData { id: 60, data: 30 })),
    );

    assert!(model_result.is_err());
    assert_eq!(
        model_result.unwrap_err(),
        ModelBuildError::ValidationFailed(ModelValidationError::TargetContextNotFound { id: 999 })
    );
}

#[test]
fn test_fails_on_multiple_root_causaloids() {
    struct MultiCausaloidGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            MultiCausaloidGenerator,
        > for MultiCausaloidGenerator
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
                MultiCausaloidGenerator,
            >,
            ModelGenerativeError,
        > {
            fn mock_fn(_: &NumericalValue) -> Result<bool, CausalityError> {
                Ok(false)
            }
            let causaloid1 = TestCausaloid::new(100, mock_fn, "First");
            let causaloid2 = TestCausaloid::new(200, mock_fn, "Second");

            Ok(GenerativeOutput::Composite(vec![
                GenerativeOutput::CreateCausaloid(100, causaloid1),
                GenerativeOutput::CreateCausaloid(200, causaloid2),
            ]))
        }
    }

    let model_result = TestModel::with_generator(
        7,
        "author",
        "desc",
        None,
        MultiCausaloidGenerator,
        &GenerativeTrigger::DataReceived(Data::new(70, MockData { id: 70, data: 35 })),
    );

    assert!(model_result.is_err());
    assert_eq!(
        model_result.unwrap_err(),
        ModelBuildError::ValidationFailed(ModelValidationError::DuplicateCausaloidID { id: 200 })
    );
}

#[test]
fn test_fails_on_update_nonexistent_causaloid() {
    struct UpdateOnlyGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            UpdateOnlyGenerator,
        > for UpdateOnlyGenerator
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
                UpdateOnlyGenerator,
            >,
            ModelGenerativeError,
        > {
            let updated_causaloid = TestCausaloid::new(123, |_| Ok(false), "Updated");
            Ok(GenerativeOutput::UpdateCausaloid(123, updated_causaloid))
        }
    }

    let model_result = TestModel::with_generator(
        8,
        "author",
        "desc",
        None,
        UpdateOnlyGenerator,
        &GenerativeTrigger::DataReceived(Data::new(80, MockData { id: 80, data: 40 })),
    );

    assert!(model_result.is_err());
    assert_eq!(
        model_result.unwrap_err(),
        ModelBuildError::ValidationFailed(ModelValidationError::TargetCausaloidNotFound {
            id: 123
        })
    );
}

#[test]
fn test_fails_on_delete_nonexistent_contextoid() {
    struct BadDeleteGenerator;
    impl
        Generatable<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            BadDeleteGenerator,
        > for BadDeleteGenerator
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
                BadDeleteGenerator,
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
            // Attempt to delete a contextoid that was never added
            let delete_nonexistent = GenerativeOutput::DeleteContextoidFromContext {
                context_id: 10,
                contextoid_id: 999,
            };

            Ok(GenerativeOutput::Composite(vec![
                create_causaloid,
                create_context,
                delete_nonexistent,
            ]))
        }
    }

    let model_result = TestModel::with_generator(
        1,
        "author",
        "desc",
        None,
        BadDeleteGenerator,
        &GenerativeTrigger::ManualIntervention("trigger".to_string()),
    );

    assert!(model_result.is_err());
    assert_eq!(
        model_result.unwrap_err(),
        ModelBuildError::ValidationFailed(ModelValidationError::TargetContextoidNotFound {
            id: 999
        })
    );
}
