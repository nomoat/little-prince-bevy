//! Bilingual string tables (zh default, en). Like the original's `Loc`, every
//! player-visible string goes through a key lookup; chapters contribute their
//! own `<id>.*` tables which are merged at startup.
//!
//! A table is a `&'static [(key, zh, en)]` — plain Rust data, so the font
//! subsetter (scripts/subset_font.sh) sees every character.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type StringTable = &'static [(&'static str, &'static str, &'static str)];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    Zh,
    En,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Zh => "zh",
            Lang::En => "en",
        }
    }
    pub fn parse(s: &str) -> Option<Lang> {
        let s = s.to_ascii_lowercase();
        if s.starts_with("zh") || s == "cn" {
            Some(Lang::Zh)
        } else if s.starts_with("en") {
            Some(Lang::En)
        } else {
            None
        }
    }
    pub fn other(self) -> Lang {
        match self {
            Lang::Zh => Lang::En,
            Lang::En => Lang::Zh,
        }
    }
}

#[derive(Resource, Default)]
pub struct I18n {
    pub lang: Lang,
    zh: HashMap<&'static str, &'static str>,
    en: HashMap<&'static str, &'static str>,
}

impl I18n {
    pub fn add_table(&mut self, table: StringTable) {
        for (k, zh, en) in table {
            if self.zh.insert(k, zh).is_some() {
                warn!("i18n: duplicate key {k}");
            }
            self.en.insert(k, en);
        }
    }

    /// Looks up `key` in the current language; falls back to the other
    /// language, then to the key itself (so missing keys are visible).
    pub fn t<'a>(&self, key: &'a str) -> &'a str
    where
        'static: 'a,
    {
        let (a, b) = match self.lang {
            Lang::Zh => (&self.zh, &self.en),
            Lang::En => (&self.en, &self.zh),
        };
        if let Some(s) = a.get(key).filter(|s| !s.is_empty()) {
            return s;
        }
        if let Some(s) = b.get(key) {
            return s;
        }
        key
    }

    /// `t` with `{name}` placeholders replaced.
    pub fn tf(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut s = self.t(key).to_string();
        for (k, v) in args {
            s = s.replace(&format!("{{{k}}}"), v);
        }
        s
    }

    pub fn has(&self, key: &str) -> bool {
        self.zh.contains_key(key)
    }

    pub fn is_zh(&self) -> bool {
        self.lang == Lang::Zh
    }
}

/// Put on an entity with `Text` to keep it translated: the text is replaced
/// whenever the language changes (or the key changes).
#[derive(Component, Clone, Debug)]
pub struct Tr(pub String);

impl Tr {
    pub fn new(key: impl Into<String>) -> Self {
        Tr(key.into())
    }
}

pub(crate) fn apply_translations(
    i18n: Res<I18n>,
    mut q: Query<(Ref<Tr>, &mut Text)>,
) {
    let all = i18n.is_changed();
    for (tr, mut text) in &mut q {
        if all || tr.is_changed() {
            let s = i18n.t(&tr.0);
            if text.0 != s {
                text.0 = s.to_string();
            }
        }
    }
}

/// Core + journey-map + UI strings.
pub const CORE_STRINGS: StringTable = &[
    ("core.game_title", "小王子", "The Little Prince"),
    ("core.game_subtitle", "星辰之旅", "A Journey Among Stars"),
    ("core.loading", "正在前往……", "Travelling…"),
    ("core.chapter_n", "第{n}章", "Chapter {n}"),
    ("core.prince", "小王子", "Little Prince"),
    ("core.narrator", "", ""),
    // Title
    ("core.title.begin", "开始旅程", "Begin the Journey"),
    ("core.title.continue", "继续旅程", "Continue"),
    ("core.title.settings", "设置", "Settings"),
    ("core.title.foot", "一段关于驯服、离别与看见的旅程 · 同人重制", "A journey of taming, leaving and seeing · fan remake"),
    ("core.title.tap", "点击任意处开始", "Click anywhere to start"),
    // HUD
    ("core.objective", "目标", "Objective"),
    ("core.hidden", "隐藏目标", "Hidden goal"),
    ("core.hidden_found", "发现隐藏目标", "Hidden goal found"),
    ("core.hidden_unknown", "？？？", "???"),
    ("core.banner.done", "完成", "Done"),
    ("core.banner.next", "接下来", "Next"),
    ("core.depart_line", "可以启程了", "You can set off now"),
    ("core.depart_hold", "按住", "Hold"),
    ("core.depart_verb", "启程", "Set off"),
    ("core.continue", "继续", "Continue"),
    ("core.hold", "按住", "Hold"),
    ("core.tab_hint", "查看任务", "Tasks"),
    ("core.tasks_short", "任务", "Tasks"),
    ("core.events", "本章", "This chapter"),
    // Tutorial
    ("core.teach.move", "移动", "Move"),
    ("core.teach.look", "拖动鼠标转动视角", "Drag the mouse to look around"),
    ("core.teach.run", "奔跑", "Run"),
    ("core.teach.jump", "跳跃", "Jump"),
    ("core.teach.interact", "互动", "Interact"),
    ("core.teach.touch_move", "左侧拖动移动 · 右侧拖动转动视角", "Drag left to move · drag right to look"),
    // Pause / settings
    ("core.menu.paused", "暂停", "Paused"),
    ("core.menu.resume", "继续", "Resume"),
    ("core.menu.settings", "设置", "Settings"),
    ("core.menu.to_map", "返回旅程图", "Journey Map"),
    ("core.menu.to_title", "返回标题", "Title Screen"),
    ("core.menu.fullscreen", "全屏", "Fullscreen"),
    ("core.menu.reset", "清除进度", "Clear Progress"),
    ("core.menu.reset_confirm", "再点一次确认清除", "Click again to confirm"),
    ("core.menu.back", "返回", "Back"),
    ("core.menu.language", "语言", "Language"),
    ("core.menu.master", "总音量", "Master volume"),
    ("core.menu.music", "音乐", "Music"),
    ("core.menu.sfx", "音效", "Sound effects"),
    ("core.menu.sensitivity", "视角灵敏度", "Look sensitivity"),
    ("core.menu.invert_y", "反转纵向视角", "Invert vertical look"),
    ("core.menu.pointer_lock", "鼠标锁定视角", "Pointer-lock mouse look"),
    ("core.menu.free_travel", "自由旅行（解锁全部星球）", "Free travel (unlock all planets)"),
    ("core.menu.quality", "画质", "Quality"),
    ("core.menu.touch", "触屏按键", "Touch controls"),
    ("core.menu.on", "开", "On"),
    ("core.menu.off", "关", "Off"),
    ("core.menu.auto", "自动", "Auto"),
    ("core.quality.low", "低", "Low"),
    ("core.quality.medium", "中", "Medium"),
    ("core.quality.high", "高", "High"),
    ("core.menu.keys", "按键", "Controls"),
    ("core.menu.keys_text", "W A S D 移动 · Shift 奔跑 · 空格 跳跃 · E 互动 · 鼠标拖动 视角 · 滚轮 缩放 · Tab 任务 · Esc 菜单", "WASD move · Shift run · Space jump · E interact · drag mouse to look · wheel zoom · Tab tasks · Esc menu"),
    // Journey map
    ("map.title", "旅程", "The Journey"),
    ("map.depart", "出发", "Set off"),
    ("map.revisit", "再去看看", "Visit again"),
    ("map.fog", "云雾还没有散开", "Still hidden in the clouds"),
    ("map.visited", "去过了", "Visited"),
    ("map.teaser", "远处还有一颗星星", "Another star, further on"),
    ("map.progress", "已经走过 {n} / {total} 颗星球", "{n} of {total} planets visited"),
    ("map.free", "自由旅行", "Free travel"),
    ("map.back", "标题", "Title"),
    ("map.hint", "点击星球 · ←→ 选择 · Enter 出发", "Click a planet · ←→ select · Enter to set off"),
    // Ending
    ("ending.1", "后来，飞行员修好了飞机，回到了有人的地方。", "Later, the pilot mended his plane and went back to where people live."),
    ("ending.2", "可是每到夜里，他都会抬头听一听星星——像五亿个小铃铛。", "But every night he listens to the stars — like five hundred million little bells."),
    ("ending.3", "他始终不知道，那只羊到底有没有吃掉那朵花。", "He never learned whether the sheep ate the flower."),
    ("ending.4", "你也抬头看看吧。你会发现，一切都变得不一样了。", "Look up, too. You will see how everything changes."),
    ("ending.fin", "终", "The End"),
    ("ending.thanks", "谢谢你陪小王子走完这段旅程", "Thank you for travelling with the little prince"),
    ("ending.back", "回到标题", "Back to Title"),
    ("ending.route", "这一路走过的星球", "The planets along the way"),
];
