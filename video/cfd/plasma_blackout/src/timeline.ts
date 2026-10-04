/**
 * The cut's timing: one scene per narration segment, with this cut's holds before the first phrase
 * and after the last. The main composition, the captions and the SRT all read this one table, so a
 * re-timed line moves all three.
 */
import { makeTimeline, type SceneTiming as Timing } from '@cfd-video/shared/timing';
import { segments, type SegmentId } from './script';

/** Seconds before a scene's first phrase and after its last. */
const LEAD_S: Record<SegmentId, number> = {
  problem: 3,
  question: 0.8,
  pause: 1.4,
  fork: 1,
  'fly-through': 0.8,
  close: 0.6,
};
const TAIL_S: Record<SegmentId, number> = {
  problem: 1,
  question: 1.2,
  pause: 1.6,
  fork: 3,
  'fly-through': 1.6,
  close: 5.5,
};

export const { sceneTimings, mainFrames, scene, segment } = makeTimeline(segments, LEAD_S, TAIL_S);
export { TRANSITION_FRAMES, type PhraseTiming } from '@cfd-video/shared/timing';
export type SceneTiming = Timing<SegmentId>;
