// A cut: its scenes in sequence, each sentence's audio at its cue, and burned-in captions when asked.
import React from "react";
import { AbsoluteFill, Audio, Sequence, staticFile, useCurrentFrame } from "remotion";
import { C } from "./tokens";
import { type Cut, type SceneSpec } from "./data";
import { Run } from "./scenes/Run";
import { Hook } from "./scenes/Hook";
import { WorldIntro } from "./scenes/WorldIntro";
import { Twist } from "./scenes/Twist";
import { Campaign } from "./scenes/Campaign";
import { Close } from "./scenes/Close";
import { Teaser } from "./scenes/Teaser";
import { LineCard } from "./scenes/LineCard";
import { Caption, Frame1080 } from "./views/Hud";

const Scene: React.FC<{ spec: SceneSpec }> = ({ spec }) => {
  switch (spec.name) {
    case "hook":
      return <Hook spec={spec} />;
    case "world":
      return <WorldIntro spec={spec} />;
    case "run1":
    case "run2":
    case "run3":
    case "run4":
      return <Run spec={spec} name={spec.name} />;
    case "twist":
      return <Twist spec={spec} />;
    case "campaign":
      return <Campaign spec={spec} />;
    case "close":
      return <Close spec={spec} />;
    case "teaser":
      return <Teaser spec={spec} />;
    default:
      return <LineCard spec={spec} />;
  }
};

export const CutVideo: React.FC<{ cut: Cut; captions: boolean }> = ({ cut, captions }) => {
  const frame = useCurrentFrame();
  const current = cut.scenes
    .flatMap((s) => s.cues.map((c) => ({ ...c, from: s.from + c.from, scene: s.name })))
    .find((c) => frame >= c.from && frame < c.from + c.durationInFrames + 8);
  const showCaption = captions && current && current.scene !== "hookCard" && current.scene !== "endCard";
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      {cut.scenes.map((s) => (
        <Sequence key={`${s.name}${s.from}`} from={s.from} durationInFrames={s.durationInFrames} name={s.name}>
          <Scene spec={s} />
          {s.cues.map((c) => (
            <Sequence key={c.file} from={c.from} durationInFrames={c.durationInFrames + 2} name={c.text.slice(0, 32)}>
              <Audio src={staticFile(c.file)} />
            </Sequence>
          ))}
        </Sequence>
      ))}
      {showCaption && (
        <Frame1080>
          <Caption text={current.text} />
        </Frame1080>
      )}
    </AbsoluteFill>
  );
};
