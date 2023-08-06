---
SPDX-License-Identifier: MIT
---

# Recommended Practices for LF AI & Data Projects Hosting Code on GitHub

These practices will help you improve your GitHub presence in an effort to help you attract more users and developers to your project, secure your account, be precise about licensing, and maintain good housekeeping. Please issue a PR to add new recommendations or update existing ones.

1. ✅ Use the [REPOLINTER](https://github.com/todogroup/repolinter) tool created by the TODO Group to identify common issues in GitHub repos.
   * [Repolinter Report](LF_Repo_Lint.md)
   
2. ✅ Secure your GitHub account with two-factor authentication.

3. ✅ Ensure that every repo includes a LICENSE file.
   * Main (mono) repo: [LICENSE](https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE)
   * Sites (website) repo: [LICENSE](https://github.com/deepcausality-rs/sites/blob/main/LICENSE)
   
4. ✅ Add a README file to your repos welcoming new community members to the project and explaining why the project is useful and how to get started.
   * [README](https://github.com/deepcausality-rs/deep_causality/blob/main/README.md)

5. ✅ Add a CONTRIBUTING file to your repos explaining to other developers and your community of users how to contribute to the project. At a high level, the file would explain what types of contributions are needed and how the process works.
   * [CONTRIBUTING](https://github.com/deepcausality-rs/deep_causality/blob/main/CONTRIBUTING.md)

6. ✅Add CODEOWNERS file to define individuals or teams that are responsible for code in a repository.
   * [CODEOWNERS](https://github.com/deepcausality-rs/deep_causality/blob/main/CODEOWNERS)

7. ✅ Add a CODE_OF_CONDUCT file that sets the ground rules for participants’ behavior associated and helps to facilitate a friendly, welcoming environment. While not every project has a CODE_OF_CONDUCT file, its presence signals that this is a welcoming project to contribute to, and defines standards for how to engage with the project’s community. You are welcome to use the Linux Foundation’s Code of Conduct if project specific CoC does not exist.
   * [CODE_OF_CONDUCT](https://github.com/deepcausality-rs/deep_causality/blob/main/CODE_OF_CONDUCT.md)
   
8. ✅ Provide documentation on the release methodology, cadence, criteria, etc.
   * [RELEASE](https://github.com/deepcausality-rs/deep_causality/blob/main/RELEASE.md)
   
9. ❌️ Document your project governance and make it available on the project’s repo.

10. ✅ Add a SUPPORT file to let users and developers know about ways to get help with your project. You can either add in this file how and where security issues are handled, or put it at the top level readme for the project, or alternatively refer to security documentation.
    * [SUPPORT](https://github.com/deepcausality-rs/deep_causality/blob/main/SUPPORT.md)
    
11. ✅ Archive inactive repos to flag to your users and other developers that you’re not maintaining them.
    * No inactive repos in the project

12. ✅ Setup issue template and pull request templates that help you customize and standardize the information you'd like contributors to include when they open issues and pull requests in your repository.
    * [issue template](https://github.com/deepcausality-rs/deep_causality/tree/main/.github/ISSUE_TEMPLATE)
    * [pull request templates](https://github.com/deepcausality-rs/deep_causality/blob/main/.github/PULL_REQUEST_TEMPLATE.md)
    
13. ✅ Achieve and maintain the [OpenSSF Best Practices Badge](https://bestpractices.coreinfrastructure.org/en) (previousely called the Core Infrastructure Initiative Best Practices Badge) for your project.
    * [deep_causality OpenSSF Best Practices Badge](https://bestpractices.coreinfrastructure.org/en/projects/7568)
    
14. Identify who on the project will be handling security issues (could be a team) and set up a separate email account.  Consider having the project become a CNA (CVE Numbering Authority).
    * [SECURITY](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md)
    
15. ✅Include an SPDX short-form identifier in a comment at the top of each file in the repo, wherever reasonably possible.
    * SPDX-License-Identifier: MIT

16. ✅ Depending on whether your project uses the DCO and/or CLAs:
    * DCO: [WHAT_IS_DCO](https://github.com/deepcausality-rs/deep_causality/blob/main/WHAT_IS_DCO.md)

17. ✅ Use English as the default universal language for anything you publish on GitHub. You can support a second language but English should be the primary language of communication towards a universal audience.
    * English is the default and only supported language of the project