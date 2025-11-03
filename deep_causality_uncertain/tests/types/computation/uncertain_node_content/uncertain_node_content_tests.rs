/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;
use deep_causality_uncertain::{
    ArithmeticOperator, BernoulliParams, ComparisonOperator, DistributionEnum, LogicalOperator,
    NormalDistributionParams, SampledBindFn, SampledFmapFn, SampledValue, UncertainNodeContent,
};
use std::sync::Arc;

type UncertainNode = ConstTree<UncertainNodeContent>;

fn create_node(content: UncertainNodeContent) -> UncertainNode {
    ConstTree::new(content)
}

#[test]
fn test_sampled_fmap_fn() {
    let func = |input: SampledValue| match input {
        SampledValue::Float(v) => SampledValue::Float(v * 2.0),
        _ => panic!("unexpected type"),
    };
    let boxed_func: Arc<dyn SampledFmapFn> = Arc::new(func);

    let input = SampledValue::Float(10.0);
    let output = boxed_func.call(input);
    assert_eq!(output, SampledValue::Float(20.0));
}

#[test]
fn test_sampled_bind_fn() {
    let func = |input: SampledValue| {
        let inner_val = match input {
            SampledValue::Float(v) => v,
            _ => panic!("unexpected type"),
        };
        create_node(UncertainNodeContent::Value(SampledValue::Float(
            inner_val + 1.0,
        )))
    };
    let boxed_func: Arc<dyn SampledBindFn> = Arc::new(func);

    let input = SampledValue::Float(5.0);
    let output_node = boxed_func.call(input);
    assert_eq!(
        *output_node.value(),
        UncertainNodeContent::Value(SampledValue::Float(6.0))
    );
}

#[test]
fn test_uncertain_node_content_debug() {
    // Value
    let content = UncertainNodeContent::Value(SampledValue::Float(1.0));
    assert_eq!(format!("{:?}", content), "Value(Float(1.0))");

    // DistributionF64
    let dist_f64 = DistributionEnum::Normal(NormalDistributionParams::new(0.0, 1.0));
    let content = UncertainNodeContent::DistributionF64(dist_f64);
    assert_eq!(
        format!("{:?}", content),
        format!("DistributionF64({:?})", dist_f64)
    );

    // DistributionBool
    let dist_bool = DistributionEnum::Bernoulli(BernoulliParams::new(0.5));
    let content = UncertainNodeContent::DistributionBool(dist_bool);
    assert_eq!(
        format!("{:?}", content),
        format!("DistributionBool({:?})", dist_bool)
    );

    // PureOp
    let content = UncertainNodeContent::PureOp {
        value: SampledValue::Bool(true),
    };
    assert_eq!(format!("{:?}", content), "PureOp { value: Bool(true) }");

    // FmapOp
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let func = Arc::new(|v| v);
    let content = UncertainNodeContent::FmapOp {
        func,
        operand: operand.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!("FmapOp {{ func: Fn, operand: {:?} }}", operand)
    );

    // ApplyOp
    let arg = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let func = Arc::new(|v| v);
    let content = UncertainNodeContent::ApplyOp {
        func,
        arg: arg.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!("ApplyOp {{ func: Fn, arg: {:?} }}", arg)
    );

    // BindOp
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let func = Arc::new(|_| create_node(UncertainNodeContent::Value(SampledValue::Float(0.0))));
    let content = UncertainNodeContent::BindOp {
        func,
        operand: operand.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!("BindOp {{ func: Fn, operand: {:?} }}", operand)
    );

    // ArithmeticOp
    let lhs = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let rhs = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let content = UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!(
            "ArithmeticOp {{ op: {:?}, lhs: {:?}, rhs: {:?} }}",
            ArithmeticOperator::Add,
            lhs,
            rhs
        )
    );

    // ComparisonOp
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let content = UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 0.5,
        operand: operand.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!(
            "ComparisonOp {{ op: {:?}, threshold: {:?}, operand: {:?} }}",
            ComparisonOperator::EqualTo,
            0.5,
            operand
        )
    );

    // LogicalOp
    let operands = vec![create_node(UncertainNodeContent::Value(
        SampledValue::Bool(true),
    ))];
    let content = UncertainNodeContent::LogicalOp {
        op: LogicalOperator::Not,
        operands: operands.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!(
            "LogicalOp {{ op: {:?}, operands: {:?} }}",
            LogicalOperator::Not,
            operands
        )
    );

    // FunctionOpF64
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let func = Arc::new(|v: f64| v);
    let content = UncertainNodeContent::FunctionOpF64 {
        func,
        operand: operand.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!("FunctionOpF64 {{ func: Fn, operand: {:?} }}", operand)
    );

    // FunctionOpBool
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let func = Arc::new(|v: f64| v > 0.0);
    let content = UncertainNodeContent::FunctionOpBool {
        func,
        operand: operand.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!("FunctionOpBool {{ func: Fn, operand: {:?} }}", operand)
    );

    // NegationOp
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let content = UncertainNodeContent::NegationOp {
        operand: operand.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!("NegationOp {{ operand: {:?} }}", operand)
    );

    // ConditionalOp
    let condition = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let if_true = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let if_false = create_node(UncertainNodeContent::Value(SampledValue::Float(0.0)));
    let content = UncertainNodeContent::ConditionalOp {
        condition: condition.clone(),
        if_true: if_true.clone(),
        if_false: if_false.clone(),
    };
    assert_eq!(
        format!("{:?}", content),
        format!(
            "ConditionalOp {{ condition: {:?}, if_true: {:?}, if_false: {:?} }}",
            condition, if_true, if_false
        )
    );
}

#[test]
fn test_uncertain_node_content_partial_eq() {
    // Value
    let v1 = UncertainNodeContent::Value(SampledValue::Float(1.0));
    let v2 = UncertainNodeContent::Value(SampledValue::Float(1.0));
    let v3 = UncertainNodeContent::Value(SampledValue::Float(2.0));
    assert_eq!(v1, v2);
    assert_ne!(v1, v3);

    // DistributionF64
    let d1 = UncertainNodeContent::DistributionF64(DistributionEnum::Normal(
        NormalDistributionParams::new(0.0, 1.0),
    ));
    let d2 = UncertainNodeContent::DistributionF64(DistributionEnum::Normal(
        NormalDistributionParams::new(0.0, 1.0),
    ));
    let d3 = UncertainNodeContent::DistributionF64(DistributionEnum::Normal(
        NormalDistributionParams::new(1.0, 1.0),
    ));
    assert_eq!(d1, d2);
    assert_ne!(d1, d3);

    // DistributionBool
    let b1 = UncertainNodeContent::DistributionBool(DistributionEnum::Bernoulli(
        BernoulliParams::new(0.5),
    ));
    let b2 = UncertainNodeContent::DistributionBool(DistributionEnum::Bernoulli(
        BernoulliParams::new(0.5),
    ));
    let b3 = UncertainNodeContent::DistributionBool(DistributionEnum::Bernoulli(
        BernoulliParams::new(0.8),
    ));
    assert_eq!(b1, b2);
    assert_ne!(b1, b3);

    // PureOp
    let p1 = UncertainNodeContent::PureOp {
        value: SampledValue::Float(1.0),
    };
    let p2 = UncertainNodeContent::PureOp {
        value: SampledValue::Float(1.0),
    };
    let p3 = UncertainNodeContent::PureOp {
        value: SampledValue::Float(2.0),
    };
    assert_eq!(p1, p2);
    assert_ne!(p1, p3);

    // FmapOp (func is ignored)
    let operand1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let operand2 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let func_dummy1 = Arc::new(|v| v);
    let func_dummy2 = Arc::new(|v| v);
    let fmap1 = UncertainNodeContent::FmapOp {
        func: func_dummy1.clone(),
        operand: operand1.clone(),
    };
    let fmap2 = UncertainNodeContent::FmapOp {
        func: func_dummy2.clone(),
        operand: operand1.clone(),
    };
    let fmap3 = UncertainNodeContent::FmapOp {
        func: func_dummy1,
        operand: operand2.clone(),
    };
    assert_eq!(fmap1, fmap2);
    assert_ne!(fmap1, fmap3);

    // ApplyOp (func is ignored)
    let arg1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let arg2 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let func_dummy1 = Arc::new(|v| v);
    let func_dummy2 = Arc::new(|v| v);
    let apply1 = UncertainNodeContent::ApplyOp {
        func: func_dummy1.clone(),
        arg: arg1.clone(),
    };
    let apply2 = UncertainNodeContent::ApplyOp {
        func: func_dummy2.clone(),
        arg: arg1.clone(),
    };
    let apply3 = UncertainNodeContent::ApplyOp {
        func: func_dummy1,
        arg: arg2.clone(),
    };
    assert_eq!(apply1, apply2);
    assert_ne!(apply1, apply3);

    // BindOp (func is ignored)
    let operand1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let operand2 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let func_dummy1 =
        Arc::new(|_| create_node(UncertainNodeContent::Value(SampledValue::Float(0.0))));
    let func_dummy2 =
        Arc::new(|_| create_node(UncertainNodeContent::Value(SampledValue::Float(0.0))));
    let bind1 = UncertainNodeContent::BindOp {
        func: func_dummy1.clone(),
        operand: operand1.clone(),
    };
    let bind2 = UncertainNodeContent::BindOp {
        func: func_dummy2.clone(),
        operand: operand1.clone(),
    };
    let bind3 = UncertainNodeContent::BindOp {
        func: func_dummy1,
        operand: operand2.clone(),
    };
    assert_eq!(bind1, bind2);
    assert_ne!(bind1, bind3);

    // ArithmeticOp
    let lhs1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let rhs1 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let lhs2 = create_node(UncertainNodeContent::Value(SampledValue::Float(3.0)));
    let arith1 = UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs1.clone(),
        rhs: rhs1.clone(),
    };
    let arith2 = UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs1.clone(),
        rhs: rhs1.clone(),
    };
    let arith3 = UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Sub,
        lhs: lhs1.clone(),
        rhs: rhs1.clone(),
    };
    let arith4 = UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs2.clone(),
        rhs: rhs1.clone(),
    };
    assert_eq!(arith1, arith2);
    assert_ne!(arith1, arith3);
    assert_ne!(arith1, arith4);

    // ComparisonOp
    let operand1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let operand2 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let comp1 = UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 0.5,
        operand: operand1.clone(),
    };
    let comp2 = UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 0.5,
        operand: operand1.clone(),
    };
    let comp3 = UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::GreaterThan,
        threshold: 0.5,
        operand: operand1.clone(),
    };
    let comp4 = UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 0.8,
        operand: operand1.clone(),
    };
    let comp5 = UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 0.5,
        operand: operand2.clone(),
    };
    assert_eq!(comp1, comp2);
    assert_ne!(comp1, comp3);
    assert_ne!(comp1, comp4);
    assert_ne!(comp1, comp5);

    // LogicalOp
    let operands1 = vec![create_node(UncertainNodeContent::Value(
        SampledValue::Bool(true),
    ))];
    let operands2 = vec![create_node(UncertainNodeContent::Value(
        SampledValue::Bool(false),
    ))];
    let operands3 = vec![
        create_node(UncertainNodeContent::Value(SampledValue::Bool(true))),
        create_node(UncertainNodeContent::Value(SampledValue::Bool(false))),
    ];
    let logic1 = UncertainNodeContent::LogicalOp {
        op: LogicalOperator::Not,
        operands: operands1.clone(),
    };
    let logic2 = UncertainNodeContent::LogicalOp {
        op: LogicalOperator::Not,
        operands: operands1.clone(),
    };
    let logic3 = UncertainNodeContent::LogicalOp {
        op: LogicalOperator::And,
        operands: operands1.clone(),
    };
    let logic4 = UncertainNodeContent::LogicalOp {
        op: LogicalOperator::Not,
        operands: operands2.clone(),
    };
    let logic5 = UncertainNodeContent::LogicalOp {
        op: LogicalOperator::Not,
        operands: operands3.clone(),
    };
    assert_eq!(logic1, logic2);
    assert_ne!(logic1, logic3);
    assert_ne!(logic1, logic4);
    assert_ne!(logic1, logic5);

    // FunctionOpF64 (func is ignored)
    let operand1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let operand2 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let func_dummy1 = Arc::new(|v: f64| v);
    let func_dummy2 = Arc::new(|v: f64| v);
    let f64_1 = UncertainNodeContent::FunctionOpF64 {
        func: func_dummy1.clone(),
        operand: operand1.clone(),
    };
    let f64_2 = UncertainNodeContent::FunctionOpF64 {
        func: func_dummy2.clone(),
        operand: operand1.clone(),
    };
    let f64_3 = UncertainNodeContent::FunctionOpF64 {
        func: func_dummy1,
        operand: operand2.clone(),
    };
    assert_eq!(f64_1, f64_2);
    assert_ne!(f64_1, f64_3);

    // FunctionOpBool (func is ignored)
    let operand1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let operand2 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let func_dummy1 = Arc::new(|v: f64| v > 0.0);
    let func_dummy2 = Arc::new(|v: f64| v > 0.0);
    let bool_1 = UncertainNodeContent::FunctionOpBool {
        func: func_dummy1.clone(),
        operand: operand1.clone(),
    };
    let bool_2 = UncertainNodeContent::FunctionOpBool {
        func: func_dummy2.clone(),
        operand: operand1.clone(),
    };
    let bool_3 = UncertainNodeContent::FunctionOpBool {
        func: func_dummy1,
        operand: operand2.clone(),
    };
    assert_eq!(bool_1, bool_2);
    assert_ne!(bool_1, bool_3);

    // NegationOp
    let operand1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let operand2 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));
    let neg1 = UncertainNodeContent::NegationOp {
        operand: operand1.clone(),
    };
    let neg2 = UncertainNodeContent::NegationOp {
        operand: operand1.clone(),
    };
    let neg3 = UncertainNodeContent::NegationOp {
        operand: operand2.clone(),
    };
    assert_eq!(neg1, neg2);
    assert_ne!(neg1, neg3);

    // ConditionalOp
    let cond1 = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let if_true1 = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0)));
    let if_false1 = create_node(UncertainNodeContent::Value(SampledValue::Float(0.0)));
    let cond2 = create_node(UncertainNodeContent::Value(SampledValue::Bool(false)));
    let if_true2 = create_node(UncertainNodeContent::Value(SampledValue::Float(2.0)));

    let cond_op1 = UncertainNodeContent::ConditionalOp {
        condition: cond1.clone(),
        if_true: if_true1.clone(),
        if_false: if_false1.clone(),
    };
    let cond_op2 = UncertainNodeContent::ConditionalOp {
        condition: cond1.clone(),
        if_true: if_true1.clone(),
        if_false: if_false1.clone(),
    };
    let cond_op3 = UncertainNodeContent::ConditionalOp {
        condition: cond2.clone(),
        if_true: if_true1.clone(),
        if_false: if_false1.clone(),
    };
    let cond_op4 = UncertainNodeContent::ConditionalOp {
        condition: cond1.clone(),
        if_true: if_true2.clone(),
        if_false: if_false1.clone(),
    };
    assert_eq!(cond_op1, cond_op2);
    assert_ne!(cond_op1, cond_op3);
    assert_ne!(cond_op1, cond_op4);

    // Cross-variant inequality
    assert_ne!(v1, d1);
    assert_ne!(v1, fmap1);
    assert_ne!(arith1, comp1);
}
