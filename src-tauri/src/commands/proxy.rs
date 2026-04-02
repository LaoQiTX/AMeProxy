//! 代理命令模块
//! 
//! 该模块包含与代理相关的Tauri命令，包括：
//! 1. 启动代理
//! 2. 停止代理

use tauri::{State, command};
use crate::proxy::*;

// Windows 平台特定导入
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows 平台创建无窗口进程的标志
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 启动代理
/// 
/// 该函数负责启动Mihomo代理进程
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程信息
/// 
/// # 返回
/// * `Ok(String)` - 启动成功的消息
/// * `Err(String)` - 启动失败的错误信息
#[command]
pub fn start_proxy(state: State<AppState>) -> Result<String, String> {
    // 获取代理进程的互斥锁
    let mut process = state.proxy_process.lock().map_err(|e| e.to_string())?;
    
    // 检查代理是否已经在运行
    if process.is_some() {
        return Ok("Proxy is already running".into());
    }

    // 获取当前目录
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // 尝试不同的路径查找 mihomo.exe
    let mut possible_paths = Vec::new();
    
    // 路径1: 当前目录下的 configs/mihomo
    let mut mihomo_dir1 = current_dir.clone();
    mihomo_dir1.push("configs");
    mihomo_dir1.push("mihomo");
    let mut exe_path1 = mihomo_dir1.clone();
    exe_path1.push("mihomo.exe");
    possible_paths.push((mihomo_dir1, exe_path1));
    
    // 路径2: 向上两级目录的 configs/mihomo (适用于打包后的环境)
    let mut mihomo_dir2 = current_dir.clone();
    mihomo_dir2.pop();
    mihomo_dir2.pop();
    mihomo_dir2.push("configs");
    mihomo_dir2.push("mihomo");
    let mut exe_path2 = mihomo_dir2.clone();
    exe_path2.push("mihomo.exe");
    possible_paths.push((mihomo_dir2, exe_path2));
    
    // 路径3: 向上三级目录的 configs/mihomo (适用于更深层次的打包环境)
    let mut mihomo_dir3 = current_dir.clone();
    mihomo_dir3.pop();
    mihomo_dir3.pop();
    mihomo_dir3.pop();
    mihomo_dir3.push("configs");
    mihomo_dir3.push("mihomo");
    let mut exe_path3 = mihomo_dir3.clone();
    exe_path3.push("mihomo.exe");
    possible_paths.push((mihomo_dir3, exe_path3));
    
    // 查找存在的 mihomo.exe
    let mut found_path = None;
    for (m_dir, e_path) in &possible_paths {
        if e_path.exists() {
            found_path = Some((m_dir.clone(), e_path.clone()));
            break;
        }
    }
    
    let (mihomo_dir, exe_path) = found_path.ok_or_else(|| {
        format!("mihomo.exe not found. Tried paths: {:?}", 
            possible_paths.iter().map(|(_, e)| e).collect::<Vec<_>>())
    })?;

    // 创建启动命令
    let mut command = std::process::Command::new(exe_path);
    command.arg("-d").arg(mihomo_dir);

    // Windows平台特定设置：创建无窗口进程
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    // 启动进程
    let child = command.spawn()
        .map_err(|e| format!("Failed to start proxy: {}", e))?;

    // 保存进程信息
    *process = Some(child);
    
    Ok("Proxy started successfully".into())
}

/// 停止代理
/// 
/// 该函数负责停止Mihomo代理进程
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程信息
/// 
/// # 返回
/// * `Ok(String)` - 停止成功的消息或代理未运行的消息
/// * `Err(String)` - 停止失败的错误信息
#[command]
pub fn stop_proxy(state: State<AppState>) -> Result<String, String> {
    // 获取代理进程的互斥锁
    let mut process = state.proxy_process.lock().map_err(|e| e.to_string())?;
    
    // 检查代理是否在运行
    if let Some(mut child) = process.take() {
        // 终止进程
        let _ = child.kill();
        // 等待进程结束
        let _ = child.wait();
        Ok("Proxy stopped".into())
    } else {
        Ok("Proxy is not running".into())
    }
}
