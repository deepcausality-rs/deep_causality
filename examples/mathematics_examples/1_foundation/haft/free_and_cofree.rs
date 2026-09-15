/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `Free` and `Cofree`: a program, and the structure it runs over
//!
//! Two witnesses sit opposite each other, both built on top of any functor `F`:
//!
//! ```text
//! FreeWitness<F>     Free<F, A>     HKT, Pure          a program, collapsed by an algebra
//! CofreeWitness<F>   Cofree<F, A>   HKT, Functor,      a labelled structure, grown by a
//!                                   CoMonad            coalgebra and queried at every node
//! ```
//!
//! `Free<F, A>` holds a computation as data: a leaf carries a value and a node carries an
//! `F`-structure of sub-programs. Meaning arrives later, from an interpreter: `fold` takes a
//! `pure` case for the leaves and an algebra `F<X> → X` for the nodes, and collapses the whole
//! tree to one `X`. One program, as many meanings as there are interpreters.
//!
//! `Cofree<F, A>` runs the other way. `unfold` takes a coalgebra `X → (A, F<X>)` and grows a
//! structure from a seed, and `extend` hands a closure the whole structure focused at one node, so
//! a node can be relabelled from everything hanging below it.
//!
//! ```text
//! Free::fold      algebra     F<X> → X        collapses a structure into a value
//! Cofree::unfold  coalgebra   X → (A, F<X>)   grows a structure out of a value
//! ```
//!
//! Both run here over `VecWitness`, so an `F`-structure of children is a `Vec` and both objects
//! are rose trees. The setting is a project work-breakdown structure: the `Cofree` side carries
//! the hours booked at each task, and the `Free` side is the plan of work as data.

use deep_causality_haft::{CoMonad, Cofree, CofreeWitness, Free, Functor, VecWitness};
use deep_causality_num::{lift, lower};

/// The work-breakdown structure: a name, the hours booked directly on the task, and its subtasks.
const TASKS: [(&str, f64, &[usize]); 7] = [
    ("Release 2.0", 0.0, &[1, 4]),
    ("Backend", 2.0, &[2, 3]),
    ("Schema migration", 12.0, &[]),
    ("API endpoints", 20.0, &[]),
    ("Frontend", 1.0, &[5, 6]),
    ("Dashboard", 16.0, &[]),
    ("Settings page", 8.0, &[]),
];
const ROOT: usize = 0;

/// The blended rate the cost map in section 2 charges, per hour.
const HOURLY_RATE: f64 = 95.0;

/// The working scalar. Hours and costs carry it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. Cofree: grow the tree from a seed with a coalgebra.
    // ---------------------------------------------------------------------
    // The coalgebra answers one question per node: what is my label, and what are my children's
    // seeds? `unfold` calls it until the seeds run out.
    let hours = Cofree::<VecWitness, FloatType>::unfold(ROOT, &|node: usize| {
        let (_, own_hours, children) = TASKS[node];
        (lift::<FloatType>(own_hours), children.to_vec())
    });

    print_tree("hours booked directly", &hours, 0);

    // ---------------------------------------------------------------------
    // 2. Functor: relabel every node, tree shape untouched.
    // ---------------------------------------------------------------------
    let rate = lift::<FloatType>(HOURLY_RATE);
    let cost = CofreeWitness::<VecWitness>::fmap(hours.clone(), move |h| h * rate);
    print_tree("cost at the blended rate", &cost, 0);

    // ---------------------------------------------------------------------
    // 3. CoMonad: relabel every node from what hangs below it.
    // ---------------------------------------------------------------------
    // `extend` gives the closure the sub-tree rooted at each node, so the roll-up is written once
    // and applies at every level. `extract` reads the label back off the root.
    let rolled_up = CofreeWitness::<VecWitness>::extend(&hours, subtree_total);
    print_tree("hours rolled up", &rolled_up, 0);

    let project_total = CofreeWitness::<VecWitness>::extract(&rolled_up);
    let own_hours = CofreeWitness::<VecWitness>::extract(&hours);
    print_roll_up(own_hours, project_total);

    // The root's roll-up is every task's own hours added together.
    let booked = TASKS
        .iter()
        .fold(lift::<FloatType>(0.0), |acc, &(_, h, _)| {
            acc + lift::<FloatType>(h)
        });
    assert_eq!(project_total, booked);

    // ---------------------------------------------------------------------
    // 4. Free: the plan as data.
    // ---------------------------------------------------------------------
    // Nothing runs while the program is built. A leaf is a task name and a node is the `Vec` of
    // sub-plans underneath it.
    let plan = plan_from(ROOT);

    // `bind` substitutes at the leaves: every task becomes two steps. This is the free monad's
    // whole job, and it still runs nothing.
    let staged = plan.bind(&|task: &'static str| {
        Free::<VecWitness, String>::Suspend(vec![
            Box::new(Free::pure(format!("estimate {task}"))),
            Box::new(Free::pure(format!("build {task}"))),
        ])
    });

    // ---------------------------------------------------------------------
    // 5. Two interpreters over one program.
    // ---------------------------------------------------------------------
    // `fold` takes the leaf case and the algebra. Changing the pair changes what the program
    // means, and the program itself is untouched.
    let steps: usize = staged.fold(&|_: String| 1usize, &|counts: Vec<usize>| {
        counts.into_iter().sum()
    });
    let outline: String =
        plan_from(ROOT).fold(&|task: &'static str| task.to_string(), &|parts: Vec<
            String,
        >| {
            format!("({})", parts.join(" + "))
        });

    print_interpreters(steps, &outline);

    // Two steps per leaf task, and there are four leaves.
    let leaves = TASKS.iter().filter(|(_, _, kids)| kids.is_empty()).count();
    assert_eq!(steps, 2 * leaves);

    print_footer();
    Ok(())
}

/// The plan for one task and everything under it, as a `Free` program over `Vec`.
fn plan_from(node: usize) -> Free<VecWitness, &'static str> {
    let (name, _, children) = TASKS[node];

    match children.is_empty() {
        true => Free::pure(name),
        false => Free::Suspend(children.iter().map(|&c| Box::new(plan_from(c))).collect()),
    }
}

/// Every hour booked at this node and below it.
fn subtree_total(node: &Cofree<VecWitness, FloatType>) -> FloatType {
    node.tail()
        .iter()
        .fold(*node.head(), |acc, child| acc + subtree_total(child))
}

/// The task names in the order `unfold` and `extend` visit them, for the printed tree.
fn task_name(node: usize) -> &'static str {
    TASKS[node].0
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== `Free` and `Cofree`: a program, and the structure it runs over ===\n");
    println!("  Both witnesses are built over `VecWitness`, so both objects are rose trees.");
    println!("  A project work-breakdown structure supplies the shape.\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_tree(title: &str, tree: &Cofree<VecWitness, FloatType>, depth: usize) {
    if depth == 0 {
        println!("--- {title} ---");
    }
    print_node(tree, ROOT, depth);
    if depth == 0 {
        println!();
    }
}

fn print_node(node: &Cofree<VecWitness, FloatType>, index: usize, depth: usize) {
    let indent = "  ".repeat(depth + 1);
    let name = task_name(index);
    println!("{indent}{name:<20} {:8.2}", lower(*node.head()));

    let children = TASKS[index].2;
    for (child, &child_index) in node.tail().iter().zip(children.iter()) {
        print_node(child, child_index, depth + 1);
    }
}

fn print_roll_up(own: FloatType, total: FloatType) {
    println!("--- extract at the root ---");
    println!("  booked on the root task itself   {:8.2} h", lower(own));
    println!("  rolled up across the project     {:8.2} h", lower(total));
    println!("  One closure ran at every node, and each saw the sub-tree below it.");
}

fn print_interpreters(steps: usize, outline: &str) {
    println!("\n--- 5. Two interpreters over one program ---");
    println!("  counting interpreter   {steps} steps after `bind` staged each task");
    println!("  rendering interpreter  {outline}");
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  `Free` holds a computation as data and waits for an interpreter, so the plan");
    println!("  above yields a step count and an outline from the same value. `Cofree` holds a");
    println!("  structure and answers questions at every position, so one roll-up closure");
    println!("  labels the whole tree. An algebra collapses; a coalgebra grows.");
}
