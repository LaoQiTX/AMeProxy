//! 配置命令模块
//! 
//! 该模块包含与配置相关的Tauri命令，包括：
//! 1. 保存订阅
//! 2. 设置代理提供者URL
//! 3. 添加代理提供者
//! 4. 更新代理提供者
//! 5. 删除代理提供者

use tauri::command;
use std::fs;
use std::env;

/// 保存订阅
/// 
/// 该函数负责保存订阅内容到配置文件
/// 
/// # 参数
/// * `content` - 订阅内容
/// 
/// # 返回
/// * `Ok(String)` - 保存成功的消息
/// * `Err(String)` - 保存失败的错误信息
#[command]
pub fn save_subscription(content: String) -> Result<String, String> {
    // 获取当前目录
    let mut current_dir = env::current_dir().map_err(|e| e.to_string())?;
    
    // 检查当前目录是否已经是 src-tauri
    if current_dir.file_name().unwrap_or_default() == "src-tauri" {
        // 如果是 src-tauri 目录，直接使用 sidecar 子目录
        current_dir = current_dir.parent().unwrap_or(&current_dir).to_path_buf();
    }
    
    // 构建配置文件路径
    let mut config_path = current_dir.clone();
    config_path.push("src-tauri");
    config_path.push("sidecar");
    config_path.push("config.yaml");

    // 确保external-controller配置存在，以便我们可以控制代理
    let mut final_content = content;
    if !final_content.contains("external-controller:") {
        final_content.push_str("\nexternal-controller: 127.0.0.1:9090\n");
    }

    // 写入配置文件
    fs::write(config_path, final_content).map_err(|e| format!("Failed to save config: {}", e))?;

    Ok("Subscription saved".into())
}

/// 设置代理提供者URL
/// 
/// 该函数负责更新指定代理提供者的URL
/// 
/// # 参数
/// * `provider` - 代理提供者名称
/// * `url` - 新的URL
/// 
/// # 返回
/// * `Ok(String)` - 更新成功的消息
/// * `Err(String)` - 更新失败的错误信息
#[command]
pub fn set_proxy_provider_url(provider: String, url: String) -> Result<String, String> {
    // 获取当前目录
    let mut current_dir = env::current_dir().map_err(|e| e.to_string())?;
    
    // 检查当前目录是否已经是 src-tauri
    if current_dir.file_name().unwrap_or_default() == "src-tauri" {
        // 如果是 src-tauri 目录，直接使用 sidecar 子目录
        current_dir = current_dir.parent().unwrap_or(&current_dir).to_path_buf();
    }
    
    // 构建配置文件路径
    let mut config_path = current_dir.clone();
    config_path.push("src-tauri");
    config_path.push("sidecar");
    config_path.push("config.yaml");

    // 读取配置文件内容
    let original = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // 将配置文件内容按行分割
    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    let needle = format!("{}:", provider);

    // 查找指定的代理提供者
    let mut provider_index: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == needle {
            provider_index = Some(i);
            break;
        }
    }

    // 确保找到了代理提供者
    let provider_index = provider_index.ok_or_else(|| format!("Provider not found: {}", provider))?;
    let provider_indent = lines[provider_index].chars().take_while(|c| *c == ' ').count();

    // 查找URL配置行
    let mut url_index: Option<usize> = None;
    for i in (provider_index + 1)..lines.len() {
        let line = &lines[i];
        if line.trim().is_empty() {
            continue;
        }

        let indent = line.chars().take_while(|c| *c == ' ').count();
        if indent <= provider_indent && line.trim_end().ends_with(':') {
            break;
        }

        if line.trim_start().starts_with("url:") {
            url_index = Some(i);
            break;
        }
    }

    // 确保找到了URL配置行
    let url_index = url_index.ok_or_else(|| format!("url not found under provider: {}", provider))?;
    let url_indent = lines[url_index].chars().take_while(|c| *c == ' ').count();
    // 转义URL中的特殊字符
    let escaped = url.replace('\\', "\\\\").replace('"', "\\\"");
    // 更新URL配置
    lines[url_index] = format!("{}url: \"{}\"", " ".repeat(url_indent), escaped);

    // 重新组合配置文件内容
    let mut updated = lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    // 写入配置文件
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider url updated".into())
}

/// 添加代理提供者
/// 
/// 该函数负责向配置文件中添加新的代理提供者
/// 
/// # 参数
/// * `name` - 代理提供者名称
/// * `url` - 代理提供者URL
/// 
/// # 返回
/// * `Ok(String)` - 添加成功的消息
/// * `Err(String)` - 添加失败的错误信息
#[command]
pub fn add_proxy_provider(name: String, url: String) -> Result<String, String> {
    // 获取当前目录
    let mut current_dir = env::current_dir().map_err(|e| e.to_string())?;
    
    // 检查当前目录是否已经是 src-tauri
    if current_dir.file_name().unwrap_or_default() == "src-tauri" {
        // 如果是 src-tauri 目录，直接使用 sidecar 子目录
        current_dir = current_dir.parent().unwrap_or(&current_dir).to_path_buf();
    }
    
    // 构建配置文件路径
    let mut config_path = current_dir.clone();
    config_path.push("src-tauri");
    config_path.push("sidecar");
    config_path.push("config.yaml");

    // 读取配置文件内容
    let original = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // 将配置文件内容按行分割
    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    
    // 找到proxy-providers部分的结束位置
    let mut proxy_providers_end: Option<usize> = None;
    let mut in_proxy_providers = false;
    
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "proxy-providers:" {
            in_proxy_providers = true;
            continue;
        }
        
        if in_proxy_providers && !line.trim().is_empty() && !line.starts_with(' ') {
            proxy_providers_end = Some(i);
            break;
        }
        
        if in_proxy_providers && i == lines.len() - 1 {
            proxy_providers_end = Some(lines.len());
            break;
        }
    }
    
    // 确保找到了proxy-providers部分
    let proxy_providers_end = proxy_providers_end.ok_or_else(|| "proxy-providers section not found".to_string())?;
    
    // 生成新的provider配置
    let new_provider = format!("  {}:\n    url: \"{}\"\n    type: http\n    interval: 86400\n    health-check: {{enable: true,url: \"https://www.gstatic.com/generate_204\", interval: 300}}\n    override:\n      additional-prefix: \"[{}]\"\n", name, url.replace('\\', "\\\\").replace('\"', "\\\"") , name);
    
    // 插入新的provider
    lines.insert(proxy_providers_end, new_provider);
    
    // 重新组合配置文件内容
    let mut updated = lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    // 写入配置文件
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider added".into())
}

/// 更新代理提供者
/// 
/// 该函数负责更新指定代理提供者的名称和URL
/// 
/// # 参数
/// * `old_name` - 旧的代理提供者名称
/// * `new_name` - 新的代理提供者名称
/// * `url` - 新的URL
/// 
/// # 返回
/// * `Ok(String)` - 更新成功的消息
/// * `Err(String)` - 更新失败的错误信息
#[command]
pub fn update_proxy_provider(old_name: String, new_name: String, url: String) -> Result<String, String> {
    // 获取当前目录
    let mut current_dir = env::current_dir().map_err(|e| e.to_string())?;
    
    // 检查当前目录是否已经是 src-tauri
    if current_dir.file_name().unwrap_or_default() == "src-tauri" {
        // 如果是 src-tauri 目录，直接使用 sidecar 子目录
        current_dir = current_dir.parent().unwrap_or(&current_dir).to_path_buf();
    }
    
    // 构建配置文件路径
    let mut config_path = current_dir.clone();
    config_path.push("src-tauri");
    config_path.push("sidecar");
    config_path.push("config.yaml");

    // 读取配置文件内容
    let original = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // 将配置文件内容按行分割
    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    let old_needle = format!("{}:", old_name);
    
    // 找到旧provider的位置
    let mut provider_index: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == old_needle {
            provider_index = Some(i);
            break;
        }
    }
    
    // 确保找到了旧provider
    let provider_index = provider_index.ok_or_else(|| format!("Provider not found: {}", old_name))?;
    
    // 更新provider名称
    let indent = lines[provider_index].chars().take_while(|c| *c == ' ').count();
    lines[provider_index] = format!("{}{}:", " ".repeat(indent), new_name);
    
    // 更新URL和prefix
    let provider_indent = indent;
    for i in (provider_index + 1)..lines.len() {
        let line = &lines[i];
        if line.trim().is_empty() {
            continue;
        }
        
        let current_indent = line.chars().take_while(|c| *c == ' ').count();
        if current_indent <= provider_indent && line.trim_end().ends_with(':') {
            break;
        }
        
        if line.trim_start().starts_with("url:") {
            let url_indent = current_indent;
            let escaped = url.replace('\\', "\\\\").replace('"', "\\\"");
            lines[i] = format!("{}url: \"{}\"", " ".repeat(url_indent), escaped);
        } else if line.trim_start().starts_with("additional-prefix:") {
            let prefix_indent = current_indent;
            lines[i] = format!("{}additional-prefix: \"[{}]\"", " ".repeat(prefix_indent), new_name);
        }
    }
    
    // 重新组合配置文件内容
    let mut updated = lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    // 写入配置文件
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider updated".into())
}

/// 删除代理提供者
/// 
/// 该函数负责从配置文件中删除指定的代理提供者
/// 
/// # 参数
/// * `name` - 代理提供者名称
/// 
/// # 返回
/// * `Ok(String)` - 删除成功的消息
/// * `Err(String)` - 删除失败的错误信息
#[command]
pub fn remove_proxy_provider(name: String) -> Result<String, String> {
    // 获取当前目录
    let mut current_dir = env::current_dir().map_err(|e| e.to_string())?;
    
    // 检查当前目录是否已经是 src-tauri
    if current_dir.file_name().unwrap_or_default() == "src-tauri" {
        // 如果是 src-tauri 目录，直接使用 sidecar 子目录
        current_dir = current_dir.parent().unwrap_or(&current_dir).to_path_buf();
    }
    
    // 构建配置文件路径
    let mut config_path = current_dir.clone();
    config_path.push("src-tauri");
    config_path.push("sidecar");
    config_path.push("config.yaml");

    // 读取配置文件内容
    let original = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // 将配置文件内容按行分割
    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    let needle = format!("{}:", name);
    
    // 找到provider的位置
    let mut provider_index: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == needle {
            provider_index = Some(i);
            break;
        }
    }
    
    // 确保找到了provider
    let provider_index = provider_index.ok_or_else(|| format!("Provider not found: {}", name))?;
    let provider_indent = lines[provider_index].chars().take_while(|c| *c == ' ').count();
    
    // 找到provider的结束位置
    let mut provider_end = provider_index + 1;
    while provider_end < lines.len() {
        let line = &lines[provider_end];
        if line.trim().is_empty() {
            provider_end += 1;
            continue;
        }
        
        let indent = line.chars().take_while(|c| *c == ' ').count();
        if indent <= provider_indent && line.trim_end().ends_with(':') {
            break;
        }
        
        provider_end += 1;
    }
    
    // 删除provider相关的行
    lines.drain(provider_index..provider_end);
    
    // 清理可能的空行
    let mut cleaned_lines: Vec<String> = Vec::new();
    let mut in_proxy_providers = false;
    let mut last_was_empty = false;
    
    for line in lines {
        if line.trim() == "proxy-providers:" {
            in_proxy_providers = true;
            cleaned_lines.push(line);
            last_was_empty = false;
        } else if in_proxy_providers {
            if line.trim().is_empty() {
                if !last_was_empty {
                    cleaned_lines.push(line);
                    last_was_empty = true;
                }
            } else {
                cleaned_lines.push(line);
                last_was_empty = false;
            }
        } else {
            let is_empty = line.trim().is_empty();
            cleaned_lines.push(line);
            last_was_empty = is_empty;
        }
    }
    
    // 重新组合配置文件内容
    let mut updated = cleaned_lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    // 写入配置文件
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider removed".into())
}

/// 获取配置文件内容
/// 
/// 该函数负责读取配置文件内容并返回解析后的对象
/// 
/// # 返回
/// * `Ok(serde_json::Value)` - 配置文件内容
/// * `Err(String)` - 读取失败的错误信息
#[command]
pub fn get_config() -> Result<serde_json::Value, String> {
    // 获取当前目录
    let mut current_dir = env::current_dir().map_err(|e| e.to_string())?;
    
    // 检查当前目录是否已经是 src-tauri
    if current_dir.file_name().unwrap_or_default() == "src-tauri" {
        // 如果是 src-tauri 目录，直接使用 sidecar 子目录
        current_dir = current_dir.parent().unwrap_or(&current_dir).to_path_buf();
    }
    
    // 构建配置文件路径
    let mut config_path = current_dir.clone();
    config_path.push("src-tauri");
    config_path.push("sidecar");
    config_path.push("config.yaml");

    // 读取配置文件内容
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // 简单解析 YAML 文件，提取 proxy-providers 部分
    let mut result = serde_json::Map::new();
    let mut proxy_providers = serde_json::Map::new();
    
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut in_proxy_providers = false;
    let mut current_provider = String::new();
    let mut current_provider_data = serde_json::Map::new();
    let mut provider_indent = 0;
    
    for line in lines {
        let trimmed = line.trim();
        
        if trimmed == "proxy-providers:" {
            in_proxy_providers = true;
            continue;
        }
        
        if in_proxy_providers && !trimmed.is_empty() && !line.starts_with(' ') {
            in_proxy_providers = false;
            break;
        }
        
        if in_proxy_providers {
            // 跳过空行
            if trimmed.is_empty() {
                continue;
            }
            
            let indent = line.chars().take_while(|c| *c == ' ').count();
            
            // 检查是否是新的提供者（以冒号结尾）
            if trimmed.ends_with(':') {
                // 新的提供者
                if !current_provider.is_empty() {
                    proxy_providers.insert(current_provider.clone(), serde_json::Value::Object(current_provider_data.clone()));
                }
                current_provider = trimmed.trim_end_matches(':').to_string();
                current_provider_data = serde_json::Map::new();
                provider_indent = indent;
            } else if !current_provider.is_empty() && indent > provider_indent {
                // 提供者的属性
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim().trim_matches('"');
                    current_provider_data.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                }
            }
        }
    }
    
    // 添加最后一个提供者
    if !current_provider.is_empty() {
        proxy_providers.insert(current_provider, serde_json::Value::Object(current_provider_data));
    }
    
    result.insert("proxy-providers".to_string(), serde_json::Value::Object(proxy_providers));

    Ok(serde_json::Value::Object(result))
}
