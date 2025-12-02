# DeepCausality Technical Steering Committee (TSC)

The DeepCausality TSC will be responsible for all technical oversight of the open source Project.

## DeepCausality Technical Charter

The DeepCausality Project Charter is located in [CHARTER](../DeepCausalityProjectCharter.pdf)

## Collaboration Tools

### Public Mailing List

The TSC for DeepCausality Foundation can be reached at their [mailing list](https://deepcausality.com/community/). This list is intended for public technical discussions.

### Discord

The DeepCausality Foundation maintains a [Discord Server](https://discord.gg/Bxj9P7JXSj) for communication and collaboration.
The Discord is open for anyone to join and participate in the public channels.

### DCO

Everyone must sign the DeepCausality DCO prior to making a contribution. You may either sign the DCO on your own, or your company can sign it for you. At a high level, if you aren't covered by a DCO, you will be notified the first time you open a PR. Please see our [documentation on the DCO](../WHAT_IS_DCO.md).

## Members

The current members of the DeepCausality TSC are:

| Name            | Github       | Term begins | Term ends   | Affiliation      |
|-----------------|--------------|-------------|-------------|------------------|
| Marvin Hansen   | marvin-hansen  | Dec 1, 2025 | Dec 1, 2026 | Emet-Labs        |
| Michael Freeman | mfreeman451    | Dec 1, 2024 | Dec 1, 2026 | Cerver Automation |


TSC membership is open to all DeepCausality project committers. Prior to each election, candidates must submit a self 
nomination to one or more of the active maintainers either by email or via the Discord Server. 

Beginning in 2026, the TSC has 5 seats. Per the charter, TSC voting member terms are one year. If a member does 
not step down, the term extends automatically by another year. 

## Policies and procedures

The DeepCausality TSC is governed by the [CHARTER](../DeepCausalityProjectCharter.pdf)  The Charter provides a foundational structure for the TSC on topics such as its scope, how to make decisions, and how to make changes to itself.  At the same time, it grants the TSC a high degree of freedom when determining how to implement the policies of the DeepCausality Foundation.

The following policies and procedures have been adopted by the TSC.

### Making decisions

Per the [CHARTER](../DeepCausalityProjectCharter.pdf), wherever possible the TSC will attempt to make decisions by consensus.  In circumstances where consensus is not possible or if a vote is explicitly required, a majority (or higher, if required by the governance) of TSC voting members must approve in order for the action to proceed.  Votes will be taken over email, and documented in the next meeting.

### Merging PRs into the TSC repository

Pull requests that do not change the charter or governance of the TSC can be merged into this repository provided the following conditions have been met:

* There are no outstanding objections
* There are two approvals by TSC members

Pull requests that change governance of the TSC (excluding the charter) must be open for at least 14 days, unless consensus is reached in a meeting with quorum of voting members.

If consensus cannot be reached, a pull request may still be landed after a vote by the Voting members to override outstanding objections.

### Fast-Tracking PRs

Special exception is made for pull requests seeking to make any of the following changes to this repository:

- Errata fixes.
- Editorial changes.
- Meeting minutes.
- Updates to team lists.
- Doc fixes.

Charter changes cannot be fast-tracked.

To propose fast-tracking a pull request, apply the ***fast-track*** label. Then add a comment that TSC members may upvote. If someone disagrees with the fast-tracking request, remove the label. Do not fast-track the pull request in that case.

The pull request may be fast-tracked if two TSC members approve the fast-tracking request. To land, the pull request itself still needs two TSC member approvals.

TSC members may request fast-tracking of pull requests they did not author. In that case only, the request itself is also one fast-track approval. Upvote the comment anyway to avoid any doubt.


#### Copyright notices

DeepCausality project follows the [community best practice](https://www.linuxfoundation.org/blog/2020/01/copyright-notices-in-open-source-software-projects/) of not requiring contributors to add a notice to each file. Instead, a one time entry to the [MAINTAINERS.md](../MAINTAINERS.md) file should be added after or during the merge of the first PR. 

#### SPDX

Contributors are encouraged (but not required) to adopt the practice of including [SPDX short form identifiers](https://spdx.dev/about/overview/) in their files. The DeepCausality uses MIT licence and the SPDX designator is: "SPDX-License-Identifier: MIT"