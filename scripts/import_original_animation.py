#!/usr/bin/env python3
"""Resample original Unity Humanoid clips onto the supplied prince's Avatar.

Reads the source Avatar axes/limits and streamed, dense and constant curves.
Writes local bone quaternions plus hip translation; no Unity runtime is needed.
Swing/twist conversion follows the Avatar pre/post-rotation convention. Unity's
proprietary limb stretch/IK correction is not reproduced by this converter.
"""
import argparse
import bisect
import json
import math
import struct
from pathlib import Path

import numpy as np
import UnityPy

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/original/shared'


def vec(d, fields='xyz'):
    return np.array([d[k] for k in fields], dtype=np.float64)


def mul(a, b):
    return np.r_[a[3]*b[:3] + b[3]*a[:3] + np.cross(a[:3], b[:3]), a[3]*b[3]-np.dot(a[:3], b[:3])]


def inv(q):
    return q*np.array([-1, -1, -1, 1]) / np.dot(q, q)


def rotate(q, v):
    return v + 2*np.cross(q[:3], np.cross(q[:3], v) + q[3]*v)


def axis_angle(v):
    angle = np.linalg.norm(v)
    return np.r_[v*(math.sin(angle/2)/angle if angle > 1e-10 else .5), math.cos(angle/2)]


def sampled_curves(d, times):
    """All binding curves in streamed → dense → constant storage order."""
    clip = d['m_MuscleClip']['m_Clip']['data']
    stream, dense, constant = [clip[k] for k in ['m_StreamedClip', 'm_DenseClip', 'm_ConstantClip']]
    ns, nd = stream['curveCount'], dense['m_CurveCount']
    out = np.zeros((len(times), ns+nd+len(constant['data'])))
    assert out.shape[1] == len(d['m_ClipBindingConstant']['genericBindings'])
    if ns:
        data = struct.pack('<'+'I'*len(stream['data']), *stream['data'])
        cursor = 0
        keys = [[] for _ in range(ns)]
        while cursor < len(data):
            time, count = struct.unpack_from('<fi', data, cursor); cursor += 8
            for _ in range(count):
                index, *co = struct.unpack_from('<i4f', data, cursor); cursor += 20
                if math.isfinite(time): keys[index].append((time, co))
        for i, curve in enumerate(keys):
            assert curve, (d['m_Name'], 'empty streamed curve', i)
            stamps = [k[0] for k in curve]
            for j, t in enumerate(times):
                k = max(0, bisect.bisect_right(stamps, t)-1)
                start, (a,b,c,value) = curve[k]
                dt = max(0., t-start)
                out[j,i] = ((a*dt+b)*dt+c)*dt+value
    if nd:
        values = np.array(dense['m_SampleArray']).reshape(dense['m_FrameCount'], nd)
        sample_times = np.arange(len(values))/dense['m_SampleRate'] + dense['m_BeginTime']
        for i in range(nd): out[:,ns+i] = np.interp(times, sample_times, values[:,i])
    out[:,ns+nd:] = constant['data']
    # Bindings 0..6 are motion root, 7..13 body root, 14..41 limb goals,
    # and 42..136 are the 95 Humanoid muscle channels.
    human = np.zeros((len(times), 137)); human[:,13] = 1
    for i, binding in enumerate(d['m_ClipBindingConstant']['genericBindings']):
        assert binding['typeID'] == 95 and binding['customType'] == 8
        human[:,binding['attribute']] = out[:,i]
    assert np.isfinite(human).all(), d['m_Name']
    return human


def muscle_axes():
    """Unity muscle index for each Avatar swing/twist axis (x,y,z)."""
    axes = {'spine_01.x':(2,1,0), 'spine_02.x':(5,4,3), 'spine_03.x':(8,7,6),
            'neck.x':(11,10,9), 'head.x':(14,13,12)}
    for side, leg, arm, hand in [('l',21,37,55),('r',29,46,75)]:
        axes.update({f'thigh_stretch.{side}':(leg+2,leg+1,leg),
                     f'leg_stretch.{side}':(leg+4,-1,leg+3),
                     f'foot.{side}':(-1,leg+6,leg+5), f'toes_01.{side}':(-1,-1,leg+7),
                     f'shoulder.{side}':(-1,arm+1,arm), f'arm_stretch.{side}':(arm+4,arm+3,arm+2),
                     f'forearm_stretch.{side}':(arm+6,-1,arm+5), f'hand.{side}':(-1,arm+8,arm+7)})
        for finger, offset in zip(['thumb','index','middle','ring','pinky'],range(0,20,4)):
            i=hand+offset
            axes[f'c_{finger}1.{side}']=(-1,i+1,i)
            axes[f'c_{finger}2.{side}']=(-1,-1,i+2)
            axes[f'c_{finger}3.{side}']=(-1,-1,i+3)
    return axes


def convert(source):
    bundle = next((source/'bundles').glob('shared*core_prefabs*'))
    avatar = next(o.read_typetree() for o in UnityPy.load(str(bundle)).objects if o.type.name=='Avatar')
    a=avatar['m_Avatar']; human=a['m_Human']['data']; skeleton=human['m_Skeleton']['data']
    tos=dict(avatar['m_TOS']); bones=[tos[i].split('/')[-1] for i in skeleton['m_ID'][1:]]
    rest=human['m_SkeletonPose']['data']['m_X'][1:]; axes=muscle_axes()
    reference=human['m_RootX']; ref_q=vec(reference['q'],'xyzw')
    hip_offset=vec(rest[0]['t'])-vec(reference['t'])
    clips=[]; packed=[]; report=[]; frame_start=0
    folder=next((source/'extracted').glob('*core_animation*'))
    for path in sorted(folder.glob('LP_*.json')):
        d=json.loads(path.read_text()); mc=d['m_MuscleClip']; duration=mc['m_StopTime']-mc['m_StartTime']
        count=max(2,math.ceil(duration*30)+1); times=np.linspace(mc['m_StartTime'],mc['m_StopTime'],count)
        curves=sampled_curves(d,times); frames=np.zeros((count,len(bones)*4+3))
        root_positions=curves[:,7:10]*human['m_Scale']
        # Horizontal root motion is owned by the kinematic controller. Keep
        # only local sway; remove travel so looping clips cannot teleport him.
        root_positions[:,[0,2]]-=np.linspace(root_positions[0,[0,2]],root_positions[-1,[0,2]],count)
        for fi, curve in enumerate(curves):
            body_q=curve[10:14]; body_q=body_q/max(np.linalg.norm(body_q),1e-9)
            body_delta=mul(body_q,inv(ref_q))
            for bi,name in enumerate(bones):
                if bi == 0:
                    q=mul(body_delta,vec(rest[0]['q'],'xyzw'))
                elif name in axes:
                    ax=skeleton['m_AxesArray'][skeleton['m_Node'][bi+1]['m_AxesId']]
                    muscle=np.array([curve[42+i] if i>=0 else 0 for i in axes[name]])
                    angle=muscle*np.where(muscle>=0,vec(ax['m_Limit']['m_Max']),-vec(ax['m_Limit']['m_Min']))*vec(ax['m_Sgn'])
                    swing=axis_angle(np.array([0,angle[1],angle[2]])); twist=axis_angle(np.array([angle[0],0,0]))
                    q=mul(mul(mul(vec(ax['m_PreQ'],'xyzw'),swing),twist),inv(vec(ax['m_PostQ'],'xyzw')))
                else: q=vec(rest[bi]['q'],'xyzw')
                q/=np.linalg.norm(q)
                q*=np.array([1,-1,-1,1]) # Unity LH → Bevy RH
                if fi and np.dot(frames[fi-1,bi*4:bi*4+4],q)<0: q=-q
                frames[fi,bi*4:bi*4+4]=q
            hip=root_positions[fi]+rotate(body_delta,hip_offset)
            frames[fi,-3:]=hip*np.array([-1,1,1])
        assert np.isfinite(frames).all()
        packed.append(frames.astype('<f4').tobytes())
        name=d['m_Name'].removeprefix('LP_')
        clips.append(f'("{name}",(start:{frame_start},frames:{count},duration:{duration:.9f},looped:{str(mc["m_LoopTime"]).lower()}))')
        report.append(dict(name=name,source=str(path.relative_to(source)),frames=count,duration=duration))
        frame_start+=count
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/'animations.bin').write_bytes(b''.join(packed))
    (OUT/'animations.ron').write_text('(bones:'+json.dumps(bones)+',clips:['+','.join(clips)+'])\n')
    (OUT/'animations-report.json').write_text(json.dumps(dict(source=str(source),avatar_bundle=bundle.name,bones=bones,clips=report,notes=['Original muscle curves resampled at 30 Hz with source Avatar axes and asymmetric limits.','Horizontal root motion removed; gameplay controller owns movement.','Unity humanoid IK/stretch solver is not reproduced.']),indent=2)+'\n')
    print(f'Converted {len(clips)} original clips, {len(bones)} bones, {frame_start} frames.')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--source',type=Path,default=Path.home()/'lp_extract')
    convert(parser.parse_args().source)
