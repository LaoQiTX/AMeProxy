//! 路径管理模块
//!
//! 该模块集中管理项目中的所有路径，避免路径逻辑分散在各个模块中。

use std::env;
use std::path::PathBuf;

/// 获取项目根目录路径
pub fn get_project_root() -> Result<PathBuf, String> {
    let mut path =
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    if path.file_name().unwrap_or_default() == "src-tauri" {
        path = path.parent().unwrap_or(&path).to_path_buf();
    }
    Ok(path)
}

/// 获取配置文件目录 `configs/mihomo/`
pub fn get_config_dir() -> Result<PathBuf, String> {
    Ok(get_project_root()?.join("configs").join("mihomo"))
}

/// 获取配置文件路径 `configs/mihomo/config.yaml`
pub fn get_config_path() -> Result<PathBuf, String> {
    Ok(get_config_dir()?.join("config.yaml"))
}

/// 获取 sidecar 目录 `src-tauri/sidecar/`（存放内核可执行文件）
pub fn get_sidecar_dir() -> Result<PathBuf, String> {
    Ok(get_project_root()?
        .join("src-tauri")
        .join("sidecar"))
}
