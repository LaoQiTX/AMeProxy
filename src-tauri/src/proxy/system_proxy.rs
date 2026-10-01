//! A recoverable lease on the current user's Windows LAN proxy settings.
//! The journal is written before changing Windows; never overwrite another app's edits.
use super::{config_file::atomic_write, paths};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Settings {
    flags: u32,
    server: String,
    bypass: String,
    pac: String,
}

#[derive(Serialize, Deserialize)]
struct Lease {
    original: Settings,
    applied: Settings,
}

trait Backend {
    fn read(&self) -> Result<Settings, String>;
    fn write(&self, settings: &Settings) -> Result<(), String>;
}

struct Manager<B> {
    backend: B,
    journal: PathBuf,
}

impl<B: Backend> Manager<B> {
    fn load(&self) -> Result<Option<Lease>, String> {
        match fs::read_to_string(&self.journal) {
            Ok(data) => serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| format!("系统代理恢复记录损坏: {e}")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("无法读取系统代理恢复记录: {e}")),
        }
    }

    fn enabled(&self) -> Result<bool, String> {
        match self.load()? {
            Some(lease) => Ok(self.backend.read()? == lease.applied),
            None => Ok(false),
        }
    }

    fn restore(&self) -> Result<(), String> {
        let Some(lease) = self.load()? else {
            return Ok(());
        };
        let current = self.backend.read()?;
        if current == lease.applied {
            self.backend.write(&lease.original)?;
            if self.backend.read()? != lease.original {
                return Err("系统代理恢复未生效，请重试关闭代理".into());
            }
        }
        // If changed externally, relinquish ownership without clobbering the new settings.
        fs::remove_file(&self.journal).map_err(|e| format!("无法清理系统代理恢复记录: {e}"))
    }

    fn enable(&self, port: u16) -> Result<(), String> {
        if port == 0 {
            return Err("没有可用的 HTTP 或混合代理端口".into());
        }
        if self.enabled()? {
            return Ok(());
        }
        self.restore()?;
        let lease = Lease {
            original: self.backend.read()?,
            applied: Settings {
                flags: 3, // PROXY_TYPE_DIRECT | PROXY_TYPE_PROXY; temporarily disable PAC/WPAD.
                server: format!("http=127.0.0.1:{port};https=127.0.0.1:{port}"),
                bypass: "localhost;127.*;[::1];<local>".into(),
                pac: String::new(),
            },
        };
        let parent = self.journal.parent().ok_or("无效的系统代理恢复路径")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        atomic_write(
            &self.journal,
            &serde_json::to_string(&lease).map_err(|e| e.to_string())?,
        )?;
        let result = self.backend.write(&lease.applied).and_then(|_| {
            if self.backend.read()? == lease.applied {
                Ok(())
            } else {
                Err("Windows 未接受系统代理设置".into())
            }
        });
        if let Err(error) = result {
            return match self.restore() {
                Ok(()) => Err(format!("开启系统代理失败，已恢复原设置: {error}")),
                Err(restore) => Err(format!(
                    "开启系统代理失败: {error}；恢复失败: {restore}。请重试关闭代理"
                )),
            };
        }
        Ok(())
    }
}

fn manager() -> Result<Manager<Native>, String> {
    Ok(Manager {
        backend: Native,
        journal: paths::get_config_dir()?.join("system-proxy-recovery.json"),
    })
}
pub fn enable(port: u16) -> Result<(), String> {
    manager()?.enable(port)
}
pub fn restore() -> Result<(), String> {
    manager()?.restore()
}
pub fn enabled() -> Result<bool, String> {
    manager()?.enabled()
}
pub fn recovery_pending() -> Result<bool, String> {
    Ok(manager()?.journal.try_exists().map_err(|e| e.to_string())?)
}

struct Native;
#[cfg(not(windows))]
impl Backend for Native {
    fn read(&self) -> Result<Settings, String> {
        Err("自动系统代理目前仅支持 Windows".into())
    }
    fn write(&self, _: &Settings) -> Result<(), String> {
        Err("自动系统代理目前仅支持 Windows".into())
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::{
        mem::size_of,
        os::windows::io::{FromRawHandle, OwnedHandle},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{GetLastError, GlobalFree, ERROR_ALREADY_EXISTS},
        Networking::WinInet::*,
        System::Threading::CreateMutexW,
    };

    fn options() -> [INTERNET_PER_CONN_OPTIONW; 4] {
        [
            INTERNET_PER_CONN_FLAGS_UI,
            INTERNET_PER_CONN_PROXY_SERVER,
            INTERNET_PER_CONN_PROXY_BYPASS,
            INTERNET_PER_CONN_AUTOCONFIG_URL,
        ]
        .map(|dw_option| INTERNET_PER_CONN_OPTIONW {
            dwOption: dw_option,
            ..Default::default()
        })
    }
    fn list(options: &mut [INTERNET_PER_CONN_OPTIONW]) -> INTERNET_PER_CONN_OPTION_LISTW {
        INTERNET_PER_CONN_OPTION_LISTW {
            dwSize: size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
            dwOptionCount: options.len() as u32,
            pOptions: options.as_mut_ptr(),
            ..Default::default() // null connection selects LAN/current user's system proxy.
        }
    }
    fn error() -> String {
        format!(
            "Windows 系统代理操作失败: {}",
            std::io::Error::last_os_error()
        )
    }
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }

    impl Backend for Native {
        fn read(&self) -> Result<Settings, String> {
            let mut options = options();
            let mut list = list(&mut options);
            let mut size = list.dwSize;
            // WinINet owns the returned NUL-terminated strings. Copy then GlobalFree each one.
            unsafe {
                let success = InternetQueryOptionW(
                    ptr::null(),
                    INTERNET_OPTION_PER_CONNECTION_OPTION,
                    &mut list as *mut _ as *mut _,
                    &mut size,
                );
                let failure = if success == 0 { Some(error()) } else { None };
                let mut strings = Vec::new();
                for option in &options[1..] {
                    let value = option.Value.pszValue;
                    let text = if value.is_null() {
                        String::new()
                    } else {
                        let mut len = 0;
                        while *value.add(len) != 0 {
                            len += 1;
                        }
                        let text = String::from_utf16_lossy(std::slice::from_raw_parts(value, len));
                        GlobalFree(value as *mut _);
                        text
                    };
                    strings.push(text);
                }
                if let Some(error) = failure {
                    return Err(error);
                }
                Ok(Settings {
                    flags: options[0].Value.dwValue,
                    server: strings[0].clone(),
                    bypass: strings[1].clone(),
                    pac: strings[2].clone(),
                })
            }
        }
        fn write(&self, settings: &Settings) -> Result<(), String> {
            let mut strings = [
                wide(&settings.server),
                wide(&settings.bypass),
                wide(&settings.pac),
            ];
            let mut options = options();
            options[0].dwOption = INTERNET_PER_CONN_FLAGS;
            options[0].Value.dwValue = settings.flags;
            for (option, value) in options[1..].iter_mut().zip(strings.iter_mut()) {
                option.Value.pszValue = value.as_mut_ptr();
            }
            let list = list(&mut options);
            // Owned buffers and the list live through these synchronous calls.
            unsafe {
                if InternetSetOptionW(
                    ptr::null(),
                    INTERNET_OPTION_PER_CONNECTION_OPTION,
                    &list as *const _ as *const _,
                    list.dwSize,
                ) == 0
                {
                    return Err(error());
                }
                for option in [INTERNET_OPTION_SETTINGS_CHANGED, INTERNET_OPTION_REFRESH] {
                    if InternetSetOptionW(ptr::null(), option, ptr::null(), 0) == 0 {
                        return Err(error());
                    }
                }
            }
            Ok(())
        }
    }

    /// Keep one desktop instance in this logon session so recovery cannot undo a live lease.
    pub struct InstanceGuard {
        _handle: OwnedHandle,
    }
    impl InstanceGuard {
        pub fn acquire() -> Result<Self, String> {
            let name = wide("Local\\AMeProxy.SystemProxy.Owner");
            unsafe {
                let raw = CreateMutexW(ptr::null(), 0, name.as_ptr());
                if raw.is_null() {
                    return Err(error());
                }
                let exists = GetLastError() == ERROR_ALREADY_EXISTS;
                let handle = OwnedHandle::from_raw_handle(raw);
                if exists {
                    return Err("AMeProxy 已在运行，请使用已打开的窗口".into());
                }
                Ok(Self { _handle: handle })
            }
        }
    }
}
#[cfg(windows)]
pub use windows::InstanceGuard;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    #[cfg(windows)]
    #[test]
    fn native_snapshot_is_read_only_and_has_valid_flags() {
        let snapshot = Native
            .read()
            .expect("WinINet should expose current-user LAN proxy settings");
        assert_eq!(snapshot.flags & !15, 0);
    }
    struct Fake {
        settings: RefCell<Settings>,
        fail_next: Cell<bool>,
        fail_all: Cell<bool>,
    }
    impl Backend for Fake {
        fn read(&self) -> Result<Settings, String> {
            Ok(self.settings.borrow().clone())
        }
        fn write(&self, value: &Settings) -> Result<(), String> {
            if self.fail_all.get() {
                return Err("blocked".into());
            }
            *self.settings.borrow_mut() = value.clone();
            if self.fail_next.replace(false) {
                Err("notification failed".into())
            } else {
                Ok(())
            }
        }
    }
    fn fixture(name: &str) -> Manager<Fake> {
        Manager {
            backend: Fake {
                settings: RefCell::new(Settings {
                    flags: 13,
                    server: "old:8080".into(),
                    bypass: "*.internal".into(),
                    pac: "https://example.invalid/proxy.pac".into(),
                }),
                fail_next: Cell::new(false),
                fail_all: Cell::new(false),
            },
            journal: std::env::temp_dir()
                .join(format!("ameproxy-lease-{}-{name}", std::process::id()))
                .join("recovery.json"),
        }
    }
    #[test]
    fn restores_pac_bypass_and_autodetect_after_restart() {
        let manager = fixture("restore");
        let original = manager.backend.read().unwrap();
        manager.enable(17890).unwrap();
        manager.enable(17890).unwrap(); // double-enable must not overwrite the original snapshot
        assert!(manager.enabled().unwrap());
        let restarted = Manager {
            backend: manager.backend,
            journal: manager.journal,
        };
        restarted.restore().unwrap();
        assert_eq!(restarted.backend.read().unwrap(), original);
        assert!(!restarted.journal.exists());
    }
    #[test]
    fn external_changes_are_preserved() {
        let manager = fixture("external");
        manager.enable(17890).unwrap();
        manager.backend.settings.borrow_mut().server = "other:7890".into();
        assert!(!manager.enabled().unwrap());
        manager.restore().unwrap();
        assert_eq!(manager.backend.read().unwrap().server, "other:7890");
    }
    #[test]
    fn failed_enable_rolls_back_and_failed_restore_keeps_journal() {
        let manager = fixture("failure");
        let original = manager.backend.read().unwrap();
        manager.backend.fail_next.set(true);
        assert!(manager.enable(17890).is_err());
        assert_eq!(manager.backend.read().unwrap(), original);
        manager.enable(17890).unwrap();
        manager.backend.fail_all.set(true);
        assert!(manager.restore().is_err());
        assert!(manager.journal.exists());
        manager.backend.fail_all.set(false);
        manager.restore().unwrap();
    }
    #[test]
    fn invalid_port_and_corrupt_journal_do_not_change_settings() {
        let manager = fixture("invalid");
        let original = manager.backend.read().unwrap();
        assert!(manager.enable(0).is_err());
        fs::create_dir_all(manager.journal.parent().unwrap()).unwrap();
        fs::write(&manager.journal, "broken").unwrap();
        assert!(manager.enable(7890).is_err());
        assert!(manager.restore().is_err());
        assert_eq!(manager.backend.read().unwrap(), original);
        fs::remove_file(&manager.journal).unwrap();
    }
}
