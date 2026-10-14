# 原版核心设计笔记（只记机制，不含原版文字/美术/代码）

来源：原版 Unity 6000.3 IL2CPP WebGL 构建的 Cpp2IL 骨架（`out_cs/DiffableCs`）、
wasm-decompile 函数体、metadata 字面量（仅用来理解设计）。程序集：B612.Core、
B612.Rendering、LittlePrince.Journey / Story / UI / Web，以及 Assembly-CSharp 里的
B612 世界构建代码。下文的数值凡标明 "tooltip" 的，来自原版字段注释；其余是本重制的取值。

## 1. 世界模型：穹顶地形（B612PlanetParams / B612World）

- 星球**不是真球**。地面是一块**高度场**，形状为**球冠**（sphere cap）：
  `y(r) = topY − (R − √(R² − r²))`，可混入一个四次项把中间压平（`capFlatten`，0–0.8，
  在 `walkRadius` 处的落差保持不变）。tooltip："capRadius 50–60 m，可走区域边缘下沉 4–5 m"。
- **重力是世界 Y 轴向下**；角色控制器就是普通的 CharacterController。
- 可走圆盘 `walkRadius`；从 `lipStart` 到 `holeRadius` 地面"翻过边沿"再下沉 `lipDepth`，
  外面是岩石**边沿 + 底面**网格（`undersideDepth`、`rimNoise`、`rimSegments`），看起来像
  漂浮的小星球。
- 轻微起伏：`noiseAmp` / `noiseWavelength`。
- **边界**（B612PlanetBoundary）：`softRadius` 开始把玩家**柔和地推回中心**，到 `hardRadius`
  饱和；推速 `minPush → maxPush`（tooltip："jog 2.4，Shift 跑 3.5"，所以 maxPush 要大于跑速）；
  外圈还有 failsafe 碰撞环。出口闸（B612ExitGate）同理是一段软推区。
- 场景布局用"站点"（site：x, z, yaw, size）放置：睡觉的小窝、工具棚、玫瑰（玫瑰所在土丘
  `roseKnoll` 高出球冠，让它从远处可见）、玻璃罩石台等；地形对站点做**整平垫**（pad：半径+过渡）
  和**土丘**（mound）。
- 区域分类（B612Zone：Home / Meadow / Ridge / StreamBank / Seedling / Volcano / SunsetSlope /
  Exit / Boundary / Far），驱动脚步声表面、植被密度等。

本重制：`core/planet.rs` 的 `PlanetParams` 保留同名参数（center, top_y, cap_radius,
cap_flatten, walk_radius, soft/hard/failsafe_radius, lip_start, hole_radius, lip_depth,
noise_amp/wavelength, underside_depth, rim_noise, rim_segments, max/min_push），
`TerrainMod::{Mound, Flatten, Paint}` 对应土丘/整平垫/地表涂色。

## 2. 角色控制器（PlayerController）

- DriveMode：`Player`（玩家输入）/ `Scripted`（剧情走位，SetScripted(dir, speed)、
  ScriptedApproach.WalkTo）/ `External`（坐下、被鸟带走等，外部驱动变换）。
- 速度（tooltip）：自由操作默认是**孩子的小跑 jog（2.4 m/s）**；按住 **Shift 快跑（3.5 m/s）**；
  `walkSpeed` 用于剧情走位（序列里约 0.9–1 m/s）。`runEnabled` 可被剧情临时关闭。
- 加速/刹车分开（acceleration / deceleration）。
- 转向：`turnSmoothTime`、`maxTurnSpeed`；朝向误差小于 `fullSpeedAngle` 才全速，大于
  `turnInPlaceAngle` 时**先原地转身再前进**；原地转身时腿部仍有踏步动画（turnStepAnimSpeed）。
- 坡度：大于 `slopeSlowStartAngle` 逐渐减速（步幅变短），到坡度上限 **35°**（tooltip）时乘
  `slopeSlowAtLimit`；比 35° 略陡的 `slideAngle` 开始下滑（slideAcceleration / maxSlideSpeed）。
- 地面吸附 `groundSnapDistance`；台阶由 CharacterController 一帧抬升，**模型用 stepSmoothTime
  平滑**跟随（只影响视觉）。
- 跳跃"可选、从不必需"：`jumpEnabled`、`jumpHeight`、`jumpCooldown`，跳跃动画按空中时间配速。
- 动画参数：Speed / Grounded / Jump / Airborne / LocoRate / JumpRate；Idle–Walk–Running
  混合树，`animWalkSpeed`、`animTopSpeed`、`animFullWalkFrom(Moving)` 控制混合与播放速率。
- 角色身高约 **1.2 m**（tooltip："1.2 m child, eyes at ~1.05 m"）。
- 其它：PlayerSitController（椅子坐下/起身、坐着时相机进入"观景模式"）、PlayerCarryController
  （搬运道具、CarryPutSpot 放下点、FillTarget/WaterTarget 灌水浇水）、FootstepPlayer +
  AudioSurfaceMap（按区域/材质换脚步声）、ScarfRig（围巾物理）、CarryHandIK。

本重制：`core/player.rs`，同样的三种驱动方式（`Drive::Player / WalkTo / External`），
jog 2.4 / run 3.5 / walk 1.0，转身减速与原地转身、20°起减速 35°上限、可选小跳、台阶 0.36 m；
程序化小王子（绿外套、金发、长围巾随风飘）+ 程序动画，可用 glTF 替换。

## 3. 第三人称相机（ThirdPersonCamera / B612CameraSpec）

- 轨道中心在脚上 `pivotHeight`（≈1.05 m，孩子的眼高）；`defaultDistance / min / max`、
  `defaultPitch / minPitch / maxPitch`、`zoomSmoothTime`。
- 跟随：XZ 与 Y 分开平滑（followSmoothTimeXZ / Y），`maxFollowLag` 限制最大落后。
- 碰撞：只对地形和环境层；`collisionRadius`、`collisionRecoverTime`；镜头离头太近时隐藏角色
  （hide/show 有滞回）；`groundClearance` 保持在地面以上。
- **遮挡时抬高而不是拉近**（obstacle lift）：先尝试更陡的俯角，再在 `maxSwing` 内转向找空位，
  再不行才拉近；草丛覆盖（GroundCover）时保持在草顶以上。
- 剧情镜头：BeginShot / CutTo / MoveShot / EndShot；B612CameraSpec（名称、位置、看向点、fov 默认 50）。
- 坐下后进入观景模式（viewEnterTime / viewExitTime）。

本重制：`core/camera.rs`（pivot 1.05、距离 4.6 [1.6, 9]、俯仰 14° [-20°, 70°]、fov 50、
XZ/Y 分开平滑 + 最大落后、碰撞拉近 + 平滑恢复、地面间隙）与 `Shot` 剧情镜头。
"抬高优先"的复杂避障未实现（已知差距）。

## 4. 输入（PlayerInputHub / PlayerInputFrame / KeyboardMouseInputSource / GameInput）

- 每帧合成一个 **PlayerInputFrame**：`move, run, look, zoom, interact, interactHeld, jump, clap`。
- 多个输入源注册到 Hub（键鼠、AgentInput 测试驱动……）；Hub 可 Suspended / MovementBlocked。
- 键鼠：度/像素的 yaw/pitch 灵敏度、invertY、滚轮每格米数；`requireCursorLockForLook`
  （点击锁定指针，Esc 释放）；网页端 `freelook` 调试参数可免锁定。
- GameInput.Look：设置里的灵敏度滑条（FromSlider/ToSlider）。

本重制：`core/input.rs` 的 `InputFrame` 同构（另加 `action2`、`menu`、`tasks_held`、`advance`），
来源：键鼠（拖动看或指针锁定）、触屏（虚拟摇杆/按钮/拖动看/捏合缩放）、`AgentInput`（测试）。

## 5. 交互（Interactable / PlayerInteractor / InteractionPromptUI）

- Interactable 字段：`prompt`（动词，如"坐下"→显示"E  坐下"）、`keyLabel`、`radius`、
  `focusPoint`、`requireFacing` + `maxFacingAngle`、`requireSide` + `sideHalfAngle`（必须站在
  物体朝向那一侧）、`priority`、`interactable`、`usableWhileCarrying`、**`holdSeconds`（按住几秒，
  提示上有填充圆环）**、`promptAnchor` / `promptHeight`、事件 onInteract / Interacted。
- PlayerInteractor 每帧 FindBest（距离、朝向、优先级），可设 Exclusive（独占某个交互物），
  按住进度 HoldProgress；按下后需松开才能再次触发（waitRelease）。

本重制：`core/interact.rs`，字段与语义一一对应（`side_half_angle`、`hold_seconds`、`priority`、
`once`），提示 UI 锚定在世界坐标上方，按住时显示进度条。

## 6. 剧情系统（StoryRunner / StoryChapter / StoryEvent）

- 一章 = `StoryChapter { Number, Title, Subtitle, Narration[], Events[] }`；
  每个 `StoryEvent { Name, Steps[] }`。
- StoryRunner：`Begin(chapter)`、`ChapterCard()`（章节卡）、`StartEvent(i)`、`SetStep(s)`、
  `CompleteEvent(i)`（横幅"完成→接下来"）、`MarkDone`、`Say(speaker, line, key, soft, extra)`、
  `SayFor(speaker, line, seconds)`、`Beat(seconds)`、`SetTarget(transform, label)`、
  `SetHidden(text, done)`（隐藏目标）、`ShowFree(step)`、`Unmet(line)`。
- **停滞提示**：`stallSeconds` 内没有更接近目标 → 标记亮一次（stallMarkerSeconds）；
  `unmetMarkerSeconds` 用于"条件未满足"时。
- 剧情用协程写在各章节自己的 Story 类里（B612Story、VanityStory、LighterStory……）。
- ChapterView（Tab 任务页）：标题、旁白、事件列表及状态（Done / Current / Future）、当前步骤、
  自由步骤、目标、隐藏目标及是否完成。

本重制：`core/story.rs` —— `Story { events, hidden, script }`；脚本是 `Cmd` 列表（数据驱动），
覆盖上面全部操作，外加标签/跳转/旗标/选择、等待交互/旗标/靠近/条件、走位、镜头、日落、音乐，
`Cmd::Run` 可调用章节自己的 Rust 代码；停滞 14 s 自动闪标记。

## 7. 界面（GameUI 及各 View）

- **ChapterCardView**：章节卡（"第 N 章" + 标题 + 副标题，默认停留 3 s）。
- **ObjectiveView**：左上角目标，三种状态 EventStart（事件名+步骤）/ Step / Depart（"可以启程了"）。
- **BannerView**：事件完成横幅，In 0.6 s / Hold 2.2 s / Out 0.8 s，含"接下来：…"。
- **DialogueView**：底部对话框，宽 1240、底边距 76；说话人名牌 + 颜色徽章（Speaker：小王子、
  玫瑰、国王、爱虚荣的人、酒鬼、商人、点灯人、地理学家）；**逐字显示 0.025 s/字**；按键继续的
  闪烁提示；分页。
- **MonologueView**（内心独白，居中）、Memory（回忆淡显）、**TutorialView**（按键帽 + 文字）、
  **KeyHintsView**、**PromptView / AnchoredPromptView**（交互提示，按住圆环）、
  **MarkerView**（目标星标+箭头+文字）、**TrailView**（地面上的星点轨迹指向目标）、
  **DepartView**（按住 **1.6 s** 启程，`DepartHoldSeconds`）、**HiddenTaskView**、**TabView**、
  **PauseView**（暂停/设置：语言、音量滑条、灵敏度、按键说明、清除进度需二次确认、网页全屏）、
  **Fade**（墨色转场）。
- UITheme：衬线中文字体、羊皮纸/墨色配色。

本重制：`core/ui/hud.rs`（卡片、目标、横幅、对话打字机 0.04 s/字(中文)、名牌徽章、选择、独白、
教学、提示、标记、隐藏目标、按住启程条、Tab 任务页、墨色淡入淡出）、`ui/menu.rs`、`ui/touch.rs`。
TrailView（地面轨迹）未实现。

## 8. 旅程图与章节流程（LittlePrince.Journey）

- **JourneyAtlas** 列出星球；**JourneyPlanet**：id、场景名、地图位置（舞台坐标 **1672×941**，左上原点）、
  半径/光环半径、清晰图/雾图、云量/光晕、文案（标题、副标题、旁白、按钮、去过后的副标题/旁白/按钮、
  加载页文案）、**离开条件** `JourneyGoal`（None / ReachObject / Story）+ `goalObject` + `goalRadius`
  + `goalFallbackSeconds`（剧情目标超时也允许离开）。
- **PlanetState**：`Fog`（雾中）/ `Lit`（已揭示）/ `Visited`（去过）/ `Teaser`（预告）。
- **JourneyProgress**（存档）：`revealed[]`、`visited[]`；`Leave(id)` → 标记去过并
  **RevealToNextStop**（揭示路线上下一个可玩星球）；`CatchUp`；`HasStarted`。
- **JourneyOptions.FreeTravel**：自由旅行（全部可选）。
- **JourneyMapScreen**：星空背景、点状航线、星球节点、标题/副标题/旁白逐字出现（0.038 s/字）、
  出发按钮、鸟群载着小王子沿航线飞到下一站（Travel / PassOver / PopDot / RevealAndPresent）。
- **JourneyFlow**：LoadPlanet（墨色 Wash 转场 + 加载页）、LeavePlanet、ReturnToMap、ContinueInPlanet。
- **JourneyDeparture**：故事完成后进入 Ready；提示"可以启程了"；**按住 E 1.6 s** 启程（或出口闸）；
  然后鸟群带走 → 回到地图。
- 8 章：b612、king、vanity（剧场）、bar（酒鬼）、businessman（商人/天文台）、lamp（点灯人）、
  geo（地理学家）、earth（沙漠，最大），earth 之后进入结局。

本重制：`core/state.rs`（AppState：Boot→Title→Map→Loading→Chapter→Ending；Flow 消息；
`leave_planet` = JourneyProgress.Leave；`departure` 按住 1.6 s；墨色转场）与 `ui/map.rs`
（同样的四种状态、同样的舞台坐标系、航线、鸟群飞行动画、自由旅行）。

## 9. 天空与昼夜（B612TimeOfDay / SunCycle / DayNightCycle）

- 太阳按真实太阳轨迹：纬度（默认 55°N）、赤纬（+2.555°）、"北"的游戏方位（+Z=东，北=-X=270°）。
- 自由游玩：`hoursPerMinute`（0.5 = 每 2 分钟一个太阳小时），永不超过 `capHour`；
  **第一次坐下才开始日落**（FirstSunsetRequested）。
- 日落：从当前时刻扫到 `endElevation`（半没入远山），时长随剩余弧线在 min/max 之间；
  可选"按高度配速"：高太阳段快速 S 曲线、金色时段慢、最后缓缓沉没。
- DayPhase：Day / Golden / Dusk / Night / Dawn / Morning；TimeOfDayPreset 定义各阶段
  太阳颜色强度、环境光、雾、天顶/地平线色、太阳辉光、反暮光带、云等。
- 星空随时间旋转；DayNightTimeline 把时间映射到阶段。

本重制：`core/sky.rs`（同样的太阳位置公式和参数、cap、`start_sunset` 三段配速曲线、
阶段判定、调色板插值天空穹顶、星空、太阳盘、平行光与环境光、雾）。

## 10. 存档、语言、网页层、音频

- 存档：JourneyProgress 文件 + PlayerPrefs（语言、音量、灵敏度、自由旅行）。
- Loc：中英（另有日韩等）表格，`T(zh, context)` 以中文为键；语言切换事件 Changed；
  WebPlatform 用浏览器语言（LPWebGetLang/SetLang）。
- WebPlatform：URL 参数 `?chapter=`（调试直接进章）、画质覆盖、`freelook`、帧统计、清进度重启、
  全屏、按章节分包下载、纹理分级加载。
- GameAudio / ChapterMusic / AudioZone / InteriorAmbience / WebAudioRolloff：章节音乐、
  区域环境声、室内混响感、脚步声表。

本重制：`core/save.rs`（localStorage / 文件）、`core/i18n.rs`（键值表）、`core/platform.rs`、
`core/params.rs`、`core/audio.rs` + `synth.rs`（清单驱动 + 合成兜底）。

## 11. 测试驱动（AgentInput / AgentInputDriver / PlaytestRoute）

- AgentInput 静态 API：`Move(x, y, seconds, run)`、`MoveTo(x, z, run, timeout, stopRadius)`、
  `MoveWorld`、`Look`、`LookToward`、`Zoom`、`Jump`、`Interact`、`Wait`、`Stop`、
  `DebugTeleport(anchor)`、`Capture`（截图）；可查询 Position / FacingYaw / NearbyInteractable /
  IsBusy / IsSettled。
- AgentInputDriver 把命令队列（Move/MoveTo/Look/Press/Wait）逐帧写入 PlayerInputFrame，
  真实驱动 PlayerController（不是瞬移）。
- PlaytestRoute：按脚本跑一条路线，每步超时，记录日志与截图；PlaytestRecorder 记录。

本重制：`core/testing.rs` 的 Autopilot：读取剧情当前等待，像玩家一样走过去、对准、按/按住 E、
推进对话、选择、执行章节给的 `TestAction` 提示、最后按住 E 启程；卡住时传送到目标附近；
每个等待 60 s 超时；输出 `[test:ok id]` / `[test:fail id] 原因`。
