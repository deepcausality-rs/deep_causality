/**
 * Scene 7, the two landings: the landing leg flown twice from the same state, by a guidance sized
 * for today's dispersion and by one sized for a standard day. Height above the touchdown plane
 * against flight time, from `retropropulsion_trace.csv` and `retropropulsion_uninformed_trace.csv`; the
 * burn-light altitudes, contact speeds and the propellant difference are the run's printed belief
 * table, which the traces reproduce.
 */
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from 'remotion';
import type { Landing, Retro } from '../data/retro';
import type { SceneTiming } from '../timeline';
import { at, Captions, color, Eyebrow, font, Stage } from '@cfd-video/shared';

const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const Beliefs: React.FC<{ retro: Retro; timing: SceneTiming }> = ({ retro: r, timing }) => {
  const frame = useCurrentFrame();
  const ph = timing.phrases;
  const span = (i: number, a = 0, b = 1) => interpolate(frame, [at(ph, i, a), at(ph, i, b)], [0, 1], { ...clamp, easing: ease });
  const inf = r.landings.informed;
  const uni = r.landings.uninformed;

  const C = { x: 160, y: 240, w: 1000, h: 520 };
  const t0 = Math.min(inf.lightT, uni.lightT) - 2;
  const t1 = Math.max(inf.touchdownT, uni.touchdownT) + 0.5;
  const hMax = Math.ceil(Math.max(...[inf, uni].map((l) => l.leg.find((s) => s.t >= t0)!.altitudeKm * 1000 - r.groundM)) / 50) * 50;
  const cx = (t: number) => C.x + ((t - t0) / (t1 - t0)) * C.w;
  const cy = (h: number) => C.y + C.h - (Math.max(0, Math.min(h, hMax)) / hMax) * C.h;
  const height = (l: Landing) => l.leg.filter((s) => s.t >= t0).map((s) => ({ t: s.t, h: s.altitudeKm * 1000 - r.groundM }));
  const path = (l: Landing) => height(l).map((p, i) => `${i === 0 ? 'M' : 'L'}${cx(p.t).toFixed(1)},${cy(p.h).toFixed(1)}`).join(' ');
  const draw = interpolate(frame, [at(ph, 0, 0.2), at(ph, 3, 0.5)], [0, 1], clamp);
  const ticks = Array.from({ length: hMax / 50 + 1 }, (_, i) => i * 50);
  // The printed burn-light altitudes, on the chart's height axis.
  const lightH = (l: Landing) => l.printed.lightM - r.groundM;
  const curves: Array<[Landing, string]> = [
    [uni, color.fg1],
    [inf, color.accent],
  ];
  const markers: Array<[Landing, string]> = [
    [inf, color.accent],
    [uni, color.fg1],
  ];

  const card = (l: Landing, y: number, title: string, tone: string, p: number) => (
    <div style={{ position: 'absolute', left: 1280, top: y, width: 520, opacity: p, fontFamily: font.mono }}>
      <div style={{ fontSize: 21, color: tone }}>{title}</div>
      <div style={{ marginTop: 10, display: 'grid', gridTemplateColumns: '1fr auto', rowGap: 8, fontSize: 21, color: color.fg1 }}>
        <span>ignition margin</span>
        <span style={{ color: tone }}>{l.printed.margin.toFixed(2)} m</span>
        <span style={{ opacity: span(1, 0.2, 0.5) }}>lights its burn at</span>
        <span style={{ color: tone, opacity: span(1, 0.2, 0.5) }}>{l.printed.lightM.toFixed(2)} m</span>
        <span style={{ opacity: span(2, 0.1, 0.4) }}>touches down at</span>
        <span style={{ color: tone, opacity: span(2, 0.1, 0.4) }}>{l.contactMs.toFixed(2)} m/s</span>
      </div>
    </div>
  );

  return (
    <AbsoluteFill style={{ background: `radial-gradient(ellipse at 40% 45%, ${color.bg2} 0%, ${color.bg0} 72%)` }}>
      <Stage>
        <Eyebrow text="Answer 2 · two counterfactual landings" />
        <svg width={1920} height={1080} style={{ position: 'absolute', inset: 0, overflow: 'visible' }}>
          {ticks.map((h) => (
            <line key={h} x1={C.x} y1={cy(h)} x2={C.x + C.w} y2={cy(h)} stroke={color.line1} strokeWidth={1} />
          ))}
          <line x1={C.x} y1={cy(0)} x2={C.x + C.w} y2={cy(0)} stroke={color.line2} strokeWidth={2} />
          {curves.map(([l, tone]) => (
            <path key={tone} d={path(l)} fill="none" stroke={tone} strokeWidth={3} strokeLinecap="round" pathLength={1} strokeDasharray={1} strokeDashoffset={1 - draw} />
          ))}
          {markers.map(([l, tone]) => (
            <g key={`m${tone}`} opacity={span(1, 0.1, 0.4)}>
              <line x1={cx(l.lightT) - 30} y1={cy(lightH(l))} x2={cx(l.lightT) + 30} y2={cy(lightH(l))} stroke={tone} strokeWidth={2} />
              <circle cx={cx(l.lightT)} cy={cy(lightH(l))} r={8} fill={tone} />
            </g>
          ))}
          <g opacity={span(1, 0.4, 0.8)}>
            <line x1={cx(inf.lightT) - 50} y1={cy(lightH(inf))} x2={cx(inf.lightT) - 50} y2={cy(lightH(uni))} stroke={color.fg0} strokeWidth={1.5} />
          </g>
        </svg>
        <div style={{ position: 'absolute', left: C.x - 30, top: C.y - 54, fontFamily: font.mono, fontSize: 19, color: color.fg1 }}>height above the touchdown plane, m</div>
        {ticks.map((h) => (
          <div key={h} style={{ position: 'absolute', left: C.x - 76, top: cy(h) - 12, width: 60, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg2 }}>
            {h}
          </div>
        ))}
        <div style={{ position: 'absolute', left: C.x + C.w - 240, top: C.y + C.h + 16, width: 240, textAlign: 'right', fontFamily: font.mono, fontSize: 18, color: color.fg1 }}>flight time →</div>
        <div style={{ position: 'absolute', left: cx(inf.lightT) - 270, top: (cy(lightH(inf)) + cy(lightH(uni))) / 2 - 14, width: 200, textAlign: 'right', fontFamily: font.mono, fontSize: 21, color: color.fg0, opacity: span(1, 0.5, 0.9) }}>
          {(inf.printed.lightM - uni.printed.lightM).toFixed(2)} m higher
        </div>
        {card(inf, 240, `sized for today, −${Math.abs(r.belief.measuredDT)} K`, color.accent, span(0, 0.3, 0.6))}
        {card(uni, 470, 'sized for a standard day', color.fg1, span(0, 0.5, 0.8))}
        <div style={{ position: 'absolute', left: 1280, top: 700, width: 540, fontFamily: font.sans, fontSize: 30, color: color.fg0, opacity: span(3, 0.1, 0.5) }}>
          The extra {(inf.printed.lightM - uni.printed.lightM).toFixed(2)} m of margin costs <span style={{ color: color.accent }}>{inf.printed.reserveKg.toFixed(2)} kg</span> of propellant.
        </div>
        <Captions phrases={ph} />
      </Stage>
    </AbsoluteFill>
  );
};
