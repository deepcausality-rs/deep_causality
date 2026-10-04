/**
 * Narration and timing, shared by every cut: a scene per narration segment, each phrase given its
 * own frames at the narration pace, with a lead before the first phrase and a tail after the last.
 * A project's main composition, captions and SRT all read the one table `makeTimeline` returns, so
 * a re-timed line moves all three.
 *
 * Free of React and three.js, so scripts run under plain Node import it.
 */
import { WORDS_PER_MINUTE } from './tokens';

export interface Phrase {
  /** The caption, on screen and in the SRT. Numbers are numerals; `10^19` sets an exponent. */
  text: string;
  /** The spoken line, where it differs from the caption. Its word count sets the phrase's length. */
  say?: string;
}

export interface Segment<Id extends string> {
  id: Id;
  /** Phrases, shown in order; together they are the spoken line. */
  phrases: Phrase[];
}

export interface PhraseTiming {
  text: string;
  /** Start within the scene, frames. */
  start: number;
  length: number;
}

export interface SceneTiming<Id extends string> {
  id: Id;
  /** Start within the main cut, frames (after the overlaps of earlier transitions). */
  from: number;
  duration: number;
  phrases: PhraseTiming[];
}

/** A short line still gets this long on screen. */
const MIN_PHRASE_S = 1.8;
/** Frames two adjacent scenes overlap while one fades into the next. */
export const TRANSITION_FRAMES = 18;

const words = (s: string) => s.split(/\s+/).filter(Boolean).length;

/** Seconds a phrase takes at the narration pace, counted on the spoken line. */
export const phraseSeconds = (p: Phrase) => (words(p.say ?? p.text) / WORDS_PER_MINUTE) * 60;

/** Seconds a segment takes at the narration pace. */
export const segmentSeconds = <Id extends string>(s: Segment<Id>) => s.phrases.reduce((a, p) => a + phraseSeconds(p), 0);

/**
 * The timing of a cut: `lead` and `tail` are the seconds before each scene's first phrase and after
 * its last. Throws on a cut with no segments, or a segment without a finite, non-negative lead and
 * tail.
 */
export function makeTimeline<Id extends string>(segments: Segment<Id>[], lead: Record<Id, number>, tail: Record<Id, number>) {
  if (segments.length === 0) throw new Error('makeTimeline: the cut has no narration segments');
  for (const s of segments) {
    for (const [name, table] of [['lead', lead], ['tail', tail]] as const) {
      const seconds = table[s.id];
      if (!Number.isFinite(seconds) || seconds < 0) throw new Error(`makeTimeline: segment '${s.id}' has ${name} ${seconds}; expected seconds ≥ 0`);
    }
  }
  const sceneTimings = (fps: number): SceneTiming<Id>[] => {
    let from = 0;
    return segments.map((s, i) => {
      let at = Math.round(lead[s.id] * fps);
      const phrases = s.phrases.map((phrase) => {
        const length = Math.round(Math.max(MIN_PHRASE_S, phraseSeconds(phrase)) * fps);
        const p = { text: phrase.text, start: at, length };
        at += length;
        return p;
      });
      const duration = at + Math.round(tail[s.id] * fps);
      const scene = { id: s.id, from, duration, phrases };
      from += duration - (i < segments.length - 1 ? TRANSITION_FRAMES : 0);
      return scene;
    });
  };
  const mainFrames = (fps: number) => {
    const scenes = sceneTimings(fps);
    const last = scenes[scenes.length - 1];
    return last.from + last.duration;
  };
  const scene = (fps: number, id: Id) => {
    const s = sceneTimings(fps).find((x) => x.id === id);
    if (!s) throw new Error(`no scene ${id}`);
    return s;
  };
  const segment = (id: Id) => {
    const s = segments.find((x) => x.id === id);
    if (!s) throw new Error(`no narration segment ${id}`);
    return s;
  };
  return { sceneTimings, mainFrames, scene, segment };
}
