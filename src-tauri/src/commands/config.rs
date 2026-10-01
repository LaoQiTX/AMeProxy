use crate::proxy::{config::ClashConfig, config_file, subscriptions, AppState};
use serde_yaml::{Mapping, Value};
use tauri::{command, State};

fn key(name: &str) -> Value {
    Value::String(name.to_owned())
}

fn validate_provider(name: &str, url: &str) -> Result<(), String> {
    if name.trim().is_empty() || name != name.trim() || name.chars().any(char::is_control) {
        return Err("订阅名称不能为空、包含控制字符或首尾空格".into());
    }
    let url = reqwest::Url::parse(url).map_err(|_| "订阅链接格式不正确")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("订阅链接必须是 HTTP 或 HTTPS 地址".into());
    }
    Ok(())
}

fn providers(yaml: &mut Value) -> Result<&mut Mapping, String> {
    let root = yaml.as_mapping_mut().ok_or("配置必须是 YAML 对象")?;
    root.entry(key("proxy-providers"))
        .or_insert(Value::Mapping(Mapping::new()))
        .as_mapping_mut()
        .ok_or_else(|| "proxy-providers 必须是对象".into())
}

fn build_provider_mapping(name: &str, url: &str) -> serde_yaml::Mapping {
    let mut provider = serde_yaml::Mapping::new();
    provider.insert(Value::String("url".into()), Value::String(url.into()));
    provider.insert(Value::String("type".into()), Value::String("http".into()));
    provider.insert(
        Value::String("interval".into()),
        Value::Number(86400.into()),
    );

    let mut health_check = serde_yaml::Mapping::new();
    health_check.insert(Value::String("enable".into()), Value::Bool(true));
    health_check.insert(
        Value::String("url".into()),
        Value::String("https://www.gstatic.com/generate_204".into()),
    );
    health_check.insert(Value::String("interval".into()), Value::Number(300.into()));
    provider.insert(
        Value::String("health-check".into()),
        Value::Mapping(health_check),
    );

    let mut override_cfg = serde_yaml::Mapping::new();
    override_cfg.insert(
        Value::String("additional-prefix".into()),
        Value::String(format!("[{}]", name)),
    );
    provider.insert(
        Value::String("override".into()),
        Value::Mapping(override_cfg),
    );

    provider
}

fn add_provider(yaml: &mut Value, name: &str, url: &str) -> Result<(), String> {
    validate_provider(name, url)?;
    let active = subscriptions::active_name(yaml);
    let items = providers(yaml)?;
    if items.contains_key(key(name)) {
        return Err(format!("订阅名称已存在: {name}"));
    }
    items.insert(key(name), Value::Mapping(build_provider_mapping(name, url)));
    let groups = yaml
        .get_mut("proxy-groups")
        .and_then(Value::as_sequence_mut)
        .ok_or("缺少策略组")?;
    let index = groups
        .iter()
        .position(|g| g["name"].as_str() == Some("默认"))
        .or_else(|| {
            groups
                .iter()
                .position(|g| g["type"].as_str() == Some("select"))
        })
        .ok_or("未找到可添加订阅的手动策略组")?;
    let group = groups[index].as_mapping_mut().ok_or("策略组格式不正确")?;
    let uses = group
        .entry(key("use"))
        .or_insert(Value::Sequence(vec![]))
        .as_sequence_mut()
        .ok_or("策略组 use 必须是列表")?;
    if !uses.contains(&key(name)) {
        uses.push(key(name));
    }
    subscriptions::persist_selection(yaml, active.as_deref().or(Some(name)));
    Ok(())
}

fn update_provider(yaml: &mut Value, old: &str, new: &str, url: &str) -> Result<(), String> {
    validate_provider(new, url)?;
    let active = subscriptions::active_name(yaml);
    let items = providers(yaml)?;
    if old != new && items.contains_key(key(new)) {
        return Err(format!("订阅名称已存在: {new}"));
    }
    let mut value = items
        .remove(key(old))
        .ok_or_else(|| format!("订阅不存在: {old}"))?;
    let provider = value.as_mapping_mut().ok_or("订阅配置格式不正确")?;
    provider.insert(key("url"), key(url));
    if let Some(overrides) = provider
        .get_mut(key("override"))
        .and_then(Value::as_mapping_mut)
    {
        overrides.insert(key("additional-prefix"), key(&format!("[{new}]")));
    }
    items.insert(key(new), value);
    update_references(yaml, old, Some(new))?;
    subscriptions::persist_selection(
        yaml,
        active
            .as_deref()
            .map(|name| if name == old { new } else { name }),
    );
    Ok(())
}

fn remove_provider(yaml: &mut Value, name: &str) -> Result<(), String> {
    let active = subscriptions::active_name(yaml);
    providers(yaml)?
        .remove(key(name))
        .ok_or_else(|| format!("订阅不存在: {name}"))?;
    update_references(yaml, name, None)?;
    subscriptions::persist_selection(yaml, active.as_deref().filter(|active| *active != name));
    Ok(())
}

fn update_references(yaml: &mut Value, old: &str, new: Option<&str>) -> Result<(), String> {
    if let Some(groups) = yaml
        .get_mut("proxy-groups")
        .and_then(Value::as_sequence_mut)
    {
        for group in groups {
            let group = group.as_mapping_mut().ok_or("策略组格式不正确")?;
            if let Some(uses) = group.get_mut(key("use")) {
                let uses = uses.as_sequence_mut().ok_or("策略组 use 必须是列表")?;
                uses.retain(|v| new.is_some() || v.as_str() != Some(old));
                for value in uses.iter_mut() {
                    if value.as_str() == Some(old) {
                        *value = key(new.unwrap());
                    }
                }
                let mut unique = vec![];
                uses.retain(|v| {
                    if unique.contains(v) {
                        false
                    } else {
                        unique.push(v.clone());
                        true
                    }
                });
                if uses.is_empty() {
                    group.remove(key("use"));
                }
            }
            let has_uses = group
                .get(key("use"))
                .and_then(Value::as_sequence)
                .is_some_and(|v| !v.is_empty());
            let has_proxies = group
                .get(key("proxies"))
                .and_then(Value::as_sequence)
                .is_some_and(|v| !v.is_empty());
            let includes_all = [
                "include-all",
                "include-all-providers",
                "include-all-proxies",
            ]
            .iter()
            .any(|k| group.get(key(k)).and_then(Value::as_bool) == Some(true));
            if !has_uses && !has_proxies && !includes_all {
                group.insert(key("proxies"), Value::Sequence(vec![key("DIRECT")]));
            }
        }
    }
    Ok(())
}

#[command]
pub async fn set_active_subscription(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    config_file::update(&state, |yaml| subscriptions::select(yaml, &name)).await
}

#[command]
pub async fn add_proxy_provider(
    state: State<'_, AppState>,
    name: String,
    url: String,
) -> Result<(), String> {
    config_file::update(&state, |yaml| add_provider(yaml, &name, &url)).await
}

#[command]
pub async fn update_proxy_provider(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
    url: String,
) -> Result<(), String> {
    config_file::update(&state, |yaml| {
        update_provider(yaml, &old_name, &new_name, &url)
    })
    .await
}

#[command]
pub async fn remove_proxy_provider(state: State<'_, AppState>, name: String) -> Result<(), String> {
    config_file::update(&state, |yaml| remove_provider(yaml, &name)).await
}

#[command]
pub async fn set_proxy_provider_url(
    state: State<'_, AppState>,
    provider: String,
    url: String,
) -> Result<(), String> {
    config_file::update(&state, |yaml| {
        update_provider(yaml, &provider, &provider, &url)
    })
    .await
}

#[command]
pub async fn save_subscription(state: State<'_, AppState>, content: String) -> Result<(), String> {
    config_file::update(&state, |yaml| {
        let mut replacement: Value = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
        let root = replacement.as_mapping_mut().ok_or("订阅必须是 YAML 对象")?;
        for field in ["external-controller", "secret"] {
            root.insert(key(field), yaml[field].clone());
        }
        *yaml = replacement;
        Ok(())
    })
    .await
}

#[command]
pub async fn get_config(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let _operation = state.operation.lock().await;
    let path = ClashConfig::generate_file().map_err(|e| e.to_string())?;
    let yaml: Value =
        serde_yaml::from_str(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "proxy-providers": yaml["proxy-providers"],
        "active-subscription": subscriptions::active_name(&yaml),
        "mixed-port": yaml["mixed-port"],
        "allow-lan": yaml["allow-lan"],
    }))
}

#[command]
pub async fn get_rule_config(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let _operation = state.operation.lock().await;
    let path = ClashConfig::generate_file().map_err(|e| e.to_string())?;
    let yaml: Value =
        serde_yaml::from_str(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    serde_yaml::from_value(
        yaml.get("rules")
            .cloned()
            .unwrap_or(Value::Sequence(vec![])),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Value {
        serde_yaml::from_str("proxy-providers:\n  old: {type: http, url: 'https://example.invalid/proxies', interval: 86400}\n  other: {type: file, path: ./other.yaml}\nproxy-groups:\n  - {name: 默认, type: select, use: [old, other]}\n  - {name: fallback, type: fallback, use: [old]}\nrules: [MATCH,默认]").unwrap()
    }
    #[test]
    fn rename_updates_every_reference_and_preserves_custom_fields() {
        let mut yaml = fixture();
        update_provider(&mut yaml, "old", "new", "https://example.invalid/new").unwrap();
        assert!(yaml["proxy-providers"]["old"].is_null());
        assert_eq!(
            yaml["proxy-providers"]["new"]["interval"].as_u64(),
            Some(86400)
        );
        assert_eq!(
            yaml["proxy-groups"][0]["use"],
            serde_yaml::from_str::<Value>("[new, other]").unwrap()
        );
        assert_eq!(yaml["proxy-groups"][1]["use"][0].as_str(), Some("new"));
    }
    #[test]
    fn deleting_last_provider_keeps_group_usable() {
        let mut yaml = fixture();
        remove_provider(&mut yaml, "old").unwrap();
        assert_eq!(
            yaml["proxy-groups"][0]["use"],
            serde_yaml::from_str::<Value>("[other]").unwrap()
        );
        assert!(yaml["proxy-groups"][1]["use"].is_null());
        assert_eq!(
            yaml["proxy-groups"][1]["proxies"][0].as_str(),
            Some("DIRECT")
        );
    }
    #[test]
    fn duplicate_names_and_invalid_urls_do_not_overwrite() {
        let mut yaml = fixture();
        let original = yaml.clone();
        assert!(add_provider(&mut yaml, "old", "https://example.invalid").is_err());
        assert!(update_provider(&mut yaml, "old", "other", "https://example.invalid").is_err());
        assert!(add_provider(&mut yaml, "new", "file:///secret").is_err());
        assert_eq!(yaml, original);
    }

    #[test]
    fn adding_renaming_and_deleting_preserve_one_active_subscription() {
        let mut yaml = fixture();
        add_provider(&mut yaml, "new", "https://example.invalid/new").unwrap();
        assert_eq!(subscriptions::active_name(&yaml).as_deref(), Some("old"));
        let runtime = subscriptions::runtime(&yaml).unwrap();
        assert_eq!(runtime["proxy-providers"].as_mapping().unwrap().len(), 1);
        subscriptions::select(&mut yaml, "new").unwrap();
        update_provider(
            &mut yaml,
            "old",
            "renamed-inactive",
            "https://example.invalid/old",
        )
        .unwrap();
        assert_eq!(subscriptions::active_name(&yaml).as_deref(), Some("new"));
        update_provider(
            &mut yaml,
            "new",
            "renamed-active",
            "https://example.invalid/new",
        )
        .unwrap();
        assert_eq!(
            subscriptions::active_name(&yaml).as_deref(),
            Some("renamed-active")
        );
        remove_provider(&mut yaml, "renamed-inactive").unwrap();
        assert_eq!(
            subscriptions::active_name(&yaml).as_deref(),
            Some("renamed-active")
        );
        remove_provider(&mut yaml, "renamed-active").unwrap();
        assert_eq!(subscriptions::active_name(&yaml).as_deref(), Some("other"));
        remove_provider(&mut yaml, "other").unwrap();
        assert!(subscriptions::active_name(&yaml).is_none());
        add_provider(&mut yaml, "first", "https://example.invalid/first").unwrap();
        assert_eq!(subscriptions::active_name(&yaml).as_deref(), Some("first"));
    }

    #[tokio::test]
    #[ignore = "requires the local mihomo sidecar; uses only isolated files and loopback ports"]
    async fn real_kernel_configuration_and_lifecycle() {
        use crate::{
            commands::api_client::{path_segment, ApiClient},
            proxy::paths,
        };
        use std::{fs, net::TcpListener};
        let dir = std::env::temp_dir().join(format!(
            "ameproxy-integration-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        paths::init_test_paths(
            dir.clone(),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sidecar"),
        );
        let occupied = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = occupied.local_addr().unwrap();
        let mixed_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mixed = mixed_listener.local_addr().unwrap().port();
        drop(mixed_listener);
        let state = AppState::default();
        let yaml = serde_json::json!({
            "mixed-port": mixed, "allow-lan": false,
            "external-controller": address.to_string(), "secret": "integration-token",
            "proxy-providers": { "old": { "type": "file", "path": "./provider.yaml" } },
            "proxy-groups": [{ "name": "默认", "type": "select", "proxies": ["DIRECT"], "use": ["old"] }],
            "rules": ["MATCH,默认"]
        });
        let config_path = dir.join("config.yaml");
        fs::write(&config_path, serde_yaml::to_string(&yaml).unwrap()).unwrap();
        fs::write(
            dir.join("provider.yaml"),
            "proxies:\n  - {name: '日本/%?#', type: direct}\n",
        )
        .unwrap();

        // An occupied controller must fail without claiming the other process.
        assert!(state.start_core().await.unwrap_err().contains("控制端口"));
        assert!(!state.is_running().unwrap());
        drop(occupied);
        let (first, second) = tokio::join!(state.start_core(), state.start_core());
        first.unwrap();
        second.unwrap();
        let api = ApiClient::new().unwrap();
        assert!(
            api.get_json::<serde_json::Value>("/version").await.unwrap()["version"].is_string()
        );
        let unauthenticated = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(format!("http://{address}/version"))
            .send()
            .await
            .unwrap();
        assert_eq!(unauthenticated.status(), 401);

        // Reference updates must pass the real parser and become visible at runtime.
        config_file::update(&state, |v| {
            update_provider(v, "old", "renamed", "https://example.invalid/subscription")
        })
        .await
        .unwrap();
        let providers: serde_json::Value = api.get_json("/providers/proxies").await.unwrap();
        assert!(providers["providers"]["renamed"].is_object());
        assert!(providers["providers"]["old"].is_null());
        api.put_json(
            &format!("/proxies/{}", path_segment("默认")),
            &serde_json::json!({"name":"日本/%?#"}),
        )
        .await
        .unwrap();

        // Exercise forwarding and latency against a local origin, never the public network.
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin_address = origin.local_addr().unwrap();
        let origin_url = format!("http://{origin_address}/check");
        let server = tokio::spawn(async move {
            for index in 0..2 {
                let (mut stream, _) = origin.accept().await.unwrap();
                let mut request = [0u8; 4096];
                stream.read(&mut request).await.unwrap();
                // A sub-millisecond loopback round trip is reported as a zero-delay
                // failure by mihomo; give the test a measurable network interval.
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                if index == 0 {
                    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 18\r\nConnection: close\r\n\r\nintegration-origin").await.unwrap();
                } else {
                    stream
                        .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
                        .await
                        .unwrap();
                }
            }
        });
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", mixed))
            .await
            .unwrap();
        stream
            .write_all(
                format!(
                    "GET {origin_url} HTTP/1.1\r\nHost: {origin_address}\r\nConnection: close\r\n\r\n"
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        let mut response = String::new();
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            stream.read_to_string(&mut response),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(response.ends_with("integration-origin"), "{response}");
        api.test_delay("日本/%?#", &origin_url).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap();

        // Changing rules must persist and affect the running kernel, unlike PATCH.
        config_file::update(&state, |v| {
            v["rules"] =
                serde_yaml::to_value(vec!["DOMAIN,audit.invalid,REJECT", "MATCH,默认"]).unwrap();
            Ok(())
        })
        .await
        .unwrap();
        let rules: serde_json::Value = api.get_json("/rules").await.unwrap();
        assert_eq!(rules["rules"].as_array().unwrap().len(), 2);
        assert_eq!(rules["rules"][0]["payload"], "audit.invalid");
        let saved = fs::read_to_string(&config_path).unwrap();
        assert!(saved.contains("DOMAIN,audit.invalid,REJECT"));

        // Validation failure leaves both the file and live configuration untouched.
        assert!(config_file::update(&state, |v| {
            v["rules"] = serde_yaml::to_value(vec!["INVALID-RULE,value,DIRECT"]).unwrap();
            Ok(())
        })
        .await
        .is_err());
        assert_eq!(fs::read_to_string(&config_path).unwrap(), saved);
        assert_eq!(
            api.get_json::<serde_json::Value>("/rules").await.unwrap()["rules"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        // Saving a second subscription must not load it. Switching replaces runtime
        // providers, disconnects old sessions, and survives a complete kernel restart.
        fs::write(
            dir.join("second.yaml"),
            "proxies:\n  - {name: second-node, type: direct}\n",
        )
        .unwrap();
        config_file::update(&state, |v| {
            add_provider(v, "second", "https://example.invalid/second")?;
            v["proxy-providers"]["second"] =
                serde_yaml::from_str("{type: file, path: ./second.yaml}").unwrap();
            Ok(())
        })
        .await
        .unwrap();
        let providers: serde_json::Value = api.get_json("/providers/proxies").await.unwrap();
        assert!(providers["providers"]["second"].is_null());
        assert!(providers["providers"]["renamed"].is_object());

        let hold_origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let hold_address = hold_origin.local_addr().unwrap();
        let mut held_connection = tokio::net::TcpStream::connect(("127.0.0.1", mixed))
            .await
            .unwrap();
        held_connection
            .write_all(
                format!("GET http://{hold_address}/hold HTTP/1.1\r\nHost: {hold_address}\r\n\r\n")
                    .as_bytes(),
            )
            .await
            .unwrap();
        let (mut held_origin, _) =
            tokio::time::timeout(std::time::Duration::from_secs(5), hold_origin.accept())
                .await
                .unwrap()
                .unwrap();
        let mut request = [0u8; 4096];
        held_origin.read(&mut request).await.unwrap();
        assert!(!api
            .get_json::<serde_json::Value>("/connections")
            .await
            .unwrap()["connections"]
            .as_array()
            .unwrap()
            .is_empty());
        config_file::update(&state, |v| subscriptions::select(v, "second"))
            .await
            .unwrap();
        let mut closed = String::new();
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            held_connection.read_to_string(&mut closed),
        )
        .await
        .unwrap()
        .unwrap();
        let providers: serde_json::Value = api.get_json("/providers/proxies").await.unwrap();
        assert!(providers["providers"]["renamed"].is_null());
        assert!(providers["providers"]["second"].is_object());
        let proxies: serde_json::Value = api.get_json("/proxies").await.unwrap();
        let options = proxies["proxies"]["默认"]["all"].as_array().unwrap();
        assert!(options.iter().any(|name| name == "second-node"));
        assert!(!options.iter().any(|name| name == "日本/%?#"));
        state.stop_core().await.unwrap();
        state.start_core().await.unwrap();
        assert!(api
            .get_json::<serde_json::Value>("/providers/proxies")
            .await
            .unwrap()["providers"]["renamed"]
            .is_null());
        let before_failure = fs::read_to_string(&config_path).unwrap();
        assert!(config_file::update(&state, |v| {
            v["proxy-providers"]["renamed"]["type"] = key("not-a-provider-type");
            subscriptions::select(v, "renamed")
        })
        .await
        .is_err());
        assert_eq!(fs::read_to_string(&config_path).unwrap(), before_failure);
        assert!(api
            .get_json::<serde_json::Value>("/providers/proxies")
            .await
            .unwrap()["providers"]["second"]
            .is_object());
        config_file::update(&state, |v| subscriptions::select(v, "renamed"))
            .await
            .unwrap();
        config_file::update(&state, |v| remove_provider(v, "second"))
            .await
            .unwrap();

        config_file::update(&state, |v| remove_provider(v, "renamed"))
            .await
            .unwrap();
        assert!(api
            .get_json::<serde_json::Value>("/providers/proxies")
            .await
            .unwrap()["providers"]["renamed"]
            .is_null());
        api.delete("/connections/nonexistent").await.unwrap();
        api.delete("/connections").await.unwrap();
        state.stop_core().await.unwrap();
        assert!(!state.is_running().unwrap());
        assert_eq!(state.get_uptime_secs(), 0);
        state.start_core().await.unwrap();
        assert_eq!(
            api.get_json::<serde_json::Value>("/rules").await.unwrap()["rules"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        state.stop_core().await.unwrap();
        // Remove only known test-owned files after shutdown (no recursive deletion).
        for file in fs::read_dir(&dir).unwrap() {
            let file = file.unwrap().path();
            if file.is_file() {
                fs::remove_file(file).unwrap();
            }
        }
        let _ = fs::remove_dir(dir);
    }
}
