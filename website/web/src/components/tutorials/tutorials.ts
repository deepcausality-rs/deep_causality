/** The Tutorials section. `tutorials` feeds the Tutorials index and the header's Tutorials menu
 * (desktop and mobile); `droneFailsafeParts` feeds the part list on the drone tutorial's overview
 * and at the foot of each of its parts. Add a tutorial or a part here, not in the pages. */
import type { OverviewTopic } from '../overview/topics';

/** Route of the Tutorials section. */
export const TUTORIALS_BASE = '/tutorials/';

/** Route of the drone fail-safe tutorial; its parts live below it. */
export const DRONE_FAILSAFE_BASE = `${TUTORIALS_BASE}dynamic-drone-failsafe/`;

/** Repository path of the drone fail-safe tutorial's source. */
export const DRONE_FAILSAFE_SOURCE = 'end_to_end_tutorials/dynamic_drone_failsafe';

export const tutorials: OverviewTopic[] = [
  {
    slug: 'dynamic-drone-failsafe',
    title: 'Dynamic drone fail-safe',
    summary:
      'An inspection drone loses GPS, a battery cell and the command link its pilot steers it by, at night over a mountain slope. Five parts build a fail-safe that brings it down clear of people, and verify it over 1,000 scenarios.',
  },
];

export const droneFailsafeParts: OverviewTopic[] = [
  {
    slug: 'dynamic-causality',
    title: 'Part 1: Dynamic causality',
    summary:
      'A causal process confirms each fault and applies the textbook fail-safe. The wind carries the drone into the creek.',
  },
  {
    slug: 'dynamic-context',
    title: 'Part 2: Dynamic context',
    summary:
      'A thermal camera and a LiDAR fill a context with uncertain readings of the ground. The controller knows the ground and still lands in the creek.',
  },
  {
    slug: 'dynamic-action',
    title: 'Part 3: Dynamic action',
    summary:
      'A causal state machine flies the drone to the nearest safe patch. It lands 3 m from a member of the crew.',
  },
  {
    slug: 'effect-ethos',
    title: 'Part 4: Effect Ethos',
    summary:
      'The published emergency procedures for drone pilots become norms that rule on every maneuver. The drone lands upright, 34 m from the nearest person.',
  },
  {
    slug: 'verification',
    title: 'Part 5: Verification',
    summary:
      'Parts 1, 3 and 4 fly the same 1,000 randomised scenarios. Part 4 puts a person at risk in 0.1 % of them, against 3.5 % for the textbook fail-safe.',
  },
];
