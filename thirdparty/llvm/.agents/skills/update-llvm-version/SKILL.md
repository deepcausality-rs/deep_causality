---
name: update-llvm-version
description: 'Update a minor or patch LLVM version in this repository. Use when bumping LLVM_VERSION, creating llvm-<version>-<suffix> prebuilt branches, watching the LLVM Prebuilt Release workflow, collecting release SHA256 values, updating MODULE.bazel and prebuilt metadata, and opening the stacked update-llvm-<version> pull request.'
argument-hint: 'Target LLVM version, optionally with prebuilt suffix number'
---

# Update LLVM Version

## When to Use

- Bumping LLVM to a new minor or patch version in this repository.
- Publishing a new set of minimal prebuilts before wiring them into MODULE.bazel.
- Recovering from a failed llvm-* prebuilt branch run.
- Creating the follow-up update branch and PR after the release artifacts exist.

## Inputs

- Target LLVM version such as 22.1.2.
- Prebuilt suffix number. Default to 1 unless a variant of the prebuilts is needed.
- Git and GitHub CLI access with permission to push branches, inspect workflow runs, inspect releases, and create PRs.

## Workflow Summary

This workflow has two phases:

1. Build and publish prebuilts from a branch named llvm-<version>-<suffix>.
2. Consume those prebuilts from a branch named update-llvm-<version> by updating the prebuilt index and opening a PR.

The first branch changes LLVM_VERSION before the new prebuilt release exists, so the llvm_toolchain_minimal extension uses default_prebuilt_llvm_version from the checked-in index as the bootstrap seed. The second branch maps the new LLVM_VERSION to the newly published release in the checked-in llvm-toolchain-minimal index.
Pushes, release inspection, and PR creation should be performed automatically unless the user overrides that behavior.
Do not run local builds unless the user explicitly asks. Default verification is the LLVM Prebuilt Release workflow plus the final PR CI.

## Procedure

### Phase 1: Prepare branch
1. Start from the latest main branch unless the user says otherwise.
2. Create a branch named llvm-<version>-<suffix>.

### Phase 2: Prepare llvm_versions.json
1. Check whether hermeticbuild/llvm-redist already has the repacked source release tag llvmorg-<version>-r1 for the target LLVM version.
2. If it does not, dispatch hermeticbuild/llvm-redist/.github/workflows/repack.yml with workflow_dispatch inputs version=<version> and publish=true, then wait for that run to finish successfully.
3. After the run succeeds, retrieve the repacked source archive URL and SHA256 from that release in hermeticbuild/llvm-redist and add them to llvm_versions.json in this repository under the new version entry.

### Phase 3: Publish the new prebuilts

3. Edit MODULE.bazel:
   - Set LLVM_VERSION to the new version.
   - Do not change default_prebuilt_llvm_version in extensions/llvm_toolchain_minimal_index.json; it remains the bootstrap seed until the new release is published.
4. Commit the change and push the branch.
5. Monitor the LLVM Prebuilt Release workflow for that branch push.
   - Expect the successful run to often take roughly 20 to 30 minutes.
   - Prefer polling with gh run view ... --json ... over gh run watch for cleaner automation output.
6. Confirm the branch naming matches the version in MODULE.bazel.
   - The workflow script rejects the run if the branch does not start with llvm-.
   - The workflow script also rejects the run if the branch payload version does not equal LLVM_VERSION.
7. Wait for the workflow to publish the release tagged with the branch name.
   - In this repository, the published release tag is exactly github.ref_name, so the release tag is the branch name.
8. If the run fails:
   - If the fix is trivial, patch the same llvm-* branch, commit, push, and watch the next run.
   - If the failure is not trivial, stop after producing a detailed failure analysis, likely root cause, and a concrete fix plan, then ask the user for guidance.

### Phase 4: Consume the published prebuilts

1. After the llvm-* release succeeds, create or switch to update-llvm-<version>.
2. Base update-llvm-<version> on the successful llvm-<version>-<suffix> branch.
   - The default and expected behavior is a stacked PR that includes both phases of work.
3. Locate the release created by the first phase.
   - Expected tag: llvm-<version>-<suffix>.
   - Expected assets: six llvm-toolchain-minimal-<version>-<target>.tar.zst archives plus SHA256.txt.
4. Retrieve the SHA256 values for all minimal toolchain artifacts from the release.
   - Use SHA256.txt as the source of truth when updating extensions/llvm_toolchain_minimal_index.json.
5. Edit extensions/llvm_toolchain_minimal_index.json:
   - Add or replace the new `llvm-<version>-<suffix>` entry under `releases` with the published URLs and sha256 values.
   - Set `latest_by_llvm_version["<version>"]` to `llvm-<version>-<suffix>`.
6. Set `default_prebuilt_llvm_version` to the new version when the repository default should advance to the new release.
7. Keep LLVM_VERSION in MODULE.bazel at the new version. The llvm_toolchain_minimal extension automatically materializes the release selected by latest_by_llvm_version.
8. Commit and push update-llvm-<version>.
9. Open a PR from update-llvm-<version> to main.
10. Make it explicit in the PR body or branch strategy that the PR intentionally includes the commits from the prior llvm-* branch when the branch is stacked that way.

## Suggested Commands

Use these as a baseline and adapt only when the user asks for a different flow.

```bash
git switch main
git pull --ff-only
git switch -c llvm-<version>-<suffix>
```

```bash
gh run list --branch llvm-<version>-<suffix> --workflow "LLVM Prebuilt Release"
gh run view <run-id> --json status,conclusion,jobs
gh run view <run-id> --log-failed
```

```bash
gh release view llvm-<version>-<suffix>
gh release download llvm-<version>-<suffix> -p SHA256.txt -D /tmp/llvm-prebuilt-<version>
```

```bash
git switch -c update-llvm-<version>
gh pr create --base main --head update-llvm-<version> --fill
```

## Decision Points

- Suffix selection:
  Use 1 by default. Increase only when a new prebuilt variant is required for the same LLVM version.
- Failure handling:
   Keep iterating on the llvm-* branch only for obvious workflow, syntax, config, or similarly low-risk fixes. Escalate with analysis and plan when the cause is not obvious or the fix is risky.
- Branch base for the PR branch:
   Always branch update-llvm-<version> from llvm-<version>-<suffix> so the final PR contains both the prebuilt-generation preparation and the final metadata update.
- Remote actions:
   Push branches, inspect GitHub runs and releases, and create the PR automatically unless the user explicitly asks to pause before remote changes.

## Completion Checks

- LLVM_VERSION is updated to the target version.
- The llvm-<version>-<suffix> branch successfully published its GitHub release.
- The release contains SHA256.txt and all six minimal toolchain archives.
- extensions/llvm_toolchain_minimal_index.json maps the LLVM version to the chosen release suffix.
- default_prebuilt_llvm_version identifies the intended fallback bootstrap seed.
- extensions/llvm_toolchain_minimal_index.json matches the published release URLs and checksums exactly.
- The update-llvm-<version> branch is pushed.
- A PR targeting main exists for the final branch.

## Repository-Specific Notes

- The LLVM prebuilt workflow is defined in .github/workflows/llvm-prebuilt.yml.
- The packaging script is .github/workflows/llvm-prebuilt.sh.
- The workflow is triggered by pushes to branches matching llvm-*.
- The prebuilt release tag is the branch name, not a derived semver-only tag.
- The workflow writes SHA256 values into release/SHA256.txt.
