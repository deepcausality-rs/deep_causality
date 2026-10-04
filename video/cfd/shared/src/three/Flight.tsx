/**
 * The 3D shot: the capsule on its flight path, rolled by the bank angle flown, a halo at the
 * stagnation point, the ground once it is near, and a camera on a slow orbit. Rendered at the full
 * composition size, transparent, over the 2D backdrop. The capsule is modelled in metres and drawn
 * at `VIEW_SCALE` scene units per metre, so the camera's distances stay in scene units.
 */
import { ThreeCanvas } from '@remotion/three';
import { useMemo } from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import * as THREE from 'three';
import { useThree } from '@react-three/fiber';
import { stage } from '../tokens';
import { BODY_CENTRE, Capsule, plumeLength, stagnationPoint, T_REF, TRIM_ALPHA } from './Capsule';

/**
 * A soft radial sprite: the eye's bloom around a bright nose, drawn rather than post-processed. It
 * ignores depth, as bloom does; a depth-tested sprite is clipped where its plane cuts the body.
 */
const Halo: React.FC<{ intensity: number; size: number; position: [number, number, number] }> = ({ intensity, size, position }) => {
  const texture = useMemo(() => {
    const c = document.createElement('canvas');
    c.width = c.height = 256;
    const g = c.getContext('2d')!;
    const grad = g.createRadialGradient(128, 128, 0, 128, 128, 128);
    grad.addColorStop(0, 'rgba(255,214,160,1)');
    grad.addColorStop(0.18, 'rgba(255,150,90,0.55)');
    grad.addColorStop(0.5, 'rgba(255,110,70,0.12)');
    grad.addColorStop(1, 'rgba(255,90,60,0)');
    g.fillStyle = grad;
    g.fillRect(0, 0, 256, 256);
    return new THREE.CanvasTexture(c);
  }, []);
  return (
    <sprite position={position} scale={[size, size, 1]}>
      <spriteMaterial map={texture} transparent depthWrite={false} depthTest={false} blending={THREE.AdditiveBlending} opacity={intensity} />
    </sprite>
  );
};

const Camera: React.FC<{ position: THREE.Vector3; target: THREE.Vector3; fov: number }> = ({ position, target, fov }) => {
  const { camera } = useThree();
  camera.position.copy(position);
  camera.lookAt(target);
  (camera as THREE.PerspectiveCamera).fov = fov;
  camera.updateProjectionMatrix();
  return null;
};

/** A fixed camera, scene units, in place of a shot's orbit: for the view from the ground. */
export interface FixedCamera {
  position: [number, number, number];
  target: [number, number, number];
  fov: number;
}

/** The flight-path angle a shot assumes when the scene does not pass the run's own. */
const GAMMA = (10 * Math.PI) / 180;
const FOV = 18;
const TARGET = new THREE.Vector3(0.8, 0.2, 0);
/** Scene units per metre of vehicle. */
export const VIEW_SCALE = 0.19;
const Z = new THREE.Vector3(0, 0, 1);
/** The flow frame's rotation: the capsule flies along −y of its frame, onto (−cos γ, −sin γ, 0). */
const flowRotation = (gamma: number) => -(Math.PI / 2 - gamma);
const flightDir = (gamma: number) => new THREE.Vector3(-Math.cos(gamma), -Math.sin(gamma), 0);

/** The stagnation point in the scene, rolled by `bank` about the flight direction. */
function noseAt(bank: number, gamma: number, alpha: number): THREE.Vector3 {
  return stagnationPoint(alpha)
    .applyAxisAngle(Z, alpha)
    .applyAxisAngle(Z, flowRotation(gamma))
    .multiplyScalar(VIEW_SCALE)
    .applyAxisAngle(flightDir(gamma), bank);
}

export interface Shot {
  /** 0 → 1 moves the camera closer. */
  dolly: number;
  /** Orbit about the vertical, radians; negative swings toward the nose. */
  azimuth: number;
  /** Lowers the camera and its target together, which raises the capsule in the frame; scene units. */
  lift?: number;
  /** Moves the camera and its target toward the flight direction, which moves the capsule right. */
  pan?: number;
}

/**
 * How far the body's middle sits from where it sits at the reference attitude (γ 10°, the trim angle
 * of attack), scene units. The camera follows it, so a steeper descent keeps the capsule framed.
 */
function framingShift(gamma: number, alpha: number): THREE.Vector3 {
  const at = (g: number, a: number) => BODY_CENTRE.clone().applyAxisAngle(Z, a).applyAxisAngle(Z, flowRotation(g)).multiplyScalar(VIEW_SCALE);
  return at(gamma, alpha).sub(at(GAMMA, TRIM_ALPHA));
}

const targetFor = (shot: Shot, gamma: number, alpha: number) =>
  new THREE.Vector3(TARGET.x - (shot.pan ?? 0), TARGET.y - (shot.lift ?? 0), TARGET.z).add(framingShift(gamma, alpha));

/** Camera position for a shot at a frame: a slow drift on top of the chosen orbit and dolly. */
function cameraAt(shot: Shot, frame: number, gamma: number, alpha: number): THREE.Vector3 {
  const radius = 7.4 - 1.1 * shot.dolly;
  const az = shot.azimuth + Math.sin(frame / 240) * 0.016;
  return new THREE.Vector3(TARGET.x - (shot.pan ?? 0) + radius * Math.sin(az), 0.75 - (shot.lift ?? 0), radius * Math.cos(az)).add(framingShift(gamma, alpha));
}

/**
 * Where the stagnation point lands on the 1920 × 1080 stage, for 2D overlays that point at the
 * vehicle; with `ahead`, the point that many scene units further along the flight path.
 */
export function noseOnStage(shot: Shot, frame: number, ahead = 0, gamma = GAMMA, alpha = TRIM_ALPHA): { x: number; y: number } {
  const cam = new THREE.PerspectiveCamera(FOV, stage.width / stage.height, 0.01, 200);
  cam.position.copy(cameraAt(shot, frame, gamma, alpha));
  cam.lookAt(targetFor(shot, gamma, alpha));
  cam.updateMatrixWorld();
  cam.updateProjectionMatrix();
  const p = noseAt(0, gamma, alpha).addScaledVector(flightDir(gamma), ahead).project(cam);
  return { x: ((p.x + 1) / 2) * stage.width, y: ((1 - p.y) / 2) * stage.height };
}

/** The terrain under a landing: dark ground, lit where the plume reaches it. Scene units. */
const groundFragment = /* glsl */ `
  uniform vec2 uHit;
  uniform float uLight;
  uniform float uReach;
  varying vec2 vXZ;
  float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
  float noise(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x), mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
  }
  void main() {
    float n = noise(vXZ * 2.5) * 0.55 + noise(vXZ * 9.0) * 0.3 + noise(vXZ * 31.0) * 0.15;
    vec3 base = vec3(0.05, 0.047, 0.044) * (0.6 + 0.8 * n);
    float d = distance(vXZ, uHit);
    vec3 lit = vec3(1.0, 0.55, 0.22) * uLight * exp(-d / uReach) * (0.7 + 0.6 * n);
    float far = smoothstep(6.0, 60.0, length(vXZ - uHit));
    gl_FragColor = vec4(mix(base + lit, vec3(0.02, 0.031, 0.047), far), 1.0);
  }
`;
const groundVertex = /* glsl */ `
  varying vec2 vXZ;
  void main() {
    vec4 world = modelMatrix * vec4(position, 1.0);
    vXZ = world.xz;
    gl_Position = projectionMatrix * viewMatrix * world;
  }
`;

const Ground: React.FC<{ y: number; hit: [number, number]; light: number; reach: number }> = ({ y, hit, light, reach }) => {
  const geo = useMemo(() => new THREE.PlaneGeometry(400, 400, 1, 1), []);
  const uniforms = { uHit: { value: new THREE.Vector2(hit[0], hit[1]) }, uLight: { value: light }, uReach: { value: reach } };
  return (
    <mesh geometry={geo} rotation={[-Math.PI / 2, 0, 0]} position={[0, y, 0]}>
      <shaderMaterial vertexShader={groundVertex} fragmentShader={groundFragment} uniforms={uniforms} />
    </mesh>
  );
};

/** Shown once the ground is this close, m. */
const GROUND_VISIBLE_M = 60;

export interface FlightProps {
  tStag: number;
  sheath: number;
  shot: Shot;
  /** Bank angle flown, rad: a roll about the flight direction. */
  bank?: number;
  /** Seconds that advect the wake's and the plume's texture; defaults to the video time. */
  wakeTime?: number;
  /** Flight-path angle below the horizontal, rad. */
  gamma?: number;
  /** Angle of attack, rad. */
  alpha?: number;
  /** Throttle flown, 0 to 1. */
  throttle?: number;
  /** Flight Mach number, which shapes the plume and shows the shock's edge above Mach 1; without it, no edge is drawn. */
  mach?: number;
  /** Height of the heatshield above the touchdown plane, m; the ground shows once it is near. */
  groundM?: number;
  /** A fixed camera in place of the shot's orbit. */
  camera?: FixedCamera;
}

export const Flight: React.FC<FlightProps> = ({ tStag, sheath, shot, bank = 0, wakeTime, gamma = GAMMA, alpha = TRIM_ALPHA, throttle = 0, mach, groundM, camera }) => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();
  const dir = flightDir(gamma);
  const roll = new THREE.Quaternion().setFromAxisAngle(dir, bank);
  const nose = noseAt(bank, gamma, alpha).toArray() as [number, number, number];
  const glow = Math.min(1.6, (tStag / T_REF) ** 2);
  const plume = plumeLength(throttle, mach ?? 10);
  // Where the plume's axis meets the ground, and how much of the plume still reaches it.
  const near = groundM !== undefined && groundM < GROUND_VISIBLE_M;
  const groundY = near ? -groundM! * VIEW_SCALE : 0;
  const along = near ? -groundY / Math.max(Math.sin(gamma), 0.05) : 0;
  const hit: [number, number] = [dir.x * along, dir.z * along];
  const reachM = groundM !== undefined ? Math.max(0, 1 - groundM / (plume * 3)) : 0;
  const dust = throttle * reachM;
  // The plume's glow sits partway down the plume, and never below the ground.
  const plumeGlow = dir.clone().multiplyScalar(0.35 * plume * VIEW_SCALE);
  if (near && plumeGlow.y < groundY + 0.05) plumeGlow.multiplyScalar((groundY + 0.05) / plumeGlow.y);
  return (
    <ThreeCanvas width={width} height={height} gl={{ alpha: true, antialias: true }} camera={{ fov: FOV, near: 0.01, far: 400 }} style={{ position: 'absolute', inset: 0 }}>
      {camera ? (
        <Camera position={new THREE.Vector3(...camera.position)} target={new THREE.Vector3(...camera.target)} fov={camera.fov} />
      ) : (
        <Camera position={cameraAt(shot, frame, gamma, alpha)} target={targetFor(shot, gamma, alpha)} fov={FOV} />
      )}
      {near && <Ground y={groundY} hit={hit} light={0.25 + 2.2 * dust} reach={0.6 + 0.6 * dust} />}
      <group quaternion={roll}>
        <group rotation={[0, 0, flowRotation(gamma)]} scale={VIEW_SCALE}>
          <Capsule
            heat={1}
            tStag={tStag}
            sheath={sheath}
            time={wakeTime ?? frame / fps}
            alpha={alpha}
            throttle={throttle}
            plume={plume}
            shockEdge={mach !== undefined && mach > 1 ? 0.35 : 0}
          />
        </group>
      </group>
      <Halo intensity={Math.min(1, 0.55 * glow)} size={0.75 * Math.max(1, glow)} position={nose} />
      <Halo intensity={0.35 * sheath} size={1.6 + 0.8 * sheath} position={nose} />
      {throttle > 0 && <Halo intensity={Math.min(1, 0.5 + throttle) * (1 - 0.6 * reachM)} size={0.6 + 1.2 * throttle} position={plumeGlow.toArray() as [number, number, number]} />}
      {near && dust > 0 && <Halo intensity={Math.min(1, 1.4 * dust)} size={1.2 + 3 * dust} position={[hit[0], groundY + 0.15, hit[1]]} />}
    </ThreeCanvas>
  );
};

/**
 * Sheath brightness from the electron density, log-scaled between 10¹⁴ and 10²⁰ m⁻³, over a faint
 * floor of shock-layer glow that only hypersonic flight keeps (above Mach 5).
 */
export const sheathFromDensity = (ne: number, mach = 30) =>
  (mach > 5 ? 0.12 : 0) + 0.88 * Math.min(1, Math.max(0, (Math.log10(Math.max(ne, 1)) - 14) / 6));
