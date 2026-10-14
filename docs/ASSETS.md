# 素材与替换说明（Assets）

游戏里**所有**模型、贴图、音效、音乐、字体都通过「清单键」引用，代码里没有写死的文件路径。

- 核心清单：`assets/manifest.ron`
- 每章清单：`assets/chapters/<章节id>.ron`（后加载，同名键覆盖核心清单）
- 清单在编译时会内嵌一份（离线也能启动），运行时再从 `assets/` 读取一次覆盖——
  **改清单、换文件都不需要重新编译**（网页版需重新运行 `scripts/build_web.sh` 或直接改 `dist/assets/`）。

## 条目格式

```ron
#![enable(implicit_some)]
(
  entries: {
    "model.b612.rose": (
      path: "models/b612/rose.glb",   // assets/ 下的文件；不写 = 用程序生成
      procedural: "b612.rose",        // 兜底的程序生成器（文件缺失/加载失败时也会用它）
      scale: 1.0, offset: [0, 0, 0], yaw: 0,   // 模型的缩放/偏移/朝向修正（度）
      note: "玫瑰，约 0.55 m 高",
    ),
    "sfx.step": (variants: ["audio/sfx/a.ogg", "audio/sfx/b.ogg"], procedural: "step", volume: 0.5),
    "music.b612": (path: "audio/music/b612.ogg", procedural: "pad_home", volume: 0.7, looped: true),
    "font.ui": (path: "fonts/ui.ttf"),
  },
)
```

| 字段 | 用于 | 说明 |
|---|---|---|
| `path` | 全部 | 相对 `assets/` 的文件：`.glb/.gltf`、`.ogg/.wav`、`.png/.jpg`、`.ttf/.otf` |
| `variants` | 音效 | 多个文件随机选一个（脚步声） |
| `procedural` | 模型/音频 | 程序生成器 id，没有 `path` 或加载失败时使用 |
| `scale` `offset` `yaw` | 模型 | 加载文件后的修正 |
| `volume` `looped` | 音频 | 音量倍率、循环 |
| `anims` | 角色模型 | glTF 动画片段名，如 `{"idle": "Idle", "walk": "Walk"}`（预留） |
| `note` | — | 说明（尺寸、朝向要求） |

## 怎么换成你自己的素材

1. 把文件放进 `assets/` 下任意目录，例如 `assets/models/b612/rose.glb`。
2. 在对应清单里把该键的 `path` 指向它（保留 `procedural` 作兜底）。
3. 原生运行 `cargo r` 立即生效；网页版重新运行 `scripts/build_web.sh`。

模型约定：单位米，+Y 向上，**正面朝 +Z**，原点在脚底/底面中心。小王子约 1.2 m 高。
需要换主角时设置 `"model.prince": (path: "models/prince.glb", ...)`；当前版本加载替换模型
但只做整体移动/转向（骨骼动画片段播放是已知待办，`anims` 字段已预留）。

音乐目前全部是程序合成的柔和 pad（没有合适的 CC0 曲目）；把 `.ogg` 放进
`assets/audio/music/` 并填 `path` 即可替换。

字体：`font.ui` 指向 `fonts/ui.ttf`，它是 `scripts/subset_font.sh` 从完整字体中裁出的子集
（只含游戏用到的字）。新增中文文本后要重新运行该脚本（`build_web.sh` 会自动执行）。
换字体：把完整字体放到 `assets/fonts/_full_LXGWWenKai-Regular.ttf`（或修改脚本里的路径）。

## 获取 CC0 素材

`scripts/fetch_assets.sh` 会下载并解压（缓存在 `assets/_downloads/`，已 gitignore）：

- Kenney Nature Kit（CC0）全部 glTF → `assets/models/nature/`
- Kenney Impact Sounds / Interface Sounds / Music Jingles（CC0）精选 → `assets/audio/`
- 霞鹜文楷 LXGW WenKai（SIL OFL 1.1，唯一非 CC0 项，字体）→ 裁剪为 `assets/fonts/ui.ttf`

来源与许可见 `assets/CREDITS.md`。新增素材时请同样登记来源 URL 与许可，只用 CC0
（Poly Haven、Kenney、Quaternius、ambientCG、OpenGameArt 的 CC0 条目等）。
