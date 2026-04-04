//! 代理进程管理模块
//! 
//! 该模块负责代理进程的启动和停止操作，包括：
//! 1. 启动 Mihomo 代理进程
//! 2. 停止 Mihomo 代理进程
//! 3. 管理代理进程的状态

use std::sync::Mutex;
use std::process::Child;
use tauri::AppHandle;
use crate::proxy::config::ClashConfig;
use std::thread;
use std::time::Duration;

/// 应用状态结构体
/// 
/// 用于管理应用的全局状态，主要是代理进程的状态
pub struct AppState {
    /// 代理进程的互斥锁，用于线程安全地访问和修改进程状态
    pub proxy_process: Mutex<Option<Child>>,
}

impl AppState {
    /// 启动代理内核
    pub async fn start_core(&self, _app_handle: &AppHandle) -> Result<(), anyhow::Error> {
        // 生成配置文件
        let config_file = ClashConfig::generate_file().await?;
        println!("Config file generated at: {:?}", config_file);
        
        // 配置目录应该和内核在同一目录下
        let sidecar_dir = config_file.parent().unwrap();
        println!("Sidecar directory: {:?}", sidecar_dir);
        
        // 动态查找内核文件
        let mut sidecar_path = None;
        
        // 尝试查找所有可能的内核文件
        let entries = std::fs::read_dir(sidecar_dir)?;
        println!("Files in sidecar directory:");
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_str().unwrap_or("");
                println!("  {:?}", file_name);
                
                // 查找包含 "mihomo" 的可执行文件
                if file_name_str.contains("mihomo") && file_name_str.ends_with(".exe") {
                    sidecar_path = Some(entry.path());
                    break;
                }
            }
        }
        
        // 如果没有找到内核文件，返回错误
        let sidecar_path = sidecar_path.ok_or_else(|| {
            anyhow::anyhow!("Kernel file not found in sidecar directory")
        })?;
        
        println!("Kernel path: {:?}", sidecar_path);
        
        // 启动进程
        println!("Starting kernel process...");
        println!("Working directory: {:?}", sidecar_dir);
        println!("Config file: {:?}", config_file);
        
        let child = std::process::Command::new(&sidecar_path)
            .current_dir(&sidecar_dir)
            .args([
                "-f", config_file.to_str().unwrap(),
            ])
            .spawn()?;
        
        // 保存进程
        *self.proxy_process.lock().unwrap() = Some(child);
        println!("Kernel process started successfully");
        
        // 等待内核启动并初始化
        println!("Waiting for kernel to initialize...");
        thread::sleep(Duration::from_secs(2));
        
        // 检查内核是否真的启动了（尝试连接 API）
        match Self::check_kernel_running().await {
            Ok(true) => {
                println!("Kernel is running and API is accessible");
                Ok(())
            }
            Ok(false) => {
                println!("Warning: Kernel process started but API is not accessible yet");
                // 再等待一段时间
                thread::sleep(Duration::from_secs(3));
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
        if let Some(mut child) = self.proxy_process.lock().unwrap().take() {
            println!("Stopping kernel process...");
            child.kill()?;
            child.wait()?;
            println!("Kernel process stopped");
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
