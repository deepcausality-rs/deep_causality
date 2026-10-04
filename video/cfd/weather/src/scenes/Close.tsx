/**
 * Scene 7, the close. First the two answers, each from the table: the six blackout windows on one
 * clock, and polar winter's drift against the standard day's. Then the campaign condensed, the eight
 * gate lines as printed, and the end card with the tutorial address and the command that reproduces
 * every number in the cut.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Weather } from '../data/weather';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, EndCard, Eyebrow, fade, font, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Close: React.FC<{ weather: Weather; timing: SceneTiming }> = ({ weather: w, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const draw = (from: number, to: number) => interpolate(frame, [from, to], [0, 1], { ...clamp, easing: ease });
  const base = w.worlds[0];
  const polar = w.worlds.find((x) => x.name === 'polar_winter')!;

  const results = fade(frame, at(ph, 0, 0), at(ph, 0, 0.08)) * (1 - fade(frame, at(ph, 2, 0), at(ph, 2, 0.12)));
  const one = draw(at(ph, 0, 0.1), at(ph, 0, 0.6));
  const two = draw(at(ph, 1, 0.1), at(ph, 1, 0.6));
  // One layer at a time: the answers leave before the campaign arrives, the campaign before the end card.
  const run = fade(frame, at(ph, 2, 0.08), at(ph, 2, 0.16)) * (1 - fade(frame, at(ph, 2, 0.72), at(ph, 2, 0.8)));
  const gatesIn = (i: number) => fade(frame, at(ph, 2, 0.16) + i * 3, at(ph, 2, 0.16) + i * 3 + 10);
  const card = fade(frame, at(ph, 2, 0.82), at(ph, 2, 0.95));
  const passed = w.gates.filter((g) => g.passed).length;

  // Answer one: the six windows on one clock.
  const A = { x: 160, y: 360, w: 640, rowH: 44 };
  const ax = (t: number) => A.x + (t / w.flightS) * A.w * one;
  const windows = [...w.worlds].sort((a, b) => b.rho - a.rho);
  // Answer two: the two drifts on one metre scale.
  const B = { x: 1100, y: 330, h: 380, max: Math.ceil(polar.driftMean / 10) * 10 + 10 };
  const by = (m: number) => B.y + B.h - (m / B.max) * B.h * two;

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 50% 35%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <div style={{ opacity: results }}>
          <Eyebrow text="DeepCausality · the answers" />
          <div style={{ position: 'absolute', left: A.x, top: 200, fontFamily: font.sans, fontSize: 36, fontWeight: 500, color: color.fg0, opacity: one }}>
            1 · Does the weather move the blackout?
            <div style={{ marginTop: 8, fontFamily: font.mono, fontSize: 21, fontWeight: 400, color: color.fg1 }}>
              it shifts by {w.spreads.onset.toFixed(1)} s; its length moves {w.spreads.dwell.toFixed(1)} s
            </div>
          </div>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0, opacity: one }}>
            {windows.map((x, k) => (
              <rect key={x.name} x={ax(x.onset)} y={A.y + k * A.rowH} width={ax(x.exit) - ax(x.onset)} height={A.rowH - 16} fill={color.warn} opacity={0.4} />
            ))}
            {windows.map((x, k) => (
              <text key={`l${x.name}`} x={ax(x.exit) + 14} y={A.y + k * A.rowH + 21} fontFamily={font.mono} fontSize={17} fill={color.fg2}>
                {x.label}
              </text>
            ))}
            <line x1={A.x} y1={A.y + windows.length * A.rowH} x2={A.x + A.w} y2={A.y + windows.length * A.rowH} stroke={color.line2} strokeWidth={2} />
          </svg>
          <div style={{ position: 'absolute', left: A.x, top: A.y + windows.length * A.rowH + 14, width: A.w, fontFamily: font.mono, fontSize: 18, color: color.fg2, opacity: one }}>
            GPS lost, six worlds, 0 to {w.flightS.toFixed(0)} s of flight
          </div>

          <div style={{ position: 'absolute', left: B.x, top: 200, fontFamily: font.sans, fontSize: 36, fontWeight: 500, color: color.fg0, opacity: two }}>
            2 · Does it move the navigation error?
            <div style={{ marginTop: 8, fontFamily: font.mono, fontSize: 21, fontWeight: 400, color: color.fg1 }}>
              polar winter drifts {Math.round((w.coldFactor - 1) * 100)}% further: its bias flies at {polar.departure.toFixed(2)}×
            </div>
          </div>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0, opacity: two }}>
            <line x1={B.x} y1={B.y + B.h} x2={B.x + 560} y2={B.y + B.h} stroke={color.line2} strokeWidth={2} />
            <line x1={B.x + 140} y1={by(base.driftMean)} x2={B.x + 140} y2={B.y + B.h} stroke={color.fg1} strokeWidth={12} />
            <line x1={B.x + 400} y1={by(polar.driftMean)} x2={B.x + 400} y2={B.y + B.h} stroke={color.accent} strokeWidth={12} />
          </svg>
          {[
            { at: 140, x: base, tone: color.fg1 },
            { at: 400, x: polar, tone: color.accent },
          ].map((b) => (
            <div key={b.x.name} style={{ position: 'absolute', left: B.x + b.at - 160, top: by(b.x.driftMean) - 76, width: 320, textAlign: 'center', fontFamily: font.mono, fontSize: 19, color: b.tone, opacity: two }}>
              {b.x.label}
              <div style={{ fontSize: 24 }}>{b.x.driftMean.toFixed(2)} m</div>
            </div>
          ))}
          <div style={{ position: 'absolute', left: B.x, top: B.y + B.h + 20, width: 780, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: two }}>
            largest error while GPS is lost, mean of {w.drawsPerWorld} draws · {w.resolved.ratio.toFixed(1)} sigma apart
          </div>
        </div>

        <div style={{ opacity: run }}>
          <Eyebrow text="DeepCausality · the campaign, end to end" />
          <div style={{ position: 'absolute', left: 160, top: 220, fontFamily: font.sans, fontSize: 40, fontWeight: 500, color: color.fg0 }}>
            {w.worlds.length} worlds × {w.drawsPerWorld} draws = <span style={{ color: color.accent }}>{w.worlds.length * w.drawsPerWorld} counterfactual descents</span>, one table
          </div>
          <div style={{ position: 'absolute', left: 160, top: 330, width: 1600, display: 'grid', gridTemplateColumns: '1fr', columnGap: 64 }}>
            {w.gates.map((g, i) => (
              <div
                key={g.label}
                style={{ display: 'flex', gap: 18, padding: '8px 0', borderBottom: `1px solid ${color.line1}`, fontFamily: font.mono, fontSize: 21, color: color.fg1, opacity: gatesIn(i) }}
              >
                <span style={{ color: g.passed ? color.ok : '#f47174', minWidth: 64 }}>{g.passed ? 'PASS' : 'FAIL'}</span>
                <span>{g.label}</span>
              </div>
            ))}
          </div>
          <div style={{ position: 'absolute', left: 160, top: 760, fontFamily: font.mono, fontSize: 20, color: color.fg2, opacity: gatesIn(w.gates.length) }}>
            {passed} of {w.gates.length} gates pass · copied from output.txt
          </div>
        </div>

        <EndCard
          line="Six counterfactual atmospheres, one table."
          url="cfd.deepcausality.com/tutorial/stage-2-weather"
          command="cargo run --release -p avionics_examples --example plasma_blackout_weather"
          note={`the whole campaign: ${w.runSeconds.toFixed(1)} s on an Apple M3 Max laptop · open source, in Rust`}
          opacity={card}
        />
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
