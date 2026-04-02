//! 代理进程管理模块
//! 
//! 该模块负责代理进程的启动和停止操作，包括：
//! 1. 启动 Mihomo 代理进程
//! 2. 停止 Mihomo 代理进程
//! 3. 管理代理进程的状态

use std::sync::Mutex;
use std::process::{Command, Child};
use std::env;

// Windows 平台特定导入
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows 平台创建无窗口进程的标志
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 应用状态结构体
/// 
/// 用于管理应用的全局状态，主要是代理进程的状态
pub struct AppState {
    /// 代理进程的互斥锁，用于线程安全地访问和修改进程状态
    pub proxy_process: Mutex<Option<Child>>,
}

/// 启动代理进程
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程的状态
/// 
/// # 返回
/// * `Ok(String)` - 启动成功的消息
/// * `Err(String)` - 启动失败的错误信息
pub fn start_proxy_process(state: &mut AppState) -> Result<String, String> {
    // 尝试获取进程锁
    let mut process = state.proxy_process.lock().map_err(|e| e.to_string())?;
    
    // 检查进程是否已经在运行
    if process.is_some() {
        return Ok("Proxy is already running".into());
    }

    // 获取当前目录
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    // 构建 Mihomo 配置目录路径
    let mut mihomo_dir = current_dir.clone();
    mihomo_dir.push("configs");
    mihomo_dir.push("mihomo");
    
    // 构建 Mihomo 可执行文件路径
    let mut exe_path = mihomo_dir.clone();
    exe_path.push("mihomo.exe");
    
    // 检查可执行文件是否存在
    if !exe_path.exists() {
        return Err(format!("mihomo.exe not found at {:?}", exe_path));
    }

    // 创建命令并设置参数
    let mut command = Command::new(exe_path);
    command.arg("-d").arg(mihomo_dir);

    // Windows 平台设置创建无窗口进程
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    // 启动进程
    let child = command.spawn()
        .map_err(|e| format!("Failed to start proxy: {}", e))?;

    // 更新进程状态
    *process = Some(child);
    
    Ok("Proxy started successfully".into())
}

/// 停止代理进程
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程的状态
/// 
/// # 返回
/// * `Ok(String)` - 停止成功的消息或进程未运行的消息
pub fn stop_proxy_process(state: &mut AppState) -> Result<String, String> {
    // 尝试获取进程锁
    let mut process = state.proxy_process.lock().map_err(|e| e.to_string())?;
    
    // 检查进程是否在运行
    if let Some(mut child) = process.take() {
        // 尝试杀死进程
        let _ = child.kill();
        // 等待进程结束
        let _ = child.wait();
        Ok("Proxy stopped".into())
    } else {
        Ok("Proxy is not running".into())
    }
}
