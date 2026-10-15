## MODIFIED Requirements

### Requirement: Algorithms on the graph run on the simple projection
Every algorithm, read and traversal called on a frozen graph directly SHALL return what it returns on the simple projection: a view whose node and edge tests pass everything, under the graph's `ParallelEdges` mode. That mode is `Merge` for every graph built by `new` or `with_capacity`, is the view's mode for a copy `materialize` returns, and is kept by every transition and by `freeze_clone`.

#### Scenario: Direct calls equal the simple projection
- **WHEN** random frozen graphs with parallel edges of several keys are generated and every algorithm, read and traversal runs directly and on the simple projection
- **THEN** each pair of results is equal

#### Scenario: A materialised copy answers under its view's mode
- **WHEN** a view under `Distinct` is materialised, and the copy is unfrozen and frozen again
- **THEN** every algorithm on the copy returns the result it returns on the copy's simple projection under `Distinct`
