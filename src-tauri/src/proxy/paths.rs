//! 路径管理模块
//!
//! 该模块集中管理项目中的所有路径，避免路径逻辑分散在各个模块中。
//!
//! 路径在应用 setup 阶段通过 Tauri 的路径 API 初始化并缓存：
//! - 配置目录：开发环境使用项目目录 `configs/mihomo/`，生产环境使用应用数据目录
//! - sidecar 目录：优先资源目录，开发环境回退 `src-tauri/sidecar/`

use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::Manager;

/// 配置目录缓存
static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
/// sidecar 目录缓存
static SIDECAR_DIR: OnceLock<PathBuf> = OnceLock::new();

#[cfg(test)]
pub(crate) fn init_test_paths(config: PathBuf, sidecar: PathBuf) {
    CONFIG_DIR
        .set(config)
        .expect("test paths already initialized");
    SIDECAR_DIR
        .set(sidecar)
        .expect("test paths already initialized");
}

/// 获取项目根目录路径（仅作为开发环境回退手段）
pub fn get_project_root() -> Result<PathBuf, String> {
    let mut path =
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    if path.file_name().unwrap_or_default() == "src-tauri" {
        path = path.parent().unwrap_or(&path).to_path_buf();
    }
    Ok(path)
}

/// 查找 sidecar 目录：优先返回包含 mihomo 可执行文件的目录
fn find_sidecar_dir(app: &tauri::AppHandle) -> PathBuf {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 生产环境：资源目录（externalBin 会被打包到此处）
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.clone());
        candidates.push(resource_dir.join("sidecar"));
    }

    // 开发环境：项目目录下的 src-tauri/sidecar
    if let Ok(root) = get_project_root() {
        candidates.push(root.join("src-tauri").join("sidecar"));
        candidates.push(root.join("sidecar"));
    }

    candidates.push(PathBuf::from("sidecar"));

    // 优先选择包含 mihomo 可执行文件的目录
    for dir in &candidates {
        if let Ok(entries) = std::fs::read_dir(dir) {
            if entries
                .flatten()
                .any(|e| e.file_name().to_string_lossy().contains("mihomo"))
            {
                return dir.clone();
            }
        }
    }

    // 回退：第一个存在的目录
    candidates
        .into_iter()
        .find(|d| d.exists())
        .unwrap_or_else(|| PathBuf::from("sidecar"))
}

/// 初始化路径（在 setup 阶段调用一次）
///
/// 生产环境（release）使用应用数据目录存放配置，避免依赖当前工作目录；
/// 开发环境（debug）保持项目相对路径，兼容现有开发流程。
pub fn init_paths(app: &tauri::AppHandle) {
    #[cfg(debug_assertions)]
    let config_dir = get_project_root()
        .map(|r| r.join("configs").join("mihomo"))
        .unwrap_or_else(|_| {
            app.path()
                .app_config_dir()
                .map(|d| d.join("configs").join("mihomo"))
                .unwrap_or_else(|_| PathBuf::from("configs/mihomo"))
        });

    #[cfg(not(debug_assertions))]
    let config_dir = app
        .path()
        .app_config_dir()
        .map(|d| d.join("configs").join("mihomo"))
        .unwrap_or_else(|_| {
            get_project_root()
                .map(|r| r.join("configs").join("mihomo"))
                .unwrap_or_else(|_| PathBuf::from("configs/mihomo"))
        });

    let _ = CONFIG_DIR.set(config_dir);
    let _ = SIDECAR_DIR.set(find_sidecar_dir(app));
}

/// 获取配置文件目录 `configs/mihomo/`
pub fn get_config_dir() -> Result<PathBuf, String> {
    if let Some(dir) = CONFIG_DIR.get() {
        return Ok(dir.clone());
    }
    Ok(get_project_root()?.join("configs").join("mihomo"))
}

/// 获取配置文件路径 `configs/mihomo/config.yaml`
pub fn get_config_path() -> Result<PathBuf, String> {
    Ok(get_config_dir()?.join("config.yaml"))
}

/// 获取 sidecar 目录 `src-tauri/sidecar/`（存放内核可执行文件）
pub fn get_sidecar_dir() -> Result<PathBuf, String> {
    if let Some(dir) = SIDECAR_DIR.get() {
        return Ok(dir.clone());
    }
    Ok(get_project_root()?.join("src-tauri").join("sidecar"))
}

/// Resolve only executable sidecars, never archives or configuration files.
pub fn get_kernel_path() -> Result<PathBuf, String> {
    let dir = get_sidecar_dir()?;
    let packaged = dir.join(format!("my-mihomo{}", std::env::consts::EXE_SUFFIX));
    if packaged.is_file() {
        return Ok(packaged);
    }
    let mut candidates: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name().and_then(|n| n.to_str()).is_some_and(|name| {
                    name.starts_with("my-mihomo-")
                        && if cfg!(windows) {
                            name.ends_with(".exe")
                        } else {
                            p.extension().is_none()
                        }
                })
        })
        .collect();
    candidates.sort();
    match candidates.len() {
        1 => Ok(candidates.remove(0)),
        0 => Err(format!("未找到 mihomo 内核: {}", dir.display())),
        _ => Err("发现多个内核文件，请仅保留当前平台的 mihomo".into()),
    }
}
