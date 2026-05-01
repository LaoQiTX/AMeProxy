//! Mihomo API 客户端模块
//!
//! 封装与 Mihomo REST API (127.0.0.1:9090) 的 HTTP 通信。
//! 使用 reqwest 替代原始 TcpStream 手动组装 HTTP 请求。

use reqwest::Client;
use serde::de::DeserializeOwned;

/// Mihomo REST API 基础地址
const MIHOMO_API_BASE: &str = "http://127.0.0.1:9090";

/// API 客户端，封装与 Mihomo 内核的 HTTP 通信
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    /// 创建新的 API 客户端实例
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: MIHOMO_API_BASE.to_string(),
        }
    }

    /// 获取基础 URL
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 发送 GET 请求并返回纯文本响应体
    pub async fn get_text(&self, path: &str) -> Result<String, String> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))
    }

    /// 发送 GET 请求并解析 JSON 响应
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// 发送 PUT 请求，附带 JSON body
    pub async fn put_json(&self, path: &str, body: &serde_json::Value) -> Result<(), String> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .client
            .put(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        Ok(())
    }
}
