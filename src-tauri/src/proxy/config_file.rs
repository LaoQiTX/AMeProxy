//! Validate candidate configurations before replacing the last usable file.
use super::{config::ClashConfig, paths, subscriptions, AppState};
use crate::commands::api_client::{ApiClient, ControllerConfig};
use serde_yaml::Value;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

struct TemporaryFile(PathBuf);
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn temporary_file(target: &Path, content: &str) -> Result<TemporaryFile, String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let name = format!(
        ".candidate-{}-{}-{}.yaml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let temporary = TemporaryFile(target.with_file_name(name));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary.0)
        .map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(temporary)
}

pub fn atomic_write(target: &Path, content: &str) -> Result<(), String> {
    let temporary = temporary_file(target, content)?;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        };
        let from: Vec<u16> = temporary
            .0
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let to: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
        // Both paths are owned, NUL-terminated UTF-16 buffers, valid for this call.
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    #[cfg(not(windows))]
    fs::rename(&temporary.0, target).map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn validate(config_path: &Path, content: &str) -> Result<(), String> {
    let temporary = temporary_file(config_path, content)?;
    let mut command = tokio::process::Command::new(paths::get_kernel_path()?);
    command
        .args(["-t", "-d"])
        .arg(config_path.parent().unwrap())
        .arg("-f")
        .arg(&temporary.0)
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let output = tokio::time::timeout(Duration::from_secs(15), command.output())
        .await
        .map_err(|_| "配置校验超时，原配置保持不变".to_string())?
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "配置校验失败，原配置保持不变: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub async fn update<F>(state: &AppState, change: F) -> Result<(), String>
where
    F: FnOnce(&mut Value) -> Result<(), String> + Send,
{
    let _operation = state.operation.lock().await;
    if state.closing.load(Ordering::SeqCst) {
        return Err("应用正在退出".into());
    }
    let path = ClashConfig::generate_file().map_err(|e| e.to_string())?;
    let old = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut yaml: Value = serde_yaml::from_str(&old).map_err(|e| e.to_string())?;
    let old_runtime = subscriptions::runtime_text(&yaml)?;
    let old_active = subscriptions::active_name(&yaml);
    let controller = ControllerConfig::from_yaml(&yaml)?;
    let network = [
        yaml["mixed-port"].clone(),
        yaml["port"].clone(),
        yaml["bind-address"].clone(),
    ];
    change(&mut yaml)?;
    if super::system_proxy::recovery_pending()?
        && network
            != [
                yaml["mixed-port"].clone(),
                yaml["port"].clone(),
                yaml["bind-address"].clone(),
            ]
    {
        return Err("请先关闭系统代理，再修改监听地址或端口".into());
    }
    let next_controller = ControllerConfig::from_yaml(&yaml)?;
    if controller.address != next_controller.address || controller.secret != next_controller.secret
    {
        return Err("本次编辑不能更改控制接口或认证信息".into());
    }
    let candidate = serde_yaml::to_string(&yaml).map_err(|e| e.to_string())?;
    let runtime = subscriptions::runtime_text(&yaml)?;
    let switching = old_active != subscriptions::active_name(&yaml);
    validate(&path, &runtime).await?;
    let running = state.is_running()?;
    atomic_write(&path.with_extension("yaml.bak"), &old)?;
    atomic_write(&path, &candidate)?;
    if running && runtime != old_runtime {
        let api = ApiClient::with_config(controller)?;
        let applied = async {
            api.put_json(
                "/configs?force=true",
                &serde_json::json!({"payload": runtime}),
            )
            .await?;
            // Existing TCP sessions otherwise retain the old provider after reload.
            if switching {
                api.delete("/connections").await?;
            }
            Ok::<(), String>(())
        }
        .await;
        if let Err(error) = applied {
            // A timeout may happen after a reload. Restore both disk and runtime.
            let disk = atomic_write(&path, &old);
            let runtime = api
                .put_json(
                    "/configs?force=true",
                    &serde_json::json!({"payload": old_runtime}),
                )
                .await;
            if disk.is_err() || runtime.is_err() {
                return Err(format!(
                    "重载失败: {error}；恢复文件: {disk:?}；恢复内核: {runtime:?}。备份: {}",
                    path.with_extension("yaml.bak").display()
                ));
            }
            return Err(format!("重载失败，已恢复原配置: {error}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atomically_replaces_existing_file_and_cleans_temporary_file() {
        let dir = std::env::temp_dir().join(format!("ameproxy-atomic-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.yaml");
        fs::write(&path, "original").unwrap();
        atomic_write(&path, "replacement").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "replacement");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
