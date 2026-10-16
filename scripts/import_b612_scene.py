#!/usr/bin/env python3
"""Import B-612's authored scene, terrain and vegetation, without re-layout.

Run AFTER import_original_assets.py using lp_extract/venv/bin/python.
Coordinates reflect Unity X consistently, including gameplay anchors.
"""
import json
import math
import struct
from pathlib import Path

from import_original_assets import *


def ron(value):
    if isinstance(value, dict):
        fields=[]
        for k,v in value.items():
            text='{' + ','.join(json.dumps(n)+':'+ron(item) for n,item in v.items()) + '}' if k in ('models','anchors','cameras') else ron(v)
            if k in ('position','scale','size','look_at'): text='('+','.join(ron(x) for x in v)+')'
            fields.append(f'{k}:{text}')
        return '(' + ','.join(fields) + ')'
    if isinstance(value, list): return '[' + ','.join(ron(v) for v in value) + ']'
    if isinstance(value, bool): return str(value).lower()
    return json.dumps(value, ensure_ascii=False)


class SceneImport:
    def __init__(self, source):
        self.exporter = Exporter(source)
        self.cat = self.exporter.catalogs['b612'] = Catalog(source, 'b612')
        self.out = OUT / 'b612'
        self.paths, self.worlds = {}, {}
        for tid,t in self.cat.transforms.items():
            if not hasattr(t,'m_LocalPosition'): continue
            chain = self.cat.chain(tid)[::-1]
            path = '/'.join(self.cat.transforms[k].m_GameObject.read().m_Name for k in chain)
            self.paths[path] = tid
            w = np.eye(4)
            for k in chain: w = w @ matrix(self.cat.transforms[k])
            self.worlds[tid] = w
        self.layout = json.loads(next((source / 'extracted' / self.cat.path.stem).glob('B612LayoutData_*.json')).read_text())
        self.metadata = dict(models={}, anchors={}, slabs=[], colliders=[], vegetation=[], cameras={})

    def world(self, tid): return REFLECT @ self.worlds[tid] @ REFLECT

    def active(self, tid):
        return all(self.cat.transforms[k].m_GameObject.read().m_IsActive for k in self.cat.chain(tid))

    def read(self, data, owner): return PPtr(**data, assetsfile=owner.assets_file).read()

    def scene(self):
        dynamic = dict(rose='World/Props/RoseArea/Rose', can='World/Props/RoseArea/WateringCan',
                       cloche='World/Props/RoseArea/Cloche', chair='World/Props/SunsetChair/SunsetChair')
        excluded = [*dynamic.values(), 'Player', 'World/Sky/CloudTowers']
        selected = []
        for path,tid in self.paths.items():
            if tid not in self.cat.renderers or not self.active(tid): continue
            r,m = self.cat.renderers[tid]
            if not r.m_Enabled or re.search(r'LOD[1-9]',m.m_Name): continue
            if any(path == p or path.startswith(p+'/') for p in excluded): continue
            selected.append(tid)
        self.exporter.export('model.b612.world', 'b612', selected_ids=selected,
                             root_id=self.paths['World'], preserve=True)
        for key,path in dynamic.items():
            tid = self.paths[path]
            selected = [t for p,t in self.paths.items() if t in self.cat.renderers and (p==path or p.startswith(path+'/'))]
            self.exporter.export(f'model.b612.{key}_authored','b612',selected_ids=selected,root_id=tid,
                                 preserve=True,aliases={'Rose_Head':'part:rose_head'})
            w = self.world(tid)
            self.metadata['models'][key] = dict(position=w[:3,3].tolist(),
                yaw=float(math.degrees(math.atan2(w[0,2],w[2,2]))),
                scale=np.linalg.norm(w[:3,:3],axis=0).tolist())
        for a in self.layout['anchors']:
            self.metadata['anchors'][a['name']] = dict(x=-a['x'],z=a['z'],height=a['heightOffset'],yaw=-a['yaw'])
        # Original transform anchors include the exact feet height on the slabs.
        for o in self.cat.env.objects:
            if o.type.name!='MonoBehaviour': continue
            d=o.read_typetree()
            if 'moreStands' not in d: continue
            self.metadata['water_y']=d['waterLevel']
            for p in [d['stand'],*d['moreStands']]:
                w=self.world(ident(self.read(p,o)))
                self.metadata['slabs'].append(dict(position=w[:3,3].tolist(),yaw=float(math.degrees(math.atan2(w[0,2],w[2,2])))))
        for c in self.layout['reviewCameras']:
            p,l=c['position'],c['lookAt']
            self.metadata['cameras'][c['name']]=dict(position=[-p['x'],p['y'],p['z']],look_at=[-l['x'],l['y'],l['z']],absolute=bool(c['absoluteHeight']),fov=c['fov'])
        # Source boxes are retained with their yaw. Skip triggers and camera-only
        # layer 9. Mesh colliders remain represented by the baked ground surface.
        for o in self.cat.env.objects:
            if o.type.name!='BoxCollider': continue
            d=o.read(); go=d.m_GameObject.read(); tid=ident(transform(go))
            if not d.m_Enabled or d.m_IsTrigger or not self.active(tid) or go.m_Layer!=8: continue
            w=self.world(tid)
            center=(w @ np.array([-d.m_Center.x,d.m_Center.y,d.m_Center.z,1]))[:3]
            size=np.asarray(vec(d.m_Size))*np.linalg.norm(w[:3,:3],axis=0)
            # Only yaw-aligned boxes are supported by the kinematic controller.
            up=w[:3,1]/np.linalg.norm(w[:3,1])
            if abs(up[1])<.98: continue
            self.metadata['colliders'].append(dict(position=(center-[0,size[1]/2,0]).tolist(),size=size.tolist(),yaw=float(math.degrees(math.atan2(w[0,2],w[2,2])))))

    def terrain(self):
        obj=next(o for o in self.cat.env.objects if o.type.name=='TerrainData')
        d=obj.read_typetree(); hm=d['m_Heightmap']; n=hm['m_Resolution']
        # Unity Terrain stores a 15-bit normalized height, row-major Z then X.
        height=np.asarray(hm['m_Heights'],dtype=np.float32).reshape(n,n)/32766*16-8
        holes=np.asarray(hm['m_Holes'],dtype=bool).reshape(n-1,n-1)
        self.height=height
        # Physics samples the same grid as the visible terrain. X reflection is
        # done at lookup time, never by guessing a new sphere-cap formula.
        (self.out/'terrain.f32').write_bytes(height.astype('<f4').tobytes())
        z,x=np.meshgrid(np.linspace(-32,32,n),np.linspace(-32,32,n),indexing='ij')
        positions=np.stack((-x,height,z),-1).reshape(-1,3)
        dz,dx=np.gradient(height,.125)
        normal=np.stack((dx,np.ones_like(dx),-dz),-1).reshape(-1,3)
        normal/=np.linalg.norm(normal,axis=1,keepdims=True)
        uv=np.stack(((x+32)/64,1-(z+32)/64),-1).reshape(-1,2)
        iz,ix=np.where(holes)
        a=iz*n+ix;b=a+1;c=a+n;e=c+1
        tris=np.stack((a,b,c,b,e,c),-1).reshape(-1).astype('<u4')
        print('terrain:',n,'grid;',len(iz),'visible cells; height range',height.min(),height.max(),flush=True)
        binary=bytearray(); doc={'asset':{'version':'2.0'},'scene':0,'scenes':[{'nodes':[0]}],
            'nodes':[{'mesh':0,'name':'B612_TerrainData'}], 'meshes':[{'primitives':[]}],
            'accessors':[],'bufferViews':[], 'materials':[{'name':'B612 baked terrain layers','pbrMetallicRoughness':{'baseColorTexture':{'index':0},'metallicFactor':0,'roughnessFactor':.92}}],
            'textures':[{'source':0}],'images':[{'uri':'b612_ground.png'}]}
        def access(a,kind,comp=5126):
            a=np.asarray(a,dtype='<f4' if comp==5126 else '<u4')
            doc['bufferViews'].append(dict(buffer=0,byteOffset=len(binary),byteLength=a.nbytes));binary.extend(a.tobytes())
            ac=dict(bufferView=len(doc['bufferViews'])-1,componentType=comp,count=len(a),type=kind)
            if kind=='VEC3':ac.update(min=a.min(0).tolist(),max=a.max(0).tolist())
            doc['accessors'].append(ac);return len(doc['accessors'])-1
        doc['meshes'][0]['primitives']=[dict(attributes=dict(POSITION=access(positions,'VEC3'),NORMAL=access(normal,'VEC3'),TEXCOORD_0=access(uv,'VEC2')),indices=access(tris,'SCALAR',5125),material=0)]
        doc['buffers']=[dict(byteLength=len(binary))]
        raw=json.dumps(doc,separators=(',',':')).encode();raw+=b' '*(-len(raw)%4)
        (self.out/'b612_terrain.glb').write_bytes(struct.pack('<4sII',b'glTF',2,28+len(raw)+len(binary))+struct.pack('<I4s',len(raw),b'JSON')+raw+struct.pack('<I4s',len(binary),b'BIN\0')+binary)
        # Bake the eight ORIGINAL splat layers with their authored tint/tiling.
        # Work in rows to keep memory bounded at 4096².
        res=4096
        splat=d['m_SplatDatabase']
        # RGBA here contains FOUR independent weights. PIL's RGBA resize
        # premultiplies RGB by A, which would erase every non-mud region.
        alpha=np.stack([np.asarray(channel.resize((res,res),Image.Resampling.BILINEAR))
            for p in splat['m_AlphaTextures'] for channel in self.read(p,obj).image.convert('RGBA').split()],-1)/255
        layers=[]
        for p in splat['m_TerrainLayers']:
            layer=self.read(p,obj);ld=layer.object_reader.read_typetree()
            im=np.asarray(layer.m_DiffuseTexture.read().image.convert('RGB'),dtype=np.float32)/255
            # Texture pixels are sRGB; layer remap multipliers are linear.
            im=np.where(im<=.04045,im/12.92,((im+.055)/1.055)**2.4)
            layers.append((im,ld))
        result=np.empty((res,res,3),dtype=np.uint8)
        for row in range(res):
            # PNG row 0 is north (+Z), as glTF V=0 at north.
            z=64*(1-(row+.5)/res)
            accum=np.zeros((res,3),dtype=np.float32)
            for i,(im,l) in enumerate(layers):
                tx=((np.arange(res)+.5)/res*64+l['m_TileOffset']['x'])/l['m_TileSize']['x']
                tz=(z+l['m_TileOffset']['y'])/l['m_TileSize']['y']
                xx=(tx*im.shape[1]).astype(int)%im.shape[1]
                zz=int((1-tz)*im.shape[0])%im.shape[0]
                tint=np.array([l['m_DiffuseRemapMax'][k] for k in 'xyz'])
                accum+=im[zz,xx]*tint*alpha[row,:,i,None]
            srgb=np.where(accum<=.0031308,accum*12.92,1.055*np.maximum(accum,0)**(1/2.4)-.055)
            result[row]=np.clip(srgb*255,0,255).astype(np.uint8)
        Image.fromarray(result).save(self.out/'b612_ground.png')
        # Verify source road heights against the decoded grid, catching flips or
        # the common erroneous 65535 denominator before anything reaches runtime.
        errors=[]
        for p in self.layout['road']:
            errors.append(abs(self.sample(-p['x'],p['z'])-p['y']))
        # Road control points predate the final nook/rose/chair sculpt. Most
        # centres are 3.5 cm below this guide, the authored path sink.
        assert np.median(errors)<.06 and max(errors)<.6,errors
        self.metadata['height_validation_max_error']=max(errors)

    def sample(self,x,z):
        gx=np.clip((-x+32)*8,0,512);gz=np.clip((z+32)*8,0,512)
        ix,iz=min(int(gx),511),min(int(gz),511);fx,fz=gx-ix,gz-iz
        h=self.height
        return float((h[iz,ix]*(1-fx)+h[iz,ix+1]*fx)*(1-fz)+(h[iz+1,ix]*(1-fx)+h[iz+1,ix+1]*fx)*fz)

    def vegetation(self):
        owner=next(o for o in self.cat.env.objects if o.type.name=='MonoBehaviour' and o.read_typetree().get('m_Name')=='B612_Main_GroundCover_Vegetation')
        d=owner.read_typetree()
        payload=self.read(d['payload'],owner).m_Script.encode('utf-8','surrogateescape')
        assert len(payload)==sum(l['instanceCount'] for l in d['layers'])*48
        instances=np.frombuffer(payload,dtype='<f4').reshape(-1,12)
        assert np.max(abs(np.linalg.norm(instances[:,4:8],axis=1)-1))<.001
        (self.out/'vegetation.bin').write_bytes(payload)
        for spec in d['layers']:
            obj=self.read(spec['layer'],owner).object_reader; layer=obj.read_typetree(); paths=[]
            for lod in range(2):
                p=layer[f'meshLOD{lod}']
                if not p['m_PathID']:paths.append(paths[0]);continue
                mesh=self.read(p,obj);mats=[self.read(p,obj) for p in layer['materials']]
                self.cat.add_loose(mesh,mats)
                # Force the loose node: a mesh may also occur in a hand-placed
                # nook tuft, whose parent transform must not affect a prototype.
                tid=ident(mesh)
                self.cat.transforms[tid]=SimpleNamespace(object_reader=mesh.object_reader,m_GameObject=SimpleNamespace(read=lambda m=mesh:SimpleNamespace(m_Name=m.m_Name)))
                self.cat.parents[tid]=None
                self.cat.renderers[tid]=(SimpleNamespace(m_Materials=[SimpleNamespace(read=lambda m=m:m) for m in mats]),mesh)
                path=self.exporter.export(f'model.b612.veg_{layer["m_Name"]}_{lod}','b612',selected_ids=[tid],preserve=True)
                paths.append(path)
            self.metadata['vegetation'].append(dict(paths=paths,start=spec['instanceStart'],count=spec['instanceCount'],lod_distance=layer['lod1Distance'],thin_start=layer['thinStart'],thin_power=layer['thinPower'],fade_end=layer['fadeEnd']))
        print('vegetation:',len(instances),'authored instances',flush=True)

    def run(self):
        import shutil
        self.out.mkdir(parents=True,exist_ok=True)
        self.scene();self.terrain();self.vegetation()
        source=next((self.exporter.source/'extracted').glob('shared_assets_assets_shared_rendering_textures_steam*/Steam_Soft_*.png'))
        shutil.copy2(source,self.out/'steam.png')
        (self.out/'scene.ron').write_text(ron(self.metadata)+'\n')
        (self.out/'scene-report.json').write_text(json.dumps(self.metadata,indent=2)+'\n')
        print('B-612 authored scene imported.',flush=True)


if __name__=='__main__': SceneImport(Path.home()/'lp_extract').run()
