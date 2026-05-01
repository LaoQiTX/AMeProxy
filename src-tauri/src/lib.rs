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
/// 1. 初始化 Tauri 应用
/// 2. 设置应用状态
/// 3. 注册插件和命令
/// 4. 启动应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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

      // 自动启动代理内核
      if let Some(app_state) = app.try_state::<AppState>() {
        if let Ok(mut process) = app_state.proxy_process.lock() {
          if process.is_none() {
            // 获取 sidecar 和配置目录
            let sidecar_dir = match proxy::paths::get_sidecar_dir() {
              Ok(d) => d,
              Err(e) => {
                println!("Warning: Failed to get sidecar directory: {}", e);
                return Ok(());
              }
            };
            let config_dir = match proxy::paths::get_config_dir() {
              Ok(d) => d,
              Err(e) => {
                println!("Warning: Failed to get config directory: {}", e);
                return Ok(());
              }
            };

            // 查找 mihomo 可执行文件
            let mut sidecar_path = None;
            if let Ok(entries) = std::fs::read_dir(&sidecar_dir) {
              for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_str().unwrap_or("");
                if file_name_str.contains("mihomo") {
                  sidecar_path = Some(entry.path());
                  break;
                }
              }
            }

            if let Some(exe_path) = sidecar_path {
              println!("Starting proxy kernel automatically from: {:?}", exe_path);
              println!("Config directory: {:?}", config_dir);

              let mut command = std::process::Command::new(&exe_path);
              command.arg("-d").arg(&config_dir);

              // Windows 平台特定设置：创建无窗口进程
              #[cfg(target_os = "windows")]
              {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000); // CREATE_NO_WINDOW
              }

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
              println!("Warning: mihomo executable not found in {:?}", sidecar_dir);
            }
          }
        }
      }

      Ok(())
    })
    // 注册所有 Tauri 命令
    .invoke_handler(generate_handler![start_core, stop_core, start_proxy, stop_proxy, is_proxy_running, get_proxies, change_proxy, test_proxy, get_providers, get_rules, save_subscription, set_proxy_provider_url, add_proxy_provider, update_proxy_provider, remove_proxy_provider, get_config, get_connections, close_connection, get_logs, get_uptime, toggle_tun, get_tun_status])
    // 应用关闭时停止代理内核
    .on_window_event(|app, event| {
      if let tauri::WindowEvent::CloseRequested { .. } = event {
        // 停止代理内核
        println!("App is closing, stopping proxy kernel...");

        // 获取应用状态并停止内核
        if let Some(app_state) = app.try_state::<AppState>() {
          if let Err(e) = app_state.stop_core() {
            println!("Error stopping proxy kernel: {}", e);
          } else {
            println!("Proxy kernel stopped successfully");
          }
        }
      }
    })
    // 运行应用
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
