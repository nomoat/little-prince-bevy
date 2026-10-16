// Decode once; let Web Audio's rendering thread play whole buffers. No 46 ms
// Rust/main-thread refill loop. Fades use the audio clock, not animation frames.
let engine;
const cache = new Map();
const tracks = new Map();
const generations = new Map();
const effects = new Set();
const pendingEffects = new Set();
let decodedBytes = 0;
const CACHE_BUDGET = 64 * 1024 * 1024;
const MAX_EFFECTS = 24;

function audio() {
  if (engine) return engine;
  const Context = globalThis.AudioContext || globalThis.webkitAudioContext;
  const context = new Context({ latencyHint: 'interactive' });
  const buses = Object.fromEntries(['music', 'ambience', 'sfx'].map(name => {
    const gain = context.createGain();
    gain.connect(context.destination);
    return [name, gain];
  }));
  engine = { context, buses };
  const resume = () => { if (context.state === 'suspended') context.resume().catch(report); };
  if (globalThis.addEventListener) {
    for (const event of ['pointerdown', 'keydown', 'touchend']) globalThis.addEventListener(event, resume, { capture: true });
  }
  return engine;
}
function report(error) { console.warn('Audio playback:', error); }
function trimCache() {
  for (const [key, entry] of cache) {
    if (decodedBytes <= CACHE_BUDGET && cache.size <= 48) break;
    if (!entry.bytes || [...tracks.values()].some(t => t.url === key)) continue;
    cache.delete(key); decodedBytes -= entry.bytes;
  }
}
function load(url, bytes) {
  if (cache.has(url)) {
    const entry = cache.get(url); cache.delete(url); cache.set(url, entry);
    return entry.promise;
  }
  // wasm-bindgen's borrowed byte slice expires on return. Copy before await.
  const copy = bytes?.length ? bytes.slice().buffer : null;
  const entry = { bytes: 0, promise: null };
  entry.promise = (async () => {
    const ctx = audio().context;
    let encoded = copy;
    if (!encoded) {
      const response = await fetch(url);
      if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
      encoded = await response.arrayBuffer();
    }
    const buffer = await ctx.decodeAudioData(encoded);
    entry.bytes = buffer.length * buffer.numberOfChannels * 4;
    decodedBytes += entry.bytes;
    trimCache();
    return buffer;
  })().catch(error => { cache.delete(url); throw error; });
  cache.set(url, entry);
  return entry.promise;
}
function ramp(param, value, seconds) {
  const now = audio().context.currentTime;
  if (param.cancelAndHoldAtTime) param.cancelAndHoldAtTime(now);
  else { param.cancelScheduledValues(now); param.setValueAtTime(param.value, now); }
  param.linearRampToValueAtTime(value, now + seconds);
}
function retire(track) {
  if (!track?.source || track.stopped) return;
  track.stopped = true;
  ramp(track.gain.gain, 0, 1.67);
  track.source.stop(audio().context.currentTime + 1.7);
}

export function setVolumes(master, music, sfx) {
  const { buses } = audio();
  ramp(buses.music.gain, master * music, 0.03);
  ramp(buses.ambience.gain, master * sfx * 0.8, 0.03);
  ramp(buses.sfx.gain, master * sfx, 0.03);
}
export async function playTrack(bus, key, url, volume, bytes) {
  const generation = (generations.get(bus) || 0) + 1;
  generations.set(bus, generation);
  if (!url) { retire(tracks.get(bus)); tracks.delete(bus); trimCache(); return; }
  try {
    const buffer = await load(url, bytes);
    // Changing chapters while decoding must never resurrect an old track.
    if (generations.get(bus) !== generation) return;
    const { context, buses } = audio();
    retire(tracks.get(bus));
    const source = context.createBufferSource();
    const gain = context.createGain();
    source.buffer = buffer; source.loop = true;
    gain.gain.setValueAtTime(0, context.currentTime);
    source.connect(gain); gain.connect(buses[bus]);
    const track = { key, url, source, gain, stopped: false, started: context.currentTime };
    tracks.set(bus, track);
    source.onended = () => { source.disconnect(); gain.disconnect(); trimCache(); };
    source.start();
    ramp(gain.gain, volume, 2.0);
  } catch (error) { report(error); }
}
export async function playEffect(url, volume, bytes) {
  if (volume <= 0.001 || pendingEffects.has(url)) return;
  pendingEffects.add(url);
  const requested = performance.now();
  try {
    const buffer = await load(url, bytes);
    // Do not play a burst of stale footsteps after a slow first download.
    if (performance.now() - requested > 1000) return;
    const { context, buses } = audio();
    if (effects.size >= MAX_EFFECTS) {
      const oldest = effects.values().next().value; effects.delete(oldest); oldest.stop();
    }
    const source = context.createBufferSource();
    const gain = context.createGain();
    source.buffer = buffer; gain.gain.value = volume;
    source.connect(gain); gain.connect(buses.sfx); effects.add(source);
    source.onended = () => { effects.delete(source); source.disconnect(); gain.disconnect(); };
    source.start();
  } catch (error) { report(error); }
  finally { pendingEffects.delete(url); }
}
// Read-only diagnostics for browser regression checks, including stalled-frame tests.
export function audioStatus() {
  return { state: engine?.context.state || 'uninitialized', time: engine?.context.currentTime || 0,
    tracks: [...tracks.values()].map(t => ({ key: t.key, loop: t.source.loop, duration: t.source.buffer.duration,
      position: (engine.context.currentTime - t.started) % t.source.buffer.duration })),
    voices: effects.size, cachedBytes: decodedBytes, cachedFiles: cache.size };
}
if (typeof window !== 'undefined') window.__lpAudioStatus = audioStatus;
