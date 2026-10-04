/**
 * The narration, one segment per scene: the single source for the audio file names, the captions
 * and, until recordings exist, the scene lengths. The text matches SCRIPT.md.
 */
import type { Segment as NarrationSegment } from '@cfd-video/shared/timing';

export type SegmentId = 'problem' | 'question' | 'pause' | 'fork' | 'fly-through' | 'close';
export type Segment = NarrationSegment<SegmentId>;

export const segments: Segment[] = [
  {
    id: 'problem',
    phrases: [
      { text: 'A capsule enters the atmosphere at Mach 29.', say: 'A capsule enters the atmosphere at Mach twenty-nine.' },
      { text: 'The air in front of it turns to plasma,' },
      {
        text: 'and 13 seconds later, at 73 km, the plasma blocks GPS.',
        say: 'and thirteen seconds later, at seventy-three kilometres, the plasma blocks GPS.',
      },
      { text: 'From here it flies blind.' },
    ],
  },
  {
    id: 'question',
    phrases: [
      { text: 'Guidance must choose a bank angle now:' },
      {
        text: 'how far to roll its lift sideways to reach an aim point 20 m off its path.',
        say: 'how far to roll its lift sideways to reach an aim point twenty metres off its path.',
      },
      { text: 'Which angle lands closest?' },
    ],
  },
  {
    id: 'pause',
    phrases: [
      {
        text: 'DeepCausality pauses the flight the moment GPS is lost, at 13.1 s and 73.2 km.',
        say: 'DeepCausality pauses the flight the moment GPS is lost, at thirteen point one seconds and seventy-three point two kilometres.',
      },
      { text: 'The pause holds the whole coupled state:' },
      {
        text: 'the flow around the vehicle, the plasma sheath at 3.3 × 10^16 electrons per m^3,',
        say: 'the flow around the vehicle, the plasma sheath at three point three times ten to the sixteen electrons per cubic metre,',
      },
      { text: 'and the navigation filter, already 0.18 m off.', say: 'and the navigation filter, already zero point one eight metres off.' },
      {
        text: 'Every candidate bank angle forks from this one state, so all 17 alternatives share the same past.',
        say: 'Every candidate bank angle forks from this one state, so all seventeen alternatives share the same past.',
      },
    ],
  },
  {
    id: 'fork',
    phrases: [
      {
        text: 'Each branch flies the same coupled simulation 10 s ahead with its own bank angle.',
        say: 'Each branch flies the same coupled simulation ten seconds ahead with its own bank angle.',
      },
      {
        text: 'The first round flies 6 angles in parallel, from 0° to 40°.',
        say: 'The first round flies six angles in parallel, from zero to forty degrees.',
      },
      {
        text: '40° exceeds the safety envelope, so the gate clamps it to 28.6°.',
        say: 'Forty degrees exceeds the safety envelope, so the gate clamps it to twenty-eight point six.',
      },
      { text: '10° lands 3.5 m from the aim, the best of the round.', say: 'Ten degrees lands three point five metres from the aim, the best of the round.' },
      {
        text: 'A second round forks the same pause again: 11 angles around 10°, 0.5° apart.',
        say: 'A second round forks the same pause again: eleven angles around ten degrees, half a degree apart.',
      },
      { text: 'Every end lands on one arc set by the bank angle, and the aim sits just off it.' },
      {
        text: '11.5° comes closest: 2.07 m from the aim, against 20 m with no bank.',
        say: 'Eleven and a half degrees comes closest: two point oh seven metres from the aim, against twenty with no bank.',
      },
    ],
  },
  {
    id: 'fly-through',
    phrases: [
      {
        text: 'DeepCausality commits 11.5° and resumes the paused flight in that world.',
        say: 'DeepCausality commits eleven and a half degrees and resumes the paused flight in that world.',
      },
      {
        text: 'The plasma thickens: at 61 km the sheath holds 2.6 × 10^19 electrons per m^3,',
        say: 'The plasma thickens: at sixty-one kilometres the sheath holds two point six times ten to the nineteen electrons per cubic metre,',
      },
      {
        text: 'against 10^19 measured on the RAM-C II flight in 1970.',
        say: 'against ten to the nineteen measured on the RAM-C two flight in nineteen seventy.',
      },
      { text: 'With no GPS, the vehicle dead-reckons on its inertial sensors.' },
      {
        text: 'Over 56 s of blackout, its navigation error grows from 0.18 m to 42.4 m.',
        say: 'Over fifty-six seconds of blackout, its navigation error grows from zero point one eight to forty-two point four metres.',
      },
      { text: 'At 46.8 km the sheath clears and GPS returns;', say: 'At forty-six point eight kilometres the sheath clears and GPS returns;' },
      {
        text: 'the first fix cuts the error to 2.29 m, and 3 s later to 0.28 m.',
        say: 'the first fix cuts the error to two point two nine metres, and three seconds later to zero point two eight.',
      },
    ],
  },
  {
    id: 'close',
    phrases: [
      { text: 'With GPS gone, flying uncorrected would miss the aim by 20 m.', say: 'With GPS gone, flying uncorrected would miss the aim by twenty metres.' },
      {
        text: 'Counterfactuals from the last known state cut that to 2.07 m, inside the 5.6 m the navigation filter reports as its own uncertainty.',
        say: 'Counterfactuals from the last known state cut that to two point oh seven metres, inside the five point six metres the navigation filter reports as its own uncertainty.',
      },
      {
        text: 'The whole run takes 44 s on a laptop, and the corridor is open source, in Rust.',
        say: 'The whole run takes forty-four seconds on a laptop, and the corridor is open source, in Rust.',
      },
    ],
  },
];
