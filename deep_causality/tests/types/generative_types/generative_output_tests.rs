use deep_causality::utils_test::test_utils_generator::MockData;
use deep_causality::*;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum MyCustomGenerativeAction {
    Spawn(u32),
}

impl
    Generatable<
        bool, // I
        bool, // O
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
            bool, // I
            bool, // O
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

type TestGenerativeOutput = GenerativeOutput<
    bool, // I
    bool, // O
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
    MyCustomGenerativeAction,
>;

type TestCausaloid = Causaloid<
    bool, // I
    bool, // O
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

fn test_causal_fn(value: bool) -> Result<CausalFnOutput<bool>, CausalityError> {
    Ok(CausalFnOutput::new(value, CausalEffectLog::default()))
}

pub fn get_test_causaloid() -> TestCausaloid {
    let id: IdentificationValue = 1;
    let description = "A simple causal function that returns its boolean input.";

    TestCausaloid::new(id, test_causal_fn, description)
}

#[test]
fn test_noop() {
    let output = TestGenerativeOutput::NoOp;
    assert_eq!(output, TestGenerativeOutput::NoOp);
}

#[test]
fn test_create_causaloid() {
    let causaloid_id = 123;
    let causaloid_instance = get_test_causaloid();
    let output = TestGenerativeOutput::CreateCausaloid(causaloid_id, causaloid_instance.clone());

    match output {
        TestGenerativeOutput::CreateCausaloid(id, caus) => {
            assert_eq!(id, causaloid_id);
            assert_eq!(caus, causaloid_instance);
        }
        _ => panic!("Expected CreateCausaloid variant"),
    }
}

#[test]
fn test_delete_causaloid() {
    let causaloid_id = 456;
    let output = TestGenerativeOutput::DeleteCausaloid(causaloid_id);

    match output {
        TestGenerativeOutput::DeleteCausaloid(id) => {
            assert_eq!(id, causaloid_id);
        }
        _ => panic!("Expected DeleteCausaloid variant"),
    }
}

#[test]
fn test_create_base_context() {
    let context_id = 1;
    let name = "Test Context".to_string();
    let capacity = 100;
    let output = TestGenerativeOutput::CreateBaseContext {
        id: context_id,
        name: name.clone(),
        capacity,
    };

    match output {
        TestGenerativeOutput::CreateBaseContext {
            id,
            name: out_name,
            capacity: out_capacity,
        } => {
            assert_eq!(id, context_id);
            assert_eq!(out_name, name);
            assert_eq!(out_capacity, capacity);
        }
        _ => panic!("Expected CreateBaseContext variant"),
    }
}

#[test]
fn test_add_contextoid_to_context() {
    let context_id = 789;
    let contextoid_instance = TestContextoid::new(101, ContextoidType::Root(Root::new(101)));
    let output = TestGenerativeOutput::AddContextoidToContext {
        context_id,
        contextoid: contextoid_instance.clone(),
    };

    match output {
        TestGenerativeOutput::AddContextoidToContext {
            context_id: ctx_id,
            contextoid: coid,
        } => {
            assert_eq!(ctx_id, context_id);
            assert_eq!(coid, contextoid_instance);
        }
        _ => panic!("Expected AddContextoidToContext variant"),
    }
}

#[test]
fn test_composite_output() {
    let inner_output1 = TestGenerativeOutput::NoOp;
    let inner_output2 = TestGenerativeOutput::DeleteCausaloid(99);
    let output =
        TestGenerativeOutput::Composite(vec![inner_output1.clone(), inner_output2.clone()]);

    match output {
        TestGenerativeOutput::Composite(actions) => {
            assert_eq!(actions.len(), 2);
            assert_eq!(actions[0], inner_output1);
            assert_eq!(actions[1], inner_output2);
        }
        _ => panic!("Expected Composite variant"),
    }
}

#[test]
fn test_evolve_variant() {
    let custom_action = MyCustomGenerativeAction::Spawn(5);
    let output = TestGenerativeOutput::Evolve(custom_action);

    match output {
        TestGenerativeOutput::Evolve(action) => {
            assert_eq!(action, custom_action);
            assert_eq!(action, MyCustomGenerativeAction::Spawn(5));
        }
        _ => panic!("Expected Evolve variant"),
    }
}

#[test]
fn test_update_context_with_name() {
    let context_id = 10;
    let new_name = "Updated Name".to_string();
    let output = TestGenerativeOutput::UpdateContext {
        id: context_id,
        new_name: Some(new_name.clone()),
    };

    if let TestGenerativeOutput::UpdateContext {
        id,
        new_name: Some(name),
    } = output
    {
        assert_eq!(id, context_id);
        assert_eq!(name, new_name);
    } else {
        panic!("Expected UpdateContext with Some(new_name)");
    }
}

#[test]
fn test_update_context_without_name() {
    let context_id = 11;
    let output = TestGenerativeOutput::UpdateContext {
        id: context_id,
        new_name: None,
    };

    if let TestGenerativeOutput::UpdateContext { id, new_name: None } = output {
        assert_eq!(id, context_id);
    } else {
        panic!("Expected UpdateContext with None for new_name");
    }
}

#[test]
fn test_delete_context() {
    let context_id = 20;
    let output = TestGenerativeOutput::DeleteContext { id: context_id };

    if let TestGenerativeOutput::DeleteContext { id } = output {
        assert_eq!(id, context_id);
    } else {
        panic!("Expected DeleteContext");
    }
}

#[test]
fn test_create_extra_context() {
    let extra_context_id = 30;
    let capacity = 50;
    let output = TestGenerativeOutput::CreateExtraContext {
        extra_context_id,
        capacity,
    };

    if let TestGenerativeOutput::CreateExtraContext {
        extra_context_id: id,
        capacity: cap,
    } = output
    {
        assert_eq!(id, extra_context_id);
        assert_eq!(cap, capacity);
    } else {
        panic!("Expected CreateExtraContext");
    }
}

#[test]
fn test_update_contextoid_in_context() {
    let context_id = 40;
    let existing_contextoid_id = 41;
    let new_contextoid_instance = TestContextoid::new(101, ContextoidType::Root(Root::new(101)));
    let output = TestGenerativeOutput::UpdateContextoidInContext {
        context_id,
        existing_contextoid: existing_contextoid_id,
        new_contextoid: new_contextoid_instance.clone(),
    };

    if let TestGenerativeOutput::UpdateContextoidInContext {
        context_id: ctx_id,
        existing_contextoid: existing_coid_id,
        new_contextoid: new_coid,
    } = output
    {
        assert_eq!(ctx_id, context_id);
        assert_eq!(existing_coid_id, existing_contextoid_id);
        assert_eq!(new_coid, new_contextoid_instance);
    } else {
        panic!("Expected UpdateContextoidInContext");
    }
}

#[test]
fn test_delete_contextoid_from_context() {
    let context_id = 50;
    let contextoid_id = 51;
    let output = TestGenerativeOutput::DeleteContextoidFromContext {
        context_id,
        contextoid_id,
    };

    if let TestGenerativeOutput::DeleteContextoidFromContext {
        context_id: ctx_id,
        contextoid_id: coid_id,
    } = output
    {
        assert_eq!(ctx_id, context_id);
        assert_eq!(coid_id, contextoid_id);
    } else {
        panic!("Expected DeleteContextoidFromContext");
    }
}
