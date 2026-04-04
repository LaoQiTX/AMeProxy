//! 代理命令模块
//! 
//! 该模块包含与代理相关的Tauri命令，包括：
//! 1. 启动代理内核
//! 2. 停止代理内核
//! 3. 获取代理列表
//! 4. 切换代理
//! 5. 测试代理延迟

use tauri::{State, command, AppHandle};
use crate::proxy::*;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;

/// Mihomo API 地址
const MIHOMO_API_HOST: &str = "127.0.0.1:9090";

/// 发送 HTTP GET 请求
fn http_get(path: &str) -> Result<String, String> {
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, MIHOMO_API_HOST
    );
    
    let mut stream = TcpStream::connect(MIHOMO_API_HOST)
        .map_err(|e| format!("Failed to connect to Mihomo API: {}", e))?;
    
    stream.write_all(request.as_bytes())
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    // 解析 HTTP 响应
    if let Some(body_start) = response.find("\r\n\r\n") {
        let body = &response[body_start + 4..];
        // 检查响应是否为空
        if body.is_empty() {
            return Err("Empty response body".to_string());
        }
        // 检查响应是否包含错误信息
        if body.contains("error") || body.contains("Error") {
            return Err(format!("API error: {}", body));
        }
        // 清理响应体，去除 BOM 和其他非 JSON 字符
        let cleaned_body = body.trim_matches(|c: char| !c.is_ascii_graphic() && c != ' ' && c != '\t' && c != '\n' && c != '\r');
        Ok(cleaned_body.to_string())
    } else {
        Err("Invalid HTTP response".to_string())
    }
}

/// 发送 HTTP PUT 请求
fn http_put(path: &str, body: &str) -> Result<(), String> {
    let request = format!(
        "PUT {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, MIHOMO_API_HOST, body.len(), body
    );
    
    let mut stream = TcpStream::connect(MIHOMO_API_HOST)
        .map_err(|e| format!("Failed to connect to Mihomo API: {}", e))?;
    
    stream.write_all(request.as_bytes())
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    // 检查响应状态
    if response.contains("200 OK") || response.contains("204 No Content") {
        Ok(())
    } else {
        Err(format!("HTTP request failed: {}", response.lines().next().unwrap_or("Unknown error")))
    }
}

/// URL编码辅助函数
fn encode_url(s: &str) -> String {
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
/// 
/// 该函数负责启动Mihomo代理内核
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程信息
/// * `app_handle` - Tauri应用句柄
/// 
/// # 返回
/// * `Ok(String)` - 启动成功的消息
/// * `Err(String)` - 启动失败的错误信息
#[command]
pub async fn start_core(state: State<'_, AppState>, app_handle: AppHandle) -> Result<String, String> {
    // 启动内核
    state.start_core(&app_handle)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok("Core started successfully".into())
}

/// 停止代理内核
/// 
/// 该函数负责停止Mihomo代理内核
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程信息
/// 
/// # 返回
/// * `Ok(String)` - 停止成功的消息
/// * `Err(String)` - 停止失败的错误信息
#[command]
pub fn stop_core(state: State<AppState>) -> Result<String, String> {
    // 停止内核
    state.stop_core()
        .map_err(|e| e.to_string())?;
    
    Ok("Core stopped successfully".into())
}

/// 获取代理列表
/// 
/// 该函数负责从Mihomo内核获取代理列表
/// 
/// # 返回
/// * `Ok(Value)` - 代理列表
/// * `Err(String)` - 获取失败的错误信息
#[command]
pub async fn get_proxies() -> Result<Value, String> {
    // 调用Mihomo API获取代理列表
    let body = http_get("/proxies")?;
    
    let data: Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(data)
}

/// 切换代理
/// 
/// 该函数负责通过Mihomo API切换代理
/// 
/// # 参数
/// * `group` - 代理组名称
/// * `proxy` - 代理名称
/// 
/// # 返回
/// * `Ok(())` - 切换成功
/// * `Err(String)` - 切换失败的错误信息
#[command]
pub async fn change_proxy(group: String, proxy: String) -> Result<(), String> {
    let body = serde_json::json!({ "name": proxy }).to_string();
    http_put(&format!("/proxies/{}", encode_url(&group)), &body)?;
    Ok(())
}

/// 测试代理延迟
/// 
/// 该函数负责通过Mihomo API测试代理延迟
/// 
/// # 参数
/// * `proxy` - 代理名称
/// 
/// # 返回
/// * `Ok(u64)` - 延迟时间（毫秒）
/// * `Err(String)` - 测试失败的错误信息
#[command]
pub async fn test_proxy(proxy: String) -> Result<u64, String> {
    let path = format!("/proxies/{}/delay?timeout=5000&url=http://www.gstatic.com/generate_204", encode_url(&proxy));
    let body = http_get(&path)?;
    
    let data: Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let delay = data["delay"]
        .as_u64()
        .ok_or("Invalid delay value")?;
    
    Ok(delay)
}

/// 获取代理提供者列表
/// 
/// 该函数负责通过Mihomo API获取代理提供者列表
/// 
/// # 返回
/// * `Ok(Value)` - 代理提供者列表
/// * `Err(String)` - 获取失败的错误信息
#[command]
pub async fn get_providers() -> Result<Value, String> {
    // 调用Mihomo API获取代理提供者列表
    let body = http_get("/providers/proxies")?;
    
    // 尝试解析响应
    match serde_json::from_str(&body) {
        Ok(data) => Ok(data),
        Err(e) => {
            // 打印响应内容，便于调试
            println!("Response body: {}", body);
            Err(format!("Failed to parse response: {}", e))
        }
    }
}

/// 获取规则列表
/// 
/// 该函数负责通过Mihomo API获取规则列表
/// 
/// # 返回
/// * `Ok(Value)` - 规则列表
/// * `Err(String)` - 获取失败的错误信息
#[command]
pub async fn get_rules() -> Result<Value, String> {
    // 调用Mihomo API获取规则列表
    let body = http_get("/rules")?;
    
    let data: Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(data)
}

/// 检查代理是否正在运行
/// 
/// 该函数负责检查Mihomo代理进程是否正在运行
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程信息
/// 
/// # 返回
/// * `Ok(bool)` - 代理是否正在运行
#[command]
pub fn is_proxy_running(state: State<AppState>) -> Result<bool, String> {
    // 获取代理进程的互斥锁
    let process = state.proxy_process.lock().map_err(|e| e.to_string())?;
    
    // 检查进程是否存在
    Ok(process.is_some())
}

/// 启动代理（兼容前端的命令名称）
/// 
/// 该函数是 `start_core` 的别名，用于兼容前端的命令名称
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程信息
/// * `app_handle` - Tauri应用句柄
/// 
/// # 返回
/// * `Ok(String)` - 启动成功的消息
/// * `Err(String)` - 启动失败的错误信息
#[command]
pub async fn start_proxy(state: State<'_, AppState>, app_handle: AppHandle) -> Result<String, String> {
    // 调用 start_core 函数
    start_core(state, app_handle).await
}

/// 停止代理（兼容前端的命令名称）
/// 
/// 该函数是 `stop_core` 的别名，用于兼容前端的命令名称
/// 
/// # 参数
/// * `state` - 应用状态，包含代理进程信息
/// 
/// # 返回
/// * `Ok(String)` - 停止成功的消息
/// * `Err(String)` - 停止失败的错误信息
#[command]
pub fn stop_proxy(state: State<AppState>) -> Result<String, String> {
    // 调用 stop_core 函数
    stop_core(state)
}
