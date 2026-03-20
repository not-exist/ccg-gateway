use crate::db::models::{InstalledPlugin, MarketplaceInfo, MarketplacePlugin, PluginItem};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

type Result<T> = std::result::Result<T, String>;

/// 执行 claude 命令
fn run_claude(args: &[&str]) -> Result<String> {
    let args_str = args.iter().map(|s| {
        if s.contains(' ') || s.contains('"') {
            format!("\"{}\"", s.replace('"', "\\\""))
        } else {
            s.to_string()
        }
    }).collect::<Vec<_>>().join(" ");

    let output = Command::new("cmd")
        .args(["/c", &format!("claude {}", args_str)])
        .output()
        .map_err(|e| format!("执行 claude 命令失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("命令执行失败，退出码: {:?}", output.status.code())
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 从 plugin id 解析插件名和市场名
fn parse_plugin_id(id: &str) -> (String, Option<String>) {
    match id.split_once('@') {
        Some((name, marketplace)) => (name.to_string(), Some(marketplace.to_string())),
        None => (id.to_string(), None),
    }
}

/// 获取市场目录
fn get_marketplaces_dir(config_dir: &std::path::Path) -> PathBuf {
    config_dir.join("plugins").join("marketplaces")
}

/// 获取已安装插件列表
pub async fn get_installed_plugins_impl() -> Result<Vec<InstalledPlugin>> {
    let stdout = run_claude(&["plugin", "list", "--json"])?;

    if stdout.is_empty() {
        return Ok(vec![]);
    }

    #[derive(Deserialize)]
    struct PluginInfo {
        id: String,
        version: Option<String>,
        #[serde(default)]
        enabled: bool,
    }

    let plugins: Vec<PluginInfo> = serde_json::from_str(&stdout)
        .map_err(|e| format!("解析插件列表失败: {}", e))?;

    Ok(plugins.into_iter().map(|p| {
        let (name, marketplace) = parse_plugin_id(&p.id);
        InstalledPlugin {
            name,
            version: p.version,
            description: None,
            marketplace_name: marketplace,
            is_enabled: p.enabled,
        }
    }).collect())
}

/// 获取已安装市场列表
pub fn get_installed_marketplaces_impl(config_dir: &std::path::Path) -> Result<Vec<MarketplaceInfo>> {
    let marketplaces_dir = get_marketplaces_dir(config_dir);

    if !marketplaces_dir.exists() {
        return Ok(vec![]);
    }

    let mut marketplaces = Vec::new();

    let entries = std::fs::read_dir(&marketplaces_dir)
        .map_err(|e| format!("读取市场目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            let config_path = path.join(".claude-plugin").join("marketplace.json");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(mut info) = serde_json::from_str::<MarketplaceInfo>(&content) {
                        if info.name.is_empty() {
                            info.name = entry.file_name().to_string_lossy().to_string();
                        }
                        marketplaces.push(info);
                    }
                }
            }
        }
    }

    Ok(marketplaces)
}

/// 读取市场中的插件列表
pub fn read_marketplace_plugins(marketplace_name: &str, config_dir: &std::path::Path) -> Result<Vec<MarketplacePlugin>> {
    let marketplaces_dir = get_marketplaces_dir(config_dir);
    let config_path = marketplaces_dir.join(marketplace_name).join(".claude-plugin").join("marketplace.json");

    if !config_path.exists() {
        return Err(format!("市场配置文件不存在: {}", config_path.display()));
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取市场配置失败: {}", e))?;

    #[derive(Deserialize)]
    struct MarketplaceConfig {
        plugins: Vec<PluginEntry>,
    }

    #[derive(Deserialize)]
    struct PluginEntry {
        name: String,
        description: Option<String>,
    }

    let config: MarketplaceConfig = serde_json::from_str(&content)
        .map_err(|e| format!("解析市场配置失败: {}", e))?;

    Ok(config.plugins.into_iter().map(|p| MarketplacePlugin {
        name: p.name,
        version: None,
        description: p.description,
        marketplace_name: marketplace_name.to_string(),
    }).collect())
}

/// 安装插件
pub async fn install_plugin_impl(plugin_id: String) -> Result<()> {
    run_claude(&["plugin", "install", &plugin_id])?;
    Ok(())
}

/// 卸载插件
pub async fn uninstall_plugin_impl(plugin_id: String) -> Result<()> {
    run_claude(&["plugin", "uninstall", &plugin_id])?;
    Ok(())
}

/// 启用插件
pub async fn enable_plugin_impl(plugin_id: String) -> Result<()> {
    run_claude(&["plugin", "enable", &plugin_id])?;
    Ok(())
}

/// 停用插件
pub async fn disable_plugin_impl(plugin_id: String) -> Result<()> {
    run_claude(&["plugin", "disable", &plugin_id])?;
    Ok(())
}

/// 更新插件
pub async fn update_plugin_impl(plugin_id: String) -> Result<()> {
    run_claude(&["plugin", "update", &plugin_id])?;
    Ok(())
}

/// 添加市场
pub async fn add_marketplace_impl(url: String) -> Result<MarketplaceInfo> {
    let output = run_claude(&["marketplace", "add", &url])?;

    let name = output
        .split("Added marketplace:")
        .nth(1)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| url.split('/').last().unwrap_or("unknown").to_string());

    Ok(MarketplaceInfo {
        name,
        description: None,
        url: Some(url),
    })
}

/// 移除市场
pub async fn remove_marketplace_impl(name: String) -> Result<()> {
    run_claude(&["marketplace", "remove", &name])?;
    Ok(())
}

/// 更新市场
pub async fn update_marketplace_impl(name: String) -> Result<()> {
    run_claude(&["marketplace", "update", &name])?;
    Ok(())
}

/// 检查市场是否存在
pub fn check_marketplace_exists_impl(name: &str, config_dir: &std::path::Path) -> Result<bool> {
    let marketplaces = get_installed_marketplaces_impl(config_dir)?;
    Ok(marketplaces.iter().any(|m| m.name == name))
}

/// 获取所有插件列表
pub fn get_all_plugins_impl(config_dir: &std::path::Path) -> Result<Vec<PluginItem>> {
    let installed = get_installed_plugins_impl_sync()?;
    let marketplaces = get_installed_marketplaces_impl(config_dir)?;

    // 建立 name@marketplace → (version, enabled) 映射
    let installed_map: std::collections::HashMap<String, (Option<String>, bool)> = installed
        .iter()
        .map(|p| {
            let key = format!("{}@{}", p.name, p.marketplace_name.as_deref().unwrap_or(""));
            (key, (p.version.clone(), p.is_enabled))
        })
        .collect();

    let mut installed_plugins = Vec::new();
    let mut not_installed_plugins = Vec::new();

    // 以市场配置为基准遍历
    for market in &marketplaces {
        if let Ok(plugins) = read_marketplace_plugins(&market.name, config_dir) {
            for plugin in plugins {
                let key = format!("{}@{}", plugin.name, plugin.marketplace_name);
                if let Some((version, enabled)) = installed_map.get(&key) {
                    installed_plugins.push(PluginItem {
                        name: plugin.name,
                        version: version.clone(),
                        description: plugin.description,
                        marketplace_name: plugin.marketplace_name,
                        is_installed: true,
                        is_enabled: Some(*enabled),
                        is_favorited: false,
                    });
                } else {
                    not_installed_plugins.push(PluginItem {
                        name: plugin.name,
                        version: plugin.version,
                        description: plugin.description,
                        marketplace_name: plugin.marketplace_name,
                        is_installed: false,
                        is_enabled: None,
                        is_favorited: false,
                    });
                }
            }
        }
    }

    // 已安装的按名称排序，未安装的也按名称排序
    installed_plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    not_installed_plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // 已安装的放前面
    installed_plugins.extend(not_installed_plugins);
    Ok(installed_plugins)
}

/// 获取已安装插件列表（同步版本）
fn get_installed_plugins_impl_sync() -> Result<Vec<InstalledPlugin>> {
    let stdout = run_claude(&["plugin", "list", "--json"])?;

    if stdout.is_empty() {
        return Ok(vec![]);
    }

    #[derive(Deserialize)]
    struct PluginInfo {
        id: String,
        version: Option<String>,
        #[serde(default)]
        enabled: bool,
    }

    let plugins: Vec<PluginInfo> = serde_json::from_str(&stdout)
        .map_err(|e| format!("解析插件列表失败: {}", e))?;

    Ok(plugins.into_iter().map(|p| {
        let (name, marketplace) = parse_plugin_id(&p.id);
        InstalledPlugin {
            name,
            version: p.version,
            description: None,
            marketplace_name: marketplace,
            is_enabled: p.enabled,
        }
    }).collect())
}