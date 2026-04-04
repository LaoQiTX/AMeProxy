//! 配置文件管理模块
//! 
//! 该模块负责配置文件的管理，包括：
//! 1. 生成默认配置文件
//! 2. 获取配置文件路径
//! 3. 添加代理订阅
//! 4. 更新代理订阅
//! 5. 删除代理订阅

use std::env;
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// 配置类型
#[derive(Debug, Clone, Copy)]
pub enum ConfigType {
    /// 运行时配置
    Run,
    /// 草稿配置
    Draft,
}

/// Tun配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TunConfig {
    pub enable: Option<bool>,
    pub stack: Option<String>,
    pub dns_hijack: Option<Vec<String>>,
    pub auto_route: Option<bool>,
    pub auto_redirect: Option<bool>,
    pub auto_detect_interface: Option<bool>,
}

/// DNS配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DnsConfig {
    pub enable: Option<bool>,
    pub ipv6: Option<bool>,
    pub enhanced_mode: Option<String>,
    pub fake_ip_range: Option<String>,
    pub listen: Option<String>,
    pub nameserver: Option<Vec<String>>,
    pub fallback: Option<Vec<String>>,
}

/// 实验性功能配置
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    pub ignore_resolve_fail: Option<bool>,
}

/// Clash配置结构体
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClashConfig {
    pub port: Option<u16>,
    pub socks_port: Option<u16>,
    pub redir_port: Option<u16>,
    pub tproxy_port: Option<u16>,
    pub mixed_port: Option<u16>,
    pub allow_lan: Option<bool>,
    pub mode: Option<String>,
    pub log_level: Option<String>,
    pub external_controller: Option<String>,
    pub secret: Option<String>,
    pub tun: Option<TunConfig>,
    pub dns: Option<DnsConfig>,
    pub experimental: Option<ExperimentalConfig>,
}

impl ClashConfig {
    /// 获取默认配置
    pub fn default() -> Self {
        Self {
            mixed_port: Some(7890),
            allow_lan: Some(true),
            mode: Some("rule".to_string()),
            log_level: Some("info".to_string()),
            external_controller: Some("127.0.0.1:9090".to_string()),
            ..Default::default()
        }
    }

    /// 获取配置
    pub async fn get(_config_type: ConfigType) -> Result<Self, anyhow::Error> {
        // 目前返回默认配置
        Ok(Self::default())
    }
    
    /// 生成配置文件
    /// 
    /// 配置文件应该和内核在同一目录下：src-tauri/sidecar/
    pub async fn generate_file() -> Result<PathBuf, anyhow::Error> {
        // 获取项目根目录
        let mut project_root = match env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                println!("获取当前目录失败: {}", e);
                return Err(anyhow::anyhow!("Failed to get current directory: {}", e));
            }
        };
        
        // 检查当前目录是否已经是 src-tauri
        if project_root.file_name().unwrap_or_default() == "src-tauri" {
            // 如果是 src-tauri 目录，直接使用 sidecar 子目录
            project_root = project_root.parent().unwrap_or(&project_root).to_path_buf();
        }
        
        let sidecar_dir = project_root
            .join("src-tauri")
            .join("sidecar");
        
        // 打印路径信息，便于调试
        println!("Project root: {:?}", project_root);
        println!("Sidecar directory: {:?}", sidecar_dir);
        
        // 确保目录存在
        match fs::create_dir_all(&sidecar_dir) {
            Ok(_) => println!("Sidecar directory created or already exists"),
            Err(e) => {
                println!("创建目录失败: {}", e);
                return Err(anyhow::anyhow!("Failed to create sidecar directory: {}", e));
            }
        }
        
        let file_path = sidecar_dir.join("config.yaml");
        println!("Config file path: {:?}", file_path);
        
        // 如果配置文件不存在，生成默认配置
        if !file_path.exists() {
            println!("Config file not found, generating default...");
            let default_config = r#"mixed-port: 7890
allow-lan: true
external-controller: 127.0.0.1:9090

proxy-providers:

proxies:
  - name: "直连"
    type: direct
    udp: true

proxy-groups:
  - name: 默认
    type: select
    proxies: [直连]

rules:
  - MATCH,默认
"#;
            
            match fs::write(&file_path, default_config) {
                Ok(_) => println!("Default config generated successfully: {:?}", file_path),
                Err(e) => {
                    println!("写入配置文件失败: {}", e);
                    return Err(anyhow::anyhow!("Failed to write config file: {}", e));
                }
            }
        } else {
            println!("Config file already exists: {:?}", file_path);
        }
        
        Ok(file_path)
    }
}
