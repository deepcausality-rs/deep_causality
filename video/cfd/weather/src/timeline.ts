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
  worlds: 0.8,
  window: 1,
  drift: 1,
  why: 0.8,
  resolved: 1,
  close: 0.6,
};
const TAIL_S: Record<SegmentId, number> = {
  entry: 0.8,
  worlds: 1.6,
  window: 2,
  drift: 1.6,
  why: 2,
  resolved: 2,
  close: 5.5,
};

export const { sceneTimings, mainFrames, scene, segment } = makeTimeline(segments, LEAD_S, TAIL_S);
export { TRANSITION_FRAMES, type PhraseTiming } from '@cfd-video/shared/timing';
export type SceneTiming = Timing<SegmentId>;
