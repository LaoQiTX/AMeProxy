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

/// 生成默认配置文件
/// 
/// 该函数负责：
/// 1. 确保配置目录存在
/// 2. 检查配置文件是否存在，如果不存在则生成默认配置
/// 3. 同时为src-tauri目录生成配置文件
/// 
/// # 返回
/// * `Ok(())` - 生成成功
/// * `Err(String)` - 生成失败的错误信息
pub fn generate_default_config() -> Result<(), String> {
    // 获取当前目录
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    
    // 确保配置目录存在
    let mut config_dir = current_dir.clone();
    config_dir.push("configs");
    config_dir.push("mihomo");
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    // 检查配置文件是否存在
    let mut config_path = config_dir.clone();
    config_path.push("config.yaml");
    
    if !config_path.exists() {
        // 生成默认配置内容
        let default_config = r#"# url 里填写自己的订阅,名称不能重复
proxy-providers:
  provider1:
    url: ""
    type: http
    interval: 86400
    health-check: {enable: true,url: "https://www.gstatic.com/generate_204", interval: 300}
    override:
      additional-prefix: "[provider1]"

  provider2:
    url: ""
    type: http
    interval: 86400
    health-check: {enable: true,url: "https://www.gstatic.com/generate_204",interval: 300}
    override:
      additional-prefix: "[provider2]"

proxies: 
  - name: "直连"
    type: direct
    udp: true

mixed-port: 7890
ipv6: true
allow-lan: true
unified-delay: false
tcp-concurrent: true
external-controller: 127.0.0.1:9090
external-ui: ui
external-ui-url: "https://github.com/MetaCubeX/metacubexd/archive/refs/heads/gh-pages.zip"

geodata-mode: false
geox-url:
  geoip: "https://github.com/MetaCubeX/meta-rules-dat/releases/download/latest/geoip-lite.dat"
  geosite: "https://github.com/MetaCubeX/meta-rules-dat/releases/download/latest/geosite.dat"
  mmdb: "https://github.com/MetaCubeX/meta-rules-dat/releases/download/latest/country-lite.mmdb"
  asn: "https://github.com/MetaCubeX/meta-rules-dat/releases/download/latest/GeoLite2-ASN.mmdb"

find-process-mode: strict
global-client-fingerprint: chrome

profile:
  store-selected: true
  store-fake-ip: true

sniffer:
  enable: true
  sniff:
    HTTP:
      ports: [80, 8080-8880]
      override-destination: true
    TLS:
      ports: [443, 8443]
    QUIC:
      ports: [443, 8443]
  skip-domain:
    - "Mijia Cloud"
    - "+.push.apple.com"

tun:
  enable: true
  stack: mixed
  dns-hijack:
    - "any:53"
    - "tcp://any:53"
  auto-route: true
  auto-redirect: true
  auto-detect-interface: true

dns:
  enable: true
  ipv6: true
  enhanced-mode: fake-ip
  fake-ip-filter:
    - "*"
    - "+.lan"
    - "+.local"
    - "+.market.xiaomi.com"
  default-nameserver:
    - tls://223.5.5.5
    - tls://223.6.6.6
  nameserver:
    - https://doh.pub/dns-query
    - https://dns.alidns.com/dns-query

proxy-groups:

  - name: 默认
    type: select
    proxies: [自动选择,直连,香港,台湾,日本,新加坡,美国,其它地区,全部节点]

  - name: Google
    type: select
    proxies: [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: Telegram
    type: select
    proxies: [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: Twitter
    type: select
    proxies: [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: 哔哩哔哩
    type: select
    proxies: [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: 巴哈姆特
    type: select
    proxies: [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: YouTube
    type: select
    proxies: [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: NETFLIX
    type: select
    proxies: [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: Spotify
    type: select
    proxies:  [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: Github
    type: select
    proxies:  [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  - name: 国内
    type: select
    proxies:  [直连,默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择]

  - name: 其他
    type: select
    proxies:  [默认,香港,台湾,日本,新加坡,美国,其它地区,全部节点,自动选择,直连]

  #分隔,下面是地区分组
  - name: 香港
    type: select
    include-all: true
    exclude-type: direct
    filter: "(?i)港|hk|hongkong|hong kong"

  - name: 台湾
    type: select
    include-all: true
    exclude-type: direct
    filter: "(?i)台|tw|taiwan"

  - name: 日本
    type: select
    include-all: true
    exclude-type: direct
    filter: "(?i)日|jp|japan"

  - name: 美国
    type: select
    include-all: true
    exclude-type: direct
    filter: "(?i)美|us|unitedstates|united states"

  - name: 新加坡
    type: select
    include-all: true
    exclude-type: direct
    filter: "(?i)(新|sg|singapore)"

  - name: 其它地区
    type: select
    include-all: true
    exclude-type: direct
    filter: "(?i)^(?!.*(?:🇭🇰|🇯🇵|🇺🇸|🇸🇬|🇨🇳|港|hk|hongkong|台|tw|taiwan|日|jp|japan|新|sg|singapore|美|us|unitedstates)).*"

  - name: 全部节点
    type: select
    include-all: true
    exclude-type: direct

  - name: 自动选择
    type: url-test
    include-all: true
    exclude-type: direct
    tolerance: 10

rules:
  - MATCH,默认
"#;
        
        // 写入默认配置文件
        fs::write(&config_path, default_config).map_err(|e| format!("Failed to write default config: {}", e))?;
        println!("Generated default config file at: {:?}", config_path);
    }
    
    // 同样为src-tauri目录生成配置文件
    let mut tauri_config_dir = current_dir.clone();
    tauri_config_dir.push("src-tauri");
    tauri_config_dir.push("configs");
    tauri_config_dir.push("mihomo");
    
    if !tauri_config_dir.exists() {
        fs::create_dir_all(&tauri_config_dir).map_err(|e| format!("Failed to create tauri config directory: {}", e))?;
    }
    
    let mut tauri_config_path = tauri_config_dir.clone();
    tauri_config_path.push("config.yaml");
    
    if !tauri_config_path.exists() {
        // 复制配置文件到src-tauri目录
        let config_content = fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config file: {}", e))?;
        fs::write(&tauri_config_path, config_content).map_err(|e| format!("Failed to write tauri config file: {}", e))?;
        println!("Copied config file to src-tauri directory: {:?}", tauri_config_path);
    }
    
    Ok(())
}

/// 获取配置文件路径
/// 
/// 该函数负责获取配置文件的绝对路径
/// 
/// # 返回
/// * `Ok(String)` - 配置文件的绝对路径
/// * `Err(String)` - 获取路径失败的错误信息
pub fn get_config_path() -> Result<String, String> {
    // 获取当前目录
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    // 构建配置文件路径
    let mut config_path = current_dir;
    config_path.push("configs");
    config_path.push("mihomo");
    config_path.push("config.yaml");
    
    // 返回配置文件路径
    Ok(config_path.to_string_lossy().to_string())
}

/// 添加代理订阅
/// 
/// 该函数负责向配置文件中添加新的代理订阅
/// 
/// # 参数
/// * `name` - 订阅名称
/// * `url` - 订阅URL
/// 
/// # 返回
/// * `Ok(String)` - 添加成功的消息
/// * `Err(String)` - 添加失败的错误信息
pub fn add_proxy_provider(name: String, url: String) -> Result<String, String> {
    // 获取配置文件路径
    let config_path = get_config_path()?;
    // 读取配置文件内容
    let content = fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config file: {}", e))?;
    
    // 找到proxy-providers部分的结束位置
    let lines: Vec<&str> = content.lines().collect();
    let mut insert_index = 0;
    let mut in_proxy_providers = false;
    
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "proxy-providers:" {
            in_proxy_providers = true;
        } else if in_proxy_providers && !line.starts_with(' ') && !line.is_empty() {
            insert_index = i;
            break;
        } else if in_proxy_providers && line.is_empty() {
            insert_index = i;
            break;
        }
    }
    
    if insert_index == 0 {
        return Err("Could not find proxy-providers section in config file".into());
    }
    
    // 构建新的provider配置
    let new_provider = format!(r#"  {}:
    url: "{}"
    type: http
    interval: 86400
    health-check: {{enable: true, url: "https://www.gstatic.com/generate_204", interval: 300}}
    override:
      additional-prefix: "[{}]"
"#, name, url, name);
    
    // 插入新的provider
    let mut new_lines = lines.clone();
    new_lines.insert(insert_index, &new_provider);
    
    // 重新组合内容
    let updated = new_lines.join("\n");
    
    // 写回配置文件
    fs::write(&config_path, &updated).map_err(|e| format!("Failed to write config file: {}", e))?;
    
    // 同时更新src-tauri目录下的配置文件
    let mut tauri_config_path = env::current_dir().map_err(|e| e.to_string())?;
    tauri_config_path.push("src-tauri");
    tauri_config_path.push("configs");
    tauri_config_path.push("mihomo");
    tauri_config_path.push("config.yaml");
    
    if tauri_config_path.exists() {
        fs::write(&tauri_config_path, &updated).map_err(|e| format!("Failed to write tauri config file: {}", e))?;
    }
    
    Ok(format!("Added proxy provider: {}", name))
}

/// 更新代理订阅
/// 
/// 该函数负责更新配置文件中指定的代理订阅
/// 
/// # 参数
/// * `old_name` - 旧的订阅名称
/// * `new_name` - 新的订阅名称
/// * `url` - 新的订阅URL
/// 
/// # 返回
/// * `Ok(String)` - 更新成功的消息
/// * `Err(String)` - 更新失败的错误信息
pub fn update_proxy_provider(old_name: String, new_name: String, url: String) -> Result<String, String> {
    // 获取配置文件路径
    let config_path = get_config_path()?;
    // 读取配置文件内容
    let content = fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config file: {}", e))?;
    
    // 查找并更新指定的provider
    let mut updated = content.clone();
    let provider_pattern = format!(r#"  {}:
    url: ".*"
    type: http
"#, old_name);
    
    let new_provider = format!(r#"  {}:
    url: "{}"
    type: http
"#, new_name, url);
    
    if updated.contains(&format!("  {}:", old_name)) {
        updated = updated.replace(&provider_pattern, &new_provider);
    } else {
        return Err(format!("Provider {} not found", old_name));
    }
    
    // 写回配置文件
    fs::write(&config_path, &updated).map_err(|e| format!("Failed to write config file: {}", e))?;
    
    // 同时更新src-tauri目录下的配置文件
    let mut tauri_config_path = env::current_dir().map_err(|e| e.to_string())?;
    tauri_config_path.push("src-tauri");
    tauri_config_path.push("configs");
    tauri_config_path.push("mihomo");
    tauri_config_path.push("config.yaml");
    
    if tauri_config_path.exists() {
        fs::write(&tauri_config_path, &updated).map_err(|e| format!("Failed to write tauri config file: {}", e))?;
    }
    
    Ok(format!("Updated proxy provider: {}", new_name))
}

/// 删除代理订阅
/// 
/// 该函数负责从配置文件中删除指定的代理订阅
/// 
/// # 参数
/// * `name` - 订阅名称
/// 
/// # 返回
/// * `Ok(String)` - 删除成功的消息
/// * `Err(String)` - 删除失败的错误信息
pub fn remove_proxy_provider(name: String) -> Result<String, String> {
    // 获取配置文件路径
    let config_path = get_config_path()?;
    // 读取配置文件内容
    let content = fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config file: {}", e))?;
    
    // 查找并删除指定的provider
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip_lines = false;
    let mut _skip_count = 0;
    
    for line in lines {
        if line.trim() == format!("{}:", name) {
            skip_lines = true;
            _skip_count = 0;
        }
        
        if skip_lines {
            _skip_count += 1;
            // 检查是否到达下一个provider或配置项
            if line.trim() != "" && !line.starts_with(' ') {
                skip_lines = false;
                new_lines.push(line);
            }
        } else {
            new_lines.push(line);
        }
    }
    
    // 重新组合内容
    let updated = new_lines.join("\n");
    
    // 写回配置文件
    fs::write(&config_path, &updated).map_err(|e| format!("Failed to write config file: {}", e))?;
    
    // 同时更新src-tauri目录下的配置文件
    let mut tauri_config_path = env::current_dir().map_err(|e| e.to_string())?;
    tauri_config_path.push("src-tauri");
    tauri_config_path.push("configs");
    tauri_config_path.push("mihomo");
    tauri_config_path.push("config.yaml");
    
    if tauri_config_path.exists() {
        fs::write(&tauri_config_path, &updated).map_err(|e| format!("Failed to write tauri config file: {}", e))?;
    }
    
    Ok(format!("Removed proxy provider: {}", name))
}
