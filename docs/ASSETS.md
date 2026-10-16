# 素材与替换说明（Assets）

通用模型、贴图、音效、音乐、字体通过「清单键」引用。第一章的原始地形、交互坐标和植被实例另有场景数据，见下文。

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
当前主角指向 `original/shared/prince.glb`，包含原版身体、围巾、贴图、骨骼和蒙皮权重。
`src/core/original_rig.rs` 播放原版 Humanoid 曲线转换出的 35 段动画，驱动 52 根身体骨骼，
包括手指、髋部起伏、动作过渡和独立的持壶手臂层。围巾跟随颈部并增加轻微摆动。
第一章已同步睡眠、醒来、灌水、浇花、拿放玻璃罩和坐下；移动使用 Idle / Walk / Run。
其他片段已转换，但尚未全部接入各章剧情。通用 glTF 的 `anims` 字段仍保留。
NPC 保留原版模型和导入时的基础姿势。

各章音乐已指向 `original/audio/` 下的原版曲目；程序合成的 pad 只作加载失败时的兜底。
标题和旅程图复用 B612 音乐，结局复用沙漠音乐；这三项是本项目的选择，不代表原版场景配置。

## 从本地原版重新导入

运行 `~/lp_extract/venv/bin/python scripts/import_original_assets.py`。
脚本通过 bundle 内的引用关联模型与材质，转换左右手坐标、三角形绕序、UV、骨骼绑定矩阵，
并按现有交互尺寸调整模型。草木实例从原版植被配置的 mesh/material 引用读取。
转换只写 `assets/original/` 和素材清单，不修改来源。映射表在
`scripts/original_asset_map.py`，来源与未匹配项记录在 `assets/original/import-report.json`。

第一章另运行 `~/lp_extract/venv/bin/python scripts/import_b612_scene.py`。它保留原版静态场景层级、
物件尺度和世界坐标；导入 513×513 高度图与八层地表贴图，并导入全部 83,955 条植被记录。
植被按原位置、距离和画质显示，复用原版近景/远景模型。可交互物件从静态场景分离，
使用 `B612LayoutData` 的站位与源物件变换。高度数据与 `scene.ron` 编译进程序，修改后需重编译。
第一章来源与坐标见 `assets/original/b612/scene-report.json`。

第一章地形碰撞使用同一份原始高度图，支持的原版盒形碰撞体也一并导入；复杂网格碰撞尚未完整移植。
其余章节的地形和碰撞仍使用重制版布局。植被与水体在标准 PBR 上增加了 Bevy 着色器：
草叶风动、角色拨草、叶片背光、水面动态法线、泉眼波纹、天空反光与瀑布流纹。它们是重新实现的效果，
不是原版 Unity 着色器；提取目录中的自定义 `.shader` 不包含可移植的源码。
原版体积云、屏幕空间水体折射、完整后处理和玻璃着色器尚未移植。
动画保留原始肌肉曲线与 Avatar 限位，但没有复刻 Unity 的 IK / 肢体伸缩求解器。
这些渲染与动画差异仍需与 https://littleprince.dev/ 对照，素材加载或自动通关不等于视觉还原。

重新转换动画：`~/lp_extract/venv/bin/python scripts/import_original_animation.py`。
输出 `shared/animations.bin`（逐帧本地旋转和髋部位置）、`animations.ron`（索引）与来源报告。
曲线包含 streamed / dense / constant 三种存储；按 30 Hz 重采样，运行时球面插值。
水平根运动由角色控制器负责，睡醒片段的床面偏移按其结束站姿对齐 Idle。
动画数据内嵌，重新导入后需重编译。转换器不修改原始目录。

可用 `?chapter=b612&review=opening` 对照开场，或 `review=CAM_Rose_Main`、
`review=CAM_Meadow_Stream` 复用原版检查镜头。去掉 `review` 即为正常游玩。
`&animation=Walk&animation_time=0.3` 可固定原始动作帧用于视觉检查；去掉这两个参数恢复剧情驱动。

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
