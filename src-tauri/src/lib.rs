//! 应用程序主入口
//! 
//! 该文件是 Tauri 应用的主入口点，负责初始化应用、设置插件和注册命令。

use tauri::{generate_handler, Manager};
use tauri_plugin_log;
use log;

// 导入代理模块，包含代理进程管理和配置文件管理
mod proxy;
// 导入命令模块，包含所有 Tauri 命令
mod commands;

// 导入代理模块中的 AppState 函数
use proxy::AppState;
// 导入 commands 模块中的所有命令
use commands::*;

/// 应用程序主入口函数
/// 
/// 该函数负责：
/// 1. 生成默认配置文件
/// 2. 初始化 Tauri 应用
/// 3. 设置应用状态
/// 4. 注册插件和命令
/// 5. 启动应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // 生成默认配置文件，如果失败则打印警告
  if let Err(err) = proxy::generate_default_config() {
    println!("Warning: Failed to generate default config: {}", err);
  }
  
  tauri::Builder::default()
    // 管理应用状态，主要是代理进程的状态
    .manage(AppState {
        proxy_process: std::sync::Mutex::new(None),
    })
    // 设置应用，添加日志插件并启动代理
    .setup(|app| {
      // 在调试模式下添加日志插件
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      
      // 启动代理内核
      let app_state = app.try_state::<AppState>().ok_or("Failed to get app state".to_string())?;
      let mut process = app_state.proxy_process.lock().map_err(|e| e.to_string())?;
      
      if process.is_none() {
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
        for (m_dir, e_path) in possible_paths {
            if e_path.exists() {
                found_path = Some((m_dir, e_path));
                break;
            }
        }
        
        if let Some((mihomo_dir, exe_path)) = found_path {
          // 创建启动命令
          let mut command = std::process::Command::new(exe_path);
          command.arg("-d").arg(mihomo_dir);

          // Windows平台特定设置：创建无窗口进程
          #[cfg(target_os = "windows")]
          {{
              use std::os::windows::process::CommandExt;
              command.creation_flags(0x08000000);
          }}

          // 启动进程
          match command.spawn() {
            Ok(child) => {
              *process = Some(child);
              println!("Proxy kernel started automatically");
            },
            Err(e) => {
              println!("Warning: Failed to start proxy kernel: {}", e);
            }
          }
        } else {
          println!("Warning: mihomo.exe not found, proxy kernel will not start automatically");
        }
      }
      
      Ok(())
    })
    // 注册所有 Tauri 命令
    .invoke_handler(generate_handler![start_proxy, stop_proxy, save_subscription, set_proxy_provider_url, add_proxy_provider, update_proxy_provider, remove_proxy_provider])
    // 运行应用
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
