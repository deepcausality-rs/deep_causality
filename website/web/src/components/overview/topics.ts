/** The Overview topic pages, in reading order. The Overview index, the topic list at the foot of
 * every topic page, and the header's Overview menu all read this list. */
export interface OverviewTopic {
  /** Route segment under /overview/. */
  slug: string;
  /** Page and menu title. */
  title: string;
  /** One sentence for the topic list. */
  summary: string;
}

export const topics: OverviewTopic[] = [
  {
    slug: 'causal-discovery',
    title: 'Causal discovery',
    summary:
      'SURD splits the information sources carry about a target; BRCD ranks the likely root causes of a failure.',
  },
  {
    slug: 'dynamic-causality',
    title: 'Dynamic causality',
    summary:
      'The causal monad: a one-directional causal order, with any computation inside each step, including time-symmetric physics.',
  },
  {
    slug: 'dynamic-context',
    title: 'Dynamic context',
    summary:
      'The world a causal step reads: data, clocks, positions and frames of reference, updated while a model runs and persisted through a store.',
  },
  {
    slug: 'dynamic-action',
    title: 'Dynamic action',
    summary:
      'The Causal State Machine fires an action when a causal result is active, and its rules can change while it runs.',
  },
  {
    slug: 'effect-ethos',
    title: 'Effect Ethos',
    summary:
      'Norms that check a proposed action and return a verdict, with the norms that produced it.',
  },
];
