//! 代理进程管理模块
//!
//! 该模块负责代理进程的启动和停止操作，包括：
//! 1. 启动 Mihomo 代理进程
//! 2. 停止 Mihomo 代理进程
//! 3. 管理代理进程的状态

use std::sync::Mutex;
use std::process::Child;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use crate::proxy::config::ClashConfig;

/// 应用状态结构体
///
/// 用于管理应用的全局状态，主要是代理进程的状态
pub struct AppState {
    /// 代理进程的互斥锁，用于线程安全地访问和修改进程状态
    pub proxy_process: Mutex<Option<Child>>,
    /// 代理启动时间，用于计算运行时长
    pub start_time: Mutex<Option<Instant>>,
}

impl AppState {
    /// 获取运行时长（秒）
    pub fn get_uptime_secs(&self) -> u64 {
        if let Ok(guard) = self.start_time.lock() {
            if let Some(start) = *guard {
                return start.elapsed().as_secs();
            }
        }
        0
    }
    /// 启动代理内核
    pub async fn start_core(&self, _app_handle: &AppHandle) -> Result<(), anyhow::Error> {
        // 检查是否已经在运行
        {
            let guard = self.proxy_process.lock()
                .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            if guard.is_some() {
                println!("Kernel already running, skipping start");
                return Ok(());
            }
        }

        // 确保配置文件存在
        let config_file = ClashConfig::generate_file()?;
        println!("Config file: {:?}", config_file);

        let config_dir = config_file.parent().unwrap();
        println!("Config directory: {:?}", config_dir);

        // 获取 sidecar 目录（存放内核可执行文件）
        let sidecar_dir = crate::proxy::paths::get_sidecar_dir()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // 查找内核可执行文件
        let mut sidecar_path = None;
        let entries = std::fs::read_dir(&sidecar_dir)?;
        println!("Files in sidecar directory:");
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_str().unwrap_or("");
                println!("  {:?}", file_name);
                if file_name_str.contains("mihomo") && file_name_str.ends_with(".exe") {
                    sidecar_path = Some(entry.path());
                    break;
                }
            }
        }

        let sidecar_path = sidecar_path.ok_or_else(|| {
            anyhow::anyhow!("Kernel file not found in sidecar directory")
        })?;

        println!("Kernel path: {:?}", sidecar_path);
        println!("Starting kernel process with -d {:?}...", config_dir);

        // 使用 -d 参数将工作目录设为配置文件目录
        // 这样 mihomo 会在该目录读取 config.yaml 并创建 cache.db 等运行时文件，
        // 避免在 src-tauri/ 下生成文件触发 Tauri 热重载
        let child = std::process::Command::new(&sidecar_path)
            .arg("-d")
            .arg(config_dir)
            .spawn()?;

        // 保存进程
        *self.proxy_process.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))? = Some(child);
        // 记录启动时间
        if let Ok(mut time_guard) = self.start_time.lock() {
            *time_guard = Some(Instant::now());
        }
        println!("Kernel process started successfully");

        // 等待内核初始化
        println!("Waiting for kernel to initialize...");
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 检查内核是否真的启动了（尝试连接 API）
        match Self::check_kernel_running().await {
            Ok(true) => {
                println!("Kernel is running and API is accessible");
                Ok(())
            }
            Ok(false) => {
                println!("Warning: Kernel process started but API is not accessible yet");
                // 再等待一段时间
                tokio::time::sleep(Duration::from_secs(3)).await;
                Ok(())
            }
            Err(e) => {
                println!("Warning: Failed to check kernel status: {}", e);
                // 进程已启动，但无法确认状态，继续
                Ok(())
            }
        }
    }

    /// 停止代理内核
    pub fn stop_core(&self) -> Result<(), anyhow::Error> {
        if let Ok(mut guard) = self.proxy_process.lock() {
            if let Some(mut child) = guard.take() {
                println!("Stopping kernel process...");
                child.kill()?;
                child.wait()?;
                println!("Kernel process stopped");
            }
        }
        // 清除启动时间
        if let Ok(mut time_guard) = self.start_time.lock() {
            *time_guard = None;
        }
        Ok(())
    }

    /// 检查内核是否正在运行
    async fn check_kernel_running() -> Result<bool, anyhow::Error> {
        // 尝试连接 mihomo API
        match std::net::TcpStream::connect("127.0.0.1:9090") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
