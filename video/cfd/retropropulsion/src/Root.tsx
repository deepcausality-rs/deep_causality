/**
 * The compositions: the main cut, and each scene alone for review. Each loads the retropropulsion run
 * in `calculateMetadata`, so the number checks, on the graphics and on the captions, run before the
 * first frame and a mismatch fails the render.
 */
import { loadFont } from '@remotion/fonts';
import { Composition, Folder, staticFile } from 'remotion';
import { checkCaptions } from './data/captions';
import { loadRetro } from './data/retro';
import { Main, SceneOnly } from './Main';
import { segments } from './script';
import { mainFrames, sceneTimings } from './timeline';
import { checkMusic, render } from '@cfd-video/shared';

loadFont({ family: 'Geist', url: staticFile('fonts/geist-latin.woff2'), weight: '100 900' });
loadFont({ family: 'JetBrains Mono', url: staticFile('fonts/jetbrains-mono-latin.woff2'), weight: '100 800' });

const withRetro = async <P extends object>({ props }: { props: P }) => {
  const retro = await loadRetro();
  checkCaptions(retro);
  return { props: { ...props, retro } };
};

export const Root: React.FC = () => (
  <>
    <Composition
      id="Main"
      component={Main}
      width={render.width}
      height={render.height}
      fps={render.fps}
      durationInFrames={mainFrames(render.fps)}
      defaultProps={{}}
      calculateMetadata={async (input) => {
        await checkMusic(mainFrames(render.fps) / render.fps, render.fps);
        return withRetro(input);
      }}
    />
    <Folder name="Scenes">
      {segments.map((s) => (
        <Composition
          key={s.id}
          id={`Scene-${s.id}`}
          component={SceneOnly}
          width={render.width}
          height={render.height}
          fps={render.fps}
          durationInFrames={sceneTimings(render.fps).find((t) => t.id === s.id)!.duration}
          defaultProps={{ id: s.id }}
          calculateMetadata={withRetro}
        />
      ))}
    </Folder>
  </>
);
