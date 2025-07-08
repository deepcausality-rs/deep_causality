/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_generator::MockData;
use deep_causality::utils_test::test_utils_generator::*;
use deep_causality::*;

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
            let causaloid = TestCausaloid::new(
                1,
                |_| Ok(PropagatingEffect::Deterministic(false)),
                "Test Causaloid",
            );
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
                TestCausaloid::new(
                    1,
                    |_| Ok(PropagatingEffect::Deterministic(false)),
                    "Initial Causaloid",
                ),
            );

            let updated_causaloid = TestCausaloid::new(
                1,
                |_| Ok(PropagatingEffect::Deterministic(false)),
                "Updated Causaloid",
            );
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
                TestCausaloid::new(
                    1,
                    |_| Ok(PropagatingEffect::Deterministic(false)),
                    "Initial Causaloid",
                ),
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
