// Headless UI tests with real input (no autopilot), using the `?probe=<s>`
// state dump the game prints to the console:
//
//   touch     phone emulation (pixel ratio 2; touch1 = ratio 1, 844×390) (hasTouch, coarse pointer): touch controls appear by
//             themselves; joystick moves the prince; drag looks; pinch zooms; jump
//             and menu buttons work.
//   keyboard  WASD moves, Space jumps, mouse wheel zooms, Esc opens the menu.
//   save      fresh start → Begin → map shows b612 only → enter b612 → reload page:
//             the title offers Continue and the save remembers b612 (localStorage).
//   lang      ?lang=en, then the settings language button switches to zh and back.
//   free      ?free=1&map=1 → every planet can be entered (clicks earth's node).
//
//   node tests/ui_web.mjs [touch touch1 keyboard save lang free]
// Env: DIST (default ../dist), PORT (default 8790), HEADED=1.
import { chromium, devices } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.env.DIST || path.join(here, '..', 'dist'));
const out = path.join(here, 'out');
fs.mkdirSync(out, { recursive: true });
const PORT = +(process.env.PORT || 8790);
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
const results = [];
function check(name, ok, detail = '') {
  results.push({ name, ok });
  console.log(`  ${ok ? 'ok  ' : 'FAIL'} ${name}${detail ? ' — ' + detail : ''}`);
}

/** Opens a page; returns helpers around the latest probe. */
async function open(ctx, query, tag) {
  const page = await ctx.newPage();
  const st = { probe: null, log: [] };
  page.on('console', m => {
    const t = m.text();
    st.log.push(t);
    if (t.startsWith('[probe] ')) { try { st.probe = JSON.parse(t.slice(8)); } catch {} }
  });
  page.on('pageerror', e => st.log.push('[pageerror] ' + e.message));
  await page.goto(`http://localhost:${PORT}/?${query}`);
  const waitFor = async (pred, secs = 60, what = 'condition') => {
    const t0 = Date.now();
    while (Date.now() - t0 < secs * 1000) {
      if (st.probe && pred(st.probe)) return st.probe;
      await sleep(200);
    }
    throw new Error(`timeout waiting for ${what}; last probe ${JSON.stringify(st.probe)}`);
  };
  const fresh = async () => { const n = st.log.length; while (st.log.length === n) await sleep(50); await sleep(300); return st.probe; };
  const shot = async name => page.screenshot({ path: path.join(out, `ui-${tag}-${name}.png`) });
  const done = async () => { fs.writeFileSync(path.join(out, `ui-${tag}.log`), st.log.join('\n')); await page.close(); };
  return { page, st, waitFor, fresh, shot, done };
}

const dist2 = (a, b) => Math.hypot(a[0] - b[0], a[2] - b[2]);

/** Plays past the chapter's opening: taps / presses to advance until the
 *  prince can be moved (his position changes when we push forward). */
async function untilControllable(p, push, secs = 90) {
  const t0 = Date.now();
  while (Date.now() - t0 < secs * 1000) {
    const a = p.st.probe;
    await push();
    await sleep(400);
    const b = p.st.probe;
    if (a && b && a.state === 'Chapter' && b.state === 'Chapter' && !b.dialogue && dist2(a.pos, b.pos) > 0.25) return true;
  }
  return false;
}

// ---------------------------------------------------------------- touch
async function touchTest(dsf = 2) {
  console.log(`touch (phone emulation, landscape, pixel ratio ${dsf})`);
  const dev = dsf === 2 ? (devices['iPhone 13 landscape'] || { viewport: { width: 750, height: 342 }, isMobile: true, hasTouch: true })
    : { viewport: { width: 844, height: 390 }, isMobile: true, hasTouch: true };
  const ctx = await browser.newContext({ ...dev, deviceScaleFactor: dsf });
  const p = await open(ctx, 'chapter=king&fresh=1&probe=0.25', `touch${dsf}`);
  const W = dev.viewport.width, H = dev.viewport.height;
  const cdp = await ctx.newCDPSession(p.page);
  const touch = (type, pts) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map(([x, y], i) => ({ x, y, id: i + 1, radiusX: 4, radiusY: 4, force: 1 })) });
  const tap = async (x, y) => { await touch('touchStart', [[x, y]]); await sleep(60); await touch('touchEnd', []); };
  const drag = async (from, to, ms = 600, holdMs = 0) => {
    await touch('touchStart', [from]);
    const n = 12;
    for (let i = 1; i <= n; i++) { await touch('touchMove', [[from[0] + (to[0] - from[0]) * i / n, from[1] + (to[1] - from[1]) * i / n]]); await sleep(ms / n); }
    if (holdMs) { const t0 = Date.now(); while (Date.now() - t0 < holdMs) { await touch('touchMove', [to]); await sleep(100); } }
    await touch('touchEnd', []);
  };
  try {
    await p.waitFor(s => s.state === 'Chapter', 90, 'chapter');
    const pr = await p.fresh();
    check('touch controls shown automatically on a phone', pr.touch === true);
    await p.shot('start');
    // The game lays the controls out in the window's logical px (pr.win);
    // convert to CSS px for the touch events. Same layout as input.rs.
    const [lw, lh] = pr.win;
    const k = lw / W;
    const L = (x, y) => [x / k, y / k];
    const btn = { interact: [lw - 92, lh - 120, 46], jump: [lw - 190, lh - 70, 36], menu: [lw - 36, 36, 24], tasks: [lw - 92, 36, 24] };
    // a free spot on the right half, far from every button
    let free = null, best = -1;
    for (let x = lw * 0.5; x < lw * 0.97; x += 4) for (let y = lh * 0.08; y < lh * 0.92; y += 4) {
      const m = Math.min(...Object.values(btn).map(([bx, by, r]) => Math.hypot(x - bx, y - by) - r - 10));
      if (m > best) { best = m; free = [x, y]; }
    }
    const joy = L(130, lh - 130);
    const ok = await untilControllable(p, async () => {
      await tap(...L(...free));
      await drag(joy, [joy[0], joy[1] - 70 / k], 300, 300);
    });
    check('opening can be advanced by tapping', ok);
    let a = await p.fresh();
    // pull the stick down: walk back from the palace door (forward may be blocked by it)
    await drag(joy, [joy[0] + 5, joy[1] + 75 / k], 300, 2000);
    let b = await p.fresh();
    check('joystick walks the prince', dist2(a.pos, b.pos) > 1.0, `moved ${dist2(a.pos, b.pos).toFixed(2)} m`);
    await p.shot('walked');
    a = await p.fresh();
    const f = L(...free);
    await drag(f, [f[0] - 160, f[1]], 600);
    b = await p.fresh();
    check('drag on the right turns the camera', Math.abs(b.cam_yaw - a.cam_yaw) > 0.15, `yaw ${a.cam_yaw.toFixed(2)} → ${b.cam_yaw.toFixed(2)}`);
    a = await p.fresh();
    await touch('touchStart', [[f[0] - 8, f[1]], [f[0] + 8, f[1]]]);
    for (let i = 1; i <= 12; i++) { await touch('touchMove', [[f[0] - 8 - i * 7, f[1] + i * 4], [f[0] + 8 + i * 2, f[1] - i * 4]]); await sleep(40); }
    await touch('touchEnd', []);
    b = await p.fresh();
    check('pinch zooms the camera', Math.abs(b.cam_dist - a.cam_dist) > 0.3, `distance ${a.cam_dist.toFixed(2)} → ${b.cam_dist.toFixed(2)}`);
    // jump button: sample the height for a moment after the tap
    a = await p.fresh();
    await tap(...L(btn.jump[0], btn.jump[1]));
    let maxY = a.pos[1];
    for (let i = 0; i < 6; i++) { await sleep(150); maxY = Math.max(maxY, p.st.probe.pos[1]); }
    check('jump button jumps', maxY - a.pos[1] > 0.15, `rose ${(maxY - a.pos[1]).toFixed(2)} m`);
    await sleep(800);
    await tap(...L(btn.menu[0], btn.menu[1]));
    await sleep(700);
    check('menu button opens the menu', (await p.fresh()).menu === true);
    await p.shot('menu');
  } catch (e) { check('touch test ran', false, e.message); }
  const errs = p.st.log.filter(l => l.includes('panicked') || l.startsWith('[pageerror]'));
  check('no panics / page errors', errs.length === 0, errs.slice(0, 2).join(' | '));
  await p.done();
  await ctx.close();
}

// ---------------------------------------------------------------- keyboard
async function keyboardTest() {
  console.log('keyboard + mouse (desktop)');
  const ctx = await browser.newContext({ viewport: { width: 960, height: 540 } });
  const p = await open(ctx, 'chapter=king&fresh=1&probe=0.25', 'kb');
  const k = p.page.keyboard;
  try {
    await p.waitFor(s => s.state === 'Chapter', 90, 'chapter');
    await p.page.mouse.click(480, 270);
    check('no touch controls on desktop', (await p.fresh()).touch === false);
    const ok = await untilControllable(p, async () => { await k.press('e'); await k.down('w'); await sleep(250); await k.up('w'); });
    check('opening can be advanced with E', ok);
    let a = await p.fresh();
    await k.down('w'); await sleep(1500); await k.up('w');
    let b = await p.fresh();
    const walk = dist2(a.pos, b.pos);
    check('W walks', walk > 1.0, `${walk.toFixed(2)} m`);
    a = await p.fresh();
    // run back the way he came (forward may be blocked by the closed door)
    await k.down('Shift'); await k.down('s'); await sleep(1500); await k.up('s'); await k.up('Shift');
    b = await p.fresh();
    check('Shift runs faster', dist2(a.pos, b.pos) > walk * 1.15, `${dist2(a.pos, b.pos).toFixed(2)} m vs ${walk.toFixed(2)} m`);
    a = await p.fresh();
    await k.press('Space');
    let maxY = a.pos[1];
    for (let i = 0; i < 6; i++) { await sleep(150); maxY = Math.max(maxY, p.st.probe.pos[1]); }
    check('Space jumps', maxY - a.pos[1] > 0.15, `rose ${(maxY - a.pos[1]).toFixed(2)} m`);
    a = await p.fresh();
    await p.page.mouse.move(480, 300);
    for (let i = 0; i < 5; i++) { await p.page.mouse.wheel(0, 300); await sleep(60); }
    b = await p.fresh();
    check('mouse wheel zooms', Math.abs(b.cam_dist - a.cam_dist) > 0.3, `${a.cam_dist.toFixed(2)} → ${b.cam_dist.toFixed(2)}`);
    a = await p.fresh();
    await p.page.mouse.move(400, 300); await p.page.mouse.down();
    for (let i = 1; i <= 10; i++) { await p.page.mouse.move(400 + i * 20, 300); await sleep(30); }
    await p.page.mouse.up();
    b = await p.fresh();
    check('mouse drag looks', Math.abs(b.cam_yaw - a.cam_yaw) > 0.1, `${a.cam_yaw.toFixed(2)} → ${b.cam_yaw.toFixed(2)}`);
    await k.press('Escape');
    await sleep(600);
    check('Esc opens the menu', (await p.fresh()).menu === true);
    await k.press('Escape');
    await sleep(600);
    check('Esc closes the menu', (await p.fresh()).menu === false);
  } catch (e) { check('keyboard test ran', false, e.message); }
  await p.done();
  await ctx.close();
}

// ---------------------------------------------------------------- save
async function saveTest() {
  console.log('save / continue (localStorage)');
  const ctx = await browser.newContext({ viewport: { width: 960, height: 540 } });
  // clear old saves for this origin
  const p0 = await ctx.newPage();
  await p0.goto(`http://localhost:${PORT}/index.html?nothing=1`).catch(() => {});
  await p0.evaluate(() => localStorage.clear()).catch(() => {});
  await p0.close();
  let p = await open(ctx, 'probe=0.5', 'save1');
  try {
    await p.waitFor(s => s.state === 'Title', 60, 'title');
    await sleep(1200);
    await p.shot('title');
    // "Begin" is the only main button on a fresh save: press Enter
    await p.page.mouse.click(480, 270);
    await p.page.keyboard.press('Enter');
    const m = await p.waitFor(s => s.state === 'Map', 20, 'map');
    check('fresh map reveals only b612', JSON.stringify(m.revealed) === '["b612"]' && m.visited.length === 0, JSON.stringify(m.revealed));
    await sleep(1500);
    await p.shot('map');
    await p.page.keyboard.press('Enter');
    await p.waitFor(s => s.state === 'Chapter' && s.chapter === 'b612', 30, 'b612');
    await sleep(2000);
    const saved = await p.page.evaluate(() => localStorage.getItem('lp.bevy.save.v1'));
    check('save written to localStorage', !!saved && saved.includes('b612'), saved ? saved.slice(0, 80) : 'none');
  } catch (e) { check('save test (first visit) ran', false, e.message); }
  await p.done();
  p = await open(ctx, 'probe=0.5', 'save2');
  try {
    const t = await p.waitFor(s => s.state === 'Title', 60, 'title after reload');
    check('progress survives a reload', t.revealed.includes('b612'));
    await sleep(1200);
    await p.shot('title-continue');
    // Enter on the title = continue to the map
    await p.page.mouse.click(200, 500);
    await p.page.keyboard.press('Enter');
    await p.waitFor(s => s.state === 'Map', 20, 'map');
    check('continue opens the journey map', true);
  } catch (e) { check('save test (reload) ran', false, e.message); }
  await p.done();
  await ctx.close();
}

// ---------------------------------------------------------------- lang
async function langTest() {
  console.log('language switch');
  const ctx = await browser.newContext({ viewport: { width: 960, height: 540 } });
  const p = await open(ctx, 'fresh=1&lang=en&probe=0.3', 'lang');
  try {
    await p.waitFor(s => s.state === 'Title', 60, 'title');
    check('?lang=en starts in English', (await p.fresh()).lang === 'En');
    await sleep(1500);
    await p.shot('title-en');
    const z = await clickLang(p, 'Zh');
    check('language button 中 switches to Chinese', z === true);
    await p.shot('title-zh');
    check('and EN switches back to English', (await clickLang(p, 'En')) === true);
  } catch (e) { check('lang test ran', false, e.message); }
  await p.done();
  await ctx.close();
}

/** Bevy UI draws its text on the canvas, so the buttons cannot be found in
 *  the DOM. The 中 / EN buttons sit at the bottom of the title's right-hand
 *  button column (left edge at 56 % of the width): click down that column
 *  from the bottom up until the probe reports the wanted language. */
async function clickLang(p, want) {
  const W = p.page.viewportSize().width, H = p.page.viewportSize().height;
  const dx = want === 'Zh' ? 0 : 1;
  // 40 px buttons, 10 px apart, in UI units scaled like the game (1280×720 reference)
  const k = Math.min(Math.max(Math.min(W / 1280, H / 720), 0.55), 1.6);
  for (let y = H * 0.95; y > H * 0.45; y -= 10) {
    for (const x of [W * 0.56 + (20 + dx * 50) * k]) {
      await p.page.mouse.click(x, y);
      await sleep(150);
      const s = await p.fresh();
      if (s.state !== 'Title') return `left the title (${s.state}) at y=${y.toFixed(0)}`;
      if (s.lang === want) return true;
    }
  }
  return false;
}

// ---------------------------------------------------------------- free travel
async function freeTest() {
  console.log('free travel');
  const ctx = await browser.newContext({ viewport: { width: 960, height: 540 } });
  const p = await open(ctx, 'fresh=1&map=1&free=1&probe=0.3', 'free');
  try {
    const m = await p.waitFor(s => s.state === 'Map', 60, 'map');
    check('?free=1 turns free travel on', m.free === true);
    await sleep(1500);
    await p.shot('map-free');
    // select the last planet with the arrow keys (wraps left from the frontier) and enter
    await p.page.mouse.click(5, 5);
    await p.page.keyboard.press('ArrowLeft');
    await sleep(300);
    await p.page.keyboard.press('Enter');
    const c = await p.waitFor(s => s.state === 'Chapter', 40, 'chapter');
    check('a locked planet can be entered with free travel', c.chapter === 'earth', `entered ${c.chapter}`);
  } catch (e) { check('free test ran', false, e.message); }
  await p.done();
  await ctx.close();
}

const all = { touch: () => touchTest(2), touch1: () => touchTest(1), keyboard: keyboardTest, save: saveTest, lang: langTest, free: freeTest };
const pick = process.argv.slice(2).length ? process.argv.slice(2) : Object.keys(all);
for (const n of pick) await all[n]();
await browser.close();
server.close();
const bad = results.filter(r => !r.ok);
console.log(`${results.length - bad.length}/${results.length} checks passed`);
process.exit(bad.length ? 1 : 0);
