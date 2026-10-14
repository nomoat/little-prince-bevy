// Screenshot helper for the b612 / king chapters: opens one URL query and
// saves screenshots at the given seconds (real time) to tests/out/.
//
//   node tests/b612_king_shots.mjs "chapter=b612" 3,6,9,12 b612-open
//   DIST=../target-x/dist PORT=8917 node tests/b612_king_shots.mjs "chapter=king&test=1&speed=1" 5,10 king-t
// Env: DIST (default ../dist), PORT (default 8766), WIDTH/HEIGHT (960x540).
import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.env.DIST || path.join(here, '..', 'dist'));
const out = path.join(here, 'out');
fs.mkdirSync(out, { recursive: true });
const PORT = +(process.env.PORT || 8766);
const W = +(process.env.WIDTH || 960), H = +(process.env.HEIGHT || 540);
const [query = 'chapter=b612', timesArg = '5,10', prefix = 'shot'] = process.argv.slice(2);
const times = timesArg.split(',').map(Number);

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
  headless: true,
  ...(exe ? { executablePath: exe } : { channel: 'chromium' }),
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist', '--enable-webgl'],
});
const page = await browser.newPage({ viewport: { width: W, height: H } });
const log = [];
page.on('console', m => log.push(m.text()));
page.on('pageerror', e => log.push(`[pageerror] ${e.message}`));
await page.goto(`http://localhost:${PORT}/?${query}`);
const t0 = Date.now();
if (process.env.FPS) { await page.waitForTimeout(12000); const n = await page.evaluate(() => new Promise(r => { let c = 0; const t = performance.now(); const f = () => { c++; if (performance.now() - t < 3000) requestAnimationFrame(f); else r(c / 3); }; requestAnimationFrame(f); })); console.log("fps", n); }
for (let i = 0; i < times.length; i++) {
  const wait = times[i] * 1000 - (Date.now() - t0);
  if (wait > 0) await page.waitForTimeout(wait);
  await page.screenshot({ path: path.join(out, `${prefix}-${String(i).padStart(2, '0')}.png`) });
}
fs.writeFileSync(path.join(out, `${prefix}.log`), log.join('\n'));
console.log(log.filter(l => /test:|panick|pageerror/.test(l)).join('\n'));
await browser.close();
server.close();
