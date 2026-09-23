/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the variants; no expectation is computed. The
//! behaviour of each write is tested through `commit` in `tests/utils_test/memory_storage/`.
//!
//! Corner cases (rows A to K): A empty payloads in `test_five_variants_all_distinct`; C the same
//! container reference in `Link` and `Attach`, same test; every other row n/a.

use deep_causality_context_store::{
    ContainerRef, ContextWrite, ContextoidRecord, NodeRecord, RelationKind, RelationRecord,
};

#[test]
fn test_five_variants_all_distinct() {
    let at = ContainerRef::Created(0);
    let all = vec![
        ContextWrite::CreateContext(String::new()),
        ContextWrite::CreateNode(vec![ContextoidRecord::new(1, NodeRecord::Root)]),
        ContextWrite::CreateEdge(vec![RelationRecord::new(1, 2, RelationKind::Datial)]),
        ContextWrite::Link {
            context: at,
            nodes: vec![],
        },
        ContextWrite::Attach {
            context: at,
            extra: at,
        },
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            assert_eq!(
                a == b,
                i == j,
                "variant {i} {a:?} against variant {j} {b:?}"
            );
        }
    }
    assert_eq!(all.clone(), all);
    assert!(matches!(
        &all[3],
        ContextWrite::Link {
            context: ContainerRef::Created(0),
            nodes,
        } if nodes.is_empty()
    ));
}
