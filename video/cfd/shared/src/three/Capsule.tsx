/**
 * The vehicle the plasma-blackout examples fly, in its own frame, lengths in metres:
 *
 * - The aeroshell diameter follows from the examples' ballistic bundle,
 *   `S_ref = CDA_OVER_M · VEHICLE_MASS_KG / VEHICLE_CD` = 14.09 m², so D = 4.23 m
 *   (`shared/constants.rs`). A central retro nozzle of `NOZZLE_EXIT_R` = 0.42 m sits in the middle of
 *   the heatshield, the configuration the Jarvinen-Adams correlation of the retropulsion example
 *   measures; during entry its lip is all that shows.
 * - The shape takes the Apollo command module's proportions, the lineage `VEHICLE_CD` cites: a
 *   spherical heatshield of radius 1.2 D, a toroidal shoulder of radius 0.05 D, a 33° backshell,
 *   and a height of 0.83 D.
 * - The wall glows by radiative equilibrium: Lees' heating `q/q_stag ≈ cos θ` over the forebody, θ
 *   between the wall normal and the oncoming flow, and an assumed 2 % of the stagnation value on the
 *   backshell, sets the local temperature `T = T_stag · (q/q_stag)^¼`. Colour comes from the
 *   blackbody approximation, brightness from `T⁴`.
 * - The bow shock is optically thin: bright where the line of sight grazes it, nearly clear across
 *   the face. `sheath` scales it with the electron density. Its shape is drawn, not computed.
 * - The wake is the cooler, recombining plasma behind the vehicle, aligned with the flow.
 * - The retro plume leaves the nozzle along the body axis, into the oncoming flow, sized by the
 *   throttle flown; its shape is drawn. Below the plasma regime the bow shock is drawn as a faint
 *   edge, the way a schlieren photograph shows it, standing off ahead of the plume.
 *
 * The group's own frame is the flow frame: the vehicle flies along −y. Inside it the body is tilted
 * by the angle of attack `alpha`, so its lift points up when the bank angle is zero.
 */
import { useMemo } from 'react';
import * as THREE from 'three';

/** The examples' ballistic bundle (`shared/constants.rs`). */
const CDA_OVER_M = 5.8e-3;
const VEHICLE_MASS_KG = 3400;
const VEHICLE_CD = 1.4;
/** Aeroshell diameter, m, from the reference area the bundle implies. */
export const DIAMETER = Math.sqrt((4 * ((CDA_OVER_M * VEHICLE_MASS_KG) / VEHICLE_CD)) / Math.PI);
/** Retro nozzle exit radius, m (`NOZZLE_EXIT_R`). */
const NOZZLE_EXIT_R = 0.42;

// Apollo command-module proportions.
const HEATSHIELD_R = 1.2 * DIAMETER;
const SHOULDER_R = 0.05 * DIAMETER;
const BACKSHELL = (33 * Math.PI) / 180;
const HEIGHT = 0.83 * DIAMETER;

/**
 * Trim angle of attack, rad. The examples fly lift as a point-mass L/D of 0.3 and carry no
 * attitude; the tilt shows where that lift comes from.
 */
export const TRIM_ALPHA = (20 * Math.PI) / 180;

/** Backshell heating as a fraction of the stagnation value (assumed). */
const BACKSHELL_Q = 0.02;

/** The stagnation temperature at the pause, K: the exposure reference for the glow. */
export const T_REF = 2148;
const STEFAN_BOLTZMANN = 5.670374419e-8;
/** Wall emissivity of the radiative-equilibrium wall. */
const EMISSIVITY = 0.85;

/** Stagnation wall temperature from the stagnation heat flux, by radiative equilibrium. */
export const stagnationTemperature = (heatFlux: number) => (heatFlux / (EMISSIVITY * STEFAN_BOLTZMANN)) ** 0.25;

/** The heatshield's widest point: the cap meets the shoulder at this polar angle. */
const CAP_END = Math.asin((DIAMETER / 2 - SHOULDER_R) / (HEATSHIELD_R - SHOULDER_R));
const SHOULDER_C = new THREE.Vector2((HEATSHIELD_R - SHOULDER_R) * Math.sin(CAP_END), HEATSHIELD_R - (HEATSHIELD_R - SHOULDER_R) * Math.cos(CAP_END));

/** The body as lathe points (radius, axis): heatshield apex at y = 0, the top at y = HEIGHT. */
function bodyProfile(): THREE.Vector2[] {
  const pts: THREE.Vector2[] = [];
  for (let i = 0; i <= 32; i++) {
    const a = (i / 32) * CAP_END;
    pts.push(new THREE.Vector2(HEATSHIELD_R * Math.sin(a), HEATSHIELD_R * (1 - Math.cos(a))));
  }
  const end = Math.PI / 2 + BACKSHELL;
  for (let i = 1; i <= 16; i++) {
    const t = CAP_END + ((end - CAP_END) * i) / 16;
    pts.push(new THREE.Vector2(SHOULDER_C.x + SHOULDER_R * Math.sin(t), SHOULDER_C.y - SHOULDER_R * Math.cos(t)));
  }
  const p = pts[pts.length - 1];
  const length = (HEIGHT - p.y) / Math.cos(BACKSHELL);
  for (let i = 1; i <= 20; i++) {
    const f = (i / 20) * length;
    pts.push(new THREE.Vector2(p.x - f * Math.sin(BACKSHELL), p.y + f * Math.cos(BACKSHELL)));
  }
  pts.push(new THREE.Vector2(0, HEIGHT));
  return pts;
}

/** The bow shock as lathe points: a drawn shell standing off the heatshield and trailing back. */
function shockProfile(): THREE.Vector2[] {
  const k = DIAMETER;
  const curve = new THREE.SplineCurve(
    [
      [0, -0.07],
      [0.24, -0.05],
      [0.43, 0.005],
      [0.56, 0.1],
      [0.63, 0.25],
      [0.68, 0.42],
    ].map(([r, y]) => new THREE.Vector2(r * k, y * k))
  );
  return curve.getPoints(48);
}

/** The wake as lathe points in the flow frame: from the shoulder, through a neck, spreading behind. */
function wakeProfile(): THREE.Vector2[] {
  const k = DIAMETER;
  return [
    [0.47, 0.18],
    [0.45, 0.5],
    [0.38, 1.0],
    [0.31, 1.7],
    [0.33, 2.6],
    [0.42, 3.6],
    [0.55, 4.8],
  ].map(([r, y]) => new THREE.Vector2(r * k, y * k));
}
const WAKE_START = 0.18 * DIAMETER;
const WAKE_LENGTH = (4.8 - 0.18) * DIAMETER;

/** The middle of the body, body frame, m: where a camera aims to keep the capsule framed. */
export const BODY_CENTRE = new THREE.Vector3(0, 0.45 * HEIGHT, 0);

/** Where the oncoming flow stagnates on the heatshield at angle of attack `alpha`, body frame, m. */
export function stagnationPoint(alpha = TRIM_ALPHA): THREE.Vector3 {
  const a = Math.min(alpha, CAP_END);
  return new THREE.Vector3(-HEATSHIELD_R * Math.sin(a), HEATSHIELD_R * (1 - Math.cos(a)), 0);
}

const wallVertex = /* glsl */ `
  varying vec3 vNormalW;
  varying vec3 vNormalO;
  varying vec3 vViewDir;
  varying vec3 vPosO;
  void main() {
    vPosO = position;
    vec4 world = modelMatrix * vec4(position, 1.0);
    vNormalO = normal;
    // Normals take the inverse transpose: the shock, which shares this shader, is widened across
    // the axis only.
    vNormalW = normalize(transpose(inverse(mat3(modelMatrix))) * normal);
    vViewDir = normalize(cameraPosition - world.xyz);
    gl_Position = projectionMatrix * viewMatrix * world;
  }
`;

// Blackbody colour after Tanner Helland's fit, valid for 1000–40000 K.
const blackbody = /* glsl */ `
  vec3 blackbody(float T) {
    float t = T / 100.0;
    float g = clamp((99.4708025861 * log(t) - 161.1195681661) / 255.0, 0.0, 1.0);
    float b = t <= 19.0 ? 0.0 : clamp((138.5177312231 * log(t - 10.0) - 305.0447927307) / 255.0, 0.0, 1.0);
    return vec3(1.0, g, b);
  }
`;

const noise = /* glsl */ `
  float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
  float noise(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x), mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
  }
`;

const wallFragment = /* glsl */ `
  uniform vec3 uFlight;
  uniform float uHeat;
  uniform float uTStag;
  uniform float uTRef;
  uniform vec3 uSun;
  varying vec3 vNormalW;
  varying vec3 vNormalO;
  varying vec3 vViewDir;
  varying vec3 vPosO;
  ${blackbody}
  ${noise}
  void main() {
    vec3 n = normalize(vNormalW);
    // Heating is a property of the body's own frame: the flow direction is fixed there.
    float lees = max(dot(normalize(vNormalO), uFlight), ${BACKSHELL_Q.toFixed(3)});
    float T = uTStag * pow(lees, 0.25);
    float radiance = pow(T / uTRef, 4.0);
    // Exposure with a soft shoulder: the stagnation region saturates toward white.
    vec3 glow = blackbody(T) * (1.0 - exp(-radiance * radiance * 3.2)) * uHeat * 1.6;

    // The charred ablator on the heatshield, a cooler grey on the backshell.
    float shield = 1.0 - smoothstep(${(SHOULDER_C.y + 0.6 * SHOULDER_R).toFixed(3)} - 0.04, ${(SHOULDER_C.y + 0.6 * SHOULDER_R).toFixed(3)} + 0.04, vPosO.y);
    float mottle = noise(vPosO.xz * 2.2 + vPosO.y * 3.0) * 0.6 + noise(vPosO.xz * 9.0) * 0.4;
    vec3 albedo = mix(vec3(0.2, 0.2, 0.21), vec3(0.13, 0.11, 0.095) * (0.75 + 0.5 * mottle), shield);
    float radial = length(vPosO.xz);
    // The retro nozzle's lip in the middle of the heatshield.
    float lip = shield * (1.0 - smoothstep(0.0, 0.035, abs(radial - ${NOZZLE_EXIT_R.toFixed(3)})));
    // Backshell panel rings.
    float ring = 0.0;
    for (int i = 1; i <= 3; i++) {
      ring += 1.0 - smoothstep(0.0, 0.012, abs(vPosO.y - (${(0.35 * DIAMETER).toFixed(3)} + ${(0.17 * DIAMETER).toFixed(3)} * float(i))));
    }

    // The ablator under a sun key, an earthshine fill and a cool rim, so the body reads as a solid
    // object and not a lamp.
    vec3 v = normalize(vViewDir);
    vec3 l = normalize(uSun);
    float diff = max(dot(n, l), 0.0);
    float spec = pow(max(dot(n, normalize(l + v)), 0.0), 40.0) * mix(0.25, 0.08, shield);
    float earth = max(-n.y, 0.0);
    float fres = pow(1.0 - max(dot(n, v), 0.0), 3.0);
    vec3 base = albedo * (0.05 + 1.15 * diff) * vec3(1.0, 0.96, 0.9)
              + albedo * vec3(0.18, 0.34, 0.6) * earth * 0.7
              + vec3(0.3, 0.48, 0.7) * fres * 0.14
              + vec3(spec);
    vec3 color = (base + glow) * (1.0 - 0.55 * lip) * (1.0 - 0.4 * clamp(ring, 0.0, 1.0));
    gl_FragColor = vec4(color, 1.0);
  }
`;

const sheathFragment = /* glsl */ `
  uniform vec3 uFlight;
  uniform float uSheath;
  uniform float uEdge;
  varying vec3 vNormalW;
  varying vec3 vNormalO;
  varying vec3 vViewDir;
  varying vec3 vPosO;
  void main() {
    vec3 n = normalize(vNormalW);
    float facing = abs(dot(n, normalize(vViewDir)));
    // Optically thin: the path length through the layer grows toward the silhouette.
    float limb = pow(1.0 - facing, 3.0);
    float windward = 0.3 + 0.7 * max(dot(normalize(vNormalO), uFlight), 0.0);
    vec3 col = mix(vec3(1.0, 0.42, 0.22), vec3(1.0, 0.8, 0.6), windward);
    // The layer thins out behind the shoulder.
    float trail = 1.0 - smoothstep(${(0.12 * DIAMETER).toFixed(3)}, ${(0.42 * DIAMETER).toFixed(3)}, vPosO.y);
    float a = limb * windward * trail * uSheath * 2.6;
    // The shock as a density edge rather than a glow.
    float e = pow(1.0 - facing, 6.0) * trail * uEdge * 0.9;
    gl_FragColor = vec4(col * a + vec3(0.62, 0.78, 0.95) * e, a + e);
  }
`;

const wakeVertex = /* glsl */ `
  varying float vAlong;
  varying float vAround;
  varying vec3 vNormalW;
  varying vec3 vViewDir;
  void main() {
    vAlong = (position.y - ${WAKE_START.toFixed(3)}) / ${WAKE_LENGTH.toFixed(3)};
    vAround = atan(position.z, position.x);
    vec4 world = modelMatrix * vec4(position, 1.0);
    vNormalW = normalize(mat3(modelMatrix) * normal);
    vViewDir = normalize(cameraPosition - world.xyz);
    gl_Position = projectionMatrix * viewMatrix * world;
  }
`;

const wakeFragment = /* glsl */ `
  uniform float uSheath;
  uniform float uTime;
  varying float vAlong;
  varying float vAround;
  varying vec3 vNormalW;
  varying vec3 vViewDir;
  ${noise}
  void main() {
    float facing = abs(dot(normalize(vNormalW), normalize(vViewDir)));
    float core = pow(facing, 1.1);
    float fade = pow(1.0 - clamp(vAlong, 0.0, 1.0), 2.0) * smoothstep(0.0, 0.06, vAlong);
    vec3 col = mix(vec3(0.9, 0.3, 0.22), vec3(0.45, 0.25, 0.6), clamp(vAlong * 1.4, 0.0, 1.0));
    float flow = 0.55 + 0.45 * noise(vec2(vAlong * 9.0 - uTime * 3.2, vAround * 2.5));
    float a = core * fade * flow * uSheath * 1.5;
    gl_FragColor = vec4(col * a, a);
  }
`;

/** The plume as lathe points for unit length along −y: radius in metres, from the nozzle exit. */
function plumeProfile(): THREE.Vector2[] {
  return [
    [NOZZLE_EXIT_R, 0],
    [0.8, -0.12],
    [1.15, -0.3],
    [1.25, -0.5],
    [1.05, -0.72],
    [0.6, -0.9],
    [0, -1],
  ].map(([r, y]) => new THREE.Vector2(r, y));
}

/** Plume length, m, at a throttle: drawn, longer below Mach 1 where no shock stands it off. */
export const plumeLength = (throttle: number, mach: number) => (mach > 1 ? 2 + 6 * throttle : 3 + 9 * throttle);

const plumeVertex = /* glsl */ `
  varying float vAlong;
  varying float vAround;
  varying vec3 vNormalW;
  varying vec3 vViewDir;
  void main() {
    vAlong = -position.y;
    vAround = atan(position.z, position.x);
    vec4 world = modelMatrix * vec4(position, 1.0);
    // Normals take the inverse transpose: the plume is stretched along its axis only.
    vNormalW = normalize(transpose(inverse(mat3(modelMatrix))) * normal);
    vViewDir = normalize(cameraPosition - world.xyz);
    gl_Position = projectionMatrix * viewMatrix * world;
  }
`;

const plumeFragment = /* glsl */ `
  uniform float uThrottle;
  uniform float uTime;
  varying float vAlong;
  varying float vAround;
  varying vec3 vNormalW;
  varying vec3 vViewDir;
  ${noise}
  void main() {
    float facing = abs(dot(normalize(vNormalW), normalize(vViewDir)));
    float core = pow(facing, 2.0);
    float along = clamp(vAlong, 0.0, 1.0);
    vec3 col = mix(vec3(1.0, 0.93, 0.78), vec3(1.0, 0.52, 0.16), clamp(smoothstep(0.0, 0.75, along) + (1.0 - core) * 0.4, 0.0, 1.0));
    float flicker = 0.75 + 0.25 * noise(vec2(along * 7.0 - uTime * 9.0, vAround * 3.0));
    float a = (0.25 + 0.75 * core) * (1.0 - along * along) * flicker * uThrottle * 2.2;
    gl_FragColor = vec4(col * a, a);
  }
`;

const SUN = new THREE.Vector3(-0.4, 0.8, 0.45).normalize();
const Z_AXIS = new THREE.Vector3(0, 0, 1);

export interface CapsuleProps {
  /** Wall emission scale, 0 (none) to 1. */
  heat: number;
  /** Stagnation wall temperature, K. */
  tStag: number;
  /** Shock-layer and wake brightness, from the electron density. */
  sheath: number;
  /** Seconds of video time, which advects the wake's and the plume's texture. */
  time: number;
  /** Angle of attack, rad. */
  alpha?: number;
  /** Throttle flown, 0 to 1; zero leaves the engine dark. */
  throttle?: number;
  /** Plume length, m. */
  plume?: number;
  /** Visibility of the bow shock's edge where it does not glow, 0 to 1. */
  shockEdge?: number;
}

export const Capsule: React.FC<CapsuleProps> = ({ heat, tStag, sheath, time, alpha = TRIM_ALPHA, throttle = 0, plume = 0, shockEdge = 0 }) => {
  const { bodyGeo, shockGeo, wakeGeo, plumeGeo } = useMemo(
    () => ({
      bodyGeo: new THREE.LatheGeometry(bodyProfile(), 128),
      shockGeo: new THREE.LatheGeometry(shockProfile(), 96),
      wakeGeo: new THREE.LatheGeometry(wakeProfile(), 64),
      plumeGeo: new THREE.LatheGeometry(plumeProfile(), 64),
    }),
    []
  );

  // The flight direction in the body frame: the flow frame's −y, seen from the tilted body.
  const flight = new THREE.Vector3(0, -1, 0).applyAxisAngle(Z_AXIS, -alpha);
  // The plume holds the bow shock off ahead of its tip: the shock stands as far ahead of the tip as
  // it stands ahead of the heatshield with the engine dark.
  const standoff = throttle > 0 ? plume : 0;
  const widen = 1 + (0.25 * standoff) / DIAMETER;

  // Fresh uniform objects every render: R3F copies their values into the material's own uniform
  // objects, so a value written into an object it already copied never reaches the shader.
  const uniforms = {
    wall: { uFlight: { value: flight }, uHeat: { value: heat }, uTStag: { value: tStag }, uTRef: { value: T_REF }, uSun: { value: SUN } },
    sheath: { uFlight: { value: flight }, uSheath: { value: sheath }, uEdge: { value: shockEdge } },
    wake: { uSheath: { value: sheath }, uTime: { value: time } },
    plume: { uThrottle: { value: throttle }, uTime: { value: time } },
  };

  return (
    <group>
      <mesh geometry={wakeGeo}>
        <shaderMaterial
          vertexShader={wakeVertex}
          fragmentShader={wakeFragment}
          uniforms={uniforms.wake}
          transparent
          depthWrite={false}
          blending={THREE.AdditiveBlending}
          side={THREE.DoubleSide}
        />
      </mesh>
      <group rotation={[0, 0, alpha]}>
        <mesh geometry={bodyGeo}>
          <shaderMaterial vertexShader={wallVertex} fragmentShader={wallFragment} uniforms={uniforms.wall} />
        </mesh>
        {throttle > 0 && (
          <mesh geometry={plumeGeo} scale={[1, plume, 1]}>
            <shaderMaterial
              vertexShader={plumeVertex}
              fragmentShader={plumeFragment}
              uniforms={uniforms.plume}
              transparent
              depthWrite={false}
              blending={THREE.AdditiveBlending}
              side={THREE.DoubleSide}
            />
          </mesh>
        )}
        <mesh geometry={shockGeo} position={[0, -standoff, 0]} scale={[widen, 1, widen]}>
          <shaderMaterial
            vertexShader={wallVertex}
            fragmentShader={sheathFragment}
            uniforms={uniforms.sheath}
            transparent
            depthWrite={false}
            blending={THREE.AdditiveBlending}
            side={THREE.DoubleSide}
          />
        </mesh>
      </group>
    </group>
  );
};
