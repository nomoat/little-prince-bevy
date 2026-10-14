#!/usr/bin/env python3
"""Generates the small one-shot sounds used by the vanity / bar / geo chapters
(dry hand claps, a pencil scratch, a bottle gulp and glass clink). Pure
synthesis from seeded noise, released as CC0 with this project.

    python3 tests/gen_vbg_sfx.py      # writes assets/chapters/{vanity,geo,bar}/*.wav
"""
import math, os, random, struct, wave

SR = 22050
ROOT = os.path.join(os.path.dirname(__file__), '..', 'assets', 'chapters')


def write(path, samples):
    peak = max(1e-6, max(abs(s) for s in samples))
    with wave.open(path, 'wb') as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
        w.writeframes(b''.join(struct.pack('<h', int(max(-1, min(1, s / peak * 0.9)) * 32767)) for s in samples))


def biquad_bp(x, f, q):
    w0 = 2 * math.pi * f / SR; a = math.sin(w0) / (2 * q); c = math.cos(w0)
    b0, b1, b2, a0, a1, a2 = a, 0, -a, 1 + a, -2 * c, 1 - a
    y = []; x1 = x2 = y1 = y2 = 0.0
    for s in x:
        o = (b0 * s + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2) / a0
        x2, x1, y2, y1 = x1, s, y1, o
        y.append(o)
    return y


def clap(seed):
    r = random.Random(seed)
    n = int(SR * 0.22)
    noise = [r.uniform(-1, 1) for _ in range(n)]
    f = 1100 + r.uniform(-250, 350)
    body = biquad_bp(noise, f, 1.4 + r.uniform(0, 0.8))
    crack = biquad_bp(noise, 2600 + r.uniform(-300, 600), 0.9)
    out = []
    for i in range(n):
        t = i / SR
        # two or three micro-bursts (palms meet unevenly) then a fast decay
        burst = sum(math.exp(-((t - d) * 900) ** 2) for d in (0.0, 0.004 + r.uniform(0, 0.004)))
        env = min(1, t * 2000) * math.exp(-t * (38 + r.uniform(-6, 8)))
        out.append((body[i] * 0.8 + crack[i] * 0.6) * env + noise[i] * burst * 0.15)
    return out


def scratch(seed, secs):
    r = random.Random(seed)
    n = int(SR * secs)
    noise = [r.uniform(-1, 1) for _ in range(n)]
    hi = biquad_bp(noise, 4200, 0.7)
    out = []
    for i in range(n):
        t = i / SR
        stroke = 0.5 + 0.5 * math.sin(t * 2 * math.pi * 7.5 + math.sin(t * 13) * 2)
        env = min(1, t * 30) * min(1, (secs - t) * 20)
        out.append(hi[i] * stroke * env)
    return out


def gulp(seed):
    r = random.Random(seed)
    n = int(SR * 0.35)
    out = []
    ph = 0.0
    for i in range(n):
        t = i / SR
        f = 180 + 260 * math.exp(-t * 14)
        ph += 2 * math.pi * f / SR
        env = math.sin(min(1, t / 0.35) * math.pi) ** 2
        out.append(math.sin(ph) * env * 0.7 + r.uniform(-1, 1) * env * 0.05)
    return out


def clink(seed):
    r = random.Random(seed)
    n = int(SR * 0.6)
    parts = [(2350, 1.0), (3920, 0.6), (5480, 0.35), (7010, 0.2)]
    return [sum(a * math.sin(2 * math.pi * f * (i / SR) + r.random()) * math.exp(-(i / SR) * (9 + f / 900)) for f, a in parts) for i in range(n)]


os.makedirs(os.path.join(ROOT, 'vanity'), exist_ok=True)
os.makedirs(os.path.join(ROOT, 'geo'), exist_ok=True)
os.makedirs(os.path.join(ROOT, 'bar'), exist_ok=True)
for i in range(4):
    write(os.path.join(ROOT, 'vanity', f'clap_{i}.wav'), clap(100 + i))
write(os.path.join(ROOT, 'geo', 'pencil.wav'), scratch(7, 1.1))
write(os.path.join(ROOT, 'bar', 'gulp.wav'), gulp(3))
write(os.path.join(ROOT, 'bar', 'clink.wav'), clink(5))
print('ok')
