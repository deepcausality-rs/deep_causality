/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError, ProjectionErrorEnum};
use deep_causality_quantum::WaveVector;

#[test]
fn test_both_directions_round_trip_as_text() {
    for (direction, text) in [(WaveVector::Up, "up"), (WaveVector::Down, "down")] {
        let record = direction.to_record();
        assert_eq!(record, DataRecord::Text(text.to_string()));
        assert_eq!(WaveVector::from_record(5, record), Ok(direction));
    }
}

#[test]
fn test_other_text_is_rejected_and_another_kind_is_the_wrong_payload() {
    match WaveVector::from_record(5, DataRecord::Text("sideways".to_string())) {
        Err(ProjectionError(ProjectionErrorEnum::Rejected { id, rule })) => {
            assert_eq!(id, 5);
            assert!(rule.contains("sideways"), "{rule}");
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
    assert_eq!(
        WaveVector::from_record(5, DataRecord::Flag(true)),
        Err(ProjectionError::WrongPayload(5, "Text", "Flag"))
    );
}
