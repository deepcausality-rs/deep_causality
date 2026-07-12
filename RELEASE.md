[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Releasing

1. Update version file accordingly.
2. Review/update [SKILLS.md](./SKILLS.md) for API or concept changes.
3. Commit changes.
4. Tag the release: `git tag -s VERSION`
5. Push changes and tags: `git push --tags`
6. Manually create
   a [release on github using the latest tag. ](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
