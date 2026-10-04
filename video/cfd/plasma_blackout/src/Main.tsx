/**
 * The main cut: the six scenes in order, each one narration segment long, joined by short fades.
 */
import { linearTiming, TransitionSeries } from '@remotion/transitions';
import { fade } from '@remotion/transitions/fade';
import { AbsoluteFill, useVideoConfig } from 'remotion';
import type { Corridor } from './data/corridor';
import { Close } from './scenes/Close';
import { FlyThrough } from './scenes/FlyThrough';
import { Fork } from './scenes/Fork';
import { Pause } from './scenes/Pause';
import { Problem } from './scenes/Problem';
import { Question } from './scenes/Question';
import { sceneTimings, TRANSITION_FRAMES, type SceneTiming } from './timeline';
import { color, Music } from '@cfd-video/shared';

const SCENES: Record<SceneTiming['id'], React.FC<{ corridor: Corridor; timing: SceneTiming }>> = {
  problem: Problem,
  question: Question,
  pause: Pause,
  fork: Fork,
  'fly-through': FlyThrough,
  close: Close,
};

export const Main: React.FC<{ corridor?: Corridor }> = ({ corridor }) => {
  const { fps } = useVideoConfig();
  if (!corridor) return <AbsoluteFill style={{ background: color.bg0 }} />;
  const timings = sceneTimings(fps);
  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <Music />
      <TransitionSeries>
        {timings.flatMap((t, i) => {
          const Scene = SCENES[t.id];
          const seq = (
            <TransitionSeries.Sequence key={t.id} durationInFrames={t.duration}>
              <Scene corridor={corridor} timing={t} />
            </TransitionSeries.Sequence>
          );
          return i < timings.length - 1
            ? [seq, <TransitionSeries.Transition key={`${t.id}-fade`} presentation={fade()} timing={linearTiming({ durationInFrames: TRANSITION_FRAMES })} />]
            : [seq];
        })}
      </TransitionSeries>
    </AbsoluteFill>
  );
};

/** One scene alone, for review. */
export const SceneOnly: React.FC<{ corridor?: Corridor; id: SceneTiming['id'] }> = ({ corridor, id }) => {
  const { fps } = useVideoConfig();
  if (!corridor) return <AbsoluteFill style={{ background: color.bg0 }} />;
  const t = sceneTimings(fps).find((x) => x.id === id)!;
  const Scene = SCENES[id];
  return <Scene corridor={corridor} timing={t} />;
};
