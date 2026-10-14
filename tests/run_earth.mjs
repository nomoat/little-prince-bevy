// Earth chapter web test: like run_web.mjs but with dense screenshots across
// the (long) chapter and an optional `?earth=dunes` start.
//   DIST=../dist node tests/run_earth.mjs [zh|en] [valley|dunes]
// Env: PORT (default 8771), TIMEOUT (s, default 600), EVERY (s between shots, default 3), WIDTH/HEIGHT.
import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.env.DIST || path.join(here, '..', 'dist'));
const out = path.join(here, 'out', 'earth');
fs.mkdirSync(out, { recursive: true });
const PORT = +(process.env.PORT || 8771);
const TIMEOUT = +(process.env.TIMEOUT || 600) * 1000;
const EVERY = +(process.env.EVERY || 3);
const W = +(process.env.WIDTH || 960), H = +(process.env.HEIGHT || 540);
const lang = process.argv[2] || 'zh';
const start = process.argv[3] || 'valley';
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.ron': 'text/plain',
  '.ttf': 'font/ttf', '.png': 'image/png', '.ogg': 'audio/ogg', '.glb': 'model/gltf-binary' };
const server = http.createServer((req, res) => {
  const u = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const f = path.join(dist, u === '/' ? 'index.html' : u);
  if (!f.startsWith(dist) || !fs.existsSync(f) || fs.statSync(f).isDirectory()) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { 'content-type': types[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});
await new Promise(r => server.listen(PORT, r));
const sysChrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const browser = await chromium.launch({
  headless: !process.env.HEADED,
  ...(fs.existsSync(sysChrome) ? { executablePath: sysChrome } : {}),
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist', '--enable-webgl', '--autoplay-policy=no-user-gesture-required'],
});
const page = await browser.newPage({ viewport: { width: W, height: H } });
const log = [];
let result = null;
page.on('console', m => {
  const t = m.text();
  log.push(`[${((Date.now() - t0) / 1000).toFixed(1)}] [${m.type()}] ${t}`);
  const ok = t.match(/\[test:ok ([\w-]+)\]/), fail = t.match(/\[test:fail ([\w-]+)\] ?(.*)/);
  if (ok && !result) result = { ok: true };
  if (fail && !result) result = { ok: false, reason: fail[2] };
});
page.on('pageerror', e => log.push(`[pageerror] ${e.message}`));
const t0 = Date.now();
const name = `earth-${lang}${start === 'dunes' ? '-dunes' : ''}`;
await page.goto(`http://localhost:${PORT}/?chapter=earth&test=1&lang=${lang}${start === 'dunes' ? '&earth=dunes' : ''}`);
let n = 0, next = 4;
while (Date.now() - t0 < TIMEOUT && !result) {
  await page.waitForTimeout(250);
  const el = (Date.now() - t0) / 1000;
  if (el >= next) {
    await page.screenshot({ path: path.join(out, `${name}-${String(n++).padStart(3, '0')}.png`) });
    next += EVERY;
  }
}
await page.waitForTimeout(1500);
await page.screenshot({ path: path.join(out, `${name}-end.png`) });
fs.writeFileSync(path.join(out, `${name}.log`), log.join('\n'));
await browser.close();
server.close();
const secs = ((Date.now() - t0) / 1000).toFixed(1);
console.log(`${result && result.ok ? 'PASS' : 'FAIL'} ${name} (${secs}s)${result && !result.ok ? ' — ' + result.reason : ''}${!result ? ' — no result' : ''}`);
process.exit(result && result.ok ? 0 : 1);
