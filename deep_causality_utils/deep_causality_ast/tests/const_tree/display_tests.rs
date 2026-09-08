/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;
use std::fmt::{self, Write};

#[test]
fn test_display() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let display_str = format!("{}", tree);
    let expected_display = "1\n    2\n";
    assert_eq!(display_str, expected_display);
}

#[test]
fn test_display_indents_four_spaces_per_level_in_sibling_order() {
    // The single depth-2 fixture observes the indent at levels 0 and 1 only, so it cannot tell
    // `indent * 4` from a flat "indented or not", and with one child it cannot see sibling
    // order either. Three levels and two siblings pin both.
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );
    assert_eq!(format!("{tree}"), "1\n    2\n        3\n    4\n");

    // A leaf is its value and a newline, with no leading space.
    assert_eq!(format!("{}", ConstTree::new(9)), "9\n");
}

/// A sink that accepts `remaining` writes and then fails, so the `?` operators in the recursive
/// formatter can be reached. `format!` never can: a `String` sink cannot fail.
struct FailAfter {
    remaining: usize,
}

impl Write for FailAfter {
    fn write_str(&mut self, _s: &str) -> fmt::Result {
        if self.remaining == 0 {
            return Err(fmt::Error);
        }
        self.remaining -= 1;
        Ok(())
    }
}

#[test]
fn test_display_propagates_a_formatter_error() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );

    // Calibrate against the real write count rather than guessing at it.
    let mut sink = FailAfter {
        remaining: usize::MAX,
    };
    write!(sink, "{tree}").expect("an unlimited sink accepts the whole tree");
    let total = usize::MAX - sink.remaining;
    assert!(total > 1, "the tree renders in more than one write");

    // Failing at any point before the last write must surface as an error, whether it happens
    // on the root's own line or inside the recursive call for a child.
    for remaining in 0..total {
        let mut sink = FailAfter { remaining };
        assert!(
            write!(sink, "{tree}").is_err(),
            "a sink failing after {remaining} of {total} writes reported success"
        );
    }
}
