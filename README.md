# 小王子：星辰之旅（Bevy 重制版）

按原版《小王子：星辰之旅》的设计（章节结构、穹顶地形星球、角色/相机手感、剧情事件系统、
旅程图解锁、按住 E 启程……）用 **Rust + Bevy 0.20** 纯代码重做，不需要任何编辑器。
素材用 CC0（Kenney 等）或程序生成，全部经由清单键引用，之后可以直接换成你的素材。

- 8 章：B612 → 国王 → 爱虚荣的人 → 酒鬼 → 商人 → 点灯人 → 地理学家 → 地球 → 结局
  （当前除 B612 示例外都是占位章节，由各章节负责人补全）
- 中文（默认）/ English，设置里切换
- 键鼠：WASD 移动、Shift 跑、空格 跳、E 互动/按住、拖动鼠标转视角（可选指针锁定）、滚轮缩放、Tab 任务、Esc 菜单
- 触屏：左半屏虚拟摇杆、右半屏拖动视角、双指缩放、互动/跳跃/菜单按钮（手机自动出现）
- 存档：网页 localStorage，原生在 `~/.little_prince_bevy/`

## 构建与运行

需要 rustup 的工具链（带 wasm32 标准库）、`wasm-bindgen` CLI 0.2.129、`wasm-opt`。

```bash
export PATH="$HOME/.cargo/bin:$PATH"   # 用 rustup 的 cargo（Homebrew 的 rustc 没有 wasm std）

scripts/fetch_assets.sh                # 第一次：下载 CC0 素材和字体（已提交的素材可跳过）
scripts/build_web.sh                   # 网页版 → dist/（默认 WebGL2，适合手机）
scripts/build_web.sh --webgpu          # WebGPU 版本
scripts/serve.sh 8080                  # 打开 http://localhost:8080/ （手机用局域网 IP）

cargo r                                # 原生桌面版
cargo r -- --chapter b612 --test       # 原生：直接进章节并自动测试，结束后退出
```

> Bevy 0.20 的着色器依赖声明需要 rustc 1.97，但在已安装的 1.95 上可以编译，所以命令要加
> `--ignore-rust-version`；`.cargo/config.toml` 里的别名 `cargo c / cw / b / r` 已经带上。

## 调试参数（网页 URL 或原生 `--参数 值`）

| 参数 | 作用 |
|---|---|
| `?chapter=<id>` | 直接进入章节（b612 king vanity bar businessman lamp geo earth） |
| `?test=1` | 自动游玩测试，控制台输出 `[test:ok <id>]` 或 `[test:fail <id>] 原因` |
| `?lang=en` / `?lang=zh` | 本次使用的语言 |
| `?free=1` | 自由旅行（全部星球可选） |
| `?map=1` | 跳过标题直接到旅程图 |
| `?ending=1` | 直接看结局 |
| `?touch=1` | 强制显示触屏按键 |
| `?quality=low/medium/high` | 画质 |
| `?fresh=1` | 不读写存档 |

## 测试

```bash
scripts/build_web.sh
cd tests && npm i && node run_web.mjs          # 8 章全部自动通关，截图在 tests/out/
LANG_EN=1 node run_web.mjs b612                 # 英文
SHOTS=title,map,ending node run_web.mjs         # 只截界面
```

## 目录

- `src/core/` 引擎：星球（穹顶地形）、角色控制器、相机、输入、交互、剧情、界面、音频、天空昼夜、存档、旅程流程、测试驱动
- `src/chapters/<id>/` 各章节（只改自己的目录）
- `assets/manifest.ron`、`assets/chapters/<id>.ron` 素材清单
- `docs/CORE_API.md` 章节开发 API；`docs/ASSETS.md` 换素材；`docs/original/core.md` 原版核心机制笔记
