/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::prelude::*;
use deep_causality::utils_test::test_utils::get_test_assumption_vec;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, Hash, Copy, PartialEq, Default)]
pub struct MockData {
    id: u64,
    data: u8,
}

impl Identifiable for MockData {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Datable for MockData {
    type Data = u8;

    fn get_data(&self) -> Self::Data {
        self.data
    }

    fn set_data(&mut self, value: Self::Data) {
        self.data = value
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum MyCustomGenerativeAction {}

impl
Generatable<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
    MyCustomGenerativeAction,
> for MyCustomGenerativeAction
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
            MyCustomGenerativeAction,
        >,
        ModelGenerativeError,
    > {
        unimplemented!()
    }
}

type TestCausaloid = Causaloid<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

type TestContextoid = Contextoid<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

type TestModel = Model<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

// Mock Generatable implementation for testing Model::with_generator
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct MockGenerator {
    // You could add fields here to control what it generates
    causaloid_id_to_create: CausaloidId,
    context_id_to_create: ContextId,
    return_context: bool,
}

// Corrected Generatable impl for MockGenerator (G is now MockGenerator itself)
impl
Generatable<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
    MockGenerator,
> for MockGenerator
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
            // Return type's G is MockGenerator
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            MockGenerator,
        >,
        ModelGenerativeError,
    > {
        let id = self.causaloid_id_to_create;
        // Define a simple mock causal_fn
        fn mock_causal_fn(_obs: &NumericalValue) -> Result<bool, CausalityError> {
            Ok(false)
        }

        let description = "TestCausaloid";

        // Corrected: create TestCausaloid with all required arguments
        let causaloid = TestCausaloid::new(id, mock_causal_fn, description);
        let create_causaloid_output =
            GenerativeOutput::CreateCausaloid(self.causaloid_id_to_create, causaloid);

        if self.return_context {
            let create_context_output = GenerativeOutput::CreateBaseContext {
                id: self.context_id_to_create,
                name: "Generated Context".to_string(),
                capacity: 10,
            };
            Ok(GenerativeOutput::Composite(vec![
                create_causaloid_output,
                create_context_output,
            ]))
        } else {
            Ok(create_causaloid_output)
        }
    }
}

#[test]
fn test_model_with_generator_creates_causaloid_and_context() {
    let model_id = 1;
    let author = "Test Author";
    let description = "Test Model generated with Causaloid and Context";
    let assumptions: Option<Arc<Vec<Assumption>>> = None;

    let trigger = GenerativeTrigger::DataReceived(Data::new(1, MockData { id: 10, data: 5 }));

    let mock_generator = MockGenerator {
        causaloid_id_to_create: 100,
        context_id_to_create: 200,
        return_context: true,
    };

    let model_result = TestModel::with_generator(
        model_id,
        author,
        description,
        assumptions.clone(),
        mock_generator,
        &trigger,
    );

    assert!(
        model_result.is_ok(),
        "Model generation should succeed: {model_result:?}"
    );
    let model = model_result.unwrap();

    assert_eq!(model.id(), model_id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_none());

    // Verify Causaloid
    let causaloid = model.causaloid();
    assert_eq!(causaloid.id(), 100); // Check causaloid ID
    assert!(causaloid.has_context()); // Check if has_context is true (tuple field 8)

    // Verify Context
    let context = model.context();
    assert!(context.is_some());
    let ctx = context.as_ref().unwrap();
    assert_eq!(ctx.id(), 200); // Check context ID
    assert_eq!(ctx.name(), "Generated Context");
}

#[test]
fn test_model_with_generator_creates_causaloid_without_context() {
    let model_id = 2;
    let author = "Another Author";
    let description = "Test Model generated with only Causaloid";
    let assumptions = None;
    let trigger = GenerativeTrigger::DataReceived(Data::new(20, MockData { id: 10, data: 5 }));

    let mock_generator = MockGenerator {
        causaloid_id_to_create: 300,
        context_id_to_create: 0, // Not used
        return_context: false,
    };

    let model_result = TestModel::with_generator(
        model_id,
        author,
        description,
        assumptions.clone(),
        mock_generator,
        &trigger,
    );

    assert!(
        model_result.is_ok(),
        "Model generation should succeed: {model_result:?}"
    );
    let model = model_result.unwrap();

    assert_eq!(model.id(), model_id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_none());

    // Verify Causaloid
    let causaloid = model.causaloid();
    assert_eq!(causaloid.id(), 300); // Check causaloid ID
    assert!(!causaloid.has_context()); // Check if has_context is false

    // Verify Context
    assert!(model.context().is_none());
}

#[test]
fn test_model_with_generator_with_assumptions() {
    let model_id = 5;
    let author = "Assumptions Author";
    let description = "Test Model generated with explicit assumptions";
    // Providing explicit assumptions
    let assumptions: Option<Arc<Vec<Assumption>>> = Some(Arc::new(get_test_assumption_vec()));
    let trigger = GenerativeTrigger::DataReceived(Data::new(50, MockData { id: 50, data: 25 }));

    let mock_generator = MockGenerator {
        causaloid_id_to_create: 400,
        context_id_to_create: 500,
        return_context: true,
    };

    let model_result = TestModel::with_generator(
        model_id,
        author,
        description,
        assumptions.clone(), // Clone assumptions for the call
        mock_generator,
        &trigger,
    );

    assert!(
        model_result.is_ok(),
        "Model generation with assumptions should succeed: {model_result:?}"
    );
    let model = model_result.unwrap();

    assert_eq!(model.id(), model_id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);

    // Verify assumptions
    assert!(model.assumptions().is_some());
    let actual_assumptions = model.assumptions().as_ref().unwrap();
    assert_eq!(actual_assumptions.len(), 3);

    // Verify Causaloid (basic check to ensure it was still created)
    let causaloid = model.causaloid();
    assert_eq!(causaloid.id(), 400);

    // Verify Context (basic check to ensure it was still created)
    let context = model.context();
    assert!(context.is_some());
    assert_eq!(context.as_ref().unwrap().id(), 500);
}

#[test]
fn test_model_with_generator_fails_on_add_contextoid_to_nonexistent_context() {
    let model_id = 6;
    let author = "Bad Generator Author";
    let description =
        "Test Model generation with generator adding contextoid to non-existent context";
    let assumptions: Option<Arc<Vec<Assumption>>> = None;
    let trigger = GenerativeTrigger::DataReceived(Data::new(60, MockData { id: 60, data: 30 }));

    // A mock generator that attempts to add a Contextoid without creating its parent Context first.
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
        BadContextoidGenerator, // G is BadContextoidGenerator
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
            // First, create a Causaloid (required by Model::with_generator)
            let causaloid = TestCausaloid::new(1000, |_| Ok(false), "TestCausaloidForBadCtx");

            // Then, attempt to add a contextoid to a context ID (e.g., 999) that has NOT been created.
            let bad_contextoid = TestContextoid::new(1001, ContextoidType::Root(Root::new(101))); // Contextoid to add
            let add_contextoid_output = GenerativeOutput::AddContextoidToContext {
                context_id: 999, // Non-existent context ID
                contextoid: bad_contextoid,
            };

            Ok(GenerativeOutput::Composite(vec![
                GenerativeOutput::CreateCausaloid(1000, causaloid), // Create the causaloid first
                add_contextoid_output,
            ]))
        }
    }

    let model_result = TestModel::with_generator(
        model_id,
        author,
        description,
        assumptions,
        BadContextoidGenerator,
        &trigger,
    );

    assert!(
        model_result.is_err(),
        "Model generation should fail when adding contextoid to non-existent context"
    );
    assert_eq!(
        model_result.unwrap_err(),
        ModelBuildError::ValidationFailed(ModelValidationError::TargetContextNotFound { id: 999 })
    );
}

#[test]
fn test_model_with_generator_with_multiple_root_causaloids_fail() {
    let model_id = 7;
    let author = "Multi-Causaloid Author";
    let description = "Test Model generation with multiple causaloid creation outputs";
    let assumptions: Option<Arc<Vec<Assumption>>> = None;
    let trigger = GenerativeTrigger::DataReceived(Data::new(70, MockData { id: 70, data: 35 }));

    // A mock generator that creates multiple causaloids
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
        MultiCausaloidGenerator, // G is MultiCausaloidGenerator
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
            fn mock_causal_fn(_obs: &NumericalValue) -> Result<bool, CausalityError> {
                Ok(false)
            }

            // This will fail because a model can only have one single root causaloid.
            let causaloid1 = TestCausaloid::new(100, mock_causal_fn, "FirstCausaloid");
            let causaloid2 = TestCausaloid::new(200, mock_causal_fn, "SecondCausaloid");

            Ok(GenerativeOutput::Composite(vec![
                GenerativeOutput::CreateCausaloid(100, causaloid1),
                GenerativeOutput::CreateCausaloid(200, causaloid2),
            ]))
        }
    }

    let mock_generator = MultiCausaloidGenerator;

    let model_result = TestModel::with_generator(
        model_id,
        author,
        description,
        assumptions,
        mock_generator,
        &trigger,
    );

    assert!(
        model_result.is_err(),
        "Model generation with multiple CreateCausaloid outputs should fail"
    );
    assert_eq!(
        model_result.unwrap_err(),
        ModelBuildError::ValidationFailed(ModelValidationError::DuplicateCausaloidID { id: 200 }) // Expect the error for the second causaloid's ID
    );
}

#[test]
fn test_model_with_generator_fails_on_update_causaloid_without_prior_creation() {
    let model_id = 8;
    let author = "Update No Create Author";
    let description = "Test Model generation with update causaloid without prior create";
    let assumptions: Option<Arc<Vec<Assumption>>> = None;
    let trigger = GenerativeTrigger::DataReceived(Data::new(80, MockData { id: 80, data: 40 }));

    // A mock generator that returns UpdateCausaloid without any CreateCausaloid first.
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
        UpdateOnlyGenerator, // G is UpdateOnlyGenerator
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
            fn mock_causal_fn(_obs: &NumericalValue) -> Result<bool, CausalityError> {
                Ok(false)
            }
            let updated_causaloid = TestCausaloid::new(123, mock_causal_fn, "UpdatedCausaloid");

            Ok(GenerativeOutput::UpdateCausaloid(123, updated_causaloid))
        }
    }

    let model_result = TestModel::with_generator(
        model_id,
        author,
        description,
        assumptions,
        UpdateOnlyGenerator,
        &trigger,
    );

    assert!(
        model_result.is_err(),
        "Model generation should fail when attempting to update a causaloid that hasn't been created."
    );
    assert_eq!(
        model_result.unwrap_err(),
        ModelBuildError::ValidationFailed(ModelValidationError::TargetCausaloidNotFound {
            id: 123
        })
    );
}

// ===================================================================
// NEW TESTS FOR UPDATE AND DELETE FUNCTIONALITY
// ===================================================================

#[test]
fn test_model_with_generator_updates_context_name() {
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
fn test_model_with_generator_deletes_context() {
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

#[test]
fn test_model_with_generator_updates_contextoid() {
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
fn test_model_with_generator_deletes_contextoid() {
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
    assert_eq!(context.node_count(), 1);
}

#[test]
fn test_model_with_generator_fails_on_delete_nonexistent_contextoid() {
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
