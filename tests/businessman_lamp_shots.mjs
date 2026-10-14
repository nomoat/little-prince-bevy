// Dense screenshots for the businessman / lamp chapters (every 2.5 s of a
// ?test=1 run), to review camera beats and the telescope view.
//   node tests/businessman_lamp_shots.mjs businessman [lang]
// Based on run_web.mjs: serves dist/, opens every chapter with ?test=1 (the
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

const id = process.argv[2] || 'businessman';
const lang = process.argv[3] || 'zh';
const shots = Array.from({ length: 80 }, (_, i) => 3 + i * 2.5);
const r = await runPage(`dense-${id}-${lang}`, `chapter=${id}&test=1&lang=${lang}`, { until: 'result', shots });
console.log(JSON.stringify(r.result), r.secs, r.errors.slice(0, 2).join(' | '));
await browser.close();
server.close();
