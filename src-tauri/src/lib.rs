use std::sync::Mutex;
use std::process::{Command, Child};
use std::env;
use std::fs;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

struct AppState {
    proxy_process: Mutex<Option<Child>>,
}

#[tauri::command]
fn start_proxy(state: tauri::State<AppState>, _app_handle: tauri::AppHandle) -> Result<String, String> {
    let mut process = state.proxy_process.lock().map_err(|e| e.to_string())?;
    
    if process.is_some() {
        return Ok("Proxy is already running".into());
    }

    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    let mut mihomo_dir = current_dir.clone();
    mihomo_dir.push("configs");
    mihomo_dir.push("mihomo");
    
    let mut exe_path = mihomo_dir.clone();
    exe_path.push("mihomo.exe");
    
    if !exe_path.exists() {
        return Err(format!("mihomo.exe not found at {:?}", exe_path));
    }

    let mut command = Command::new(exe_path);
    command.arg("-d").arg(mihomo_dir);

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let child = command.spawn()
        .map_err(|e| format!("Failed to start proxy: {}", e))?;

    *process = Some(child);
    
    Ok("Proxy started successfully".into())
}

#[tauri::command]
fn stop_proxy(state: tauri::State<AppState>) -> Result<String, String> {
    let mut process = state.proxy_process.lock().map_err(|e| e.to_string())?;
    
    if let Some(mut child) = process.take() {
        let _ = child.kill();
        let _ = child.wait();
        Ok("Proxy stopped".into())
    } else {
        Ok("Proxy is not running".into())
    }
}

#[tauri::command]
fn save_subscription(content: String) -> Result<String, String> {
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    let mut config_path = current_dir.clone();
    config_path.push("configs");
    config_path.push("mihomo");
    config_path.push("config.yaml");

    // Before saving, ensure `external-controller` is preserved or added so we can still control it
    let mut final_content = content;
    if !final_content.contains("external-controller:") {
        final_content.push_str("\nexternal-controller: 127.0.0.1:9090\n");
    }

    fs::write(config_path, final_content).map_err(|e| format!("Failed to save config: {}", e))?;

    Ok("Subscription saved".into())
}

#[tauri::command]
fn set_proxy_provider_url(provider: String, url: String) -> Result<String, String> {
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    let mut config_path = current_dir.clone();
    config_path.push("configs");
    config_path.push("mihomo");
    config_path.push("config.yaml");

    let original = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    let needle = format!("{}:", provider);

    let mut provider_index: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == needle {
            provider_index = Some(i);
            break;
        }
    }

    let provider_index = provider_index.ok_or_else(|| format!("Provider not found: {}", provider))?;
    let provider_indent = lines[provider_index].chars().take_while(|c| *c == ' ').count();

    let mut url_index: Option<usize> = None;
    for i in (provider_index + 1)..lines.len() {
        let line = &lines[i];
        if line.trim().is_empty() {
            continue;
        }

        let indent = line.chars().take_while(|c| *c == ' ').count();
        if indent <= provider_indent && line.trim_end().ends_with(':') {
            break;
        }

        if line.trim_start().starts_with("url:") {
            url_index = Some(i);
            break;
        }
    }

    let url_index = url_index.ok_or_else(|| format!("url not found under provider: {}", provider))?;
    let url_indent = lines[url_index].chars().take_while(|c| *c == ' ').count();
    let escaped = url.replace('\\', "\\\\").replace('"', "\\\"");
    lines[url_index] = format!("{}url: \"{}\"", " ".repeat(url_indent), escaped);

    let mut updated = lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider url updated".into())
}

#[tauri::command]
fn add_proxy_provider(name: String, url: String) -> Result<String, String> {
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    let mut config_path = current_dir.clone();
    config_path.push("configs");
    config_path.push("mihomo");
    config_path.push("config.yaml");

    let original = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    
    // 找到proxy-providers部分的结束位置
    let mut proxy_providers_end: Option<usize> = None;
    let mut in_proxy_providers = false;
    
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "proxy-providers:" {
            in_proxy_providers = true;
            continue;
        }
        
        if in_proxy_providers && !line.trim().is_empty() && !line.starts_with(' ') {
            proxy_providers_end = Some(i);
            break;
        }
        
        if in_proxy_providers && i == lines.len() - 1 {
            proxy_providers_end = Some(lines.len());
            break;
        }
    }
    
    let proxy_providers_end = proxy_providers_end.ok_or_else(|| "proxy-providers section not found".to_string())?;
    
    // 生成新的provider配置
    let new_provider = format!("  {}:\n    url: \"{}\"\n    type: http\n    interval: 86400\n    health-check: {{enable: true,url: \"https://www.gstatic.com/generate_204\", interval: 300}}\n    override:\n      additional-prefix: \"[{}]\"\n", name, url.replace('\\', "\\\\").replace('\"', "\\\""), name);
    
    // 插入新的provider
    lines.insert(proxy_providers_end, new_provider);
    
    let mut updated = lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider added".into())
}

#[tauri::command]
fn update_proxy_provider(old_name: String, new_name: String, url: String) -> Result<String, String> {
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    let mut config_path = current_dir.clone();
    config_path.push("configs");
    config_path.push("mihomo");
    config_path.push("config.yaml");

    let original = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    let old_needle = format!("{}:", old_name);
    
    // 找到旧provider的位置
    let mut provider_index: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == old_needle {
            provider_index = Some(i);
            break;
        }
    }
    
    let provider_index = provider_index.ok_or_else(|| format!("Provider not found: {}", old_name))?;
    
    // 更新provider名称
    let indent = lines[provider_index].chars().take_while(|c| *c == ' ').count();
    lines[provider_index] = format!("{}{}:", " ".repeat(indent), new_name);
    
    // 更新URL和prefix
    let provider_indent = indent;
    for i in (provider_index + 1)..lines.len() {
        let line = &lines[i];
        if line.trim().is_empty() {
            continue;
        }
        
        let current_indent = line.chars().take_while(|c| *c == ' ').count();
        if current_indent <= provider_indent && line.trim_end().ends_with(':') {
            break;
        }
        
        if line.trim_start().starts_with("url:") {
            let url_indent = current_indent;
            let escaped = url.replace('\\', "\\\\").replace('\"', "\\\"");
            lines[i] = format!("{}url: \"{}\"", " ".repeat(url_indent), escaped);
        } else if line.trim_start().starts_with("additional-prefix:") {
            let prefix_indent = current_indent;
            lines[i] = format!("{}additional-prefix: \"[{}]\"", " ".repeat(prefix_indent), new_name);
        }
    }
    
    let mut updated = lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider updated".into())
}

#[tauri::command]
fn remove_proxy_provider(name: String) -> Result<String, String> {
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;
    let mut config_path = current_dir.clone();
    config_path.push("configs");
    config_path.push("mihomo");
    config_path.push("config.yaml");

    let original = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let mut lines: Vec<String> = original.lines().map(|l| l.to_string()).collect();
    let needle = format!("{}:", name);
    
    // 找到provider的位置
    let mut provider_index: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == needle {
            provider_index = Some(i);
            break;
        }
    }
    
    let provider_index = provider_index.ok_or_else(|| format!("Provider not found: {}", name))?;
    let provider_indent = lines[provider_index].chars().take_while(|c| *c == ' ').count();
    
    // 找到provider的结束位置
    let mut provider_end = provider_index + 1;
    while provider_end < lines.len() {
        let line = &lines[provider_end];
        if line.trim().is_empty() {
            provider_end += 1;
            continue;
        }
        
        let indent = line.chars().take_while(|c| *c == ' ').count();
        if indent <= provider_indent && line.trim_end().ends_with(':') {
            break;
        }
        
        provider_end += 1;
    }
    
    // 删除provider相关的行
    lines.drain(provider_index..provider_end);
    
    // 清理可能的空行
    let mut cleaned_lines: Vec<String> = Vec::new();
    let mut in_proxy_providers = false;
    let mut last_was_empty = false;
    
    for line in lines {
        if line.trim() == "proxy-providers:" {
            in_proxy_providers = true;
            cleaned_lines.push(line);
            last_was_empty = false;
        } else if in_proxy_providers {
            if line.trim().is_empty() {
                if !last_was_empty {
                    cleaned_lines.push(line);
                    last_was_empty = true;
                }
            } else {
                cleaned_lines.push(line);
                last_was_empty = false;
            }
        } else {
            let is_empty = line.trim().is_empty();
            cleaned_lines.push(line);
            last_was_empty = is_empty;
        }
    }
    

    let mut updated = cleaned_lines.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(&config_path, updated).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok("Provider removed".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(AppState {
        proxy_process: Mutex::new(None),
    })
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![start_proxy, stop_proxy, save_subscription, set_proxy_provider_url, add_proxy_provider, update_proxy_provider, remove_proxy_provider])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
