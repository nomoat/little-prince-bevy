// Screenshot helper for the vanity / bar / geo chapters: opens one query and
// captures the canvas at the given seconds; prints the console test result.
//   DIST=../dist-vanity-bar-geo node tests/vbg_shots.mjs "chapter=vanity&test=1" name 4,8,12
import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.env.DIST || path.join(here, '..', 'dist'));
const out = path.join(here, 'out', 'vbg');
fs.mkdirSync(out, { recursive: true });
const [query, name, times] = process.argv.slice(2);
const shots = (times || '').split(',').filter(Boolean).map(Number);
const PORT = +(process.env.PORT || 8797);
const W = +(process.env.WIDTH || 960), H = +(process.env.HEIGHT || 540);
const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.ron': 'text/plain', '.ttf': 'font/ttf', '.png': 'image/png', '.ogg': 'audio/ogg', '.wav': 'audio/wav', '.glb': 'model/gltf-binary' };
const server = http.createServer((req, res) => {
  const u = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const f = path.join(dist, u === '/' ? 'index.html' : u);
  if (!f.startsWith(dist) || !fs.existsSync(f) || fs.statSync(f).isDirectory()) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { 'content-type': types[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});
await new Promise(r => server.listen(PORT, r));
const sysChrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const browser = await chromium.launch({ headless: true, executablePath: fs.existsSync(sysChrome) ? sysChrome : undefined,
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist', '--enable-webgl', '--autoplay-policy=no-user-gesture-required'] });
const page = await browser.newPage({ viewport: { width: W, height: H } });
let result = null; const log = [];
page.on('console', m => { const t = m.text(); log.push(t); if (/\[test:(ok|fail)/.test(t) && !result) result = t; });
page.on('pageerror', e => log.push('[pageerror] ' + e.message));
const t0 = Date.now();
await page.goto(`http://localhost:${PORT}/?${query}`);
const end = Math.max(...shots, 0) + 1;
let i = 0;
while ((Date.now() - t0) / 1000 < (process.env.UNTIL_RESULT ? 400 : end)) {
  await page.waitForTimeout(250);
  const el = (Date.now() - t0) / 1000;
  while (i < shots.length && el >= shots[i]) { await page.screenshot({ path: path.join(out, `${name}-${String(shots[i]).padStart(3, '0')}.png`) }); i++; }
  if (process.env.UNTIL_RESULT && result && i >= shots.length) break;
  if (process.env.UNTIL_RESULT && result) break;
}
fs.writeFileSync(path.join(out, `${name}.log`), log.join('\n'));
console.log(result || 'no result', `${((Date.now() - t0) / 1000).toFixed(1)}s`);
console.log(log.filter(l => /panick|pageerror|ERROR|WARN/.test(l)).slice(0, 12).join('\n'));
await browser.close(); server.close();
