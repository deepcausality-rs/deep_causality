# Bazel


In addition to Cargo and related tools, the entire mono-repo is configured to build and test with Bazel. 
Please [install bazelisk ](https://github.com/bazelbuild/bazelisk)as it is the only requirement to build the repo with Bazel.

To build all targets with Bazel, run:

```bash 
    bazel build //...
```

To build only a specific target and its dependencies, run:

```bash 
    bazel build //alias:deep_causality
```

To test all targets with Bazel, run:

```bash 
    bazel test //...
```

To test only a specific target, run:

```bash 
    bazel test //deep_causality/...
```

To query all available tests to find, for example, all spacetime  tests, run:

```bash 
    bazel query "kind('rust_test', //...)" | grep spacetime
```

Furthermore, Bazel tests can be tagged and queried as to run only specific tests with
a specific tag. For example, to test only tests tagged "reasoning_types_tests" in all targets within 
the deep_causality crate, run: 

```bash 
bazel test //deep_causality/... --test_tag_filters=reasoning_types_tests
```

To see all the individual tests that a test_suite expands to, run:

```bash 
   bazel query 'tests(//deep_causality/tests:ctx_space_time_types_tests)'
```

Bazel queries the targets, filters the tests, and only runs those tests containing the tags.
Tests targets and tags are defined in the Build.bazel file in the test folder of each crate. 


To explore all dependencies of a specific crate, run:

```bash 
    bazel query "deps(/dcl_data_structures)"
```

To find all reverse dependencies, i.e. packages that depends on a specific crate, run:

```bash 
    bazel query "rdeps(//..., //dcl_data_structures, 1)"
```

If you were to refactor the dcl_data_structures crate, the rdepds tell you
upfront were its used and thus helps you to estimate upfront the blast radius of braking changes.

To query available vendored external dependencies with Bazel, run:

```bash 
    bazel query "kind('rust_library', //thirdparty/...)"
```

Note, these vendored external dependencies are shared across all crates.

To visualize all dependencies of the top level crate deep_causality, run

```bash 
bazel query "rdeps(//..., //deep_causality_haft, 1)"  --output graph --noimplicit_deps  | dot -Tpng -o graph.png 
   
   open graph.png # Works on Mac. 
```

## References:

https://bazel.build/query/guide

https://buildkite.com/resources/blog/a-guide-to-bazel-query/