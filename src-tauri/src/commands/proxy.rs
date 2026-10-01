use super::api_client::{path_segment, ApiClient, ControllerConfig};
use crate::proxy::{config_file, AppState};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use tauri::{command, State};

fn mode_from_config(config: &Value) -> Result<&str, String> {
    match config["mode"].as_str() {
        Some(mode @ ("rule" | "global" | "direct")) => Ok(mode),
        _ => Err("内核返回了未知的代理模式".into()),
    }
}

#[command]
pub async fn get_proxy_mode() -> Result<String, String> {
    let config: Value = ApiClient::new()?.get_json("/configs").await?;
    Ok(mode_from_config(&config)?.into())
}

#[command]
pub async fn set_proxy_mode(state: State<'_, AppState>, mode: String) -> Result<String, String> {
    if !matches!(mode.as_str(), "rule" | "global" | "direct") {
        return Err("不支持的代理模式".into());
    }
    config_file::update(&state, move |yaml| {
        yaml["mode"] = serde_yaml::Value::String(mode);
        Ok(())
    })
    .await?;
    get_proxy_mode().await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticCheck {
    key: &'static str,
    state: &'static str,
    detail: String,
}

fn check(key: &'static str, state: &'static str, detail: impl Into<String>) -> DiagnosticCheck {
    DiagnosticCheck { key, state, detail: detail.into() }
}

#[command]
pub async fn check_network(state: State<'_, AppState>) -> Result<Vec<DiagnosticCheck>, String> {
    let running = state.is_running()?;
    let mut checks = vec![check("kernel", if running { "success" } else { "failed" },
        if running { "Mihomo 内核运行中" } else { "Mihomo 内核未运行" })];
    let config = crate::proxy::paths::get_config_path()
        .and_then(|path| std::fs::read_to_string(path).map_err(|e| e.to_string()))
        .and_then(|text| serde_yaml::from_str::<serde_yaml::Value>(&text).map_err(|e| e.to_string()));
    checks.push(match config {
        Ok(_) => check("config", "success", "配置文件可读取且 YAML 格式有效"),
        Err(_) => check("config", "failed", "配置文件无法读取或 YAML 格式错误"),
    });
    let enabled = crate::proxy::system_proxy::enabled();
    checks.push(match enabled {
        Ok(true) => check("systemProxy", "success", "Windows 系统代理由本应用接管"),
        Ok(false) => check("systemProxy", "notApplicable", "本应用未接管 Windows 系统代理"),
        Err(_) => check("systemProxy", "failed", "无法读取本应用的系统代理状态"),
    });
    if !running {
        checks.extend([
            check("port", "notApplicable", "内核未运行"),
            check("dns", "notApplicable", "内核未运行"),
            check("target", "notApplicable", "内核未运行"),
        ]);
        return Ok(checks);
    }
    let api = ApiClient::new()?;
    let runtime: Value = match api.get_json("/configs").await {
        Ok(runtime) => runtime,
        Err(_) => {
            checks.extend([
                check("port", "untested", "核心控制接口暂不可用"),
                check("dns", "untested", "核心控制接口暂不可用"),
                check("target", "untested", "核心控制接口暂不可用"),
            ]);
            return Ok(checks);
        }
    };
    let port = ["mixed-port", "port"].iter()
        .find_map(|key| runtime[*key].as_u64().filter(|value| *value > 0 && *value <= u16::MAX as u64));
    if let Some(port) = port {
        let reachable = tokio::time::timeout(Duration::from_secs(2),
            tokio::net::TcpStream::connect(("127.0.0.1", port as u16)))
            .await.is_ok_and(|result| result.is_ok());
        checks.push(check("port", if reachable { "success" } else { "failed" },
            format!("本地代理端口 {port} {}", if reachable { "可连接" } else { "无法连接" })));
        if reachable {
            let target = reqwest::Client::builder()
                .proxy(reqwest::Proxy::http(format!("http://127.0.0.1:{port}"))
                    .map_err(|e| e.to_string())?)
                .timeout(Duration::from_secs(6)).build().map_err(|e| e.to_string())?
                .get("https://www.gstatic.com/generate_204").send().await;
            checks.push(match target {
                Ok(response) if response.status().is_success() => check("target", "success", "经本地代理访问测试目标成功"),
                Ok(response) => check("target", "failed", format!("测试目标返回 HTTP {}", response.status())),
                Err(_) => check("target", "failed", "经本地代理访问测试目标失败或超时"),
            });
        } else {
            checks.push(check("target", "notApplicable", "本地代理端口不可用"));
        }
    } else {
        checks.push(check("port", "failed", "内核未报告可用的 HTTP 或混合端口"));
        checks.push(check("target", "notApplicable", "无可用的本地代理端口"));
    }
    let dns: Result<Value, _> = api.get_json("/dns/query?name=example.com&type=A").await;
    checks.push(match dns {
        Ok(data) if data["Answer"].as_array().is_some_and(|answers| !answers.is_empty()) =>
            check("dns", "success", "内核 DNS 查询 example.com 成功"),
        Ok(_) => check("dns", "failed", "内核 DNS 未返回 A 记录"),
        Err(_) => check("dns", "failed", "内核 DNS 查询失败或接口不可用"),
    });
    Ok(checks)
}

#[cfg(test)]
mod home_tests {
    use super::*;

    #[test]
    fn accepts_only_known_runtime_modes() {
        for mode in ["rule", "global", "direct"] {
            assert_eq!(mode_from_config(&serde_json::json!({ "mode": mode })).unwrap(), mode);
        }
        assert!(mode_from_config(&serde_json::json!({ "mode": "unknown" })).is_err());
    }
}

#[command]
pub async fn start_core(state: State<'_, AppState>) -> Result<(), String> {
    state.start_core().await
}
#[command]
pub async fn stop_core(state: State<'_, AppState>) -> Result<(), String> {
    state.stop_core().await
}
#[command]
pub async fn start_proxy(state: State<'_, AppState>) -> Result<(), String> {
    state.start_proxy().await
}
#[command]
pub async fn stop_proxy(state: State<'_, AppState>) -> Result<(), String> {
    state.stop_proxy().await
}
#[command]
pub async fn get_proxy_status(
    state: State<'_, AppState>,
) -> Result<crate::proxy::ProxyStatus, String> {
    state.proxy_status().await
}
#[command]
pub async fn is_proxy_running(state: State<'_, AppState>) -> Result<bool, String> {
    let _operation = state.operation.lock().await;
    state.is_running()
}
#[command]
pub async fn get_controller_config(state: State<'_, AppState>) -> Result<ControllerConfig, String> {
    let _operation = state.operation.lock().await;
    ControllerConfig::load()
}
#[command]
pub async fn get_proxies() -> Result<Value, String> {
    ApiClient::new()?.get_json("/proxies").await
}
#[command]
pub async fn change_proxy(group: String, proxy: String) -> Result<(), String> {
    ApiClient::new()?
        .put_json(
            &format!("/proxies/{}", path_segment(&group)),
            &serde_json::json!({"name": proxy}),
        )
        .await
}
#[command]
pub async fn test_proxy(proxy: String) -> Result<u64, String> {
    ApiClient::new()?
        .test_delay(&proxy, "https://www.gstatic.com/generate_204")
        .await
}
#[command]
pub async fn get_providers() -> Result<Value, String> {
    ApiClient::new()?.get_json("/providers/proxies").await
}
#[command]
pub async fn get_provider_proxies(provider_name: String) -> Result<Value, String> {
    ApiClient::new()?
        .get_json(&format!(
            "/providers/proxies/{}",
            path_segment(&provider_name)
        ))
        .await
}
#[command]
pub async fn trigger_provider_health_check(provider_name: String) -> Result<(), String> {
    ApiClient::new()?
        .get_text(&format!(
            "/providers/proxies/{}/healthcheck",
            path_segment(&provider_name)
        ))
        .await?;
    Ok(())
}
#[command]
pub async fn get_rules() -> Result<Value, String> {
    ApiClient::new()?.get_json("/rules").await
}
#[command]
pub async fn set_rules(state: State<'_, AppState>, rules: Vec<String>) -> Result<(), String> {
    config_file::update(&state, |yaml| {
        yaml["rules"] = serde_yaml::to_value(rules).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}
#[command]
pub async fn get_connections() -> Result<Value, String> {
    ApiClient::new()?.get_json("/connections").await
}
#[command]
pub async fn close_connection(id: String) -> Result<(), String> {
    ApiClient::new()?
        .delete(&format!("/connections/{}", path_segment(&id)))
        .await
}
#[command]
pub async fn close_all_connections() -> Result<(), String> {
    ApiClient::new()?.delete("/connections").await
}
#[command]
pub async fn get_logs(level: Option<String>) -> Result<String, String> {
    ApiClient::new()?
        .get_text(&format!(
            "/logs?level={}",
            path_segment(&level.unwrap_or_else(|| "info".into()))
        ))
        .await
}
#[command]
pub fn get_uptime(state: State<AppState>) -> Result<u64, String> {
    state.is_running()?;
    Ok(state.get_uptime_secs())
}
#[command]
pub async fn toggle_tun(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let _operation = state.operation.lock().await;
    ApiClient::new()?
        .patch_json("/configs", &serde_json::json!({"tun": {"enable": enabled}}))
        .await
}
#[command]
pub async fn get_tun_status() -> Result<bool, String> {
    let data: Value = ApiClient::new()?.get_json("/configs").await?;
    Ok(data["tun"]["enable"]
        .as_bool()
        .or_else(|| data["tun"].as_bool())
        .unwrap_or(false))
}
