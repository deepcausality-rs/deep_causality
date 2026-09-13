# build_corridor_sequence.py
# Rebuilds the plasma-blackout corridor animation from the assets of the earlier attempt.
#
#   Cut 1  f0001-0336  (14.0 s)  close-in entry, 90 km -> 73.2 km, nose-down at the flight-path
#                                angle, the sheath forms red-orange, fork ring flashes at onset
#   Cut 2  f0337-1000  (27.7 s)  wide: fork ring, coarse fan (6 translucent), fine fan (11),
#                                commit 11.5 deg goes solid, the rest fade, push back in
#   Cut 3  f1001-1440  (18.3 s)  rear-quarter close-in through the 61 km peak blaze, then the
#                                sheath abates to the 47 km exit
#
# Run inside Blender (MCP exec or Text editor) or headless:
#   blender -b --python build_corridor_sequence.py [-- <old_blend> [<out_blend>]]
# With no arguments the asset file is plasma_blackout_corridor.blend beside this script and the
# output is corridor_sequence.blend beside it.
import bpy
import math
import os
import sys
from mathutils import Vector

# ---------------------------------------------------------------- config
# The directory the defaults resolve against: this script's own, or, when the body is run from the
# Text editor or over MCP exec without a `__file__`, the open .blend's, else the cwd.
HERE = (os.path.dirname(os.path.abspath(__file__)) if "__file__" in globals()
        else os.path.dirname(bpy.data.filepath) if bpy.data.filepath else os.getcwd())
OLD_BLEND = globals().get("OLD_BLEND", os.path.join(HERE, "plasma_blackout_corridor.blend"))
OUT_BLEND = globals().get("OUT_BLEND", os.path.join(HERE, "corridor_sequence.blend"))
PREVIEW_PCT = int(globals().get("PREVIEW_PCT", 30))
SAMPLES = int(globals().get("SAMPLES", 32))
if "--" in sys.argv:
    a = sys.argv[sys.argv.index("--") + 1:]
    if len(a) > 0: OLD_BLEND = a[0]
    if len(a) > 1: OUT_BLEND = a[1]
if not os.path.isfile(OLD_BLEND):
    raise FileNotFoundError(
        f"asset file not found: {OLD_BLEND} "
        "(expected plasma_blackout_corridor.blend beside build_corridor_sequence.py; "
        "pass another path as the first argument after '--')")

FPS = 24
F_END = 1440
# --- cut 1
F_ONSET = 300          # 73.2 km, link -> DENIED
F_CUT2 = 337
# --- cut 2
F_FORK_FLASH = 340
F_COARSE0, F_COARSE1 = 360, 540
F_FINE0, F_FINE1 = 570, 730
F_COMMIT = 760
F_FADE0, F_FADE1 = 790, 880
F_PUSH0, F_PUSH1 = 850, 1000
F_CUT3 = 1001
# --- cut 3
F_PEAK = 1110          # 61 km
F_EXIT = 1380          # 47 km, link returns
R_EARTH = 6371000.0

COARSE = [0.0, 5.0, 10.0, 15.0, 20.0, 40.0]
FINE = [7.5, 8.0, 8.5, 9.0, 9.5, 10.0, 10.5, 11.0, 11.5, 12.0, 12.5]
COMMITTED = 11.5

# ---------------------------------------------------------------- helpers
def set_interp(mode):
    bpy.context.preferences.edit.keyframe_new_interpolation_type = mode

def kf(idb, path, frame, value, index=-1):
    if index >= 0:
        v = list(getattr(idb, path))
        v[index] = value
        setattr(idb, path, v)
    else:
        setattr(idb, path, value)
    idb.keyframe_insert(path, frame=frame, index=index)

def kf_sock(sock, frame, value):
    sock.default_value = value
    sock.keyframe_insert("default_value", frame=frame)

def new_coll(name, parent=None):
    c = bpy.data.collections.get(name)
    if c is None:
        c = bpy.data.collections.new(name)
        (parent or bpy.context.scene.collection).children.link(c)
    return c

def link_to(obj, coll):
    for c in list(obj.users_collection):
        c.objects.unlink(obj)
    coll.objects.link(obj)

def emission_mat(name, color, strength):
    m = bpy.data.materials.new(name)
    if not m.use_nodes: m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    em = nt.nodes.new("ShaderNodeEmission")
    em.inputs["Color"].default_value = (*color, 1)
    em.inputs["Strength"].default_value = strength
    nt.links.new(em.outputs[0], out.inputs["Surface"])
    return m

def ghost_mat(name, color, strength, fac):
    """Mix(Transparent, Emission) -- fac 0 = invisible, 1 = solid emission."""
    m = bpy.data.materials.new(name)
    if not m.use_nodes: m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    mix = nt.nodes.new("ShaderNodeMixShader")
    tr = nt.nodes.new("ShaderNodeBsdfTransparent")
    em = nt.nodes.new("ShaderNodeEmission")
    mix.name = "MIX"
    em.name = "EM"
    em.inputs["Color"].default_value = (*color, 1)
    em.inputs["Strength"].default_value = strength
    mix.inputs["Fac"].default_value = fac
    nt.links.new(tr.outputs[0], mix.inputs[1])
    nt.links.new(em.outputs[0], mix.inputs[2])
    nt.links.new(mix.outputs[0], out.inputs["Surface"])
    m.blend_method = "BLEND"
    return m

def mesh_obj(name, bm_fn, loc, mat, coll):
    import bmesh
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bm_fn(bm)
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons: p.use_smooth = True
    me.materials.append(mat)
    o = bpy.data.objects.new(name, me)
    o.location = loc
    coll.objects.link(o)
    o.parent = bpy.data.objects["FLIGHT"]
    return o

def sphere(name, r, loc, mat, coll, seg=24):
    import bmesh
    return mesh_obj(name, lambda bm: bmesh.ops.create_uvsphere(bm, u_segments=seg, v_segments=seg // 2, radius=r), loc, mat, coll)

def torus(name, R, r, loc, mat, coll, segs=64, rings=12):
    import bmesh
    def fn(bm):
        verts = []
        for i in range(segs):
            a = 2 * math.pi * i / segs
            ring = []
            for j in range(rings):
                b = 2 * math.pi * j / rings
                ring.append(bm.verts.new(((R + r * math.cos(b)) * math.cos(a), (R + r * math.cos(b)) * math.sin(a), r * math.sin(b))))
            verts.append(ring)
        for i in range(segs):
            for j in range(rings):
                bm.faces.new((verts[i][j], verts[(i + 1) % segs][j], verts[(i + 1) % segs][(j + 1) % rings], verts[i][(j + 1) % rings]))
    return mesh_obj(name, fn, loc, mat, coll)

def track_cam(name, lens, target, coll):
    cd = bpy.data.cameras.new(name)
    cd.lens = lens
    cd.clip_start = 0.1
    cd.clip_end = 2.0e7
    cd.sensor_width = 36
    cam = bpy.data.objects.new(name, cd)
    coll.objects.link(cam)
    cam.parent = bpy.data.objects["FLIGHT"]
    c = cam.constraints.new("TRACK_TO")
    c.target = target
    c.track_axis = "TRACK_NEGATIVE_Z"
    c.up_axis = "UP_Y"
    return cam

# ---------------------------------------------------------------- fresh scene
bpy.ops.wm.read_homefile(use_empty=True)
scene = bpy.context.scene
scene.name = "Corridor"
for o in list(bpy.data.objects): bpy.data.objects.remove(o)

# ---------------------------------------------------------------- append assets
want_objs = ["VEHICLE", "RAMC_probe", "raceway", "PlasmaSheath_shocklayer", "PlasmaSheath_wake",
             "Earth", "Sun", "Earthshine", "FORK_onset_step131",
             "branch_coarse_40.0deg_UNCLAMPED_ghost"]
want_objs += [f"antenna_{i}" for i in range(4)] + [f"boltring_{i}" for i in range(24)]
want_objs += [f"band_{i}" for i in range(5)] + [f"rake_{i}" for i in range(4)]
want_objs += [f"branch_coarse_{b:04.1f}deg" for b in COARSE] + [f"branch_fine_{b:04.1f}deg" for b in FINE]
with bpy.data.libraries.load(OLD_BLEND, link=False) as (src, dst):
    dst.objects = [n for n in want_objs if n in src.objects]
    dst.worlds = ["Space"]
missing = [n for n in want_objs if n not in bpy.data.objects]
if missing: print("WARNING missing from old blend:", missing)

scene.world = bpy.data.worlds["Space"]

C_probe = new_coll("probe")
C_sheath = new_coll("sheath")
C_env = new_coll("environment")
C_branches = new_coll("branches")
C_markers = new_coll("markers")
C_cams = new_coll("cameras")

for coll_ in (bpy.data.objects, bpy.data.curves, bpy.data.meshes, bpy.data.materials, bpy.data.lights, bpy.data.cameras, bpy.data.worlds, bpy.data.node_groups):
    for idb in coll_:
        if idb.animation_data: idb.animation_data_clear()
        nt = getattr(idb, "node_tree", None)
        if nt is not None and nt.animation_data: nt.animation_data_clear()
for o in bpy.data.objects:
    n = o.name
    if n == "VEHICLE" or n == "RAMC_probe" or n == "raceway" or n.startswith(("antenna_", "boltring_", "band_", "rake_")):
        link_to(o, C_probe)
    elif n.startswith("PlasmaSheath"):
        link_to(o, C_sheath)
    elif n in ("Earth", "Sun", "Earthshine"):
        link_to(o, C_env)
    elif n.startswith("branch_"):
        link_to(o, C_branches)
    elif n.startswith("FORK"):
        link_to(o, C_markers)

ROOT = bpy.data.objects.new("ROOT", None)
ROOT.empty_display_size = 2.0
scene.collection.objects.link(ROOT)
ROOT.rotation_euler = (math.radians(90), 0, 0)   # asset frame: down = -Y  ->  world: down = -Z
# FLIGHT: the vehicle frame, pitched nose-down by the flight-path angle so the descent reads
FLIGHT = bpy.data.objects.new("FLIGHT", None)
FLIGHT.empty_display_size = 2.0
scene.collection.objects.link(FLIGHT)
FLIGHT.parent = ROOT
PITCH_DEG = float(globals().get("PITCH_DEG", 24.0))
FLIGHT.rotation_euler = (0, 0, math.radians(PITCH_DEG))   # +Z rotation tips the -X nose toward -Y (down)
ENV_NAMES = ("Earth", "Sun", "Earthshine")
def under_root(o):
    if o.parent is None and o not in (ROOT, FLIGHT):
        o.parent = ROOT if o.name in ENV_NAMES else FLIGHT
for o in list(bpy.data.objects): under_root(o)

VEH = bpy.data.objects["VEHICLE"]
EARTH = bpy.data.objects["Earth"]
FORK = bpy.data.objects["FORK_onset_step131"]
SHOCK = bpy.data.objects["PlasmaSheath_shocklayer"]
WAKE = bpy.data.objects["PlasmaSheath_wake"]

# ---------------------------------------------------------------- sheath: one PLASMA level drives everything
EM_GAIN = float(globals().get("EM_GAIN", 14.0))
DENS_GAIN = float(globals().get("DENS_GAIN", 3.0))
def rig_volume(mat, ramp_colors, hot_color, em_gain=None):
    nt = mat.node_tree
    nodes = nt.nodes
    pv = next(n for n in nodes if n.type == "PRINCIPLED_VOLUME")
    ramp = next(n for n in nodes if n.type == "VALTORGB")
    # red-orange ramp: nose (x=0) white-yellow, flank orange, tail deep red
    els = ramp.color_ramp.elements
    while len(els) > 1: els.remove(els[-1])
    els[0].position, els[0].color = ramp_colors[0]
    for pos, col in ramp_colors[1:]:
        e = els.new(pos)
        e.color = col
    level = nodes.new("ShaderNodeValue")
    level.name = "PLASMA"
    level.label = "PLASMA level"
    level.outputs[0].default_value = 0.0
    # density * level
    dens_link = next(link for link in nt.links if link.to_node == pv and link.to_socket.name == "Density")
    md = nodes.new("ShaderNodeMath")
    md.operation = "MULTIPLY"
    md.name = "DENS_x_LEVEL"
    dg = nodes.new("ShaderNodeMath")
    dg.operation = "MULTIPLY"
    dg.inputs[1].default_value = DENS_GAIN
    nt.links.new(level.outputs[0], dg.inputs[0])
    nt.links.new(dens_link.from_socket, md.inputs[0])
    nt.links.new(dg.outputs[0], md.inputs[1])
    nt.links.remove(dens_link)
    nt.links.new(md.outputs[0], pv.inputs["Density"])
    # emission strength * level^1.4  (glow rises faster than the density)
    em_link = next(link for link in nt.links if link.to_node == pv and link.to_socket.name == "Emission Strength")
    pw = nodes.new("ShaderNodeMath")
    pw.operation = "POWER"
    pw.inputs[1].default_value = 1.4
    nt.links.new(level.outputs[0], pw.inputs[0])
    gain = nodes.new("ShaderNodeMath")
    gain.operation = "MULTIPLY"
    gain.inputs[1].default_value = (em_gain or EM_GAIN)
    gain.name = "EM_GAIN"
    nt.links.new(pw.outputs[0], gain.inputs[0])
    me = nodes.new("ShaderNodeMath")
    me.operation = "MULTIPLY"
    me.name = "EM_x_LEVEL"
    nt.links.new(em_link.from_socket, me.inputs[0])
    nt.links.new(gain.outputs[0], me.inputs[1])
    nt.links.remove(em_link)
    nt.links.new(me.outputs[0], pv.inputs["Emission Strength"])
    # colour: mix ramp colour toward hot at high level
    col_link = next(link for link in nt.links if link.to_node == pv and link.to_socket.name == "Emission Color")
    mix = nodes.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    mix.name = "HOT_MIX"
    heat = nodes.new("ShaderNodeMath")
    heat.operation = "MULTIPLY"
    heat.inputs[1].default_value = 0.6
    heat.use_clamp = True
    nt.links.new(level.outputs[0], heat.inputs[0])
    nt.links.new(heat.outputs[0], mix.inputs["Factor"])
    nt.links.new(col_link.from_socket, mix.inputs[6])
    mix.inputs[7].default_value = (*hot_color, 1)
    nt.links.remove(col_link)
    nt.links.new(mix.outputs[2], pv.inputs["Emission Color"])
    pv.inputs["Anisotropy"].default_value = 0.2
    return level.outputs[0]

SHOCK_LEVEL = rig_volume(bpy.data.materials["Sheath_shocklayer"],
                         [(0.0, (1.0, 0.88, 0.62, 1)), (0.18, (1.0, 0.58, 0.18, 1)), (0.6, (1.0, 0.36, 0.08, 1)), (1.0, (0.85, 0.16, 0.04, 1))],
                         (1.0, 0.42, 0.10))
WAKE_LEVEL = rig_volume(bpy.data.materials["Sheath_wake"],
                        [(0.0, (1.0, 0.5, 0.18, 1)), (0.45, (1.0, 0.3, 0.09, 1)), (1.0, (0.45, 0.08, 0.03, 1))],
                        (1.0, 0.30, 0.06), em_gain=EM_GAIN * 0.3)

# the wake's axial falloff was mapped over 0..1.3 (the body), but the wake mesh spans x = 1.3..10.3
# in its own object space, so the whole wake evaluated to zero. Map it over the wake itself.
_mr = next(n for n in bpy.data.materials["Sheath_wake"].node_tree.nodes if n.type == "MAP_RANGE" and n.name == "Map Range")
_mr.inputs["From Min"].default_value = 1.3
_mr.inputs["From Max"].default_value = 10.3

# plasma level over the descent (0 = none, 0.35 = marginal onset, 1.3 = 61 km peak)
PLASMA_KEYS = [(1, 0.0), (90, 0.02), (180, 0.10), (250, 0.22), (F_ONSET, 0.36), (F_CUT2 - 1, 0.40),
               (F_CUT3 - 1, 0.40), (F_CUT3, 0.42), (F_CUT3 + 45, 0.85), (F_PEAK, 1.35), (F_PEAK + 60, 1.10),
               (F_PEAK + 150, 0.55), (F_EXIT, 0.14), (F_EXIT + 30, 0.06), (F_END, 0.03)]
set_interp("BEZIER")
for f, v in PLASMA_KEYS:
    kf_sock(SHOCK_LEVEL, f, v)
    kf_sock(WAKE_LEVEL, f, v)

# visible shell thickens with the plasma (the true standoff is a 3 cm skin at this scale)
for f, v in PLASMA_KEYS:
    t = min(v, 1.35) / 1.35
    kf(SHOCK, "scale", f, (1.0 + 0.05 * t, 1.0 + 0.16 * t, 1.0 + 0.16 * t))
    kf(SHOCK, "location", f, (-0.03 * t, 0.0, 0.0))

# wake grows with the plasma
for f, v in PLASMA_KEYS:
    s = 0.15 + 0.7 * min(v, 1.35) / 1.35
    kf(WAKE, "scale", f, s, 0)
    kf(WAKE, "scale", f, 0.3 + 0.3 * min(v, 1.35) / 1.35, 1)
    kf(WAKE, "scale", f, 0.3 + 0.3 * min(v, 1.35) / 1.35, 2)

# wall radiative-equilibrium glow follows the heat flux (q rises 1.03e6 -> 1.83e6 -> 3e5)
for mname, base in (("HeatShield_RadEq", 0.34), ("NoseCap_Be_RadEq", 0.52), ("Hardware_RadEq", 0.22)):
    mix = next(n for n in bpy.data.materials[mname].node_tree.nodes if n.type == "MIX_SHADER")
    for f, v in PLASMA_KEYS:
        kf_sock(mix.inputs["Fac"], f, base * (0.15 + 0.85 * min(v, 1.35) / 1.35))

# ---------------------------------------------------------------- Earth surface + atmosphere shell
_en = bpy.data.materials["Earth_surface"].node_tree.nodes
_noise = next(n for n in _en if n.type == "TEX_NOISE")
_noise.inputs["Scale"].default_value = 70.0
_noise.inputs["Detail"].default_value = 10.0
_er = next(n for n in _en if n.type == "VALTORGB").color_ramp
for e, (pos, col) in zip(_er.elements, [(0.30, (0.04, 0.12, 0.32, 1)), (0.48, (0.06, 0.18, 0.42, 1)), (0.55, (0.16, 0.24, 0.18, 1)),
                                        (0.60, (0.30, 0.30, 0.20, 1)), (0.66, (0.70, 0.72, 0.74, 1)), (0.74, (0.95, 0.96, 0.98, 1)), (1.0, (1, 1, 1, 1))]):
    e.position = pos
    e.color = col
ATMO_MAT = bpy.data.materials.new("Atmosphere_shell")
if not ATMO_MAT.use_nodes: ATMO_MAT.use_nodes = True
_nt = ATMO_MAT.node_tree
_nt.nodes.clear()
_out = _nt.nodes.new("ShaderNodeOutputMaterial")
_mix = _nt.nodes.new("ShaderNodeMixShader")
_tr = _nt.nodes.new("ShaderNodeBsdfTransparent")
_em = _nt.nodes.new("ShaderNodeEmission")
_lw = _nt.nodes.new("ShaderNodeLayerWeight")
_pw = _nt.nodes.new("ShaderNodeMath")
_pw.operation = "POWER"
_lw.inputs["Blend"].default_value = 0.08
_pw.inputs[1].default_value = 5.0
_em.inputs["Color"].default_value = (0.30, 0.52, 1.0, 1)
_em.inputs["Strength"].default_value = 2.5
_nt.links.new(_lw.outputs["Facing"], _pw.inputs[0])
_nt.links.new(_pw.outputs[0], _mix.inputs["Fac"])
_nt.links.new(_tr.outputs[0], _mix.inputs[1])
_nt.links.new(_em.outputs[0], _mix.inputs[2])
_nt.links.new(_mix.outputs[0], _out.inputs["Surface"])
ATMO_MAT.blend_method = "BLEND"
import bmesh as _bm
_me = bpy.data.meshes.new("Atmosphere")
_b = _bm.new()
_bm.ops.create_uvsphere(_b, u_segments=192, v_segments=96, radius=R_EARTH + 100.0e3)
_b.to_mesh(_me)
_b.free()
for _p in _me.polygons: _p.use_smooth = True
_me.materials.append(ATMO_MAT)
ATMO = bpy.data.objects.new("Atmosphere", _me)
C_env.objects.link(ATMO)
ATMO.parent = EARTH
ATMO.visible_shadow = False
ATMO.visible_diffuse = False
ATMO.visible_glossy = False

# ---------------------------------------------------------------- altitude (Earth centre sits at -Y)
ALT_KEYS = [(1, 90.0e3), (F_ONSET, 73.2e3), (F_CUT3 - 1, 73.2e3), (F_PEAK, 61.0e3), (F_EXIT, 47.0e3), (F_END, 46.2e3)]
for f, alt in ALT_KEYS:
    kf(EARTH, "location", f, -(R_EARTH + alt), 1)
# slow ground drift under the vehicle (downrange, purely cosmetic)
set_interp("LINEAR")
kf(EARTH, "rotation_euler", 1, 0.0, 2)
kf(EARTH, "rotation_euler", F_END, 0.12, 2)

# ---------------------------------------------------------------- branches
GHOST_COARSE = ghost_mat("Ghost_coarse", (0.40, 0.68, 1.0), 1.3, 0.0)
GHOST_FINE = ghost_mat("Ghost_fine", (0.70, 0.86, 1.0), 1.6, 0.0)
GHOST_REFUSED = ghost_mat("Ghost_refused", (1.0, 0.30, 0.22), 1.6, 0.0)
COMMIT_MAT = ghost_mat("Committed", (1.0, 0.72, 0.28), 10.0, 0.0)

def curve_setup(o, mat, depth):
    d = o.data
    d.materials.clear()
    d.materials.append(mat)
    d.bevel_depth = depth
    d.bevel_resolution = 4
    d.use_fill_caps = True
    d.bevel_factor_mapping_end = "RESOLUTION"
    d.bevel_factor_mapping_start = "RESOLUTION"
    d.bevel_factor_start = 0.0
    d.bevel_factor_end = 0.0

def draw(o, f0, f1):
    set_interp("LINEAR")
    kf(o.data, "bevel_factor_end", 1, 0.0)
    kf(o.data, "bevel_factor_end", f0, 0.0)
    kf(o.data, "bevel_factor_end", f1, 1.0)

def endpoint(o):
    sp = o.data.splines[0]
    p = sp.points[-1].co
    return Vector((p[0], p[1], p[2]))

coarse_objs = {b: bpy.data.objects[f"branch_coarse_{b:04.1f}deg"] for b in COARSE}
fine_objs = {b: bpy.data.objects[f"branch_fine_{b:04.1f}deg"] for b in FINE}
refused = bpy.data.objects["branch_coarse_40.0deg_UNCLAMPED_ghost"]
committed = fine_objs[COMMITTED]

for b, o in coarse_objs.items():
    curve_setup(o, GHOST_COARSE, 0.7)
    draw(o, F_COARSE0, F_COARSE1)
for b, o in fine_objs.items():
    curve_setup(o, GHOST_FINE if b != COMMITTED else COMMIT_MAT, 0.5)
    draw(o, F_FINE0, F_FINE1)
curve_setup(refused, GHOST_REFUSED, 0.4)
draw(refused, F_COARSE0, F_COARSE1)
committed.data.bevel_depth = 0.7

# terminal markers
TERM_COARSE = ghost_mat("Term_coarse", (0.5, 0.75, 1.0), 4.0, 0.0)
TERM_FINE = ghost_mat("Term_fine", (0.8, 0.9, 1.0), 4.5, 0.0)
TERM_COMMIT = ghost_mat("Term_commit", (1.0, 0.85, 0.5), 12.0, 0.0)
terms = []
for b, o in coarse_objs.items():
    terms.append((sphere(f"term_coarse_{b:04.1f}", 1.0, endpoint(o), TERM_COARSE, C_branches), F_COARSE1, False))
for b, o in fine_objs.items():
    terms.append((sphere(f"term_fine_{b:04.1f}", 0.7 if b != COMMITTED else 1.1, endpoint(o),
                         TERM_FINE if b != COMMITTED else TERM_COMMIT, C_branches), F_FINE1, b == COMMITTED))
set_interp("BEZIER")
for o, f_land, is_commit in terms:
    kf(o, "scale", 1, (0.001,) * 3)
    kf(o, "scale", f_land - 2, (0.001,) * 3)
    kf(o, "scale", f_land + 8, (1.4,) * 3)
    kf(o, "scale", f_land + 16, (1.0,) * 3)
    if not is_commit:
        kf(o, "scale", F_FADE0, (1.0,) * 3)
        kf(o, "scale", F_FADE1, (0.001,) * 3)
    else:
        kf(o, "scale", F_COMMIT, (1.0,) * 3)
        kf(o, "scale", F_COMMIT + 10, (1.8,) * 3)
        kf(o, "scale", F_COMMIT + 24, (1.3,) * 3)
        kf(o, "scale", F_PUSH0 + 60, (1.3,) * 3)
        kf(o, "scale", F_PUSH1, (0.001,) * 3)

# material visibility over the study: appear translucent, commit goes solid, others fade
def vis(mat, keys):
    fac = mat.node_tree.nodes["MIX"].inputs["Fac"]
    for f, v in keys: kf_sock(fac, f, v)

vis(GHOST_COARSE, [(1, 0.0), (F_COARSE0 - 1, 0.0), (F_COARSE0 + 10, 0.5), (F_FADE0, 0.5), (F_FADE1, 0.0)])
vis(GHOST_REFUSED, [(1, 0.0), (F_COARSE0 - 1, 0.0), (F_COARSE0 + 10, 0.5), (F_FADE0, 0.5), (F_FADE1, 0.0)])
vis(GHOST_FINE, [(1, 0.0), (F_FINE0 - 1, 0.0), (F_FINE0 + 8, 0.55), (F_FADE0, 0.55), (F_FADE1, 0.0)])
vis(COMMIT_MAT, [(1, 0.0), (F_FINE0 - 1, 0.0), (F_FINE0 + 8, 0.55), (F_COMMIT - 4, 0.55), (F_COMMIT + 12, 1.0),
                 (F_PUSH0 + 80, 1.0), (F_PUSH1, 0.0)])
vis(TERM_COARSE, [(1, 0.0), (F_COARSE1 - 3, 0.0), (F_COARSE1 + 6, 0.6), (F_FADE0, 0.6), (F_FADE1, 0.0)])
vis(TERM_FINE, [(1, 0.0), (F_FINE1 - 3, 0.0), (F_FINE1 + 6, 0.7), (F_FADE0, 0.7), (F_FADE1, 0.0)])
vis(TERM_COMMIT, [(1, 0.0), (F_FINE1 - 3, 0.0), (F_FINE1 + 6, 0.7), (F_COMMIT - 4, 0.7), (F_COMMIT + 12, 1.0),
                  (F_PUSH0 + 80, 1.0), (F_PUSH1, 0.0)])
# the committed branch after commit: emission strength ramps up as it solidifies
em = COMMIT_MAT.node_tree.nodes["EM"].inputs["Strength"]
kf_sock(em, F_COMMIT - 4, 1.6)
kf_sock(em, F_COMMIT + 12, 10.0)

# ---------------------------------------------------------------- markers: fork ring + aim point
FORK_MAT = ghost_mat("Fork_ring", (0.45, 0.85, 1.0), 6.0, 0.0)
FORK.data.materials.clear()
FORK.data.materials.append(FORK_MAT)
FORK.scale = (2.6, 1.3, 1.3)   # ring around the vehicle at the fork point
kf(FORK, "scale", 1, (0.001,) * 3)
kf(FORK, "scale", F_ONSET - 1, (0.001,) * 3)
kf(FORK, "scale", F_ONSET + 6, (3.4, 1.7, 1.7))
kf(FORK, "scale", F_ONSET + 18, (2.6, 1.3, 1.3))
kf(FORK, "scale", F_PUSH1 - 30, (2.6, 1.3, 1.3))
kf(FORK, "scale", F_PUSH1, (0.001,) * 3)
vis(FORK_MAT, [(1, 0.0), (F_ONSET - 1, 0.0), (F_ONSET + 4, 1.0), (F_ONSET + 40, 0.55), (F_PUSH1 - 30, 0.55), (F_PUSH1, 0.0)])

AIM = Vector((-158.03, 0.0, -20.0))
AIM_MAT = ghost_mat("Aim", (1.0, 0.93, 0.45), 20.0, 0.0)
aim_ring = torus("AIM_ring", 2.2, 0.18, AIM, AIM_MAT, C_markers)
aim_ring.rotation_euler = (0, math.radians(90), 0)   # face the fork (ring normal along X)
aim_core = sphere("AIM_core", 0.5, AIM, AIM_MAT, C_markers)
vis(AIM_MAT, [(1, 0.0), (F_CUT2 - 1, 0.0), (F_CUT2 + 12, 1.0), (F_PUSH0 + 60, 1.0), (F_PUSH1, 0.0)])
for o in (aim_ring, aim_core):
    kf(o, "scale", F_PUSH0 + 60, (1.0,) * 3)
    kf(o, "scale", F_PUSH1, (0.001,) * 3)

# ---------------------------------------------------------------- RF pulses: removed (read as projectiles)
# ---------------------------------------------------------------- cameras
def target(name, loc):
    e = bpy.data.objects.new(name, None)
    e.empty_display_size = 0.5
    e.location = loc
    C_cams.objects.link(e)
    e.parent = bpy.data.objects["FLIGHT"]
    return e

set_interp("BEZIER")
# cut 1: front three-quarter, slow dolly in, the nose and the forming sheath
T1 = target("T1", (2.5, 0.0, 0.0))
CAM1 = track_cam("CAM1_entry", 50, T1, C_cams)
kf(CAM1, "location", 1, (-16.0, 13.0, 14.0))
kf(CAM1, "location", F_ONSET, (-8.0, 6.5, 8.0))
kf(CAM1, "location", F_CUT2, (-7.6, 6.2, 7.6))
kf(T1, "location", 1, (3.0, 0.2, 0.0))
kf(T1, "location", F_CUT2, (2.0, 0.0, 0.2))

# cut 2: wide fan, then push back into the vehicle
T2 = target("T2", (-40.0, -2.0, -8.0))
CAM2 = track_cam("CAM2_fan", 35, T2, C_cams)
kf(CAM2, "location", F_CUT2, (-30.0, 45.0, 60.0))
kf(CAM2, "location", F_COARSE1, (-75.0, 110.0, 135.0))
kf(CAM2, "location", F_FINE1, (-85.0, 120.0, 150.0))
kf(CAM2, "location", F_COMMIT + 30, (-82.0, 112.0, 142.0))
kf(CAM2, "location", F_PUSH0, (-78.0, 100.0, 130.0))
kf(CAM2, "location", F_PUSH1, (-9.0, 4.0, 9.5))
kf(T2, "location", F_CUT2, (-25.0, -2.0, -6.0))
kf(T2, "location", F_COARSE1, (-78.0, -6.0, -16.0))
kf(T2, "location", F_FINE1, (-80.0, -6.0, -16.0))
kf(T2, "location", F_PUSH0, (-70.0, -5.0, -14.0))
kf(T2, "location", F_PUSH1, (2.5, 0.0, 0.2))
kf(CAM2.data, "lens", F_CUT2, 35.0)
kf(CAM2.data, "lens", F_PUSH0, 35.0)
kf(CAM2.data, "lens", F_PUSH1, 50.0)

# cut 3: rear three-quarter through the peak (the wake blaze), orbit forward as it abates
T3 = target("T3", (4.5, 0.0, 0.0))
CAM3 = track_cam("CAM3_exit", 50, T3, C_cams)
kf(CAM3, "location", F_CUT3, (14.0, 9.0, 11.0))
kf(CAM3, "location", F_PEAK, (10.0, 8.5, 13.0))
kf(CAM3, "location", F_PEAK + 140, (-4.0, 8.0, 13.0))
kf(CAM3, "location", F_END, (-9.0, 7.0, 7.5))
kf(T3, "location", F_CUT3, (5.0, 0.0, 0.0))
kf(T3, "location", F_PEAK, (5.5, 0.0, 0.0))
kf(T3, "location", F_END, (2.5, 0.0, 0.2))

scene.camera = CAM1
for f, cam in ((1, CAM1), (F_CUT2, CAM2), (F_CUT3, CAM3)):
    m = scene.timeline_markers.new(cam.name, frame=f)
    m.camera = cam

# ---------------------------------------------------------------- render settings
scene.frame_start, scene.frame_end = 1, F_END
scene.render.fps = FPS
scene.render.engine = "CYCLES"
scene.render.resolution_x, scene.render.resolution_y = 1920, 1080
scene.render.resolution_percentage = PREVIEW_PCT
scene.cycles.samples = SAMPLES
scene.cycles.use_denoising = True
scene.cycles.transparent_max_bounces = 40
scene.cycles.volume_max_steps = 256
scene.cycles.volume_step_rate = 1.0
scene.cycles.device = "GPU"
try:
    prefs = bpy.context.preferences.addons["cycles"].preferences
    prefs.compute_device_type = "METAL"
    prefs.get_devices()
    for d in prefs.devices: d.use = True
except Exception as e:
    print("GPU prefs:", e)
scene.view_settings.view_transform = "AgX"
scene.view_settings.look = "AgX - Medium High Contrast"
scene.view_settings.exposure = 0.0
scene.render.image_settings.file_format = "PNG"
scene.render.image_settings.color_mode = "RGB"
scene.render.filepath = "//preview30/c_"
scene.render.use_persistent_data = True

bpy.ops.wm.save_as_mainfile(filepath=OUT_BLEND)
print("saved", OUT_BLEND)
result = {"saved": OUT_BLEND, "objects": len(bpy.data.objects), "missing": missing}
