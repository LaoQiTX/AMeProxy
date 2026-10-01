//! Keep the saved catalog intact, but expose only one provider to mihomo.
use serde_yaml::{Mapping, Value};

pub const ACTIVE_KEY: &str = "ameproxy-active-subscription";

pub fn active_name(yaml: &Value) -> Option<String> {
    let providers = yaml.get("proxy-providers")?.as_mapping()?;
    if let Some(name) = yaml[ACTIVE_KEY].as_str() {
        if providers.contains_key(Value::String(name.into())) {
            return Some(name.into());
        }
    }
    // Older merged configurations retain the first provider in the default group.
    if let Some(groups) = yaml["proxy-groups"].as_sequence() {
        let group = groups
            .iter()
            .find(|group| group["name"].as_str() == Some("默认"))
            .or_else(|| {
                groups
                    .iter()
                    .find(|group| group["type"].as_str() == Some("select"))
            });
        if let Some(uses) = group.and_then(|group| group["use"].as_sequence()) {
            if let Some(name) = uses
                .iter()
                .find(|name| providers.contains_key(*name))
                .and_then(Value::as_str)
            {
                return Some(name.into());
            }
        }
    }
    providers.keys().find_map(Value::as_str).map(str::to_owned)
}

pub fn select(yaml: &mut Value, name: &str) -> Result<(), String> {
    if !yaml["proxy-providers"]
        .as_mapping()
        .is_some_and(|items| items.contains_key(Value::String(name.into())))
    {
        return Err(format!("订阅不存在: {name}"));
    }
    yaml[ACTIVE_KEY] = Value::String(name.into());
    Ok(())
}

pub fn persist_selection(yaml: &mut Value, preferred: Option<&str>) {
    let name = preferred.map(str::to_owned).or_else(|| active_name(yaml));
    if let Some(name) = name {
        yaml[ACTIVE_KEY] = Value::String(name);
    } else if let Some(root) = yaml.as_mapping_mut() {
        root.remove(Value::String(ACTIVE_KEY.into()));
    }
}

pub fn runtime(yaml: &Value) -> Result<Value, String> {
    let active = active_name(yaml);
    let mut runtime = yaml.clone();
    let root = runtime.as_mapping_mut().ok_or("配置必须是 YAML 对象")?;
    root.remove(Value::String(ACTIVE_KEY.into()));
    let mut selected = Mapping::new();
    if let Some(name) = &active {
        selected.insert(
            Value::String(name.clone()),
            yaml["proxy-providers"][name].clone(),
        );
    }
    root.insert(
        Value::String("proxy-providers".into()),
        Value::Mapping(selected),
    );
    if let Some(groups) = runtime
        .get_mut("proxy-groups")
        .and_then(Value::as_sequence_mut)
    {
        for group in groups {
            // Every provider-backed strategy now draws from the selected subscription.
            // Static proxies, filters, group type, and routing rules remain intact.
            let uses_providers = group["use"]
                .as_sequence()
                .is_some_and(|uses| !uses.is_empty());
            if uses_providers {
                let mapping = group.as_mapping_mut().ok_or("策略组格式不正确")?;
                if let Some(name) = &active {
                    mapping.insert(
                        Value::String("use".into()),
                        Value::Sequence(vec![Value::String(name.clone())]),
                    );
                } else {
                    mapping.remove(Value::String("use".into()));
                }
            }
            let has_proxies = group["proxies"]
                .as_sequence()
                .is_some_and(|items| !items.is_empty());
            let has_uses = group["use"]
                .as_sequence()
                .is_some_and(|items| !items.is_empty());
            let includes = [
                "include-all",
                "include-all-providers",
                "include-all-proxies",
            ]
            .iter()
            .any(|key| group[*key].as_bool() == Some(true));
            if !has_proxies && !has_uses && (!includes || active.is_none()) {
                group["proxies"] = serde_yaml::to_value(["DIRECT"]).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(runtime)
}

pub fn runtime_text(yaml: &Value) -> Result<String, String> {
    serde_yaml::to_string(&runtime(yaml)?).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Value {
        serde_yaml::from_str("proxy-providers:\n  first: {type: file, path: ./a.yaml}\n  second: {type: file, path: ./b.yaml}\nproxy-groups:\n  - {name: 默认, type: select, use: [first, second]}\n  - {name: 自动, type: url-test, include-all-providers: true}\nrules: ['MATCH,默认']").unwrap()
    }
    #[test]
    fn legacy_catalog_loads_only_one_provider() {
        let yaml = fixture();
        let runtime = runtime(&yaml).unwrap();
        assert_eq!(active_name(&yaml).as_deref(), Some("first"));
        assert_eq!(runtime["proxy-providers"].as_mapping().unwrap().len(), 1);
        assert_eq!(
            runtime["proxy-groups"][0]["use"],
            serde_yaml::to_value(["first"]).unwrap()
        );
        assert_eq!(yaml["proxy-providers"].as_mapping().unwrap().len(), 2);
    }
    #[test]
    fn selection_is_persistent_and_all_provider_groups_follow_it() {
        let mut yaml = fixture();
        select(&mut yaml, "second").unwrap();
        let saved: Value = serde_yaml::from_str(&serde_yaml::to_string(&yaml).unwrap()).unwrap();
        let runtime = runtime(&saved).unwrap();
        assert_eq!(active_name(&saved).as_deref(), Some("second"));
        assert!(runtime["proxy-providers"]["first"].is_null());
        assert!(runtime[ACTIVE_KEY].is_null());
        assert_eq!(
            runtime["proxy-groups"][0]["use"],
            serde_yaml::to_value(["second"]).unwrap()
        );
        assert_eq!(
            runtime["proxy-groups"][1]["include-all-providers"],
            Value::Bool(true)
        );
        assert_eq!(runtime["rules"], saved["rules"]);
        assert!(select(&mut yaml, "missing").is_err());
        assert_eq!(active_name(&yaml).as_deref(), Some("second"));
    }
}
