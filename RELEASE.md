---
SPDX-License-Identifier: MIT
---

# Releasing

1. Update version file accordingly.
2. Commit changes.
3. Tag the release: `git tag -s VERSION`
4. Push changes and tags: `git push --tags`
5. Manually create a [release on github using the latest tag. ](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)