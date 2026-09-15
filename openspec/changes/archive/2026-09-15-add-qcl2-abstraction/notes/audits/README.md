<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Named-defect audit scripts

Each script applies one defect at a time to the sources named in it, runs the Bazel test targets
named in it, records `caught` or `SURVIVED`, and restores the file byte for byte (a copy of each
original is kept in the directory passed as the first argument). Run from the repository root:

```bash
python3 openspec/changes/archive/2026-09-15-add-qcl2-abstraction/notes/audits/audit_g6.py /tmp/audit_originals
```

`audit_g5.py`, `audit_g6.py` and `audit_g7.py` are the scripts whose tables the group 5, 6 and 7
notes report, re-created from the session that ran them where the working copy had been trimmed for
a rerun. The group 1 and group 3 audits (`tdd-group-1.md`, `tdd-groups-2-3.md`) were run by
scripts that were not retained; their tables stand as recorded and cannot be replayed from this
directory. A pattern the script cannot find any more, because the source has since been formatted
or refactored, is reported as `PATTERN NOT FOUND` rather than applied.
