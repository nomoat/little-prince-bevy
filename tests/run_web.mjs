// Headless web test: serves dist/, opens every chapter with ?test=1 (the
// autoplay harness), waits for "[test:ok <id>]" / "[test:fail <id>]" in the
// console, and saves screenshots to tests/out/.
//
//   node tests/run_web.mjs                    # all chapters, zh
//   node tests/run_web.mjs b612 king          # some chapters
//   LANG_EN=1 node tests/run_web.mjs b612     # also run each with &lang=en
//   SHOTS=title,map,ending node tests/run_web.mjs   # extra screen captures only
// Env: DIST (default ../dist), PORT (default 8765), TIMEOUT (s, default 300),
//      HEADED=1, WIDTH/HEIGHT (default 960x540).
import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.env.DIST || path.join(here, '..', 'dist'));
const out = path.join(here, 'out');
fs.mkdirSync(out, { recursive: true });
const PORT = +(process.env.PORT || 8765);
const TIMEOUT = +(process.env.TIMEOUT || 300) * 1000;
const W = +(process.env.WIDTH || 960), H = +(process.env.HEIGHT || 540);
const ALL = ['b612', 'king', 'vanity', 'bar', 'businessman', 'lamp', 'geo', 'earth'];

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

// Prefer $CHROME, then the system Google Chrome, then Playwright's cached Chromium.
const sysChrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const exe = process.env.CHROME || (fs.existsSync(sysChrome) ? sysChrome : undefined);
const browser = await chromium.launch({
  headless: !process.env.HEADED,
  ...(exe ? { executablePath: exe } : { channel: 'chromium' }),
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist', '--enable-webgl', '--autoplay-policy=no-user-gesture-required'],
});

async function runPage(name, query, { until, shots = [], timeout = TIMEOUT }) {
  const page = await browser.newPage({ viewport: { width: W, height: H } });
  const log = [];
  let result = null;
  page.on('console', m => {
    const t = m.text();
    log.push(`[${m.type()}] ${t}`);
    const ok = t.match(/\[test:ok ([\w-]+)\]/), fail = t.match(/\[test:fail ([\w-]+)\] ?(.*)/);
    if (ok && !result) result = { ok: true, id: ok[1] };
    if (fail && !result) result = { ok: false, id: fail[1], reason: fail[2] };
  });
  page.on('pageerror', e => log.push(`[pageerror] ${e.message}`));
  const t0 = Date.now();
  await page.goto(`http://localhost:${PORT}/?${query}`);
  let shotIdx = 0;
  while (Date.now() - t0 < timeout) {
    await page.waitForTimeout(500);
    const el = (Date.now() - t0) / 1000;
    while (shotIdx < shots.length && el >= shots[shotIdx]) {
      await page.screenshot({ path: path.join(out, `${name}-${String(shotIdx).padStart(2, '0')}.png`) });
      shotIdx++;
    }
    if (until === 'result' && result) break;
    if (typeof until === 'number' && el >= until) break;
  }
  await page.screenshot({ path: path.join(out, `${name}-end.png`) });
  fs.writeFileSync(path.join(out, `${name}.log`), log.join('\n'));
  await page.close();
  return { result, secs: ((Date.now() - t0) / 1000).toFixed(1), errors: log.filter(l => l.startsWith('[pageerror]') || l.includes('panicked')) };
}

let failed = 0;
const extra = (process.env.SHOTS || '').split(',').filter(Boolean);
for (const s of extra) {
  const q = { title: 'fresh=1', map: 'fresh=1&map=1', mapfree: 'fresh=1&map=1&free=1', ending: 'fresh=1&ending=1', title_en: 'fresh=1&lang=en' }[s] || s;
  const r = await runPage(`screen-${s}`, q, { until: 14, shots: [8] });
  console.log(`screen ${s}: ${r.errors.length ? 'ERRORS ' + r.errors.join(' | ') : 'ok'}`);
}
const ids = process.argv.slice(2).length ? process.argv.slice(2) : (extra.length ? [] : ALL);
const langs = process.env.LANG_EN ? ['zh', 'en'] : ['zh'];
for (const id of ids) {
  for (const lang of langs) {
    const name = lang === 'zh' ? id : `${id}-en`;
    const r = await runPage(name, `chapter=${id}&test=1&lang=${lang}`, { until: 'result', shots: [5, 9, 13, 17, 21, 25, 30, 40, 60] });
    const ok = r.result && r.result.ok && r.result.id === id;
    if (!ok) failed++;
    console.log(`${ok ? 'PASS' : 'FAIL'} ${name} (${r.secs}s)${r.result && !r.result.ok ? ' — ' + r.result.reason : ''}${!r.result ? ' — no result (timeout)' : ''}${r.errors.length ? ' errors: ' + r.errors.slice(0, 3).join(' | ') : ''}`);
  }
}
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
