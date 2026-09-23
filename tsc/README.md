# DeepCausality Technical Steering Committee (TSC)

The DeepCausality TSC oversees all technical matters of the open source project.

## DeepCausality Technical Charter

The project charter is in [CHARTER](../DeepCausalityProjectCharter.pdf).

## Collaboration Tools

### Public Mailing List

The TSC can be reached on its [mailing list](https://deepcausality.com/community/), which hosts public technical discussions.

### Discord

The DeepCausality Foundation maintains a [Discord Server](https://discord.gg/Bxj9P7JXSj).
Anyone can join and participate in its public channels.

### DCO

Every contributor must sign the DeepCausality DCO before contributing, either personally or through their company. A contributor not covered by a DCO is notified on opening their first PR. See the [documentation on the DCO](../WHAT_IS_DCO.md).

## Members

The current members of the DeepCausality TSC are:

| Name            | Github       | Term begins | Term ends   | Affiliation      |
|-----------------|--------------|-------------|-------------|------------------|
| Marvin Hansen   | marvin-hansen  | Dec 1, 2025 | Dec 1, 2026 | Emet-Labs        |
| Michael Freeman | mfreeman451    | Dec 1, 2024 | Dec 1, 2026 | Cerver Automation |


TSC membership is open to all DeepCausality project committers. Before each election, candidates nominate themselves 
to one or more active maintainers by email or on the Discord Server. 

From 2026, the TSC has 5 seats. Per the charter, a voting member's term is one year and extends automatically 
by another year unless the member steps down. 

## Policies and procedures

The [CHARTER](../DeepCausalityProjectCharter.pdf) governs the TSC. It sets the TSC's scope, how it decides, and how it changes itself, and leaves the TSC wide freedom in implementing the policies of the DeepCausality Foundation.

The TSC has adopted the following policies and procedures.

### Making decisions

Per the [CHARTER](../DeepCausalityProjectCharter.pdf), the TSC decides by consensus wherever possible. Where consensus fails or a vote is explicitly required, an action proceeds only with the approval of a majority of TSC voting members (or more, if the governance requires). Votes are taken over email and documented in the next meeting.

### Merging PRs into the TSC repository

A pull request that does not change the TSC's charter or governance can be merged if:

* There are no outstanding objections
* There are two approvals by TSC members

Pull requests that change governance of the TSC (excluding the charter) must be open for at least 14 days, unless consensus is reached in a meeting with quorum of voting members.

Without consensus, a pull request may still land after a vote of the voting members overrides outstanding objections.

### Fast-Tracking PRs

Pull requests making any of the following changes to this repository may be fast-tracked:

- Errata fixes.
- Editorial changes.
- Meeting minutes.
- Updates to team lists.
- Doc fixes.

Charter changes cannot be fast-tracked.

To propose fast-tracking a pull request, apply the ***fast-track*** label. Then add a comment that TSC members may upvote. If someone disagrees with the fast-tracking request, remove the label. Do not fast-track the pull request in that case.

The pull request may be fast-tracked if two TSC members approve the fast-tracking request. To land, the pull request itself still needs two TSC member approvals.

TSC members may request fast-tracking of pull requests they did not author; only then does the request count as one fast-track approval. Upvote the comment anyway to avoid doubt.


#### Copyright notices

The DeepCausality project follows the [community best practice](https://www.linuxfoundation.org/blog/2020/01/copyright-notices-in-open-source-software-projects/) of not requiring a notice in each file. Instead, a contributor adds a one-time entry to [MAINTAINERS.md](../MAINTAINERS.md) during or after the merge of their first PR. 

#### SPDX

Contributors are encouraged, not required, to include [SPDX short form identifiers](https://spdx.dev/about/overview/) in their files. DeepCausality uses the MIT licence; its SPDX designator is "SPDX-License-Identifier: MIT".