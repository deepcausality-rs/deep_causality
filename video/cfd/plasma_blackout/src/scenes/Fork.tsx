/**
 * Scene 4, the fork: the seventeen branches after the pause, drawn from the branch trace, then
 * where each one ends against the aim. Each step lands on its narration phrase: the axes, the
 * coarse round, the clamped 40° branch, the coarse winner, the fine round, the end arc, the result.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Corridor } from '../data/corridor';
import type { SceneTiming } from '../timeline';
import { Captions, color, Eyebrow, Stage } from '@cfd-video/shared';
import { BranchFan } from '../views/BranchFan';
import { EndArc } from '../views/EndArc';

const clamp01 = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Fork: React.FC<{ corridor: Corridor; timing: SceneTiming }> = ({ corridor, timing }) => {
  const frame = useCurrentFrame();
  const [ahead, roundOne, clamp, best, roundTwo, arc, result] = timing.phrases.map((p) => ({ start: p.start, end: p.start + p.length }));
  const span = (r: { start: number; end: number }, from = 0, to = 1) =>
    interpolate(frame, [r.start + (r.end - r.start) * from, r.start + (r.end - r.start) * to], [0, 1], { ...clamp01, easing: ease });

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 30% 40%, ${color.bg2} 0%, ${color.bg0} 70%)` }}>
      <Stage>
        <Eyebrow text={`DeepCausality · ${corridor.branches.length} counterfactual trajectories`} />
        <BranchFan
          corridor={corridor}
          x={96}
          y={190}
          w={900}
          h={620}
          p={{
            axes: span(ahead, 0, 0.4),
            roundOne: span(roundOne, 0, 0.9),
            clamp: span(clamp, 0.1, 0.4),
            best: span(best, 0, 0.3),
            roundTwo: span(roundTwo, 0.15, 1),
            committed: span(result, 0, 0.4),
          }}
        />
        <EndArc
          corridor={corridor}
          x={1110}
          y={190}
          w={720}
          p={{ points: span(arc, 0, 0.6), arc: span(arc, 0.4, 1), miss: span(result, 0.1, 0.5), result: span(result, 0.4, 0.75) }}
        />
        <Captions phrases={timing.phrases} />
      </Stage>
    </AbsoluteFill>
  );
};
