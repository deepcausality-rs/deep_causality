/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::types::telos_types::tag_index::*;
use deep_causality::{TeloidID, TeloidTag};

#[test]
fn test_new_and_default() {
    let tag_index_new = TagIndex::new();
    assert_eq!(tag_index_new.len(), 0);
    assert!(tag_index_new.get("1").is_none());

    let tag_index_default = TagIndex::default();
    assert_eq!(tag_index_default.len(), 0);
    assert_eq!(tag_index_new, tag_index_default);
}

#[test]
fn test_with_capacity() {
    let tag_index = TagIndex::with_capacity(10);
    assert_eq!(tag_index.len(), 0);
}

#[test]
fn test_add_and_get() {
    let mut tag_index = TagIndex::new();
    let tag1: TeloidTag = "tag1";
    let id1: TeloidID = 101;

    // 1. Add a single item
    tag_index.add(tag1, id1);
    assert_eq!(tag_index.len(), 1);
    assert_eq!(tag_index.get(tag1), Some(&vec![id1]));

    // 2. Add another item to the same tag
    let id2: TeloidID = 102;
    tag_index.add(tag1, id2);
    assert_eq!(tag_index.len(), 1);
    assert_eq!(tag_index.get(tag1), Some(&vec![id1, id2]));

    // 3. Add a new tag
    let tag2: TeloidTag = "tag2";
    let id3: TeloidID = 103;
    tag_index.add(tag2, id3);
    assert_eq!(tag_index.len(), 2);
    assert_eq!(tag_index.get(tag2), Some(&vec![id3]));

    // 4. Get a non-existent tag (corner case)
    let tag_non_existent: TeloidTag = "tag99";
    assert!(tag_index.get(tag_non_existent).is_none());
}

#[test]
fn test_add_duplicate_id_edge_case() {
    let mut tag_index = TagIndex::new();
    let tag: TeloidTag = "tag1";
    let id: TeloidID = 101;

    tag_index.add(tag, id);
    tag_index.add(tag, id); // Add the exact same ID again

    assert_eq!(tag_index.len(), 1);
    assert_eq!(tag_index.get(tag), Some(&vec![id, id]));
}

#[test]
fn test_remove() {
    let mut tag_index = TagIndex::new();
    let tag1: TeloidTag = "tag1";
    let id1: TeloidID = 101;
    let id2: TeloidID = 102;
    tag_index.add(tag1, id1);
    tag_index.add(tag1, id2);

    // 1. Remove an existing ID
    tag_index.remove(tag1, id1);
    assert_eq!(tag_index.get(tag1), Some(&vec![id2]));

    // 2. Remove a non-existent ID from an existing tag (corner case)
    tag_index.remove(tag1, 999);
    assert_eq!(tag_index.get(tag1), Some(&vec![id2]));

    // 3. Remove the last ID for a tag
    tag_index.remove(tag1, id2);
    // The key should still exist with an empty vec, as per implementation
    assert!(tag_index.contains_key(tag1));
    assert_eq!(tag_index.get(tag1), Some(&vec![]));

    // 4. Remove an ID from a non-existent tag (corner case)
    let tag_non_existent: TeloidTag = "tag99";
    tag_index.remove(tag_non_existent, 999);
    assert_eq!(tag_index.len(), 1); // Length should be unchanged
}

#[test]
fn test_update() {
    let mut tag_index = TagIndex::new();
    let tag1: TeloidTag = "tag1";
    let id1: TeloidID = 101;

    // 1. Try to update a non-existent tag (edge case)
    tag_index.update(tag1, id1);
    assert!(!tag_index.contains_key(tag1));
    assert_eq!(tag_index.len(), 0);

    // 2. Add the tag first, then update
    tag_index.add(tag1, id1);
    assert_eq!(tag_index.get(tag1), Some(&vec![id1]));

    let id2: TeloidID = 102;
    tag_index.update(tag1, id2);
    assert_eq!(tag_index.len(), 1);
    assert_eq!(tag_index.get(tag1), Some(&vec![id1, id2]));
}

#[test]
fn test_check() {
    let mut tag_index = TagIndex::new();
    let tag1: TeloidTag = "tag1";
    let id1: TeloidID = 101;

    // 1. Check non-existent tag
    assert!(!tag_index.contains_key(tag1));

    // 2. Check after adding
    tag_index.add(tag1, id1);
    assert!(tag_index.contains_key(tag1));

    // 3. Check after removing the only ID (key should still exist)
    tag_index.remove(tag1, id1);
    assert!(tag_index.contains_key(tag1));
}

#[test]
fn test_clear() {
    let mut tag_index = TagIndex::new();
    tag_index.add("tag1", 101);
    tag_index.add("tag2", 102);

    assert_eq!(tag_index.len(), 2);

    // 1. Clear the index
    tag_index.clear();
    assert_eq!(tag_index.len(), 0);
    assert!(!tag_index.contains_key("tag1"));

    // 2. Clear an already empty index (corner case)
    tag_index.clear();
    assert_eq!(tag_index.len(), 0);
}

#[test]
fn test_len() {
    let mut tag_index = TagIndex::new();
    assert_eq!(tag_index.len(), 0);

    tag_index.add("tag1", 101);
    assert_eq!(tag_index.len(), 1);

    tag_index.add("tag2", 102);
    assert_eq!(tag_index.len(), 2);

    // Adding to an existing tag doesn't change len
    tag_index.add("tag1", 102);
    assert_eq!(tag_index.len(), 2);

    // Removing an ID doesn't change len
    tag_index.remove("tag1", 101);
    assert_eq!(tag_index.len(), 2);

    // Clearing changes len to 0
    tag_index.clear();
    assert_eq!(tag_index.len(), 0);
    assert!(tag_index.is_empty());
}
#[test]
fn test_display() {
    let mut tag_index = TagIndex::new();

    // 1. Test empty display
    assert_eq!(format!("{}", tag_index), "TagIndex { size: 0 }");

    // 2. Test display with one tag
    tag_index.add("tag1", 101);
    assert_eq!(format!("{}", tag_index), "TagIndex { size: 1 }");

    // 3. Test display with a second tag
    tag_index.add("tag2", 201);
    assert_eq!(format!("{}", tag_index), "TagIndex { size: 2 }");
}

#[test]
fn test_is_empty() {
    let mut tag_index = TagIndex::new();
    assert!(tag_index.is_empty());

    tag_index.add("tag1", 101);
    assert!(!tag_index.is_empty());

    tag_index.clear();
    assert!(tag_index.is_empty());
}
