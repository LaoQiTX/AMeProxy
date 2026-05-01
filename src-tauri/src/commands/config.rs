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
use crate::proxy::paths;
use serde_yaml::Value as YamlValue;

/// 保存订阅内容到配置文件
#[command]
pub fn save_subscription(content: String) -> Result<String, String> {
    let config_path = paths::get_config_path()?;

    let mut final_content = content;
    if !final_content.contains("external-controller:") {
        final_content.push_str("\nexternal-controller: 127.0.0.1:9090\n");
    }

    fs::write(&config_path, final_content)
        .map_err(|e| format!("Failed to save config: {}", e))?;
    Ok("Subscription saved".into())
}

/// 更新指定代理提供者的URL
#[command]
pub fn set_proxy_provider_url(provider: String, url: String) -> Result<String, String> {
    let config_path = paths::get_config_path()?;
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    let mut yaml: YamlValue =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    let providers = yaml
        .get_mut("proxy-providers")
        .and_then(|v| v.as_mapping_mut())
        .ok_or_else(|| "proxy-providers section not found".to_string())?;

    let provider_cfg = providers
        .get_mut(&YamlValue::String(provider.clone()))
        .and_then(|v| v.as_mapping_mut())
        .ok_or_else(|| format!("Provider '{}' not found", provider))?;

    provider_cfg.insert(YamlValue::String("url".into()), YamlValue::String(url));

    let file =
        fs::File::create(&config_path).map_err(|e| format!("Failed to write config: {}", e))?;
    serde_yaml::to_writer(file, &yaml)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider url updated".into())
}

/// 构建提供者配置 Mapping
fn build_provider_mapping(name: &str, url: &str) -> serde_yaml::Mapping {
    let mut provider = serde_yaml::Mapping::new();
    provider.insert(
        YamlValue::String("url".into()),
        YamlValue::String(url.into()),
    );
    provider.insert(
        YamlValue::String("type".into()),
        YamlValue::String("http".into()),
    );
    provider.insert(
        YamlValue::String("interval".into()),
        YamlValue::Number(86400.into()),
    );

    let mut health_check = serde_yaml::Mapping::new();
    health_check.insert(
        YamlValue::String("enable".into()),
        YamlValue::Bool(true),
    );
    health_check.insert(
        YamlValue::String("url".into()),
        YamlValue::String("https://www.gstatic.com/generate_204".into()),
    );
    health_check.insert(
        YamlValue::String("interval".into()),
        YamlValue::Number(300.into()),
    );
    provider.insert(
        YamlValue::String("health-check".into()),
        YamlValue::Mapping(health_check),
    );

    let mut override_cfg = serde_yaml::Mapping::new();
    override_cfg.insert(
        YamlValue::String("additional-prefix".into()),
        YamlValue::String(format!("[{}]", name)),
    );
    provider.insert(
        YamlValue::String("override".into()),
        YamlValue::Mapping(override_cfg),
    );

    provider
}

/// 添加新的代理提供者
#[command]
pub fn add_proxy_provider(name: String, url: String) -> Result<String, String> {
    let config_path = paths::get_config_path()?;
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    let mut yaml: YamlValue =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    let providers = yaml
        .get_mut("proxy-providers")
        .and_then(|v| v.as_mapping_mut())
        .ok_or_else(|| "proxy-providers section not found".to_string())?;

    let provider = build_provider_mapping(&name, &url);
    providers.insert(YamlValue::String(name), YamlValue::Mapping(provider));

    let file =
        fs::File::create(&config_path).map_err(|e| format!("Failed to write config: {}", e))?;
    serde_yaml::to_writer(file, &yaml)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider added".into())
}

/// 更新代理提供者的名称和URL
#[command]
pub fn update_proxy_provider(old_name: String, new_name: String, url: String) -> Result<String, String> {
    let config_path = paths::get_config_path()?;
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    let mut yaml: YamlValue =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    let providers = yaml
        .get_mut("proxy-providers")
        .and_then(|v| v.as_mapping_mut())
        .ok_or_else(|| "proxy-providers section not found".to_string())?;

    let old_key = YamlValue::String(old_name.clone());
    let provider_val = providers
        .remove(&old_key)
        .ok_or_else(|| format!("Provider '{}' not found", old_name))?;

    let mut provider = provider_val
        .as_mapping()
        .cloned()
        .unwrap_or_default();

    // 更新 URL
    provider.insert(YamlValue::String("url".into()), YamlValue::String(url));

    // 更新 additional-prefix
    if let Some(override_cfg) = provider.get_mut(&YamlValue::String("override".into())) {
        if let Some(override_map) = override_cfg.as_mapping_mut() {
            override_map.insert(
                YamlValue::String("additional-prefix".into()),
                YamlValue::String(format!("[{}]", new_name)),
            );
        }
    }

    providers.insert(YamlValue::String(new_name), YamlValue::Mapping(provider));

    let file =
        fs::File::create(&config_path).map_err(|e| format!("Failed to write config: {}", e))?;
    serde_yaml::to_writer(file, &yaml)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider updated".into())
}

/// 删除代理提供者
#[command]
pub fn remove_proxy_provider(name: String) -> Result<String, String> {
    let config_path = paths::get_config_path()?;
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    let mut yaml: YamlValue =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    let providers = yaml
        .get_mut("proxy-providers")
        .and_then(|v| v.as_mapping_mut())
        .ok_or_else(|| "proxy-providers section not found".to_string())?;

    let key = YamlValue::String(name.clone());
    if providers.remove(&key).is_none() {
        return Err(format!("Provider '{}' not found", name));
    }

    let file =
        fs::File::create(&config_path).map_err(|e| format!("Failed to write config: {}", e))?;
    serde_yaml::to_writer(file, &yaml)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider removed".into())
}

/// 将 serde_yaml::Value 递归转换为 serde_json::Value
fn convert_yaml_to_json(v: &YamlValue) -> serde_json::Value {
    match v {
        YamlValue::Null => serde_json::Value::Null,
        YamlValue::Bool(b) => serde_json::Value::Bool(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map_or(serde_json::Value::Null, serde_json::Value::Number)
            } else {
                serde_json::Value::Null
            }
        }
        YamlValue::String(s) => serde_json::Value::String(s.clone()),
        YamlValue::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(convert_yaml_to_json).collect())
        }
        YamlValue::Mapping(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    YamlValue::String(s) => s.clone(),
                    other => format!("{:?}", other),
                };
                json_map.insert(key, convert_yaml_to_json(v));
            }
            serde_json::Value::Object(json_map)
        }
        _ => serde_json::Value::Null,
    }
}

/// 获取配置文件内容（proxy-providers 部分）
#[command]
pub fn get_config() -> Result<serde_json::Value, String> {
    let config_path = paths::get_config_path()?;
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    let yaml: YamlValue =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    // 提取 proxy-providers 部分并转换为 serde_json::Value
    let providers = yaml
        .get("proxy-providers")
        .map(convert_yaml_to_json)
        .unwrap_or(serde_json::Value::Null);

    let mut result = serde_json::Map::new();
    result.insert("proxy-providers".to_string(), providers);

    Ok(serde_json::Value::Object(result))
}
