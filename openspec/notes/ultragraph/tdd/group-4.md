<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Group 4: Measurement and close-out

Change `ultragraph-stable-ids-keyed-edges`, design D8. The five phases of
`openspec/specs/unified-math-tdd-protocol/spec.md`, recorded as they ran.

## Measurement

Group 4 adds no code; it measures groups 1 to 3 against the baseline and closes the change out, so
the five phases and the A–K table do not apply. The three after-runs and their aggregation are
`../BenchAfterChange1StableId`, in the baseline's layout, with every row
compared with `BenchBefore/BenchBefore.md` and every move beyond the noise floor explained by a
decision of the design (D4's keyed scan and backward list, the one-iterator traversal, the end of
tombstone filtering). Of the 54 shared rows, 24 are within spread, 25 slower and 5 faster; the six
shuffled walks the baseline listed as pending have their first three runs. No move in a frozen read
or an algorithm is unexplained, which was D9's test for a defect.

## Close-out

`make format && make fix`, `make build`, `bazel test //...` and `make check_examples` were run by
Marvin on the committed groups. The `context-store-crate-identity` spec's Purpose line still says
"Tier-0, dependency-free"; the delta of this change replaces the requirement, and the Purpose line
is updated when the change is archived (task 4.4).
