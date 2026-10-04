/**
 * The narration, one segment per scene: the single source for the audio file names, the captions
 * and, until recordings exist, the scene lengths. The text matches SCRIPT.md.
 *
 * The story is two questions, posed at the start and answered in turn: how hard to burn at
 * supersonic speed (the pause and the fork), and when to light the landing burn (the plan, the
 * landing and the two beliefs).
 */
import type { Segment as NarrationSegment } from '@cfd-video/shared/timing';

export type SegmentId = 'entry' | 'coast' | 'pause' | 'fork' | 'plan' | 'landing' | 'beliefs' | 'close';
export type Segment = NarrationSegment<SegmentId>;

export const segments: Segment[] = [
  {
    id: 'entry',
    phrases: [
      { text: 'The corridor’s capsule flies again, and this time it has to land, braking with its own engine.' },
      { text: 'Two questions shape that landing: how hard to burn at supersonic speed, and when to light the final burn.' },
      { text: 'DeepCausality answers both by flying the alternatives from one shared state.' },
    ],
  },
  {
    id: 'coast',
    phrases: [
      {
        text: 'It enters at Mach 32 on a day 32 K colder than standard, and loses GPS for 56 s.',
        say: 'It enters at Mach thirty-two on a day thirty-two kelvin colder than standard, and loses GPS for fifty-six seconds.',
      },
      {
        text: 'After blackout it coasts, engine off, for another 156 s.',
        say: 'After blackout it coasts, engine off, for another hundred and fifty-six seconds.',
      },
      {
        text: 'At 32.7 km and Mach 2.00 the engine lights, firing forward through the heatshield, into the oncoming air.',
        say: 'At thirty-two point seven kilometres and Mach two the engine lights, firing forward through the heatshield, into the oncoming air.',
      },
    ],
  },
  {
    id: 'pause',
    phrases: [
      { text: 'The first question: how hard to burn.' },
      { text: 'The plume pushes the bow shock away and destroys the drag the capsule relies on, so more thrust is not simply more braking.' },
      {
        text: 'DeepCausality pauses the lit flight and forks it 5 ways:',
        say: 'DeepCausality pauses the lit flight and forks it five ways:',
      },
      {
        text: 'coasting, and throttles of 0.20, 0.40, 0.60 and 0.85.',
        say: 'coasting, and throttles of zero point two, zero point four, zero point six and zero point eight five.',
      },
    ],
  },
  {
    id: 'fork',
    phrases: [
      { text: 'Each branch flies 12 s from the same state.', say: 'Each branch flies twelve seconds from the same state.' },
      {
        text: 'Coasting slows the capsule at 10.59 m/s^2.',
        say: 'Coasting slows the capsule at ten point five nine metres per second squared.',
      },
      {
        text: 'Burning at 0.20 slows it at only 7.47:',
        say: 'Burning at zero point two slows it at only seven point four seven:',
      },
      { text: 'the plume removes drag about as fast as thrust replaces it.' },
      {
        text: 'Only harder burns win the deceleration back: 12.89 at 0.60, and 17.20 at 0.85, which the envelope trims to 0.79.',
        say: 'Only harder burns win the deceleration back: twelve point eight nine at zero point six, and seventeen point two at zero point eight five, which the envelope trims to zero point seven nine.',
      },
      {
        text: 'The drag comes from the Jarvinen and Adams correlation of 1970, carried through each forked flight.',
        say: 'The drag comes from the Jarvinen and Adams correlation of nineteen seventy, carried through each forked flight.',
      },
    ],
  },
  {
    id: 'plan',
    phrases: [
      { text: 'The second question: when to light the landing burn.' },
      { text: 'The guidance adds a margin for its navigation error, sized from the weather table it read before entry.' },
      {
        text: 'For today’s cold, the table gives 54.43 m of drift, plus or minus 2.62;',
        say: 'For today’s cold, the table gives fifty-four point four three metres of drift, plus or minus two point six two;',
      },
      {
        text: 'the margin is the mean plus 3 standard deviations: 62.30 m.',
        say: 'the margin is the mean plus three standard deviations: sixty-two point three metres.',
      },
      {
        text: 'A guidance that assumed a standard day would use 47.52 m.',
        say: 'A guidance that assumed a standard day would use forty-seven point five two metres.',
      },
    ],
  },
  {
    id: 'landing',
    phrases: [
      {
        text: 'The flight resumes from the pause on its own guidance and burns down to Mach 0.6 at 18.5 km.',
        say: 'The flight resumes from the pause on its own guidance and burns down to Mach zero point six at eighteen point five kilometres.',
      },
      { text: 'Then the engine cuts off and the capsule falls.' },
      {
        text: 'With today’s margin it lights the stopping burn at 139.12 m,',
        say: 'With today’s margin it lights the stopping burn at a hundred and thirty-nine point one two metres,',
      },
      {
        text: 'and touches down at 1.81 m/s, with 1192.5 kg of propellant left.',
        say: 'and touches down at one point eight one metres per second, with eleven hundred ninety-two point five kilograms of propellant left.',
      },
    ],
  },
  {
    id: 'beliefs',
    phrases: [
      { text: 'To see what the margin buys, the landing is flown twice from the same state, once with each margin.' },
      {
        text: 'Sized for today, it lights at 139.12 m; sized for a standard day, at 125.06 m.',
        say: 'Sized for today, it lights at a hundred and thirty-nine point one two metres; sized for a standard day, at a hundred and twenty-five point oh six.',
      },
      {
        text: 'On this day both land softly, at 1.81 and 1.72 m/s.',
        say: 'On this day both land softly, at one point eight one and one point seven two metres per second.',
      },
      {
        text: 'The extra 14 m of margin costs 7.28 kg of propellant.',
        say: 'The extra fourteen metres of margin costs seven point two eight kilograms of propellant.',
      },
    ],
  },
  {
    id: 'close',
    phrases: [
      {
        text: 'How hard to burn? The 5 forked burns show that a light burn slows the capsule less than coasting.',
        say: 'How hard to burn? The five forked burns show that a light burn slows the capsule less than coasting.',
      },
      {
        text: 'When to light the final burn? The 2 forked landings show that knowing the day costs 7.28 kg for 14 m of extra margin.',
        say: 'When to light the final burn? The two forked landings show that knowing the day costs seven point two eight kilograms for fourteen metres of extra margin.',
      },
      {
        text: 'The whole descent, 5491 coupled steps, runs in under 6 minutes on a laptop, open source, in Rust.',
        say: 'The whole descent, five thousand four hundred ninety-one coupled steps, runs in under six minutes on a laptop, open source, in Rust.',
      },
    ],
  },
];
