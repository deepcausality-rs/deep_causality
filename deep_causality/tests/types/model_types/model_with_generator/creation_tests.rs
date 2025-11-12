/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils::get_test_assumption_vec;
use deep_causality::utils_test::test_utils_generator::*;
use deep_causality::*;
use std::sync::Arc;

// A generator specifically for creation tests.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct CreationGenerator {
    causaloid_id_to_create: CausaloidId,
    context_id_to_create: ContextId,
    return_context: bool,
}

fn mock_causal_fn(value: bool) -> Result<CausalFnOutput<bool>, CausalityError> {
    Ok(CausalFnOutput::new(value, CausalEffectLog::default()))
}

impl
    Generatable<
        bool,
        bool,
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        CreationGenerator,
    > for CreationGenerator
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
            bool,
            bool,
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            CreationGenerator,
        >,
        ModelGenerativeError,
    > {
        let id = self.causaloid_id_to_create;
        let causaloid = TestCausaloid::<bool, bool>::new(id, mock_causal_fn, "TestCausaloid");
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
fn test_creates_causaloid_and_context() {
    let model_id = 1;
    let author = "Test Author";
    let description = "Test Model generated with Causaloid and Context";

    let trigger = GenerativeTrigger::DataReceived(Data::new(1, MockData { id: 10, data: 5 }));

    let mock_generator = CreationGenerator {
        causaloid_id_to_create: 100,
        context_id_to_create: 200,
        return_context: true,
    };

    let model_result = TestModel::<bool, bool>::with_generator(
        model_id,
        author,
        description,
        None,
        mock_generator,
        &trigger,
    );

    assert!(
        model_result.is_ok(),
        "Model generation should succeed: {model_result:?}"
    );
    let model = model_result.unwrap();

    assert_eq!(model.id(), model_id);
    let causaloid = model.causaloid();
    assert_eq!(causaloid.id(), 100);

    let context = model.context();
    assert!(context.is_some());
    let ctx = context.as_ref().unwrap().read().unwrap();
    assert_eq!(ctx.id(), 200);
    assert_eq!(ctx.name(), "Generated Context");
}

#[test]
fn test_creates_causaloid_without_context() {
    let mock_generator = CreationGenerator {
        causaloid_id_to_create: 300,
        context_id_to_create: 0, // Not used
        return_context: false,
    };

    let model_result = TestModel::<bool, bool>::with_generator(
        2,
        "author",
        "desc",
        None,
        mock_generator,
        &GenerativeTrigger::DataReceived(Data::new(20, MockData { id: 10, data: 5 })),
    );

    assert!(
        model_result.is_ok(),
        "Model generation should succeed: {model_result:?}"
    );
    let model = model_result.unwrap();

    let causaloid = model.causaloid();
    assert_eq!(causaloid.id(), 300);
    assert!(model.context().is_none());
}

#[test]
fn test_creation_with_assumptions() {
    let assumptions = Some(Arc::new(get_test_assumption_vec()));
    let mock_generator = CreationGenerator {
        causaloid_id_to_create: 400,
        context_id_to_create: 500,
        return_context: true,
    };

    let model_result = TestModel::<bool, bool>::with_generator(
        5,
        "author",
        "desc",
        assumptions.clone(),
        mock_generator,
        &GenerativeTrigger::DataReceived(Data::new(50, MockData { id: 50, data: 25 })),
    );

    assert!(model_result.is_ok());
    let model = model_result.unwrap();

    assert!(model.assumptions().is_some());
    assert_eq!(model.assumptions().as_ref().unwrap().len(), 3);
    assert_eq!(model.causaloid().id(), 400);
}
