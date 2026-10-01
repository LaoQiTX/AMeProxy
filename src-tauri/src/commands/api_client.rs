//! Shared, bounded, authenticated access to the local mihomo controller.
use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Serialize};
use std::{net::SocketAddr, sync::OnceLock, time::Duration};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerConfig {
    pub ws_url: String,
    pub secret: String,
    #[serde(skip)]
    pub address: SocketAddr,
}

impl ControllerConfig {
    pub fn from_yaml(yaml: &serde_yaml::Value) -> Result<Self, String> {
        let address = yaml["external-controller"]
            .as_str()
            .unwrap_or("127.0.0.1:9090")
            .parse::<SocketAddr>()
            .map_err(|_| "控制接口必须使用回环 IP 和端口".to_string())?;
        if !address.ip().is_loopback() || address.port() == 0 {
            return Err("控制接口必须绑定有效的回环地址，例如 127.0.0.1:9090".into());
        }
        Ok(Self {
            ws_url: format!("ws://{}", address),
            secret: yaml["secret"].as_str().unwrap_or_default().to_owned(),
            address,
        })
    }

    pub fn load() -> Result<Self, String> {
        let content = std::fs::read_to_string(crate::proxy::paths::get_config_path()?)
            .map_err(|e| e.to_string())?;
        Self::from_yaml(&serde_yaml::from_str(&content).map_err(|e| e.to_string())?)
    }
}

pub struct ApiClient {
    client: Client,
    pub config: ControllerConfig,
}

impl ApiClient {
    pub fn new() -> Result<Self, String> {
        Self::with_config(ControllerConfig::load()?)
    }

    pub fn with_config(config: ControllerConfig) -> Result<Self, String> {
        static CLIENT: OnceLock<Client> = OnceLock::new();
        let client = match CLIENT.get() {
            Some(client) => client.clone(),
            None => {
                let client = Client::builder()
                    .no_proxy()
                    .connect_timeout(Duration::from_secs(2))
                    .timeout(Duration::from_secs(15))
                    .build()
                    .map_err(|e| e.to_string())?;
                let _ = CLIENT.set(client.clone());
                client
            }
        };
        Ok(Self { client, config })
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<reqwest::Response, String> {
        let mut request = self
            .client
            .request(method, format!("http://{}{}", self.config.address, path));
        if !self.config.secret.is_empty() {
            request = request.bearer_auth(&self.config.secret);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .await
            .map_err(|e| format!("内核请求失败: {}", e.without_url()))?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("内核接口错误 ({}): {}", status, body));
        }
        Ok(response)
    }

    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        self.request(Method::GET, path, None)
            .await?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_text(&self, path: &str) -> Result<String, String> {
        self.request(Method::GET, path, None)
            .await?
            .text()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn put_json(&self, path: &str, body: &serde_json::Value) -> Result<(), String> {
        self.request(Method::PUT, path, Some(body)).await?;
        Ok(())
    }

    pub async fn patch_json(&self, path: &str, body: &serde_json::Value) -> Result<(), String> {
        self.request(Method::PATCH, path, Some(body)).await?;
        Ok(())
    }

    pub async fn test_delay(&self, proxy: &str, url: &str) -> Result<u64, String> {
        let path = format!(
            "/proxies/{}/delay?timeout=5000&url={}",
            path_segment(proxy),
            path_segment(url)
        );
        let data: serde_json::Value = self.get_json(&path).await?;
        data["delay"]
            .as_u64()
            .ok_or_else(|| "内核未返回有效延迟".into())
    }

    pub async fn delete(&self, path: &str) -> Result<(), String> {
        self.request(Method::DELETE, path, None).await?;
        Ok(())
    }
}

pub fn path_segment(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encodes_reserved_and_unicode_names() {
        assert_eq!(path_segment("日本/%?#"), "%E6%97%A5%E6%9C%AC%2F%25%3F%23");
    }
    #[test]
    fn controller_requires_loopback_and_preserves_secret() {
        let yaml = serde_yaml::from_str("external-controller: 127.0.0.1:19191\nsecret: test-token")
            .unwrap();
        let config = ControllerConfig::from_yaml(&yaml).unwrap();
        assert_eq!(config.ws_url, "ws://127.0.0.1:19191");
        assert_eq!(config.secret, "test-token");
        assert!(ControllerConfig::from_yaml(
            &serde_yaml::from_str("external-controller: 0.0.0.0:9090").unwrap()
        )
        .is_err());
    }
}
