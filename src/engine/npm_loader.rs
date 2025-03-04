use deno_core::{ModuleLoader, ModuleSource, ModuleType, ModuleSpecifier};
use deno_core::error::AnyError;
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use serde_json;
use anyhow;

pub struct NpmModuleLoader {
    node_modules_path: PathBuf,
}

impl NpmModuleLoader {
    pub fn new(node_modules_path: PathBuf) -> Self {
        Self { node_modules_path }
    }

    pub fn resolve_npm_module(&self, specifier: &str) -> Option<PathBuf> {
        if specifier.starts_with("./") || specifier.starts_with("../") {
            return None;
        }
        let parts = specifier.split("/").collect::<Vec<&str>>();
        let package_name = if parts[0].starts_with('@') && parts.len() > 1 {
            // 处理 @scope/package 格式
            format!("{}/{}", parts[0], parts[1])
        } else {
            parts[0].to_string()
        };
        
        // 查找 package.json 确定入口文件
        let package_dir = self.node_modules_path.join(&package_name);
        if !package_dir.exists() {
            return None;
        }
        let package_json_path = package_dir.join("package.json");
        if package_json_path.exists() {
            if let Ok(content) = fs::read_to_string(&package_json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    // 优先使用 module 字段 (ESM)，然后是 main 字段
                    let entry_point = json.get("module")
                        .or_else(|| json.get("main"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("index.js");
                    
                    return Some(package_dir.join(entry_point));
                }
            }
        }
        
        // 默认尝试 index.js
        Some(package_dir.join("index.js"))
    }
    
    pub fn load_npm_module(&self, path: &PathBuf, specifier: &str) -> Option<ModuleSource> {
        if path.exists() {
            if let Ok(code) = fs::read_to_string(path) {
                return Some(ModuleSource::new(
                    code.into(),
                    specifier.to_string(),
                    path.to_string_lossy().to_string(),
                    None,
                ));
            }
        }
        None
    }
}

impl ModuleLoader for NpmModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: deno_core::ResolutionKind,
    ) -> Result<ModuleSpecifier, deno_core::error::ModuleLoaderError> {
        // 先尝试标准解析
        if let Ok(ms) = deno_core::resolve_import(specifier, referrer) {
            return Ok(ms);
        }
        
        // 失败时返回错误
        Err(deno_core::error::custom_error(
            "ModuleNotFound",
            format!("Cannot resolve module: {}", specifier)
        ).into())
    }
    
    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<ModuleSpecifier>,
        _is_dyn_import: bool,
        _requested_module_type: deno_core::RequestedModuleType
    ) -> deno_core::ModuleLoadResponse {
        let path = match module_specifier.to_file_path() {
            Ok(p) => p,
            Err(_) => {
                // 尝试作为 npm 模块加载
                if let Some(npm_path) = self.resolve_npm_module(module_specifier.path()) {
                    npm_path
                } else {
                    return deno_core::ModuleLoadResponse::Sync(Err(
                        deno_core::error::custom_error("ModuleNotFound", format!("Module not found: {}", module_specifier)).into()
                    ));
                }
            }
        };
        
        // 加载文件内容
        match fs::read_to_string(&path) {
            Ok(code) => {
                let module_type = if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    ModuleType::Json
                } else {
                    ModuleType::JavaScript
                };
                
                deno_core::ModuleLoadResponse::Sync(Ok(ModuleSource::new(
                    code.into(),
                    module_specifier.clone(),
                    module_specifier.clone(),
                    None,
                )))
            },
            Err(e) => deno_core::ModuleLoadResponse::Sync(Err(
                deno_core::error::custom_error("ModuleLoadError", format!("Failed to load module: {}", e)).into()
            )),
        }
    }
}
