---
SPDX-License-Identifier: MIT
---

# Checklist for hosting DeepCausality with LF AI & Data

## Basic preparations in the GH org and repo


:white_check_mark: Enable two-factor authentication for all members of the project’s GitHub org.

:white_check_mark: Install the GitHub DCO app on all repos.

:white_check_mark: [Achieve the basic OpenSSF badge](https://bestpractices.coreinfrastructure.org/en/projects/7568)

:white_check_mark: Have the following files in GitHub:

* [LICENSE.md](../LICENSE)
* [README.md](../README.md)
* [CONTRIBUTING.md](../CONTRIBUTING.md)
* [CODEOWNERS](../CODEOWNERS)
* [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)
* [RELEASE.md](../RELEASE.md)
* [SUPPORT.md](../SUPPORT.md)
* [SECURITY.md ](../SECURITY.md)

## Github Community Standards

:white_check_mark: Description

:white_check_mark: README

:white_check_mark: Code of conduct

:white_check_mark: Contributing

:white_check_mark: License

:white_check_mark: Security policy

:white_check_mark: Issue templates

:white_check_mark: Pull request template

:white_check_mark:  Repository admins accept content reports

## Github Security 

:white_check_mark: Verified & approved domains: deepcausality.com

:white_check_mark: Branch protection rule: Commit requires signature 

:white_check_mark: Security policy • Enabled

:white_check_mark: Security advisories • Enabled

:white_check_mark: Private vulnerability reporting • Enabled

:white_check_mark: Dependabot alerts • Enabled

:white_check_mark: Code scanning alerts • Enabled

:white_check_mark: Secret scanning alerts • Enabled

## OSS LICENSING

:white_check_mark: Project license exists and is OSI-approved:
* Project license: [The MIT LICENSE.md](../LICENSE)
* OSI-approved: [The MIT License](https://opensource.org/license/mit/)

:white_check_mark: Determine known components/dependencies are under a compatible license
* OSI-approved: [Apache License, Version 2.0](https://opensource.org/license/apache-2-0/)
* OSI-approved: [The MIT License](https://opensource.org/license/mit/)

:white_check_mark: **Project components (Crates):**

* dcl_data_structures: [The MIT LICENSE.md](https://github.com/deepcausality-rs/deep_causality/blob/main/dcl_data_structures/LICENSE)

* deep_causality_macros: [The MIT LICENSE](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_macros/LICENSE)

* deep_causality: [The MIT LICENSE](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/LICENSE)


:white_check_mark: **Project git repositories:**

* deep_causality: [The MIT LICENSE](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/LICENSE)

* .github (GH Org Readme): [The MIT LICENSE](https://github.com/deepcausality-rs/.github/blob/main/LICENSE)

* sites (Project websites): [The MIT LICENSE](https://github.com/deepcausality-rs/sites/blob/main/LICENSE)

:white_check_mark: **Project dependencies:**

* PetGraph ([Project](https://github.com/petgraph/petgraph)): [Apache-2.0, MIT licenses ](https://github.com/petgraph/petgraph)
* criterion.rs ([Project](https://github.com/bheisler/criterion.rs)): [Apache-2.0, MIT licenses ](https://github.com/bheisler/criterion.rs#license)
* rand ([Project](https://github.com/rust-random/rand)): [Apache-2.0, MIT licenses ](https://github.com/rust-random/rand)
