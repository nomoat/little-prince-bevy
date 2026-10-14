//! Thin platform layer: everything that differs between the browser (wasm32)
//! and a native desktop build lives here — URL/CLI parameters, persistent
//! storage, console logging, touch detection and the page title.

/// Raw `key=value&key2=value2` parameter string.
/// Web: `window.location.search`. Native: CLI args (`--chapter b612 --test`)
/// plus the `LP_PARAMS` environment variable (`chapter=b612&test=1`).
pub fn query_string() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default()
            .trim_start_matches('?')
            .to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut parts: Vec<String> = Vec::new();
        if let Ok(env) = std::env::var("LP_PARAMS") {
            parts.push(env);
        }
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < args.len() {
            let a = args[i].trim_start_matches('-').to_string();
            if let Some((k, v)) = a.split_once('=') {
                parts.push(format!("{k}={v}"));
            } else if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                parts.push(format!("{a}={}", args[i + 1]));
                i += 1;
            } else {
                parts.push(format!("{a}=1"));
            }
            i += 1;
        }
        parts.join("&")
    }
}

/// Reads a persisted string (web: localStorage, native: a file per key).
pub fn storage_get(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let s = web_sys::window()?.local_storage().ok()??;
        s.get_item(key).ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(native_path(key)).ok()
    }
}

pub fn storage_set(key: &str, value: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(Ok(Some(s))) = web_sys::window().map(|w| w.local_storage()) {
            let _ = s.set_item(key, value);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let p = native_path(key);
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, value);
    }
}

pub fn storage_remove(key: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(Ok(Some(s))) = web_sys::window().map(|w| w.local_storage()) {
            let _ = s.remove_item(key);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::remove_file(native_path(key));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn native_path(key: &str) -> std::path::PathBuf {
    let base = std::env::var("LP_SAVE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            std::path::PathBuf::from(home).join(".little_prince_bevy")
        });
    base.join(format!("{}.ron", key.replace(['/', '\\', ':'], "_")))
}

/// Writes one line to the browser console (or stdout natively). The test
/// harness greps these lines (`[test:ok b612]`).
pub fn console_log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&msg.into());
    #[cfg(not(target_arch = "wasm32"))]
    println!("{msg}");
}

/// Factor from CSS px to the window's logical px: 1.0 normally. Some
/// browsers (seen in headless Chrome with an emulated pixel ratio) size the
/// canvas in CSS px while reporting a device pixel ratio, which halves the
/// window's logical size while pointer positions stay in CSS px.
pub fn css_to_logical(window_logical_width: f32) -> f32 {
    #[cfg(target_arch = "wasm32")]
    {
        let css = web_sys::window().and_then(|w| w.inner_width().ok()).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        if css > 1.0 && window_logical_width > 1.0 {
            let k = window_logical_width / css;
            if (k - 1.0).abs() > 0.05 {
                return k;
            }
        }
        1.0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = window_logical_width;
        1.0
    }
}

/// True on phones/tablets (coarse pointer or touch points).
pub fn is_touch_device() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(w) = web_sys::window() else { return false };
        let coarse = w
            .match_media("(pointer: coarse)")
            .ok()
            .flatten()
            .map(|m| m.matches())
            .unwrap_or(false);
        // Primary pointer is a finger: phones and tablets. Touch laptops keep
        // a fine primary pointer and get the controls on their first touch.
        coarse
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Sets the document title (web only); used for status markers.
pub fn set_page_title(title: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(d) = web_sys::window().and_then(|w| w.document()) {
            d.set_title(title);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = title;
}

/// Toggles browser fullscreen (web only).
pub fn toggle_fullscreen() {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = js_sys::eval(
            "(function(){var d=document; if(d.fullscreenElement){d.exitFullscreen();}else{var c=d.getElementById('bevy')||d.documentElement; if(c.requestFullscreen) c.requestFullscreen();}})()",
        );
    }
}

/// Tells the page loader that the game is up (hides the HTML loading veil).
pub fn notify_ready() {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = js_sys::eval("window.__lpReady && window.__lpReady()");
    }
}

pub fn is_web() -> bool {
    cfg!(target_arch = "wasm32")
}
