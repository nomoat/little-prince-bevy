// Player-like bot: plays a chapter in normal (non-test) mode with REAL input only
// (keyboard on a desktop viewport, or touch on an emulated iPhone), steering by the
// on-screen objective marker the way a player would. It reads `?probe=` for the
// marker position / focus prompt / story wait, screenshots every few seconds and
// logs a timeline so a human can review where a player gets stuck.
//
//   node tests/play_web.mjs <chapter> [desk|phone] [zh|en] [maxSeconds]
// Env: DIST (default ../dist), PORT (default 8791), EXTRA (extra query string), SHOT_EVERY (s, default 6)
import { chromium, devices } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.env.DIST || path.join(here, '..', 'dist'));
const [chapter = 'b612', mode = 'desk', lang = 'zh', maxS = '420'] = process.argv.slice(2);
const tag = `play-${chapter}-${mode}-${lang}`;
const out = path.join(here, 'out', tag);
fs.rmSync(out, { recursive: true, force: true });
fs.mkdirSync(out, { recursive: true });
const PORT = +(process.env.PORT || 8791);
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.ron': 'text/plain',
  '.ttf': 'font/ttf', '.png': 'image/png', '.jpg': 'image/jpeg', '.ogg': 'audio/ogg', '.wav': 'audio/wav',
  '.glb': 'model/gltf-binary', '.gltf': 'model/gltf+json', '.json': 'application/json' };
const server = http.createServer((req, res) => {
  const u = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const f = path.join(dist, u === '/' ? 'index.html' : u);
  if (!f.startsWith(dist) || !fs.existsSync(f) || fs.statSync(f).isDirectory()) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { 'content-type': types[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});
await new Promise(r => server.listen(PORT, r));
const sysChrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const exe = process.env.CHROME || (fs.existsSync(sysChrome) ? sysChrome : undefined);
const browser = await chromium.launch({
  headless: !process.env.HEADED,
  ...(exe ? { executablePath: exe } : { channel: 'chromium' }),
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist', '--enable-webgl', '--autoplay-policy=no-user-gesture-required'],
});
const sleep = ms => new Promise(r => setTimeout(r, ms));
const phone = mode === 'phone';
const dev = phone ? { ...devices['iPhone 13 landscape'], deviceScaleFactor: +(process.env.DSF || 1) } : { viewport: { width: 1280, height: 720 } };
const ctx = await browser.newContext(dev);
const page = await ctx.newPage();
let probe = null;
const log = [];
const timeline = [];
const T0 = Date.now();
const now = () => ((Date.now() - T0) / 1000).toFixed(1);
page.on('console', m => {
  const t = m.text();
  if (t.startsWith('[probe] ')) { try { probe = JSON.parse(t.slice(8)); } catch (e) { log.push('[badprobe] ' + t); } return; }
  log.push(`${now()} ${t}`);
});
page.on('pageerror', e => log.push(`${now()} [pageerror] ${e.message}`));
await page.goto(`http://localhost:${PORT}/?chapter=${chapter}&fresh=1&probe=0.25&lang=${lang}${process.env.EXTRA ? '&' + process.env.EXTRA : ''}`);

const cdp = phone ? await ctx.newCDPSession(page) : null;
const W = dev.viewport.width, H = dev.viewport.height;
const touch = (type, pts) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map(([x, y], i) => ({ x, y, id: i + 1, radiusX: 4, radiusY: 4, force: 1 })) });
let k = 1; // logical px per CSS px
const L = (x, y) => [x / k, y / k];
const btn = () => {
  const [lw, lh] = probe.win;
  return { interact: L(lw - 92, lh - 120), jump: L(lw - 190, lh - 70), joy: L(130, lh - 130), free: L(lw * 0.62, lh * 0.3) };
};

let shotN = 0, lastShot = 0;
const shot = async (why = '') => {
  const name = `${String(shotN++).padStart(3, '0')}${why ? '-' + why : ''}.png`;
  await page.screenshot({ path: path.join(out, name) }).catch(() => {});
  timeline.push(`${now()} SHOT ${name}`);
};
const keys = new Set();
const setKeys = async want => {
  for (const kk of [...keys]) if (!want.has(kk)) { await page.keyboard.up(kk); keys.delete(kk); }
  for (const kk of want) if (!keys.has(kk)) { await page.keyboard.down(kk); keys.add(kk); }
};
let joyDown = false;
const setJoy = async (vx, vy) => { // vx right, vy forward (-1..1)
  const j = btn().joy;
  if (vx === 0 && vy === 0) { if (joyDown) { await touch('touchEnd', []); joyDown = false; } return; }
  const p = [j[0] + vx * 60 / k, j[1] - vy * 60 / k];
  if (!joyDown) { await touch('touchStart', [j]); joyDown = true; }
  await touch('touchMove', [p]);
};
const move = async (x, y) => { // camera-relative axes
  if (phone) return setJoy(x, y);
  const want = new Set();
  if (y > 0.38) want.add('w'); if (y < -0.38) want.add('s');
  if (x > 0.38) want.add('d'); if (x < -0.38) want.add('a');
  await setKeys(want);
};
const press = async (holdS = 0) => { // interact / advance
  if (phone) {
    await move(0, 0);
    const b = btn().interact;
    await touch('touchStart', [b]); await sleep(holdS ? holdS * 1000 : 80); await touch('touchEnd', []);
  } else {
    await setKeys(new Set());
    await page.keyboard.down('e'); await sleep(holdS ? holdS * 1000 : 80); await page.keyboard.up('e');
  }
};
const tapScreen = async () => { if (phone) { await move(0, 0); const f = btn().free; await touch('touchStart', [f]); await sleep(60); await touch('touchEnd', []); } else await press(); };

let lastWait = '', waitSince = Date.now(), lastObj = '', lastFocus = '', lastTgt = '', lastTeach = '', act2T = 0;
let result = 'timeout';
try {
  while (!probe || probe.state !== 'Chapter') { await sleep(300); if (Date.now() - T0 > 120000) throw new Error('never reached Chapter'); }
  k = probe.win[0] / W;
  await shot('start');
  let stuckStart = null, bestD = 1e9, jumpT = 0;
  while ((Date.now() - T0) / 1000 < +maxS) {
    const p = probe;
    if (log.some(l => /\[test:(ok|fail)|panicked|pageerror/.test(l))) { result = log.find(l => /panicked|pageerror/.test(l)) ? 'error' : 'done'; }
    if (p.state !== 'Chapter') { result = `left chapter -> ${p.state}`; await shot('left'); break; }
    if (result === 'error') break;
    const s = p.story || {};
    if (s.wait !== lastWait) { timeline.push(`${now()} WAIT ${s.wait}  (prev ${lastWait} for ${((Date.now() - waitSince) / 1000).toFixed(1)}s)`); lastWait = s.wait; waitSince = Date.now(); bestD = 1e9; }
    if (s.objective !== lastObj) { timeline.push(`${now()} OBJ ${s.objective} target=${JSON.stringify(s.target)} ${s.tlabel}`); lastObj = s.objective; await shot('obj'); }
    const tg = JSON.stringify(s.target) + s.tlabel;
    if (tg !== lastTgt) { timeline.push(`${now()} TARGET ${tg}`); lastTgt = tg; }
    if ((s.teach || '') !== lastTeach) { if (s.teach) { timeline.push(`${now()} TEACH ${s.teach}`); await shot('teach'); } lastTeach = s.teach || ''; }
    if (s.teach && /(^| )C( |\|)/.test(s.teach) && Date.now() - act2T > 3000) {
      act2T = Date.now();
      if (phone) { const [lw, lh] = p.win; const b = L(lw - 190, lh - 170); await move(0, 0); await touch('touchStart', [b]); await sleep(80); await touch('touchEnd', []); }
      else await page.keyboard.press('c');
      timeline.push(`${now()} ACTION2`);
    }
    if (s.focus !== lastFocus) { if (s.focus) timeline.push(`${now()} FOCUS ${s.focus} hold=${s.hold}`); lastFocus = s.focus; }
    if (Date.now() - lastShot > (+process.env.SHOT_EVERY || 6) * 1000) { lastShot = Date.now(); await shot(); }

    if (s.wait === 'departure' || /depart/i.test(s.wait)) { await shot('depart'); await press(2.2); await sleep(1500); continue; }
    if (s.card || s.narr) { await move(0, 0); await sleep(400); continue; }
    if (s.choice > 0) { await shot('choice'); await press(); await sleep(800); continue; }
    if (p.dialogue) { await move(0, 0); await sleep(900); await tapScreen(); await sleep(300); continue; }
    const tdist = s.target ? Math.hypot(s.target[0] - p.pos[0], s.target[2] - p.pos[2]) : 0;
    if (s.focus && !/home|put|back|drop/.test(s.focus) && (!s.target || tdist < 3.5 || /door/.test(s.focus))) { await press(s.hold > 0 ? s.hold + 0.4 : 0); await sleep(500); continue; }
    if (s.target) {
      const dx = s.target[0] - p.pos[0], dz = s.target[2] - p.pos[2];
      const d = Math.hypot(dx, dz);
      const f = [Math.sin(p.cam_yaw), Math.cos(p.cam_yaw)], r = [-f[1], f[0]];
      const y = (dx * f[0] + dz * f[1]) / d, x = (dx * r[0] + dz * r[1]) / d;
      if (d < bestD - 0.3) { bestD = d; stuckStart = Date.now(); }
      if (stuckStart && Date.now() - stuckStart > 6000 && Date.now() - jumpT > 3000) { // blocked: try sidestep + jump
        jumpT = Date.now(); timeline.push(`${now()} BLOCKED d=${d.toFixed(2)} at ${p.pos.map(v => v.toFixed(1))} locked=${s.locked} drive=${s.drive} focus=${s.focus}`);
        await move(x > 0 ? 1 : -1, -0.5); await sleep(700);
        if (phone) { await move(0, 0); const b = btn().jump; await touch('touchStart', [b]); await sleep(60); await touch('touchEnd', []); } else await page.keyboard.press('Space');
        stuckStart = Date.now();
      }
      if (d > 0.6) await move(x, y); else await move(0, 0);
      await sleep(150);
      continue;
    }
    // nothing to steer by: after a few seconds wander like a curious player (also
    // satisfies "move / look / run" teaching cards)
    if (Date.now() - waitSince > 4000 && /condition|flag/.test(s.wait)) {
      const ph = ((Date.now() - T0) / 1000) % 6;
      if (phone) { await setJoy(ph < 3 ? 0.4 : -0.4, 1); }
      else {
        await setKeys(new Set(ph < 4 ? ['w', 'Shift'] : ['w', 'd']));
        await page.mouse.move(W / 2, H / 2); await page.mouse.down(); await page.mouse.move(W / 2 + 80, H / 2, { steps: 4 }); await page.mouse.up();
      }
      await sleep(250);
      continue;
    }
    await move(0, 0);
    await sleep(300);
  }
} catch (e) { result = 'error: ' + e.message; }
await shot('end');
timeline.push(`${now()} RESULT ${result}`);
fs.writeFileSync(path.join(out, 'timeline.txt'), timeline.join('\n'));
fs.writeFileSync(path.join(out, 'console.log'), log.join('\n'));
console.log(`${tag}: ${result} in ${now()}s; ${shotN} shots in ${out}`);
await browser.close(); server.close();
