//! M2 · 文件监听 + 增量重建(build-pipeline 扩展)。
//!
//! 监听活动工程的 `src/ chr/ map/ music/`,去抖后触发一次重建(与手动构建经
//! `BuildState` 的锁串行化,避免并发竞态),并把结果经 `build-updated` 事件推给
//! 前端,使"改资源即重建/即听"。`build/` 不在监听范围内,避免产物触发回环。

use crate::build_pipeline::{run_build, BuildState};
use crate::project::{self, ProjectState};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
pub struct WatchState {
    inner: Mutex<Option<Handle>>,
}

struct Handle {
    _watcher: RecommendedWatcher, // kept alive; dropping it ends the stream
    stop: std::sync::Arc<AtomicBool>,
}

impl WatchState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tauri::command]
pub fn watch_start(
    app: AppHandle,
    project: State<ProjectState>,
    build: State<BuildState>,
    watch: State<WatchState>,
) -> Result<(), String> {
    let root = project.active_root()?;
    // stop any prior watcher first
    if let Some(h) = watch.inner.lock().unwrap().take() {
        h.stop.store(true, Ordering::Relaxed);
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })
    .map_err(|e| format!("创建文件监听失败: {e}"))?;
    for sub in ["src", "chr", "map", "music"] {
        let p = root.join(sub);
        if p.exists() {
            let _ = watcher.watch(&p, RecursiveMode::Recursive);
        }
    }

    let stop = std::sync::Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let cancel = build.cancel_flag();
    let lock = build.build_lock();
    let last_result = build.last_result_slot();
    let app2 = app.clone();
    let root2 = root.clone();

    std::thread::spawn(move || loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(_) => {
                // debounce: absorb a burst of events before rebuilding
                let deadline = Instant::now() + Duration::from_millis(300);
                while rx.recv_timeout(Duration::from_millis(60)).is_ok() {
                    if Instant::now() > deadline {
                        break;
                    }
                }
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
                let _g = lock.lock().unwrap(); // serialize with manual build
                cancel.store(false, Ordering::Relaxed);
                match project::load_manifest(&root2) {
                    Ok(m) => {
                        let result = run_build(&root2, &m, cancel.clone());
                        *last_result.lock().unwrap() = Some(result.clone());
                        let _ = app2.emit("build-updated", &result);
                    }
                    Err(e) => {
                        let _ = app2.emit("build-error", e);
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
            }
            Err(_) => break, // channel closed (watcher dropped)
        }
    });

    *watch.inner.lock().unwrap() = Some(Handle { _watcher: watcher, stop });
    Ok(())
}

#[tauri::command]
pub fn watch_stop(watch: State<WatchState>) -> Result<(), String> {
    if let Some(h) = watch.inner.lock().unwrap().take() {
        h.stop.store(true, Ordering::Relaxed);
    }
    Ok(())
}
