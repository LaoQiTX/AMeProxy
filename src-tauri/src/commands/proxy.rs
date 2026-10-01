use super::api_client::{path_segment, ApiClient, ControllerConfig};
use crate::proxy::{config_file, AppState};
use serde_json::Value;
use tauri::{command, State};

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
