/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The computation graph's node, generic in the scalar.

use std::fmt::Debug;

use deep_causality_ast::ConstTree;

use crate::{ArithmeticOperator, ComparisonOperator, DistributionEnum, LogicalOperator, Sample};

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
/// # Nothing here is a trait object
///
/// The two `FunctionOp` arms hold `fn(R) -> R` and `fn(R) -> bool`: a plain function pointer, so
/// the graph stores no `dyn` and no `Arc`, and dispatch through it is static. That matters beyond
/// tidiness — a trait object's default lifetime reaches the type it is parameterised by, so a
/// single `Arc<dyn …<R>>` anywhere in this enum would put `R: 'static` on the crate's whole public
/// surface. There is none, and the scalar bound is therefore exactly the algebra: `RandScalar`.
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
///
/// # No higher-kinded arms
///
/// `PureOp`, `FmapOp`, `ApplyOp` and `BindOp` are gone. No carrier method built one, so no public
/// API could put a graph into a state carrying them; what they did do was store an
/// `Arc<dyn …<R>>`, which is where a `'static` on the scalar came from. A stored function belongs
/// in the Arrow layer, over a composite whose type records what it holds — not in a node of a
/// homogeneous tree, where it can only be erased.
#[derive(Clone)]
pub enum Node<R> {
    // Leaf nodes
    Value(Sample<R>),
    Distribution(DistributionEnum<R>),

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
    /// Structural equality. A function pointer is not compared, so the two arms carrying one
    /// compare their operands and the rest of their shape — which is what a reader asking whether
    /// two graphs have the same form is asking.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Value(v1), Node::Value(v2)) => v1 == v2,
            (Node::Distribution(d1), Node::Distribution(d2)) => d1 == d2,
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
