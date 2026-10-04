// The explainer: a title, the setting, the textbook fail-safe and the issue with its static rule, the
// dynamic fail-safe of the Effect Ethos, the evidence over 1,000 simulations, the three steps, and the
// end screen. Scenes play in order and fade at their edges; there is no audio.
import React from "react";
import { AbsoluteFill, interpolate, Sequence, useCurrentFrame } from "remotion";
import { C } from "./tokens";
import { Title, TITLE_FRAMES } from "./scenes/Title";
import { Intro, INTRO_FRAMES } from "./scenes/Intro";
import { Part1Card, PART1_CARD_FRAMES, Part2Card, PART2_CARD_FRAMES } from "./scenes/Chapter";
import { EthosFlight, ETHOS_FRAMES, TextbookFlight, TEXTBOOK_FRAMES } from "./scenes/Flight";
import { StaticRule, STATIC_RULE_FRAMES } from "./scenes/StaticRule";
import { ThousandNights, THOUSAND_FRAMES } from "./scenes/ThousandNights";
import { Summary, SUMMARY_FRAMES } from "./scenes/Summary";
import { End, END_FRAMES } from "./scenes/End";

export const SCENES = {
  title: { component: Title, frames: TITLE_FRAMES },
  setting: { component: Intro, frames: INTRO_FRAMES },
  part1: { component: Part1Card, frames: PART1_CARD_FRAMES },
  textbook: { component: TextbookFlight, frames: TEXTBOOK_FRAMES },
  issue: { component: StaticRule, frames: STATIC_RULE_FRAMES },
  part2: { component: Part2Card, frames: PART2_CARD_FRAMES },
  ethos: { component: EthosFlight, frames: ETHOS_FRAMES },
  simulations: { component: ThousandNights, frames: THOUSAND_FRAMES },
  summary: { component: Summary, frames: SUMMARY_FRAMES },
  end: { component: End, frames: END_FRAMES },
};
export type SceneName = keyof typeof SCENES;

const FADE = 12;

const Faded: React.FC<{ frames: number; children: React.ReactNode }> = ({ frames, children }) => {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, FADE, frames - FADE, frames], [0, 1, 1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return <AbsoluteFill style={{ opacity }}>{children}</AbsoluteFill>;
};

export const totalFrames = (scenes: SceneName[]) => scenes.reduce((n, s) => n + SCENES[s].frames, 0);

export const Explainer: React.FC<{ scenes: SceneName[] }> = ({ scenes }) => {
  let from = 0;
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      {scenes.map((name) => {
        const { component: Scene, frames } = SCENES[name];
        const at = from;
        from += frames;
        return (
          <Sequence key={name} from={at} durationInFrames={frames} name={name}>
            <Faded frames={frames}>
              <Scene />
            </Faded>
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};
