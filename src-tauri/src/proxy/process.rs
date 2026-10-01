//! Serializes lifecycle and configuration operations on the owned kernel.
use crate::{
    commands::api_client::ApiClient,
    proxy::{config::ClashConfig, config_file, paths, subscriptions, system_proxy},
};
use std::{
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct AppState {
    pub operation: tokio::sync::Mutex<()>,
    proxy_process: Mutex<Option<OwnedKernel>>,
    start_time: Mutex<Option<Instant>>,
    pub closing: AtomicBool,
    recovered: AtomicBool,
    pub lifecycle_error: Mutex<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyStatus {
    kernel_running: bool,
    system_proxy_enabled: bool,
    recovery_pending: bool,
    error: String,
}

impl AppState {
    pub fn get_uptime_secs(&self) -> u64 {
        self.start_time
            .lock()
            .ok()
            .and_then(|t| *t)
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    pub fn is_running(&self) -> Result<bool, String> {
        let mut process = self.proxy_process.lock().map_err(|e| e.to_string())?;
        let alive = match process.as_mut() {
            Some(child) => child.try_wait().map_err(|e| e.to_string())?.is_none(),
            None => false,
        };
        if !alive {
            *process = None;
            *self.start_time.lock().map_err(|e| e.to_string())? = None;
        }
        Ok(alive)
    }

    pub async fn start_core(&self) -> Result<(), String> {
        let _operation = self.operation.lock().await;
        self.start_core_locked().await
    }

    async fn start_core_locked(&self) -> Result<(), String> {
        if self.closing.load(Ordering::SeqCst) {
            return Err("应用正在退出".into());
        }
        if !self.recovered.load(Ordering::SeqCst) {
            system_proxy::restore()?;
            self.recovered.store(true, Ordering::SeqCst);
        }
        if self.is_running()? {
            return Ok(());
        }
        let config = ClashConfig::generate_file().map_err(|e| e.to_string())?;
        let api = ApiClient::new()?;
        let listener = std::net::TcpListener::bind(api.config.address)
            .map_err(|e| format!("控制端口 {} 不可用: {}", api.config.address, e))?;
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(&std::fs::read_to_string(&config).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let mut proxy_listeners = Vec::new();
        for name in ["mixed-port", "port", "socks-port"] {
            if let Some(port) = yaml[name].as_u64().filter(|p| *p > 0 && *p <= 65535) {
                proxy_listeners.push(
                    std::net::TcpListener::bind(("0.0.0.0", port as u16))
                        .map_err(|e| format!("代理端口 {name}={port} 不可用: {e}"))?,
                );
            }
        }
        drop(proxy_listeners);
        drop(listener);
        let runtime_path = config.parent().unwrap().join("runtime.yaml");
        config_file::atomic_write(&runtime_path, &subscriptions::runtime_text(&yaml)?)?;
        let log_path = config.parent().unwrap().join("kernel.log");
        let log = std::fs::File::create(&log_path).map_err(|e| e.to_string())?;
        let mut command = Command::new(paths::get_kernel_path()?);
        command
            .arg("-d")
            .arg(config.parent().unwrap())
            .arg("-f")
            .arg(&runtime_path)
            .stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
            .stderr(Stdio::from(log));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let child = command
            .spawn()
            .map_err(|e| format!("无法启动内核: {}", e))?;
        *self.proxy_process.lock().map_err(|e| e.to_string())? = Some(OwnedKernel::new(child)?);
        *self.start_time.lock().map_err(|e| e.to_string())? = Some(Instant::now());
        let result = tokio::time::timeout(Duration::from_secs(12), async {
            loop {
                if !self.is_running()? {
                    return Err(format!(
                        "内核启动后退出，请检查配置和端口。日志: {}",
                        log_path.display()
                    ));
                }
                if let Ok(version) = api.get_json::<serde_json::Value>("/version").await {
                    if version["version"].is_string() && self.is_running()? {
                        return Ok(());
                    }
                }
                tokio::time::sleep(Duration::from_millis(150)).await;
            }
        })
        .await
        .unwrap_or_else(|_| Err("内核启动超时，控制接口未就绪".into()));
        if result.is_err() {
            self.stop_owned()?;
        }
        result
    }

    fn stop_owned(&self) -> Result<(), String> {
        let mut process = self.proxy_process.lock().map_err(|e| e.to_string())?;
        if let Some(child) = process.as_mut() {
            if child.try_wait().map_err(|e| e.to_string())?.is_none() {
                child.kill().map_err(|e| format!("停止内核失败: {}", e))?;
            }
            child.wait().map_err(|e| e.to_string())?;
        }
        *process = None;
        *self.start_time.lock().map_err(|e| e.to_string())? = None;
        Ok(())
    }

    pub async fn stop_core(&self) -> Result<(), String> {
        let _operation = self.operation.lock().await;
        // Never leave Windows pointing at a listener we are about to stop.
        system_proxy::restore()?;
        self.stop_owned()
    }

    pub async fn start_proxy(&self) -> Result<(), String> {
        let _operation = self.operation.lock().await;
        self.start_core_locked().await?;
        let config: serde_json::Value = ApiClient::new()?.get_json("/configs").await?;
        let port = ["mixed-port", "port"]
            .into_iter()
            .filter_map(|key| config[key].as_u64())
            .find(|port| *port > 0 && *port <= u16::MAX as u64)
            .ok_or("请在配置中启用 mixed-port 或 HTTP port")? as u16;
        // API readiness alone does not prove the HTTP listener is accepting connections.
        tokio::time::timeout(
            Duration::from_secs(3),
            tokio::net::TcpStream::connect(("127.0.0.1", port)),
        )
        .await
        .map_err(|_| "代理端口连接超时")?
        .map_err(|e| format!("代理端口尚未就绪: {e}"))?;
        if !self.is_running()? {
            return Err("内核已退出，未开启系统代理".into());
        }
        system_proxy::enable(port)?;
        *self.lifecycle_error.lock().map_err(|e| e.to_string())? = String::new();
        Ok(())
    }

    pub async fn stop_proxy(&self) -> Result<(), String> {
        let _operation = self.operation.lock().await;
        system_proxy::restore()?;
        // Closing the proxy also stops an active TUN session and existing connections.
        self.stop_owned()?;
        *self.lifecycle_error.lock().map_err(|e| e.to_string())? = String::new();
        Ok(())
    }

    pub async fn proxy_status(&self) -> Result<ProxyStatus, String> {
        let _operation = self.operation.lock().await;
        Ok(ProxyStatus {
            kernel_running: self.is_running()?,
            system_proxy_enabled: system_proxy::enabled()?,
            recovery_pending: system_proxy::recovery_pending()?,
            error: self
                .lifecycle_error
                .lock()
                .map_err(|e| e.to_string())?
                .clone(),
        })
    }

    pub async fn check_health(&self) -> Result<bool, String> {
        let _operation = self.operation.lock().await;
        if self.closing.load(Ordering::SeqCst) || !self.recovered.load(Ordering::SeqCst) {
            return Ok(false);
        }
        if !self.is_running()? && system_proxy::recovery_pending()? {
            system_proxy::restore()?;
            *self.lifecycle_error.lock().map_err(|e| e.to_string())? =
                "内核意外退出，已恢复原系统代理设置。请检查日志后重新开启。".into();
            return Ok(true);
        }
        Ok(false)
    }
}

/// Reaps the process normally; the Windows job also handles forced app exits.
struct OwnedKernel {
    child: Child,
    #[cfg(windows)]
    _job: std::os::windows::io::OwnedHandle,
}

impl OwnedKernel {
    fn new(mut child: Child) -> Result<Self, String> {
        #[cfg(windows)]
        {
            use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
            use windows_sys::Win32::System::JobObjects::*;
            let attach = || -> Result<OwnedHandle, String> {
                // Null security attributes create a non-inheritable handle.
                let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
                if raw.is_null() {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                // The non-null handle is uniquely owned and closed by OwnedHandle.
                let job = unsafe { OwnedHandle::from_raw_handle(raw) };
                let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION =
                    unsafe { std::mem::zeroed() };
                limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                if unsafe {
                    SetInformationJobObject(
                        job.as_raw_handle(),
                        JobObjectExtendedLimitInformation,
                        &limits as *const _ as *const _,
                        std::mem::size_of_val(&limits) as u32,
                    )
                } == 0
                {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                if unsafe { AssignProcessToJobObject(job.as_raw_handle(), child.as_raw_handle()) }
                    == 0
                {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                Ok(job)
            };
            match attach() {
                Ok(job) => Ok(Self { child, _job: job }),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    Err(format!("无法托管内核进程: {error}"))
                }
            }
        }
        #[cfg(not(windows))]
        Ok(Self { child })
    }
}

impl std::ops::Deref for OwnedKernel {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.child
    }
}
impl std::ops::DerefMut for OwnedKernel {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}
impl Drop for OwnedKernel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
