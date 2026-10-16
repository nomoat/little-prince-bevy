# 小王子：星辰之旅（Bevy 重制版）

这是按原版《小王子：星辰之旅》（Unity 6 网页游戏）的设计重做的新游戏。它用 **Rust + Bevy 0.20** 写成，
场景和玩法由代码组织，不需要编辑器。角色、主要道具、建筑和音乐已接入用户提供的原版素材，
来源是 `~/lp_extract`。转换结果保存在 `assets/original/`，运行游戏不需要原始目录。

- **流程：** 标题 → 旅程图（8 颗星球，按原版规则逐个解锁，可开自由旅行）→ 章节 → 结局。
- **8 章：** B612 → 国王 → 爱虚荣的人（剧院）→ 酒鬼（酒馆）→ 商人（天文台）→ 点灯人 → 地理学家 → 地球（山谷 + 沙漠）。
- **语言：** 中文（默认）和 English，可在标题页或设置里切换。
- **平台：** 网页（WebGL2，手机可玩；可选 WebGPU）和原生桌面。
- **存档：** 网页存在 localStorage（键 `lp.bevy.save.v1`），原生存在 `~/.little_prince_bevy/`。
- **动画与渲染：** 已转换 35 段原版角色动画，接入移动和第一章主要动作；新增围巾跟随、草叶风动与水面动态着色。尚未达到原版全部渲染效果，具体差异见 `docs/ASSETS.md`。
- **素材：** 优先使用原版模型、贴图和音频；第一章使用原版地形、场景摆放和植被实例；其余章节的地形、部分特效和道具仍由程序生成。每个文件的来源可查 `assets/original/import-report.json`。

## 构建与运行

需要以下工具：

- rustup 的工具链，带 wasm32 标准库
- `wasm-bindgen` CLI 0.2.129（和 Cargo.toml 里固定的版本一致）
- `wasm-opt`（binaryen）
- 可选：`brotli`，用来生成预压缩文件

```bash
export PATH="$HOME/.cargo/bin:$PATH"   # 用 rustup 的 cargo（Homebrew 的 rustc 没有 wasm std）

scripts/build_web.sh                   # 网页版 → dist/（默认 WebGL2，适合手机；wasm-opt -Oz）
scripts/build_web.sh --webgpu          # 改用 WebGPU 渲染
scripts/build_web.sh --fast            # 跳过 wasm-opt，用于快速迭代
scripts/serve.sh 8080                  # 打开 http://localhost:8080/ ；启动时会打印手机可用的局域网地址

cargo r                                # 原生桌面版
cargo r -- --chapter king --test       # 原生：直接进章节并自动通关，结束后退出
```

> Bevy 0.20 的着色器依赖要求 rustc 1.97，但在已安装的 1.95 上也能编译，所以命令要加
> `--ignore-rust-version`。`.cargo/config.toml` 里的别名 `cargo c / cw / b / r` 已经加上了这个参数。

`dist/` 是纯静态文件，放到任何静态主机都能运行。它包含以下文件：

- `index.html`：加载页，带进度条
- `little_prince.js`：wasm-bindgen 生成的胶水代码
- `little_prince_bg.wasm`：游戏本体
- `.br` 和 `.gz`：预压缩副本
- `assets/`：素材

`scripts/serve.sh` 会优先发送 `.br` 或 `.gz` 副本。

### 体积（WebGL2，`-Oz`）

| 文件 | 大小 |
|---|---|
| `little_prince_bg.wasm` | 23.7 MB（未压缩） |
| 同上，gzip -9 | 7.2 MB |
| 同上，brotli -q 11 | 4.8 MB |
| `assets/` | 接入原版场景后约 279 MB；实际大小以构建输出为准 |

Release 配置为 `opt-level="z"`、`lto="fat"`、`codegen-units=1`、`panic="abort"`、`strip`，之后再运行 `wasm-opt -Oz`。

## 操作

| 动作 | 键盘 + 鼠标 | 触屏（手机自动出现） |
|---|---|---|
| 移动 | WASD / 方向键 | 左半屏虚拟摇杆（推到底 = 跑） |
| 跑 | 按住 Shift | 摇杆推到底 |
| 跳 | 空格 | 「跳跃」按钮 |
| 互动 / 下一句 | E（部分互动要按住） | 互动按钮（显示当前动作），或轻点屏幕 |
| 第二动作（鼓掌、换镜） | C | 有用时才出现的第三个按钮 |
| 转视角 | 拖动鼠标；设置里可开指针锁定 | 右半屏拖动 |
| 缩放 | 滚轮 | 双指捏合 |
| 任务 | 按住 Tab | 「查看任务」按钮 |
| 菜单 | Esc | ≡ 按钮 |
| 启程（离开星球） | 剧情结束后按住 E 1.6 秒 | 按住互动按钮 |

旅程图上可以点星球，也可以用 ←/→ 选择、Enter 出发。标题页的 Enter 等于「继续旅程」。

## 调试参数

网页版写在 URL 里，原生版写成命令行参数 `--参数 值`。

| 参数 | 作用 |
|---|---|
| `?chapter=<id>` | 直接进入章节：`b612 king vanity bar businessman lamp geo earth` |
| `?test=1` | 自动游玩（仿原版 AgentInput/PlaytestRoute）。控制台输出 `[test:ok <id>]` 或 `[test:fail <id>] 原因` |
| `?flow=1&test=1` | 完整流程测试，见下文 |
| `?lang=en` / `?lang=zh` | 本次使用的语言 |
| `?free=1` | 自由旅行：所有星球都能选 |
| `?map=1` | 跳过标题，直接到旅程图 |
| `?ending=1` | 直接看结局 |
| `?touch=1` / `?touch=0` | 强制显示或隐藏触屏按键 |
| `?quality=low/medium/high` | 画质 |
| `?speed=2` | 游戏时间倍速（测试模式默认 1.5） |
| `?fresh=1` | 不读也不写存档（测试模式会自动打开） |
| `?probe=0.5` | 每 0.5 秒在控制台输出一行 `[probe] {json}`：状态、位置、相机、菜单、语言、进度。供浏览器测试使用 |
| `?earth=dunes` | 地球章直接从沙漠开始 |
| `?b612_real=1` / `?king_real=1` | 测试模式下保持真实时长，用于截图 |

`?flow=1&test=1` 的完整流程如下：

1. 在标题页按「开始旅程」。
2. 在旅程图上检查解锁规则：已到过的星球是 Visited，下一站是 Lit，后面的都还在雾里。
3. 尝试进入一颗未解锁的星球，确认进不去。
4. 出发去下一站，自动通关这一章。
5. 第 2 到 4 步依次走完 8 章。
6. 看完结局后回到标题页，检查顺序和进度。
7. 控制台输出 `[flow:ok]` 或 `[flow:fail] 原因`。

## 测试

```bash
scripts/build_web.sh
cd tests && npm i                                 # playwright 1.57（用本机 Chrome 或已缓存的 Chromium）
node run_web.mjs                                  # 8 章各自自动通关（中文），截图在 tests/out/
LANG_EN=1 node run_web.mjs b612 king              # 同时跑英文
FLOW=1 LANG_EN=1 node run_web.mjs                 # 完整流程：标题 → 旅程图 → 8 章 → 结局 → 标题
node ui_web.mjs                                   # 真实输入测试
SHOTS=title,map,ending node run_web.mjs           # 只截界面
node play_web.mjs earth desk zh                   # 「玩家机器人」：只用真实键鼠输入、跟着目标标记玩一章（非测试模式，真实时长）
node play_web.mjs bar phone en                    # 同上，手机横屏触屏（750×342）
./play_all.sh phone zh "king bar" 600 2           # 批量跑；截图和时间线在 tests/out/play-<章>-<设备>-<语言>/
```

`play_web.mjs` 不走自动通关，而是像玩家一样：对话时点屏幕，看到互动提示就按 E/互动键，
有目标标记就朝标记走，被挡住就绕开、跳，教学卡提示 C 就鼓掌。它会记录每一步卡了多久，
用来发现「玩家会卡住」的地方（例如路被挡、标记指向墙里、斜坡太陡）。

`node ui_web.mjs` 用真实输入测试以下几项：

- **触屏：** 用手机模拟测试摇杆、拖动视角、捏合、跳跃和菜单按钮。
- **键鼠：** WASD、Shift、空格、滚轮、拖动和 Esc。
- **存档：** 存档写入 localStorage，刷新页面后可以「继续旅程」。
- **语言：** 中文和英文之间切换。
- **自由旅行：** 可以进入未解锁的星球。

## 原版素材导入与替换

```bash
~/lp_extract/venv/bin/python scripts/import_original_assets.py
scripts/build_web.sh --fast
scripts/serve.sh 8080
```

导入脚本使用本地 UnityPy、NumPy、Pillow 和 ffmpeg。它读取原始 bundle 中的模型、材质引用、
骨骼和蒙皮权重，生成 glTF 2.0 GLB，并把音频转成 Bevy 支持的 Ogg Vorbis。
贴图最长边限制为 1024 像素。映射表在 `scripts/original_asset_map.py`。
`--source <目录>` 可指定来源；`--skip-audio` 可只重新转换模型。

主角的真实骨骼由现有移动和互动状态驱动，支持站立、行走、跳跃、坐下、挥手等姿势。
Unity 的 Humanoid 动画控制器尚未移植；NPC 使用导入的模型和基础姿势。
碰撞和地形高度仍由现有 Bevy 玩法定义，原版建筑的细节与简化碰撞体并非逐面一致。

所有模型、贴图、音效、音乐和字体都通过**清单键**引用，代码里没有写死的文件路径。

- **核心清单：** `assets/manifest.ron`。
- **各章清单：** `assets/chapters/<id>.ron`。每个键都写明了尺寸、朝向和会动的部件。
- **换法：** 把文件放进 `assets/`，在对应的键里写上 `path: "..."`。不需要重新编译；网页版重新运行 `scripts/build_web.sh`，或直接改 `dist/assets/`。
- **兜底：** 每个键都有程序生成的兜底。文件缺失或加载失败时，会自动改用程序生成的版本。
- **碰撞体：** 碰撞体写在代码里，所以换模型不会改变碰撞。

详细说明见 [docs/ASSETS.md](docs/ASSETS.md)，来源和许可证记录见 [assets/CREDITS.md](assets/CREDITS.md)。

## 与原版的对应关系

| 原版（Unity 程序集） | 本项目 |
|---|---|
| B612.Core：穹顶地形星球（球冠高度场 + 世界 Y 向重力）、角色控制器、第三人称相机 | `src/core/planet.rs`、`player.rs`、`camera.rs`、`physics.rs` |
| LittlePrince.Story：剧情事件、对话、任务、隐藏目标 | `src/core/story.rs`、`ui/hud.rs` |
| LittlePrince.Journey：JourneyFlow / JourneyProgress / JourneyDeparture、旅程图 | `src/core/state.rs`、`save.rs`、`ui/map.rs`（按住 E 1.6 s 启程，revealed/visited 规则相同） |
| LittlePrince.UI / Web：标题、菜单、设置、触屏、URL 参数 | `src/core/ui/*`、`input.rs`、`params.rs`、`platform.rs` |
| AgentInput / PlaytestRoute（开发用自动游玩） | `src/core/testing.rs`（`?test=1`、`?flow=1`） |
| Assembly-CSharp（虚荣的人、点灯人等）、King/Bar/Geography/Desert.Runtime、StarObservatory | `src/chapters/<id>/`，每章的设计笔记在 `docs/original/<id>.md` |

各章会尽量使用原版的坐标、镜头位置、视野（FOV）、时长和节拍。与原版的差异记录在各章源码的文件头注释里。

## 目录

- `src/core/`：引擎。包括星球、角色、相机、输入、交互、剧情、界面、音频、天空昼夜、存档、旅程流程和测试驱动。
- `src/chapters/<id>/`：各章节。
- `assets/`：素材和清单。`CREDITS.md` 列出每个素材的来源和许可证。
- `docs/CORE_API.md`：章节开发 API。
- `docs/ASSETS.md`：换素材说明。
- `docs/original/`：原版机制笔记。
- `scripts/`：构建、本地服务、字体子集、素材下载。
- `tests/`：Playwright 测试（`run_web.mjs`、`ui_web.mjs`）和各章截图脚本。
