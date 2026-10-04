/**
 * Scene 6, the close. First the result at one scale on both axes, from the branch ends: flying
 * uncorrected ends 20 m from the aim, the committed branch 2.07 m, inside the position uncertainty
 * the navigation filter reports for itself when the branches are scored. Then the run condensed to
 * one line, the thirteen gate lines as printed, and the end card with the tutorial address and the
 * command that reproduces every number in the cut.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Corridor } from '../data/corridor';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, fade, font, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Close: React.FC<{ corridor: Corridor; timing: SceneTiming }> = ({ corridor: c, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const draw = (from: number, to: number) => interpolate(frame, [from, to], [0, 1], { ...clamp, easing: ease });
  const result = fade(frame, at(ph, 0, 0), at(ph, 0, 0.1)) * (1 - fade(frame, at(ph, 2, 0), at(ph, 2, 0.12)));
  const uncorrected = draw(at(ph, 0, 0.15), at(ph, 0, 0.55));
  const corrected = fade(frame, at(ph, 1, 0.05), at(ph, 1, 0.2));
  const ring = draw(at(ph, 1, 0.45), at(ph, 1, 0.75));
  const run = fade(frame, at(ph, 2, 0), at(ph, 2, 0.12));
  const line = draw(at(ph, 2, 0), at(ph, 2, 0.35));
  const gatesIn = (i: number) => fade(frame, at(ph, 2, 0.1) + i * 3, at(ph, 2, 0.1) + i * 3 + 10);
  const card = fade(frame, at(ph, 2, 0.8), at(ph, 2, 1));
  const passed = c.gates.filter((g) => g.passed).length;

  // The result in the end plane, metres at S px each on both axes; the no-bank end is the origin.
  const S = 40;
  const X0 = 440;
  const Y0 = 560;
  const won = c.branches.find((b) => b.round === 2 && b.bank === c.committed.bank)!;
  const aim = { x: X0 + c.aimOffsetM * S, y: Y0 };
  const end = { x: X0 + won.end.cross * S, y: Y0 + won.end.drop * S };
  const sigma = c.scoring.navSigma * S;

  // The condensed run: entry, pause, the fan, commit, GPS back.
  const y = 300;
  const x = (frac: number) => 160 + frac * 1600;
  const pauseF = c.pause.t / c.endT;
  const exitF = c.exit.t / c.endT;
  const N = c.branches.length;

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 50% 35%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        {/* The result. */}
        <div style={{ opacity: result }}>
          <Eyebrow text="DeepCausality · the result" />
          <div style={{ position: 'absolute', left: 160, top: 170, fontFamily: font.sans, fontSize: 60, fontWeight: 500, letterSpacing: '-0.01em', color: color.fg0 }}>
            <span style={{ opacity: uncorrected }}>Without GPS: {c.noBankMiss.toFixed(1)} m</span>
            <span style={{ opacity: corrected, color: color.accent }}> → {c.committed.miss.toFixed(2)} m</span>
          </div>
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            <g opacity={uncorrected}>
              <line x1={X0} y1={Y0 - 70} x2={aim.x} y2={Y0 - 70} stroke={color.fg2} strokeWidth={1.5} />
              <line x1={X0} y1={Y0 - 80} x2={X0} y2={Y0 - 60} stroke={color.fg2} strokeWidth={1.5} />
              <line x1={aim.x} y1={Y0 - 80} x2={aim.x} y2={Y0 - 60} stroke={color.fg2} strokeWidth={1.5} />
              <circle cx={X0} cy={Y0} r={8} fill={color.fg1} />
              <circle cx={aim.x} cy={aim.y} r={13} fill="none" stroke={color.fg0} strokeWidth={2.5} />
            </g>
            <line x1={X0} y1={Y0} x2={aim.x} y2={aim.y} stroke={color.fg1} strokeWidth={2} strokeDasharray="6 8" opacity={uncorrected * 0.8} />
            <g opacity={corrected}>
              <line x1={end.x} y1={end.y} x2={aim.x} y2={aim.y} stroke={color.accent} strokeWidth={3} />
              <circle cx={end.x} cy={end.y} r={9} fill={color.accent} />
            </g>
            <circle
              cx={end.x}
              cy={end.y}
              r={sigma}
              fill="none"
              stroke={color.accent}
              strokeWidth={2}
              opacity={0.75}
              transform={`rotate(-90 ${end.x} ${end.y})`}
              pathLength={1}
              strokeDasharray={1}
              strokeDashoffset={1 - ring}
            />
          </svg>
          <div style={{ position: 'absolute', left: (X0 + aim.x) / 2 - 200, top: Y0 - 118, width: 400, textAlign: 'center', fontFamily: font.mono, fontSize: 21, color: color.fg1, opacity: uncorrected }}>
            {c.noBankMiss.toFixed(1)} m
          </div>
          <div style={{ position: 'absolute', left: X0 - 120, top: Y0 + 22, width: 240, textAlign: 'center', whiteSpace: 'nowrap', fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: uncorrected }}>
            0° · flying uncorrected
          </div>
          <div style={{ position: 'absolute', left: aim.x + 22, top: aim.y - 40, fontFamily: font.mono, fontSize: 21, color: color.fg0, opacity: uncorrected }}>aim</div>
          <div style={{ position: 'absolute', left: end.x - 150, top: end.y + 22, width: 300, textAlign: 'center', fontFamily: font.mono, fontSize: 21, color: color.accent, opacity: corrected }}>
            {c.committed.bank}° · {c.committed.miss.toFixed(2)} m
          </div>
          <div style={{ position: 'absolute', left: end.x + sigma + 24, top: end.y - 30, width: 360, fontFamily: font.mono, fontSize: 20, color: color.accent, opacity: ring }}>
            navigation filter, 1σ: {c.scoring.navSigma.toFixed(1)} m
            <div style={{ fontSize: 17, color: color.fg2 }}>the vehicle's own position uncertainty when the branches are scored</div>
          </div>
          <div style={{ position: 'absolute', right: 96, top: 196, fontFamily: font.mono, fontSize: 16, color: color.fg2, opacity: uncorrected }}>
            end of each branch, {c.branchSeconds.toFixed(0)} s after the pause · one scale on both axes
          </div>
        </div>

        <div style={{ opacity: run * (1 - card) }}>
          <Eyebrow text="DeepCausality · the corridor, end to end" />
          <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0 }}>
            <line x1={x(0)} y1={y} x2={x(1)} y2={y} stroke={color.accent} strokeWidth={3} pathLength={1} strokeDasharray={1} strokeDashoffset={1 - line} />
            <line x1={x(pauseF)} y1={y} x2={x(exitF)} y2={y} stroke={color.warn} strokeWidth={6} opacity={0.35 * line} />
            {Array.from({ length: N }, (_, k) => (
              <path
                key={k}
                d={`M${x(pauseF)},${y} C${x(pauseF) + 60},${y} ${x(pauseF) + 110},${y - 10 - k * 7} ${x(pauseF) + 180},${y - 10 - k * 7}`}
                fill="none"
                stroke={color.accent}
                strokeOpacity={0.45}
                strokeWidth={1.4}
                pathLength={1}
                strokeDasharray={1}
                strokeDashoffset={1 - interpolate(line, [0.25, 0.6], [0, 1], clamp)}
              />
            ))}
            {[0, pauseF, exitF, 1].map((f) => (
              <circle key={f} cx={x(f)} cy={y} r={8} fill={color.accent} opacity={line} />
            ))}
          </svg>
          {[
            { f: 0, label: `entry, ${c.entry.altitudeKm.toFixed(0)} km` },
            { f: pauseF, label: `pause, ${c.pause.altitudeKm.toFixed(1)} km` },
            { f: exitF, label: `GPS back, ${c.exit.altitudeKm.toFixed(1)} km` },
          ].map((l) => (
            <div key={l.label} style={{ position: 'absolute', left: x(l.f) - 10, top: y + 22, fontFamily: font.mono, fontSize: 19, color: color.fg1, opacity: line }}>
              {l.label}
            </div>
          ))}
          <div style={{ position: 'absolute', left: x(pauseF) + 200, top: y - 150, fontFamily: font.mono, fontSize: 19, color: color.accent, opacity: line }}>
            {N} branches, one committed: {c.committed.bank}°
          </div>

          <div style={{ position: 'absolute', left: 160, top: 430, width: 1600, display: 'grid', gridTemplateColumns: '1fr 1fr', columnGap: 64 }}>
            {c.gates.map((g, i) => (
              <div
                key={g.label}
                style={{
                  display: 'flex',
                  gap: 18,
                  padding: '9px 0',
                  borderBottom: `1px solid ${color.line1}`,
                  fontFamily: font.mono,
                  fontSize: 21,
                  color: color.fg1,
                  opacity: gatesIn(i),
                }}
              >
                <span style={{ color: g.passed ? color.ok : '#f47174', minWidth: 64 }}>{g.passed ? 'PASS' : 'FAIL'}</span>
                <span>{g.label}</span>
              </div>
            ))}
          </div>
          <div style={{ position: 'absolute', left: 160, top: 840, fontFamily: font.mono, fontSize: 20, color: color.fg2, opacity: gatesIn(c.gates.length) }}>
            {passed} of {c.gates.length} gates pass · copied from output.txt
          </div>
        </div>

        {/* The end card. */}
        <div style={{ position: 'absolute', left: 160, top: 300, opacity: card }}>
          <div style={{ fontFamily: font.mono, fontSize: 24, letterSpacing: '0.16em', textTransform: 'uppercase', color: color.accent }}>DeepCausality CFD</div>
          <div style={{ marginTop: 24, fontFamily: font.sans, fontSize: 84, fontWeight: 500, letterSpacing: '-0.02em', color: color.fg0, lineHeight: 1.05 }}>
            Seventeen counterfactual trajectories,
            <br />
            one shared past.
          </div>
          <div style={{ marginTop: 40, fontFamily: font.mono, fontSize: 30, color: color.fg0 }}>cfd.deepcausality.com/tutorial/stage-1-corridor</div>
          <div style={{ marginTop: 20, fontFamily: font.mono, fontSize: 23, color: color.fg1 }}>
            cargo run --release -p avionics_examples --example plasma_blackout_corridor
          </div>
          <div style={{ marginTop: 14, fontFamily: font.mono, fontSize: 19, color: color.fg2 }}>
            the whole run: {c.runSeconds.toFixed(1)} s on an Apple M3 Max laptop · open source, in Rust
          </div>
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
