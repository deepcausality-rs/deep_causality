/**
 * The cut's timing: one scene per narration segment, with this cut's holds before the first phrase
 * and after the last. The main composition, the captions and the SRT all read this one table, so a
 * re-timed line moves all three.
 */
import { makeTimeline, type SceneTiming as Timing } from '@cfd-video/shared/timing';
import { segments, type SegmentId } from './script';

/** Seconds before a scene's first phrase and after its last. */
const LEAD_S: Record<SegmentId, number> = {
  entry: 3,
  plan: 0.8,
  coast: 0.8,
  pause: 1.2,
  fork: 1,
  landing: 0.8,
  beliefs: 1,
  close: 0.6,
};
const TAIL_S: Record<SegmentId, number> = {
  entry: 0.8,
  plan: 1.4,
  coast: 1.2,
  pause: 1.4,
  fork: 2.4,
  landing: 1.6,
  beliefs: 2.4,
  close: 5.5,
};

export const { sceneTimings, mainFrames, scene, segment } = makeTimeline(segments, LEAD_S, TAIL_S);
export { TRANSITION_FRAMES, type PhraseTiming } from '@cfd-video/shared/timing';
export type SceneTiming = Timing<SegmentId>;
