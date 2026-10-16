import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
const code = await readFile(new URL('../web/audio.js', import.meta.url), 'utf8');
let serial = 0;
async function setup() {
  const requests = new Map(), sources = [], gains = [];
  class Param {
    value = 1; events = [];
    setValueAtTime(v,t) { this.value=v; this.events.push(['set',v,t]); }
    cancelAndHoldAtTime(t) { this.events.push(['hold',t]); }
    linearRampToValueAtTime(v,t) { this.events.push(['ramp',v,t]); }
  }
  class Context {
    currentTime = 10; state = 'running'; destination = {};
    createGain() { const g={gain:new Param(),connect(){},disconnect(){}};gains.push(g);return g; }
    createBufferSource() {
      const s={buffer:null,loop:false,starts:0,stops:[],connect(){},disconnect(){},start(){this.starts++},stop(t){this.stops.push(t)}};
      sources.push(s);return s;
    }
    async decodeAudioData(bytes) { assert.ok(bytes instanceof ArrayBuffer);return {length:44100*90,numberOfChannels:2,duration:90}; }
  }
  globalThis.AudioContext = Context;
  globalThis.fetch = url => new Promise(resolve => requests.set(url, () => resolve({ok:true,arrayBuffer:async()=>new ArrayBuffer(8)})));
  const api = await import('data:text/javascript;base64,' + Buffer.from(code+'\n// test '+serial++).toString('base64'));
  return {api,requests,sources,gains};
}
test('out-of-order downloads cannot start music from the previous chapter', async()=>{
  const {api,requests,sources}=await setup();
  const first=api.playTrack('music','first','first.ogg',1,[]);
  const second=api.playTrack('music','second','second.ogg',1,[]);
  requests.get('second.ogg')();await second;
  requests.get('first.ogg')();await first;
  assert.deepEqual(api.audioStatus().tracks.map(t=>t.key),['second']);
  assert.equal(sources.length,1);assert.equal(sources[0].loop,true);
  assert.equal(sources[0].buffer.duration,90);assert.equal(sources[0].starts,1);
});
test('volume changes do not restart or refill the music; crossfade stops on the audio clock',async()=>{
  const {api,requests,sources,gains}=await setup();
  const first=api.playTrack('music','first','first.ogg',.6,[]);requests.get('first.ogg')();await first;
  for(let i=0;i<100;i++)api.setVolumes(.5,.5,.5);
  assert.equal(sources.length,1);assert.equal(sources[0].starts,1);
  const second=api.playTrack('music','second','second.ogg',.6,[]);requests.get('second.ogg')();await second;
  assert.deepEqual(sources[0].stops,[11.7]);
  assert.ok(gains.some(g=>g.gain.events.some(e=>e[0]==='ramp'&&e[1]===0&&e[2]===11.67)));
});
test('stopping a track during decode cancels its eventual playback',async()=>{
  const {api,requests,sources}=await setup();
  const pending=api.playTrack('ambience','rain','rain.ogg',1,[]);
  await api.playTrack('ambience','','',0,[]);
  requests.get('rain.ogg')();await pending;
  assert.equal(sources.length,0);assert.equal(api.audioStatus().tracks.length,0);
});
