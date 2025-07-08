/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_generator::*;
use deep_causality::*;

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
