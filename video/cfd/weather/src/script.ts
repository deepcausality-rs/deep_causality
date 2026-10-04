/**
 * The narration, one segment per scene: the single source for the audio file names, the captions
 * and, until recordings exist, the scene lengths. The text matches SCRIPT.md.
 *
 * The story is two questions, posed at the start and answered in turn: does the weather move the
 * GPS blackout (the windows), and does it move the navigation error (the drift, why, and whether it
 * is resolved above the noise). The close answers both under the same headings.
 */
import type { Segment as NarrationSegment } from '@cfd-video/shared/timing';

export type SegmentId = 'entry' | 'worlds' | 'window' | 'drift' | 'why' | 'resolved' | 'close';
export type Segment = NarrationSegment<SegmentId>;

export const segments: Segment[] = [
  {
    id: 'entry',
    phrases: [
      { text: 'Before this capsule flies, its flight computer needs a table: what the weather does to its descent.' },
      { text: 'Two questions: does the weather move the GPS blackout, and does it move the navigation error?' },
      {
        text: 'DeepCausality answers both by flying one validated descent through 6 counterfactual atmospheres.',
        say: 'DeepCausality answers both by flying one validated descent through six counterfactual atmospheres.',
      },
    ],
  },
  {
    id: 'worlds',
    phrases: [
      { text: 'The standard day is the baseline: the corridor’s validated descent.' },
      {
        text: 'From it, DeepCausality alternates 5 atmospheres: hot, cold, polar winter, thin and dense, each set by a temperature offset and a density scale.',
        say: 'From it, DeepCausality alternates five atmospheres: hot, cold, polar winter, thin and dense, each set by a temperature offset and a density scale.',
      },
      { text: 'Everything else stays the baseline, and each descent’s audit log records the swap.' },
      {
        text: 'Each world flies 8 receiver-noise draws: 48 full descents, flown in parallel.',
        say: 'Each world flies eight receiver-noise draws: forty-eight full descents, flown in parallel.',
      },
    ],
  },
  {
    id: 'window',
    phrases: [
      { text: 'The first question: does the weather move the blackout?' },
      {
        text: 'Denser air ionizes earlier: the dense day loses GPS at 12.1 s, the thin day at 14.6 s.',
        say: 'Denser air ionizes earlier: the dense day loses GPS at twelve point one seconds, the thin day at fourteen point six.',
      },
      {
        text: 'But every blackout lasts between 55.0 and 56.1 s.',
        say: 'But every blackout lasts between fifty-five and fifty-six point one seconds.',
      },
      {
        text: 'The weather shifts the window by 2.5 s, and barely changes its length.',
        say: 'The weather shifts the window by two point five seconds, and barely changes its length.',
      },
    ],
  },
  {
    id: 'drift',
    phrases: [
      { text: 'The second question: does the weather move the navigation error?' },
      { text: 'Without GPS, the error grows through the whole blackout.' },
      {
        text: 'On a standard day it reaches 41.60 m on average; in polar winter, 58.71 m, 1.41 times as far.',
        say: 'On a standard day it reaches forty-one point six metres on average; in polar winter, fifty-eight point seven one, one point four one times as far.',
      },
    ],
  },
  {
    id: 'why',
    phrases: [
      {
        text: 'Why? Not the window: the longest blackout is 2% longer than the shortest, which moves the drift by about 4%.',
        say: 'Why? Not the window: the longest blackout is two percent longer than the shortest, which moves the drift by about four percent.',
      },
      {
        text: 'The instrument: the accelerometer bias departs from its calibration by 1% per kelvin, while the navigation filter assumes a standard day.',
        say: 'The instrument: the accelerometer bias departs from its calibration by one percent per kelvin, while the navigation filter assumes a standard day.',
      },
      {
        text: 'Polar winter sits 40 K from the calibration point and flies 1.40 times the bias.',
        say: 'Polar winter sits forty kelvin from the calibration point and flies one point four times the bias.',
      },
      {
        text: 'Scaled by that factor and the square of the blackout’s length, the standard day’s drift predicts every world within 3.4%.',
        say: 'Scaled by that factor and the square of the blackout’s length, the standard day’s drift predicts every world within three point four percent.',
      },
    ],
  },
  {
    id: 'resolved',
    phrases: [
      { text: 'Is the difference real, or receiver noise?' },
      {
        text: 'Each dot is one draw: receiver noise scatters a world’s drift by 1.71 to 2.93 m, one standard deviation.',
        say: 'Each dot is one draw: receiver noise scatters a world’s drift by one point seven one to two point nine three metres, one standard deviation.',
      },
      {
        text: 'Polar winter and the standard day sit 17.11 m apart: 5.7 times their combined scatter.',
        say: 'Polar winter and the standard day sit seventeen point one one metres apart: five point seven times their combined scatter.',
      },
      {
        text: 'And once GPS returns, every draw of every world recovers to within 0.22 m.',
        say: 'And once GPS returns, every draw of every world recovers to within zero point two two metres.',
      },
    ],
  },
  {
    id: 'close',
    phrases: [
      {
        text: 'Does the weather move the blackout? It shifts it by 2.5 s and barely changes its length.',
        say: 'Does the weather move the blackout? It shifts it by two point five seconds and barely changes its length.',
      },
      {
        text: 'Does it move the navigation error? Polar winter drifts 41% further, because its instrument flies furthest from calibration.',
        say: 'Does it move the navigation error? Polar winter drifts forty-one percent further, because its instrument flies furthest from calibration.',
      },
      {
        text: '48 counterfactual descents, one table, which the landing reads in flight. The campaign runs in about 3 minutes on a laptop, open source, in Rust.',
        say: 'Forty-eight counterfactual descents, one table, which the landing reads in flight. The campaign runs in about three minutes on a laptop, open source, in Rust.',
      },
    ],
  },
];
