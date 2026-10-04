/**
 * The main cut: the seven scenes in order, each one narration segment long, joined by short fades.
 */
import { linearTiming, TransitionSeries } from '@remotion/transitions';
import { fade } from '@remotion/transitions/fade';
import { AbsoluteFill, useVideoConfig } from 'remotion';
import type { Weather } from './data/weather';
import { Close } from './scenes/Close';
import { Drift } from './scenes/Drift';
import { Entry } from './scenes/Entry';
import { Resolved } from './scenes/Resolved';
import { Why } from './scenes/Why';
import { Window } from './scenes/Window';
import { Worlds } from './scenes/Worlds';
import { sceneTimings, TRANSITION_FRAMES, type SceneTiming } from './timeline';
import { color } from '@cfd-video/shared';

const SCENES: Record<SceneTiming['id'], React.FC<{ weather: Weather; timing: SceneTiming }>> = {
  entry: Entry,
  worlds: Worlds,
  window: Window,
  drift: Drift,
  why: Why,
  resolved: Resolved,
  close: Close,
};

export const Main: React.FC<{ weather?: Weather }> = ({ weather }) => {
  const { fps } = useVideoConfig();
  if (!weather) return <AbsoluteFill style={{ background: color.bg0 }} />;
  const timings = sceneTimings(fps);
  return (
    <AbsoluteFill style={{ background: color.bg0 }}>
      <TransitionSeries>
        {timings.flatMap((t, i) => {
          const Scene = SCENES[t.id];
          const seq = (
            <TransitionSeries.Sequence key={t.id} durationInFrames={t.duration}>
              <Scene weather={weather} timing={t} />
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
export const SceneOnly: React.FC<{ weather?: Weather; id: SceneTiming['id'] }> = ({ weather, id }) => {
  const { fps } = useVideoConfig();
  if (!weather) return <AbsoluteFill style={{ background: color.bg0 }} />;
  const t = sceneTimings(fps).find((x) => x.id === id)!;
  const Scene = SCENES[id];
  return <Scene weather={weather} timing={t} />;
};
