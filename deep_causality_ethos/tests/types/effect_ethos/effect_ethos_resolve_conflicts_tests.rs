/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_ethos::utils_test::test_utils_effect_ethos::{
    TestEthos, always_false_predicate, always_true_predicate, get_dummy_action, get_dummy_context,
};
use deep_causality_ethos::{DeonticInferable, TeloidModal, Verdict};

/// A norm as (id, tags, active, modality, timestamp, specificity, priority).
type Norm = (
    u64,
    &'static [&'static str],
    bool,
    TeloidModal,
    u64,
    u32,
    u32,
);

fn ethos(norms: &[Norm], defeats: &[(u64, u64)], inherits: &[(u64, u64)]) -> TestEthos {
    let mut ethos = TestEthos::new();
    for &(id, tags, active, modality, timestamp, specificity, priority) in norms {
        let predicate = if active {
            always_true_predicate
        } else {
            always_false_predicate
        };
        ethos = ethos
            .add_deterministic_norm(
                id,
                "drive",
                tags,
                predicate,
                modality,
                timestamp,
                specificity,
                priority,
            )
            .unwrap();
    }
    for &(defeater, defeated) in defeats {
        ethos = ethos.link_defeasance(defeater, defeated).unwrap();
    }
    for &(parent, child) in inherits {
        ethos = ethos.link_inheritance(parent, child).unwrap();
    }
    ethos.verify_graph().unwrap();
    ethos
}

fn verdict(ethos: &TestEthos) -> Verdict {
    ethos
        .evaluate_action(
            &get_dummy_action("drive", 40.0),
            &get_dummy_context(),
            &["drive"],
        )
        .unwrap()
}

fn sorted(verdict: &Verdict) -> Vec<u64> {
    let mut ids = verdict.justification().clone();
    ids.sort();
    ids
}

#[test]
fn test_a_defeated_norm_cannot_defeat() {
    // 1 defeats 2 and 2 defeats 3. Norm 1 stands, so norm 2 falls; norm 3's only defeater has
    // fallen, so norm 3 stands.
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Optional(1), 1, 1, 3),
            (2, &["drive"], true, TeloidModal::Optional(1), 1, 1, 2),
            (3, &["drive"], true, TeloidModal::Impermissible, 1, 1, 1),
        ],
        &[(1, 2), (2, 3)],
        &[],
    );
    let v = verdict(&ethos);
    assert_eq!(v.outcome(), TeloidModal::Impermissible);
    assert_eq!(sorted(&v), vec![1, 3]);
}

#[test]
fn test_a_chain_of_defeats_gives_the_same_verdict_every_time() {
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Optional(1), 1, 1, 3),
            (2, &["drive"], true, TeloidModal::Optional(1), 1, 1, 2),
            (3, &["drive"], true, TeloidModal::Impermissible, 1, 1, 1),
        ],
        &[(1, 2), (2, 3)],
        &[],
    );
    let first = verdict(&ethos);
    for _ in 0..500 {
        assert_eq!(verdict(&ethos), first);
    }
}

#[test]
fn test_the_justification_lists_survivors_in_one_order() {
    // Five independent norms: the justification must not depend on hash order.
    let norms: Vec<Norm> = (1..=5)
        .map(|id| (id, &["drive"][..], true, TeloidModal::Optional(1), 1, 1, 1))
        .collect();
    let ethos = ethos(&norms, &[], &[]);
    let first = verdict(&ethos).justification().clone();
    for _ in 0..200 {
        assert_eq!(verdict(&ethos).justification(), &first);
    }
}

#[test]
fn test_a_newer_norm_does_not_defeat_a_higher_priority_one() {
    // Norm 2 is newer and has the edge, but norm 1 outranks it: priority decides first.
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Impermissible, 1, 10, 10),
            (2, &["drive"], true, TeloidModal::Optional(1), 100, 1, 1),
        ],
        &[(2, 1)],
        &[],
    );
    let v = verdict(&ethos);
    assert_eq!(v.outcome(), TeloidModal::Impermissible);
    assert_eq!(sorted(&v), vec![1, 2]);
}

#[test]
fn test_a_more_specific_norm_does_not_defeat_a_higher_priority_one() {
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Impermissible, 1, 1, 10),
            (2, &["drive"], true, TeloidModal::Optional(1), 1, 100, 1),
        ],
        &[(2, 1)],
        &[],
    );
    assert_eq!(verdict(&ethos).outcome(), TeloidModal::Impermissible);
}

#[test]
fn test_at_equal_priority_specificity_decides_before_time() {
    // Equal priority: the more specific but older defeater wins.
    let older_but_specific = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Impermissible, 100, 1, 5),
            (2, &["drive"], true, TeloidModal::Optional(1), 1, 10, 5),
        ],
        &[(2, 1)],
        &[],
    );
    assert_eq!(sorted(&verdict(&older_but_specific)), vec![2]);

    // Equal priority: the newer but less specific defeater loses.
    let newer_but_general = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Impermissible, 1, 10, 5),
            (2, &["drive"], true, TeloidModal::Optional(1), 100, 1, 5),
        ],
        &[(2, 1)],
        &[],
    );
    assert_eq!(sorted(&verdict(&newer_but_general)), vec![1, 2]);
}

#[test]
fn test_at_equal_priority_and_specificity_the_newer_norm_defeats() {
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Impermissible, 1, 5, 5),
            (2, &["drive"], true, TeloidModal::Optional(1), 2, 5, 5),
        ],
        &[(2, 1)],
        &[],
    );
    assert_eq!(sorted(&verdict(&ethos)), vec![2]);
}

#[test]
fn test_equal_rank_does_not_defeat() {
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Impermissible, 5, 5, 5),
            (2, &["drive"], true, TeloidModal::Optional(1), 5, 5, 5),
        ],
        &[(2, 1)],
        &[],
    );
    assert_eq!(sorted(&verdict(&ethos)), vec![1, 2]);
}

#[test]
fn test_an_inactive_norm_does_not_defeat() {
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Impermissible, 1, 1, 1),
            (2, &["drive"], false, TeloidModal::Optional(1), 1, 1, 10),
        ],
        &[(2, 1)],
        &[],
    );
    let v = verdict(&ethos);
    assert_eq!(v.outcome(), TeloidModal::Impermissible);
    assert_eq!(sorted(&v), vec![1]);
}

#[test]
fn test_a_defeated_norm_passes_nothing_on_by_inheritance() {
    // 3 defeats 1; 1's child 2 is reachable only through 1, so it does not join.
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Optional(1), 1, 1, 1),
            (2, &[], true, TeloidModal::Impermissible, 1, 1, 1),
            (3, &["drive"], true, TeloidModal::Optional(2), 1, 1, 10),
        ],
        &[(3, 1)],
        &[(1, 2)],
    );
    let v = verdict(&ethos);
    assert_eq!(v.outcome(), TeloidModal::Optional(2));
    assert_eq!(sorted(&v), vec![3]);
}

#[test]
fn test_an_inherited_norm_can_defeat() {
    // 1 passes 2 on by inheritance; 2 outranks and defeats the active norm 3.
    let ethos = ethos(
        &[
            (1, &["drive"], true, TeloidModal::Optional(1), 1, 1, 1),
            (2, &[], true, TeloidModal::Optional(1), 1, 1, 10),
            (3, &["drive"], true, TeloidModal::Impermissible, 1, 1, 5),
        ],
        &[(2, 3)],
        &[(1, 2)],
    );
    let v = verdict(&ethos);
    assert_eq!(v.outcome(), TeloidModal::Optional(2));
    assert_eq!(sorted(&v), vec![1, 2]);
}

#[test]
fn test_an_inherited_norm_missing_from_the_store_is_reported() {
    use deep_causality_ethos::{
        DeonticError, TagIndex, Teloid, TeloidGraph, TeloidStorable, TeloidStore, Teloidable,
    };

    // Norm 1 is stored and tagged; the graph also holds node 2 as its child, which the store
    // lacks.
    let mut store = TeloidStore::new();
    store.insert(Teloid::new_deterministic(
        1,
        "drive".to_string(),
        always_true_predicate,
        TeloidModal::Optional(1),
        1,
        1,
        1,
        vec!["drive"],
        None,
    ));
    let mut index = TagIndex::new();
    index.add("drive", 1);
    let mut graph = TeloidGraph::new();
    let parent = graph.add_teloid(1).unwrap();
    let child = graph.add_teloid(2).unwrap();
    graph.add_inheritance_edge(parent, child).unwrap();

    let mut ethos = TestEthos::from(store, index, graph);
    ethos.verify_graph().unwrap();
    let result = ethos.evaluate_action(
        &get_dummy_action("drive", 40.0),
        &get_dummy_context(),
        &["drive"],
    );
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::TeloidNotFound { id: 2 }
    ));
}
