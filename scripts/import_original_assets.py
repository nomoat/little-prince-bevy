#!/usr/bin/env python3
"""Convert the user's local Unity bundles to portable glTF assets.

Run with ~/lp_extract/venv/bin/python scripts/import_original_assets.py.
No downloads. Sources remain untouched. See assets/original/import-report.json.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import struct
import subprocess
from pathlib import Path
from types import SimpleNamespace

import numpy as np
from PIL import Image
import UnityPy
from UnityPy.helpers.MeshHelper import MeshHandler
from UnityPy.classes.PPtr import PPtr

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets/original"
REFLECT = np.diag([-1., 1., 1., 1.])


def vec(v, fields="xyz"):
    return [getattr(v, k) for k in fields]


def matrix(t):
    x, y, z, w = vec(t.m_LocalRotation, "xyzw")
    q = np.array([[1-2*(y*y+z*z), 2*(x*y-z*w), 2*(x*z+y*w)],
                  [2*(x*y+z*w), 1-2*(x*x+z*z), 2*(y*z-x*w)],
                  [2*(x*z-y*w), 2*(y*z+x*w), 1-2*(x*x+y*y)]])
    m = np.eye(4)
    m[:3, :3] = q @ np.diag(vec(t.m_LocalScale))
    m[:3, 3] = vec(t.m_LocalPosition)
    return m


def ident(obj):
    return (obj.object_reader.assets_file.name, obj.object_reader.path_id)


def clean_name(name):
    return ''.join(c if c.isalnum() or c in '-_.' else '_' for c in name)


def multiply_quat(a, b):
    av, bv = np.array(a[:3]), np.array(b[:3])
    return [*(a[3]*bv + b[3]*av + np.cross(av, bv)), a[3]*b[3]-np.dot(av, bv)]


def transform(go):
    return next(c.component.read() for c in go.m_Component if c.component.type.name == "Transform")


class Catalog:
    def __init__(self, source, chapter):
        pattern = "shared*core_prefabs*" if chapter == "shared" else (chapter + '*' if '_scenes_' in chapter else chapter + "_scenes*")
        self.path = next((source / "bundles").glob(pattern))
        self.env = UnityPy.load(str(self.path))
        # Resolve materials in shared addressable bundles by their file IDs.
        for dependency in sorted((source / 'bundles').glob('shared*rendering_materials*.bundle')):
            self.env.load_file(str(dependency))
        self.renderers = {}
        self.transforms = {}
        self.parents = {}
        self.meshes = {}
        self.handlers = {}
        for obj in self.env.objects:
            if obj.type.name == "Transform":
                t = obj.read()
                self.transforms[ident(t)] = t
                self.parents[ident(t)] = ident(t.m_Father.read()) if t.m_Father.path_id else None
        for obj in self.env.objects:
            if obj.type.name not in ("MeshRenderer", "SkinnedMeshRenderer"):
                continue
            r = obj.read()
            go = r.m_GameObject.read()
            try:
                mesh = r.m_Mesh.read() if obj.type.name == "SkinnedMeshRenderer" else next(
                    c.component.read().m_Mesh.read() for c in go.m_Component if c.component.type.name == "MeshFilter")
            except (StopIteration, FileNotFoundError, ValueError):
                continue
            if "Collider" in mesh.m_Name or mesh.m_Name.startswith(("COL_", "CAM_")):
                continue
            tid = ident(transform(go))
            self.renderers[tid] = (r, mesh)
            self.meshes.setdefault(clean_name(mesh.m_Name), tid)
        # Vegetation is GPU-instanced in Unity, so it has no MeshRenderer.
        # Its ScriptableObject stores exact mesh/material PPtrs.
        for obj in self.env.objects:
            if obj.type.name != 'MonoBehaviour': continue
            try:
                data = obj.read_typetree()
                if 'meshLOD0' not in data or 'materials' not in data: continue
                read = lambda p: PPtr(**p, assetsfile=obj.assets_file).read()
                mesh = read(data['meshLOD0'])
                materials = [read(p) for p in data['materials']]
                self.add_loose(mesh, materials)
            except (ValueError, FileNotFoundError, KeyError):
                continue
        if chapter == 'lamp':
            mesh = next(o.read() for o in self.env.objects if o.type.name == 'Mesh' and clean_name(o.read().m_Name) == 'Tide_pool')
            mat = next(o.read() for o in self.env.objects if o.type.name == 'Material' and o.read().m_Name == 'TidePool')
            self.add_loose(mesh, [mat])

    def add_loose(self, mesh, materials):
        if clean_name(mesh.m_Name) in self.meshes: return
        tid = ident(mesh)
        self.transforms[tid] = SimpleNamespace(
            object_reader=mesh.object_reader,
            m_GameObject=SimpleNamespace(read=lambda: SimpleNamespace(m_Name=mesh.m_Name)))
        self.parents[tid] = None
        renderer = SimpleNamespace(m_Materials=[SimpleNamespace(read=lambda m=m: m) for m in materials])
        self.renderers[tid] = (renderer, mesh)
        self.meshes[clean_name(mesh.m_Name)] = tid

    def chain(self, tid):
        chain = []
        while tid:
            chain.append(tid)
            tid = self.parents[tid]
        return chain

    def handler(self, mesh):
        key = ident(mesh)
        if key not in self.handlers:
            h = MeshHandler(mesh)
            h.process()
            self.handlers[key] = h
        return self.handlers[key]


class Exporter:
    def __init__(self, source):
        self.source = source
        self.catalogs = {}
        self.textures = {}
        self.report = {"source": str(source), "models": {}, "audio": {}, "textures": {}, "unmapped": []}
        self.pngs = {}
        for p in sorted((source / "extracted").glob("*/*.png")):
            match = re.search(r"_(-?\d+)\.png$", p.name)
            if match:
                self.pngs.setdefault(int(match[1]), []).append(p)

    def texture(self, pointer, material, normal=False):
        pid = pointer["m_PathID"]
        if not pid:
            return None
        if pointer["m_FileID"] == 0:
            bundle = material.object_reader.assets_file.parent.name.removesuffix(".bundle")
            candidates = [p for p in self.pngs.get(pid, []) if p.parent.name == bundle]
        else:
            candidates = self.pngs.get(pid, [])
        if len(candidates) != 1:
            # Resolve external IDs through the bundle table; never guess by name.
            try:
                file = material.object_reader.assets_file
                external = file.externals[pointer["m_FileID"] - 1].path.rsplit("/", 1)[-1]
                for cat in self.catalogs.values():
                    for ob in cat.env.objects:
                        if ob.path_id == pid and ob.assets_file.name == external:
                            name = ob.read().m_Name
                            candidates = [p for p in self.pngs.get(pid, []) if p.name.startswith(name + "_")]
                            break
            except (AttributeError, IndexError):
                pass
        if len(candidates) != 1:
            raise ValueError(f"Unresolved texture {material.m_Name}: {pointer}: {candidates}")
        source = candidates[0]
        key = (source, normal)
        if key in self.textures:
            return self.textures[key]
        image = Image.open(source).convert("RGBA")
        image.thumbnail((1024, 1024), Image.Resampling.LANCZOS)
        if normal:
            # Unity normal maps use either RG or DXT5nm AG packing.
            a = np.asarray(image).astype(np.float32) / 255
            x = a[:, :, 0] * a[:, :, 3] * 2 - 1
            y = a[:, :, 1] * 2 - 1
            z = np.sqrt(np.maximum(0, 1-x*x-y*y))
            rgb = np.stack((x*.5+.5, y*.5+.5, z*.5+.5), -1)
            image = Image.fromarray(np.clip(rgb*255, 0, 255).astype(np.uint8))
        name = hashlib.sha256((str(source.relative_to(self.source)) + str(normal)).encode()).hexdigest()[:16] + ".png"
        target = OUT / "textures" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        image.save(target)
        self.textures[key] = "../textures/" + name
        self.report["textures"][name] = str(source.relative_to(self.source))
        return self.textures[key]

    def export(self, key, chapter, names=(), height=None, size=None, aliases=None, preserve=False, pose=None, keep_root_rotation=False, selected_ids=None, root_id=None):
        cat = self.catalogs.setdefault(chapter, None)
        if cat is None:
            cat = self.catalogs[chapter] = Catalog(self.source, chapter)
        selected = list(selected_ids or [])
        for name in names:
            matches = [n for n in cat.meshes if re.fullmatch(name, n)]
            if not matches:
                raise ValueError(f"{key}: no mesh matching {name}")
            selected.extend(cat.meshes[n] for n in sorted(matches))
        selected = list(dict.fromkeys(selected))
        required = list(selected)
        for tid in selected:
            r, _ = cat.renderers[tid]
            if hasattr(r, "m_Bones"):
                required.extend(ident(b.read()) for b in r.m_Bones if b.path_id)
        chains = [cat.chain(tid) for tid in required]
        common = root_id or next(tid for tid in chains[0] if all(tid in chain for chain in chains))
        include = set()
        for chain in chains:
            include.update(chain[:chain.index(common)+1])
        ordered = sorted(include, key=lambda tid: (len(cat.chain(tid)), tid))
        indices = {tid: i+1 for i, tid in enumerate(ordered)}
        doc = {"asset": {"version": "2.0", "generator": "Little Prince local Unity importer"},
               "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"name": key, "children": [indices[common]]}],
               "meshes": [], "materials": [], "textures": [], "images": [], "samplers": [{"magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497}],
               "skins": [], "accessors": [], "bufferViews": [], "buffers": []}
        binary = bytearray()
        worlds = {}
        def access(data, kind, component=5126, target=None):
            dtype = {5126: "<f4", 5125: "<u4", 5123: "<u2"}[component]
            a = np.ascontiguousarray(data, dtype=dtype)
            assert np.isfinite(a).all(), key
            binary.extend(b"\0" * (-len(binary) % 4))
            view = {"buffer": 0, "byteOffset": len(binary), "byteLength": a.nbytes}
            if target: view["target"] = target
            doc["bufferViews"].append(view)
            binary.extend(a.tobytes())
            acc = {"bufferView": len(doc["bufferViews"])-1, "componentType": component, "count": len(a), "type": kind}
            if kind == "VEC3":
                acc.update(min=a.min(0).tolist(), max=a.max(0).tolist())
            doc["accessors"].append(acc)
            return len(doc["accessors"])-1

        mats = {}
        image_ids = {}
        def material(mat):
            mk = ident(mat)
            if mk in mats: return mats[mk]
            d = mat.object_reader.read_typetree()
            props = d["m_SavedProperties"]
            floats = dict(props["m_Floats"])
            colors = dict(props["m_Colors"])
            texs = dict(props["m_TexEnvs"])
            color = colors.get("_BaseColor", colors.get("_Color", colors.get('_ScatterColor', colors.get('_DeepColor', dict(r=1, g=1, b=1, a=1)))))
            pbr = {"baseColorFactor": [max(0, min(1, color[k])) for k in "rgba"],
                   "metallicFactor": max(0, min(1, floats.get("_Metallic", 0))),
                   "roughnessFactor": max(.08, min(1, floats.get('_Roughness', 1-floats.get("_Smoothness", floats.get("_Glossiness", .2)))))}
            m = {"name": mat.m_Name, "pbrMetallicRoughness": pbr, "doubleSided": floats.get("_Cull", 2) == 0 or '_ColorRoot' in colors}
            for slots, field, normal in [(('_BaseMap', '_MainTex', '_BaseColorMap'), 'baseColorTexture', False), (('_BumpMap', '_NormalMap'), 'normalTexture', True), (('_EmissionMap',), 'emissiveTexture', False)]:
                tex = next((texs[s] for s in slots if s in texs and texs[s]['m_Texture']['m_PathID']), None)
                if not tex: continue
                uri = self.texture(tex["m_Texture"], mat, normal)
                if uri not in image_ids:
                    image_ids[uri] = len(doc["images"])
                    doc["images"].append({"uri": uri})
                    doc["textures"].append({"source": image_ids[uri], "sampler": 0})
                info = {"index": image_ids[uri]}
                if normal: info["scale"] = floats.get("_BumpScale", 1)
                (pbr if field=='baseColorTexture' else m)[field] = info
            packed=texs.get('_MetallicGlossMap')
            if packed and packed['m_Texture']['m_PathID']:
                uri=self.texture(packed['m_Texture'],mat)
                source=OUT/'textures'/Path(uri).name
                a=np.asarray(Image.open(source).convert('RGBA'),dtype=np.float32)/255
                # Unity: metal R, occlusion G, smoothness A. glTF: AO R,
                # roughness G, metal B. Do not treat the scalar slider as the
                # final roughness when a smoothness texture is present.
                packed_image=np.stack((a[:,:,1],1-a[:,:,3]*floats.get('_Smoothness',1),a[:,:,0],np.ones_like(a[:,:,0])),-1)
                name=source.stem+'_orm_'+str(round(floats.get('_Smoothness',1)*100))+'.png'
                Image.fromarray(np.clip(packed_image*255,0,255).astype(np.uint8)).save(OUT/'textures'/name)
                uri='../textures/'+name
                if uri not in image_ids:
                    image_ids[uri]=len(doc['images']);doc['images'].append({'uri':uri});doc['textures'].append({'source':image_ids[uri],'sampler':0})
                pbr.update(metallicRoughnessTexture={'index':image_ids[uri]},metallicFactor=1,roughnessFactor=1)
                if texs.get('_OcclusionMap',{}).get('m_Texture',{}).get('m_PathID')==packed['m_Texture']['m_PathID']:
                    m['occlusionTexture']={'index':image_ids[uri],'strength':floats.get('_OcclusionStrength',1)}
            foliage = '_Cutoff' in floats and re.search('Flower|Foliage|Grass|GroundCover|Leaves|Wheat|Ivy|Fern|Shrub', mat.m_Name, re.I)
            if foliage or floats.get("_AlphaClip", 0) > .5 or "_ALPHATEST_ON" in d.get("m_ValidKeywords", []):
                m.update(alphaMode="MASK", alphaCutoff=floats.get("_Cutoff", .5), doubleSided=True)
            elif floats.get("_Surface", 0) > .5 or d.get("m_CustomRenderQueue", -1) >= 3000:
                m["alphaMode"] = "BLEND"
            emission = colors.get("_EmissionColor", {})
            m["emissiveFactor"] = [max(0, min(1, emission.get(k, 0))) for k in "rgb"]
            mats[mk] = len(doc["materials"])
            doc["materials"].append(m)
            return mats[mk]

        for tid in ordered:
            t = cat.transforms[tid]
            name = t.m_GameObject.read().m_Name
            local = np.eye(4) if tid == common else matrix(t)
            parent = cat.parents[tid]
            quaternion = vec(t.m_LocalRotation, 'xyzw') if tid != common else [0, 0, 0, 1]
            if tid == common and keep_root_rotation:
                quaternion = vec(t.m_LocalRotation, 'xyzw')
                local = matrix(t)
                local[:3, 3] = 0
                local[:3, :3] /= np.array(vec(t.m_LocalScale))
            if pose and tid != common:
                basis = worlds.get(parent, np.eye(4))[:3, :3]
                basis = basis / np.maximum(np.linalg.norm(basis, axis=0), 1e-8)
                angle, axis = 0, np.array([0., 0., 1.])
                if name in ('arm_stretch.l', 'arm_stretch.r'):
                    point = worlds.get(parent, np.eye(4)) @ local[:, 3]
                    angle = -np.sign(point[0]) * 1.25
                if pose == 'seated':
                    if name.startswith('thigh_stretch.'):
                        angle, axis = -1.45, np.array([1., 0., 0.])
                    elif name.startswith('leg_stretch.'):
                        angle, axis = 1.45, np.array([1., 0., 0.])
                if angle:
                    axis = basis.T @ axis
                    axis /= np.linalg.norm(axis)
                    correction = [*(axis * math.sin(angle/2)), math.cos(angle/2)]
                    quaternion = multiply_quat(correction, quaternion)
                    fake = SimpleNamespace(m_LocalPosition=t.m_LocalPosition, m_LocalScale=t.m_LocalScale,
                                           m_LocalRotation=SimpleNamespace(**dict(zip('xyzw', quaternion))))
                    local = matrix(fake)
            worlds[tid] = worlds[parent] @ local if parent in worlds else local
            node = {"name": (aliases or {}).get(name, name)}
            if tid != common:
                node.update(translation=vec(t.m_LocalPosition), rotation=quaternion, scale=vec(t.m_LocalScale))
                node["translation"][0] *= -1
                node["rotation"][1] *= -1
                node["rotation"][2] *= -1
            elif keep_root_rotation:
                node['rotation'] = [quaternion[0], -quaternion[1], -quaternion[2], quaternion[3]]
            children = [indices[c] for c in ordered if cat.parents[c] == tid]
            if children: node["children"] = children
            doc["nodes"].append(node)

        bounds = []
        mesh_ids = {}
        for tid in selected:
            r, mesh = cat.renderers[tid]
            mesh_key = (ident(mesh), tuple(ident(p.read()) for p in r.m_Materials))
            if mesh_key in mesh_ids and not hasattr(r, 'm_Bones'):
                doc['nodes'][indices[tid]]['mesh'] = mesh_ids[mesh_key]
                continue
            h = cat.handler(mesh)
            verts = np.asarray(h.m_Vertices, dtype=np.float64)[:, :3]
            pos = verts * [-1, 1, 1]
            attrs = {"POSITION": access(pos, "VEC3", target=34962)}
            if h.m_Normals:
                normals = np.asarray(h.m_Normals)[:, :3] * [-1, 1, 1]
                normals[np.linalg.norm(normals, axis=1) < 1e-8] = [0, 1, 0]
                normals /= np.maximum(np.linalg.norm(normals, axis=1, keepdims=True), 1e-8)
                attrs["NORMAL"] = access(normals, "VEC3", target=34962)
            if h.m_UV0:
                uv = np.asarray(h.m_UV0)[:, :2].copy()
                uv[:, 1] = 1-uv[:, 1]
                attrs["TEXCOORD_0"] = access(uv, "VEC2", target=34962)
            if h.m_Colors:
                # Foliage channels encode wind weights/phase/AO, not RGB.
                saved=[p.read().object_reader.read_typetree()['m_SavedProperties'] for p in r.m_Materials]
                props = [dict(p['m_Floats']) for p in saved]
                palettes=[dict(p['m_Colors']) for p in saved]
                grass=next((p for p in palettes if '_ColorRoot' in p),None)
                if grass:
                    masks=np.asarray(h.m_Colors,dtype=np.float32)
                    if masks.max()>1:masks/=255
                    root,mid,tip=[np.array([grass[k][c] for c in 'rgb']) for k in ['_ColorRoot','_ColorMid','_ColorTip']]
                    growth=masks[:,0:1]
                    rgb=root+(mid-root)*np.minimum(growth*4,1)
                    rgb+=(tip-mid)*np.maximum((growth-.45)/.55,0)*.7
                    rgb*=.65+.35*masks[:,2:3]
                    rgb=np.where(rgb<=.04045,rgb/12.92,((rgb+.055)/1.055)**2.4)
                    attrs['COLOR_0']=access(np.c_[rgb,np.ones(len(rgb))],'VEC4',target=34962)
                elif not any('_WindBend' in p or '_WindMode' in p for p in props):
                    color = np.asarray(h.m_Colors, dtype=np.float32)
                    if color.max() > 1: color /= 255
                    attrs["COLOR_0"] = access(np.clip(color, 0, 1), "VEC4", target=34962)
            homogeneous = np.c_[verts, np.ones(len(verts))]
            if hasattr(r, "m_Bones") and r.m_Bones and h.m_BoneIndices:
                bones = [ident(b.read()) for b in r.m_Bones]
                # Unity omits the weight channel for rigid one-bone meshes
                # (hats, crowns, pens). Their first joint has implicit weight 1.
                weights = np.asarray(h.m_BoneWeights) if h.m_BoneWeights else np.tile([1., 0., 0., 0.], (len(verts), 1))
                weights /= np.maximum(weights.sum(1, keepdims=True), 1e-8)
                joints = np.asarray(h.m_BoneIndices)
                if joints.shape[1] < 4:
                    joints = np.pad(joints, ((0, 0), (0, 4-joints.shape[1])))
                attrs["JOINTS_0"] = access(joints, "VEC4", 5123, 34962)
                attrs["WEIGHTS_0"] = access(weights, "VEC4", target=34962)
                bind = [np.array([[getattr(m, f'e{i}{j}') for j in range(4)] for i in range(4)]) for m in mesh.m_BindPose]
                assert len(bind) == len(bones)
                # glTF skin matrices are jointWorld * inverseBind. The mesh's
                # own transform is cancelled by the skinning shader.
                ibm = np.array([(REFLECT @ m @ REFLECT).T.reshape(16) for m in bind])
                doc["skins"].append({"joints": [indices[b] for b in bones], "inverseBindMatrices": access(ibm, "MAT4")})
                doc["nodes"][indices[tid]]["skin"] = len(doc["skins"])-1
                skinmat = np.array([worlds[b] @ m for b, m in zip(bones, bind)])
                posed = np.zeros((len(verts), 4))
                for j in range(4):
                    posed += np.einsum('nij,nj->ni', skinmat[joints[:,j]], homogeneous) * weights[:,j,None]
                bounds.append(posed[:, :3] * [-1, 1, 1])
            else:
                bounds.append((homogeneous @ worlds[tid].T)[:, :3] * [-1, 1, 1])
            primitives = []
            for i, triangles in enumerate(h.get_triangles()):
                if not triangles: continue
                indices_array = np.asarray(triangles)[:, ::-1].reshape(-1)
                primitive = {"attributes": attrs, "indices": access(indices_array, "SCALAR", 5125, 34963)}
                if r.m_Materials:
                    primitive["material"] = material(r.m_Materials[min(i, len(r.m_Materials)-1)].read())
                primitives.append(primitive)
            doc["nodes"][indices[tid]]["mesh"] = len(doc["meshes"])
            mesh_ids[mesh_key] = len(doc['meshes'])
            doc["meshes"].append({"name": mesh.m_Name, "primitives": primitives})

        all_vertices = np.concatenate(bounds)
        lo, hi = all_vertices.min(0), all_vertices.max(0)
        extent = hi-lo
        scaling = np.ones(3)
        if height: scaling[:] = height / extent[1]
        if size: scaling = np.array([s / e if s and abs(e) > 1e-6 else 1 for s, e in zip(size, extent)])
        offset = np.zeros(3) if preserve else -np.array([(lo[0]+hi[0])*.5, lo[1], (lo[2]+hi[2])*.5]) * scaling
        doc["nodes"][0].update(scale=scaling.tolist(), translation=offset.tolist())
        binary.extend(b'\0' * (-len(binary) % 4))
        doc['buffers'] = [{'byteLength': len(binary)}]
        for field in ('skins', 'materials', 'images', 'textures', 'samplers'):
            if not doc[field]: del doc[field]
        raw = json.dumps(doc, separators=(',', ':'), allow_nan=False).encode()
        raw += b' ' * (-len(raw) % 4)
        target = OUT / chapter / (key.removeprefix('model.').replace('.', '_') + '.glb')
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(struct.pack('<4sII', b'glTF', 2, 28+len(raw)+len(binary)) + struct.pack('<I4s', len(raw), b'JSON') + raw + struct.pack('<I4s',len(binary),b'BIN\0') + binary)
        path = str(target.relative_to(ROOT / 'assets'))
        self.report['models'][key] = {'path': path, 'bundle': cat.path.name, 'meshes': [cat.renderers[t][1].m_Name for t in selected], 'source_bounds': [lo.tolist(), hi.tolist()], 'size': (extent*scaling).tolist(), 'skins': len(doc.get('skins', []))}
        print(key, '→', path, flush=True)
        return path

    def audio(self, chapter, name):
        folder = next((self.source / 'extracted').glob(chapter + '*_audio_*'))
        sources = sorted(folder.glob(name + '_*.*'))
        if not sources: raise ValueError(f'Missing audio {chapter}/{name}')
        result = []
        for i, source in enumerate(sources):
            target = OUT / 'audio' / (chapter + '_' + name + (f'_{i}' if len(sources)>1 else '') + '.ogg')
            target.parent.mkdir(parents=True, exist_ok=True)
            if not target.exists():
                temporary = target.with_suffix('.tmp.ogg')
                subprocess.run(['ffmpeg', '-v', 'error', '-y', '-i', str(source), '-ac', '2', '-c:a', 'vorbis', '-strict', 'experimental', '-b:a', '128k', str(temporary)], check=True)
                temporary.replace(target)
            path = str(target.relative_to(ROOT / 'assets'))
            self.report['audio'][path] = str(source.relative_to(self.source))
            result.append(path)
        return result


def update_manifest(paths):
    for file in [ROOT / 'assets/manifest.ron', *sorted((ROOT / 'assets/chapters').glob('*.ron'))]:
        text = file.read_text()
        for key, value in paths.items():
            pattern = r'("' + re.escape(key) + r'"\s*:\s*\()(.*?)(\),)'
            def replace(match):
                body = match[2]
                body = re.sub(r'\bpath:\s*"[^"]*",?\s*', '', body)
                body = re.sub(r'\bvariants:\s*\[[^\]]*\],?\s*', '', body)
                if key.startswith('model.'):
                    body = re.sub(r'\bscale:\s*[\d.]+,?\s*', '', body)
                field = 'variants: ' + json.dumps(value) if isinstance(value, list) else 'path: ' + json.dumps(value)
                return match[1] + field + ', ' + body + match[3]
            text = re.sub(pattern, replace, text)
        file.write_text(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path.home() / 'lp_extract')
    parser.add_argument('--only', help='Only model keys containing this string (for iteration)')
    parser.add_argument('--skip-audio', action='store_true')
    args = parser.parse_args()
    exporter = Exporter(args.source)
    # Reviewed semantic mapping: regexes select original render meshes, never
    # colliders or similarly named placeholders. Sizes match the Bevy contracts.
    from original_asset_map import MODELS, AUDIO
    paths = {}
    for key, spec in MODELS.items():
        if args.only and args.only not in key: continue
        try:
            paths[key] = exporter.export(key, **spec)
        except (ValueError, FileNotFoundError) as error:
            exporter.report['unmapped'].append({'key': key, 'reason': str(error)})
            print('UNMAPPED', key, str(error), flush=True)
    if not args.skip_audio and not args.only:
        for key, (chapter, name) in AUDIO.items():
            files = exporter.audio(chapter, name)
            paths[key] = files if len(files)>1 else files[0]
    update_manifest(paths)
    report_path = OUT / 'import-report.json'
    if report_path.exists() and (args.only or args.skip_audio):
        previous = json.loads(report_path.read_text())
        for field in ('models', 'audio', 'textures'):
            previous[field].update(exporter.report[field])
        if not args.only:
            previous['unmapped'] = exporter.report['unmapped']
        exporter.report = previous
    report_path.write_text(json.dumps(exporter.report, indent=2, ensure_ascii=False) + '\n')
    print(f'Imported {len(paths)} manifest entries.')
    if exporter.report['unmapped']:
        raise SystemExit('Some assets could not be imported; inspect assets/original/import-report.json')


if __name__ == '__main__':
    main()
