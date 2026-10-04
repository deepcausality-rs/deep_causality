/**
 * The main cut: the eight scenes in order, each one narration segment long, joined by short fades.
 */
import { linearTiming, TransitionSeries } from '@remotion/transitions';
import { fade } from '@remotion/transitions/fade';
import { AbsoluteFill, useVideoConfig } from 'remotion';
import type { Retro } from './data/retro';
import { Beliefs } from './scenes/Beliefs';
import { Close } from './scenes/Close';
import { Coast } from './scenes/Coast';
import { Entry } from './scenes/Entry';
import { Fork } from './scenes/Fork';
import { Landing } from './scenes/Landing';
import { Pause } from './scenes/Pause';
import { Plan } from './scenes/Plan';
import { sceneTimings, TRANSITION_FRAMES, type SceneTiming } from './timeline';
import { color } from '@cfd-video/shared';

const SCENES: Record<SceneTiming['id'], React.FC<{ retro: Retro; timing: SceneTiming }>> = {
  entry: Entry,
  plan: Plan,
  coast: Coast,
  pause: Pause,
  fork: Fork,
  landing: Landing,
  beliefs: Beliefs,
  close: Close,
};

export const Main: React.FC<{ retro?: Retro }> = ({ retro }) => {
  const { fps } = useVideoConfig();
  if (!retro) return <AbsoluteFill style={{ background: color.bg0 }} />;
  const timings = sceneTimings(fps);
  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <TransitionSeries>
        {timings.flatMap((t, i) => {
          const Scene = SCENES[t.id];
          const seq = (
            <TransitionSeries.Sequence key={t.id} durationInFrames={t.duration}>
              <Scene retro={retro} timing={t} />
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
export const SceneOnly: React.FC<{ retro?: Retro; id: SceneTiming['id'] }> = ({ retro, id }) => {
  const { fps } = useVideoConfig();
  if (!retro) return <AbsoluteFill style={{ background: color.bg0 }} />;
  const t = sceneTimings(fps).find((x) => x.id === id)!;
  const Scene = SCENES[id];
  return <Scene retro={retro} timing={t} />;
};
