/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ethos::{TagIndex, TeloidID, TeloidTag};
use std::collections::HashSet;

#[test]
fn test_new_and_default() {
    let tag_index_new = TagIndex::new();
    assert!(tag_index_new.is_empty());
    assert!(tag_index_new.get("1").is_none());

    let tag_index_default = TagIndex::default();
    assert!(tag_index_default.is_empty());
    assert_eq!(tag_index_new, tag_index_default);
}

#[test]
fn test_with_capacity() {
    let tag_index = TagIndex::with_capacity(10);
    assert!(tag_index.is_empty());
}

#[test]
fn test_add_and_get() {
    let mut tag_index = TagIndex::new();
    let tag1: TeloidTag = "tag1";
    let id1: TeloidID = 101;

    // 1. Add a single item
    tag_index.add(tag1, id1);
    assert_eq!(tag_index.len(), 1);
    assert_eq!(tag_index.get(tag1), Some(&HashSet::from([id1])));

    // 2. Add another item to the same tag
    let id2: TeloidID = 102;
    tag_index.add(tag1, id2);
    assert_eq!(tag_index.len(), 1);
    assert_eq!(tag_index.get(tag1), Some(&HashSet::from([id1, id2])));

    // 3. Add a new tag
    let tag2: TeloidTag = "tag2";
    let id3: TeloidID = 103;
    tag_index.add(tag2, id3);
    assert_eq!(tag_index.len(), 2);
    assert_eq!(tag_index.get(tag2), Some(&HashSet::from([id3])));

    // 4. Get a non-existent tag (corner case)
    let tag_non_existent: TeloidTag = "tag99";
    assert!(tag_index.get(tag_non_existent).is_none());
}

#[test]
fn test_add_duplicate_id_is_ignored() {
    let mut tag_index = TagIndex::new();
    let tag: TeloidTag = "tag1";
    let id: TeloidID = 101;

    tag_index.add(tag, id);
    tag_index.add(tag, id); // Add the exact same ID again

    assert_eq!(tag_index.len(), 1);
    assert_eq!(tag_index.get(tag), Some(&HashSet::from([id])));
    assert_eq!(
        tag_index.get(tag).unwrap().len(),
        1,
        "HashSet should contain only one unique ID"
    );
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
    assert_eq!(tag_index.get(tag1), Some(&HashSet::from([id2])));

    // 2. Remove a non-existent ID from an existing tag (corner case)
    tag_index.remove(tag1, 999);
    assert_eq!(tag_index.get(tag1), Some(&HashSet::from([id2])));

    // 3. Remove the last ID for a tag, which should remove the tag itself
    tag_index.remove(tag1, id2);
    assert!(!tag_index.contains_key(tag1));
    assert!(tag_index.get(tag1).is_none());

    // 4. Remove an ID from a non-existent tag (corner case)
    let tag_non_existent: TeloidTag = "tag99";
    tag_index.remove(tag_non_existent, 999);
    assert!(tag_index.is_empty()); // Length should be 0
}

#[test]
fn test_update() {
    let mut tag_index = TagIndex::new();
    let tag1: TeloidTag = "tag1";
    let id1: TeloidID = 101;

    // 1. Try to update a non-existent tag (edge case)
    tag_index.update(tag1, id1);
    assert!(!tag_index.contains_key(tag1));
    assert!(tag_index.is_empty());

    // 2. Add the tag first, then update
    tag_index.add(tag1, id1);
    assert_eq!(tag_index.get(tag1), Some(&HashSet::from([id1])));

    let id2: TeloidID = 102;
    tag_index.update(tag1, id2);
    assert_eq!(tag_index.len(), 1);
    assert_eq!(tag_index.get(tag1), Some(&HashSet::from([id1, id2])));
}

#[test]
fn test_contains_key() {
    let mut tag_index = TagIndex::new();
    let tag1: TeloidTag = "tag1";
    let id1: TeloidID = 101;

    // 1. Check non-existent tag
    assert!(!tag_index.contains_key(tag1));

    // 2. Check after adding
    tag_index.add(tag1, id1);
    assert!(tag_index.contains_key(tag1));

    // 3. Check after removing the only ID (key should be removed)
    tag_index.remove(tag1, id1);
    assert!(!tag_index.contains_key(tag1));
}

#[test]
fn test_clear() {
    let mut tag_index = TagIndex::new();
    tag_index.add("tag1", 101);
    tag_index.add("tag2", 102);

    assert_eq!(tag_index.len(), 2);

    // 1. Clear the index
    tag_index.clear();
    assert!(tag_index.is_empty());
    assert!(!tag_index.contains_key("tag1"));

    // 2. Clear an already empty index (corner case)
    tag_index.clear();
    assert!(tag_index.is_empty());
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

    // Removing an ID from a tag with multiple IDs doesn't change len
    tag_index.remove("tag1", 101);
    assert_eq!(tag_index.len(), 2);

    // Removing the last ID from a tag *does* change len
    tag_index.remove("tag1", 102);
    assert_eq!(tag_index.len(), 1);

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
