//! 代理命令模块
//!
//! 该模块包含与代理相关的Tauri命令，包括：
//! 1. 启动/停止代理内核
//! 2. 获取/切换代理
//! 3. 测试延迟
//! 4. 获取连接、规则、日志等

use tauri::{State, command, AppHandle};
use crate::proxy::*;
use serde_json::Value;

/// 获取 API 客户端辅助函数
fn api() -> super::api_client::ApiClient {
    super::api_client::ApiClient::new()
}

/// URL编码辅助函数（用于路径中的代理名称等）
fn url_encode(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('!', "%21")
        .replace('#', "%23")
        .replace('$', "%24")
        .replace('&', "%26")
        .replace('\'', "%27")
        .replace('(', "%28")
        .replace(')', "%29")
        .replace('*', "%2A")
        .replace('+', "%2B")
        .replace(',', "%2C")
        .replace('/', "%2F")
        .replace(':', "%3A")
        .replace(';', "%3B")
        .replace('=', "%3D")
        .replace('?', "%3F")
        .replace('@', "%40")
        .replace('[', "%5B")
        .replace(']', "%5D")
}

/// 启动代理内核
#[command]
pub async fn start_core(state: State<'_, AppState>, app_handle: AppHandle) -> Result<String, String> {
    state
        .start_core(&app_handle)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Core started successfully".into())
}

/// 停止代理内核
#[command]
pub fn stop_core(state: State<AppState>) -> Result<String, String> {
    state
        .stop_core()
        .map_err(|e| e.to_string())?;

    Ok("Core stopped successfully".into())
}

/// 获取代理列表
#[command]
pub async fn get_proxies() -> Result<Value, String> {
    api().get_json("/proxies").await
}

/// 切换代理
#[command]
pub async fn change_proxy(group: String, proxy: String) -> Result<(), String> {
    let body = serde_json::json!({ "name": proxy });
    api().put_json(&format!("/proxies/{}", url_encode(&group)), &body).await
}

/// 测试代理延迟
#[command]
pub async fn test_proxy(proxy: String) -> Result<u64, String> {
    let path = format!(
        "/proxies/{}/delay?timeout=5000&url=http://www.gstatic.com/generate_204",
        url_encode(&proxy)
    );
    let data: Value = api().get_json(&path).await?;

    data["delay"]
        .as_u64()
        .ok_or_else(|| "Invalid delay value".to_string())
}

/// 获取代理提供者列表
#[command]
pub async fn get_providers() -> Result<Value, String> {
    api().get_json("/providers/proxies").await
}

/// 获取规则列表
#[command]
pub async fn get_rules() -> Result<Value, String> {
    api().get_json("/rules").await
}

/// 检查代理是否正在运行
#[command]
pub fn is_proxy_running(state: State<AppState>) -> Result<bool, String> {
    let process = state
        .proxy_process
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(process.is_some())
}

/// 启动代理（兼容前端的命令名称）
#[command]
pub async fn start_proxy(state: State<'_, AppState>, app_handle: AppHandle) -> Result<String, String> {
    start_core(state, app_handle).await
}

/// 停止代理（兼容前端的命令名称）
#[command]
pub fn stop_proxy(state: State<AppState>) -> Result<String, String> {
    stop_core(state)
}

/// 获取实时连接信息
#[command]
pub async fn get_connections() -> Result<Value, String> {
    api().get_json("/connections").await
}

/// 关闭指定连接
#[command]
pub async fn close_connection(id: String) -> Result<(), String> {
    let body = serde_json::json!({ "id": id });
    api().put_json("/connections/close", &body).await
}

/// 获取运行日志
#[command]
pub async fn get_logs(level: Option<String>) -> Result<String, String> {
    let path = match level {
        Some(lvl) => format!("/logs?level={}", url_encode(&lvl)),
        None => "/logs".to_string(),
    };
    api().get_text(&path).await
}

/// 获取运行时长（从进程启动时间计算）
#[command]
pub fn get_uptime(state: State<AppState>) -> Result<u64, String> {
    Ok(state.get_uptime_secs())
}

/// 切换TUN模式
#[command]
pub async fn toggle_tun(enabled: bool) -> Result<(), String> {
    let body = serde_json::json!({ "enabled": enabled });
    api().put_json("/tun", &body).await
}

/// 获取TUN模式状态
#[command]
pub async fn get_tun_status() -> Result<bool, String> {
    let data: Value = api().get_json("/tun").await?;

    data["enabled"]
        .as_bool()
        .ok_or_else(|| "Invalid tun status".to_string())
}
