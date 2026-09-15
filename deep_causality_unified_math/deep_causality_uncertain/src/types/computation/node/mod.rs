/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The computation graph's node, generic in the scalar.

use std::fmt::Debug;
use std::sync::Arc;

use deep_causality_ast::ConstTree;

use crate::{ArithmeticOperator, ComparisonOperator, DistributionEnum, LogicalOperator, Sample};

pub trait SampledFmapFn<R>: Send + Sync + 'static {
    fn call(&self, input: Sample<R>) -> Sample<R>;
}

impl<R, F> SampledFmapFn<R> for F
where
    F: Fn(Sample<R>) -> Sample<R> + Send + Sync + 'static,
{
    fn call(&self, input: Sample<R>) -> Sample<R> {
        self(input)
    }
}

pub trait SampledBindFn<R>: Send + Sync + 'static {
    fn call(&self, input: Sample<R>) -> ConstTree<Node<R>>;
}

impl<R, F> SampledBindFn<R> for F
where
    F: Fn(Sample<R>) -> ConstTree<Node<R>> + Send + Sync + 'static,
{
    fn call(&self, input: Sample<R>) -> ConstTree<Node<R>> {
        self(input)
    }
}

/// One node of a lazy computation graph over the scalar `R`.
///
/// # One distribution arm
///
/// The three per-precision distribution arms (`DistributionF64`, `DistributionF106`,
/// `DistributionBool`) are one `Distribution(DistributionEnum<R>)`. The Boolean case is the
/// Bernoulli variant of that same enum, which is where it always belonged: a Bernoulli is a
/// distribution parameterised by a probability, not by the type of the thing it produces.
///
/// Every node below the root carries `R`, including the Boolean ones — a comparison holds a
/// threshold of type `R` and a Bernoulli's parameter is stated at `R`. That is why the Boolean
/// carrier keeps the scalar: there is one tree, and it has one scalar.
///
/// # A mapped function is a pointer, not a trait object
///
/// The two `FunctionOp` arms hold `fn(R) -> R` and `fn(R) -> bool`: a plain function pointer, so
/// the graph stores no `dyn` and no `Arc`, and dispatch through it is static.
///
/// Holding the function as a **type parameter**, the way `deep_causality_calculus`'s `Euler` and
/// `Rk4` hold their rate field, is what a non-recursive arrow can do and this tree cannot: every
/// node of a `ConstTree<Node<R>>` has one type, so one `F` would have to serve every mapped node in
/// the graph — `x.map(f).map(g)` would not typecheck, and neither would `a.map(f) + b`. A pointer
/// is the largest thing a homogeneous node can hold without a trait object.
///
/// The cost is that a **capturing** closure cannot be mapped. A captured parameter belongs in the
/// graph rather than in a closure over it: `x.map(|v| v * k)` is `x * Uncertain::point(k)`, which
/// the sampler and the quasi-Monte-Carlo pre-pass can both see into, where a captured `k` is opaque
/// to them.
#[derive(Clone)]
pub enum Node<R> {
    // Leaf nodes
    Value(Sample<R>),
    Distribution(DistributionEnum<R>),

    // HKT Operations
    PureOp {
        value: Sample<R>,
    },
    FmapOp {
        func: Arc<dyn SampledFmapFn<R>>,
        operand: ConstTree<Node<R>>,
    },
    ApplyOp {
        func: Arc<dyn SampledFmapFn<R>>,
        arg: ConstTree<Node<R>>,
    },
    BindOp {
        func: Arc<dyn SampledBindFn<R>>,
        operand: ConstTree<Node<R>>,
    },

    // Graph operations
    ArithmeticOp {
        op: ArithmeticOperator,
        lhs: ConstTree<Node<R>>,
        rhs: ConstTree<Node<R>>,
    },
    ComparisonOp {
        op: ComparisonOperator,
        threshold: R,
        operand: ConstTree<Node<R>>,
    },
    LogicalOp {
        op: LogicalOperator,
        operands: Vec<ConstTree<Node<R>>>,
    },
    FunctionOpReal {
        func: fn(R) -> R,
        operand: ConstTree<Node<R>>,
    },
    FunctionOpBool {
        func: fn(R) -> bool,
        operand: ConstTree<Node<R>>,
    },
    NegationOp {
        operand: ConstTree<Node<R>>,
    },
    ConditionalOp {
        condition: ConstTree<Node<R>>,
        if_true: ConstTree<Node<R>>,
        if_false: ConstTree<Node<R>>,
    },
}

impl<R: Debug> Debug for Node<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Value(v) => write!(f, "Value({:?})", v),
            Node::Distribution(d) => write!(f, "Distribution({:?})", d),
            Node::PureOp { value } => write!(f, "PureOp {{ value: {:?} }}", value),
            Node::FmapOp { func: _, operand } => {
                write!(f, "FmapOp {{ func: Fn, operand: {:?} }}", operand)
            }
            Node::ApplyOp { func: _, arg } => write!(f, "ApplyOp {{ func: Fn, arg: {:?} }}", arg),
            Node::BindOp { func: _, operand } => {
                write!(f, "BindOp {{ func: Fn, operand: {:?} }}", operand)
            }
            Node::ArithmeticOp { op, lhs, rhs } => write!(
                f,
                "ArithmeticOp {{ op: {:?}, lhs: {:?}, rhs: {:?} }}",
                op, lhs, rhs
            ),
            Node::ComparisonOp {
                op,
                threshold,
                operand,
            } => write!(
                f,
                "ComparisonOp {{ op: {:?}, threshold: {:?}, operand: {:?} }}",
                op, threshold, operand
            ),
            Node::LogicalOp { op, operands } => {
                write!(f, "LogicalOp {{ op: {:?}, operands: {:?} }}", op, operands)
            }
            Node::FunctionOpReal { func: _, operand } => {
                write!(f, "FunctionOpReal {{ func: Fn, operand: {:?} }}", operand)
            }
            Node::FunctionOpBool { func: _, operand } => {
                write!(f, "FunctionOpBool {{ func: Fn, operand: {:?} }}", operand)
            }
            Node::NegationOp { operand } => write!(f, "NegationOp {{ operand: {:?} }}", operand),
            Node::ConditionalOp {
                condition,
                if_true,
                if_false,
            } => write!(
                f,
                "ConditionalOp {{ condition: {:?}, if_true: {:?}, if_false: {:?} }}",
                condition, if_true, if_false
            ),
        }
    }
}

impl<R: PartialEq> PartialEq for Node<R> {
    /// Structural equality. A stored function cannot be compared, so the arms carrying one
    /// compare their operands and the rest of their shape — which is what a reader asking whether
    /// two graphs have the same form is asking.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Value(v1), Node::Value(v2)) => v1 == v2,
            (Node::Distribution(d1), Node::Distribution(d2)) => d1 == d2,
            (Node::PureOp { value: v1 }, Node::PureOp { value: v2 }) => v1 == v2,
            (Node::FmapOp { operand: o1, .. }, Node::FmapOp { operand: o2, .. }) => o1 == o2,
            (Node::ApplyOp { arg: a1, .. }, Node::ApplyOp { arg: a2, .. }) => a1 == a2,
            (Node::BindOp { operand: o1, .. }, Node::BindOp { operand: o2, .. }) => o1 == o2,
            (
                Node::ArithmeticOp {
                    op: op1,
                    lhs: l1,
                    rhs: r1,
                },
                Node::ArithmeticOp {
                    op: op2,
                    lhs: l2,
                    rhs: r2,
                },
            ) => op1 == op2 && l1 == l2 && r1 == r2,
            (
                Node::ComparisonOp {
                    op: op1,
                    threshold: t1,
                    operand: o1,
                },
                Node::ComparisonOp {
                    op: op2,
                    threshold: t2,
                    operand: o2,
                },
            ) => op1 == op2 && t1 == t2 && o1 == o2,
            (
                Node::LogicalOp {
                    op: op1,
                    operands: ops1,
                },
                Node::LogicalOp {
                    op: op2,
                    operands: ops2,
                },
            ) => op1 == op2 && ops1 == ops2,
            (
                Node::FunctionOpReal { operand: o1, .. },
                Node::FunctionOpReal { operand: o2, .. },
            ) => o1 == o2,
            (
                Node::FunctionOpBool { operand: o1, .. },
                Node::FunctionOpBool { operand: o2, .. },
            ) => o1 == o2,
            (Node::NegationOp { operand: o1 }, Node::NegationOp { operand: o2 }) => o1 == o2,
            (
                Node::ConditionalOp {
                    condition: c1,
                    if_true: t1,
                    if_false: f1,
                },
                Node::ConditionalOp {
                    condition: c2,
                    if_true: t2,
                    if_false: f2,
                },
            ) => c1 == c2 && t1 == t2 && f1 == f2,
            _ => false,
        }
    }
}
