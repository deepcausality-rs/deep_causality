/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_generator::*;
use deep_causality::*;

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
    > = GenerativeOutput::UpdateCausaloid(
        99,
        TestCausaloid::new(99, |_| Ok(PropagatingEffect::Deterministic(false)), ""),
    );
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
    > = GenerativeOutput::CreateCausaloid(
        1,
        TestCausaloid::new(1, |_| Ok(PropagatingEffect::Deterministic(false)), ""),
    );
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
    > = GenerativeOutput::CreateCausaloid(
        1,
        TestCausaloid::new(1, |_| Ok(PropagatingEffect::Deterministic(false)), ""),
    );
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
