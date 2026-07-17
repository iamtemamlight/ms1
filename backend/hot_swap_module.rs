// ==============================================================================
// MODULE: Hot-Swap Module System
// Purpose: Dynamic module loading with version compatibility checking and rollback
//          Enables zero-downtime updates for fleet modules
// ==============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Module version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub hash: String,
}

impl ModuleVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch, hash: String::new() }
    }

    pub fn is_compatible_with(&self, other: &ModuleVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::fmt::Display for ModuleVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleStatus {
    Loading,
    Active,
    RollingBack,
    Failed,
    Unloaded,
}

pub trait HotSwapModule: Send + Sync {
    fn init(&self) -> Result<(), String>;
    fn name(&self) -> &str;
    fn version(&self) -> ModuleVersion;
    fn validate(&self) -> Result<bool, String>;
    fn shutdown(&self) -> Result<(), String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDescriptor {
    pub name: String,
    pub version: ModuleVersion,
    pub dependencies: Vec<String>,
    pub config: HashMap<String, String>,
    pub status: ModuleStatus,
    pub load_time_ms: u64,
}

pub struct HotSwapRegistry {
    modules: Arc<RwLock<HashMap<String, ModuleDescriptor>>>,
    previous_versions: Arc<RwLock<HashMap<String, ModuleDescriptor>>>,
    max_rollback_storage: usize,
}

impl HotSwapRegistry {
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            previous_versions: Arc::new(RwLock::new(HashMap::new())),
            max_rollback_storage: 5,
        }
    }

    pub async fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), String> {
        let name = descriptor.name.clone();
        let mut modules = self.modules.write().await;
        
        if let Some(existing) = modules.get(&name) {
            if !descriptor.version.is_compatible_with(&existing.version) {
                return Err(format!("Version incompatibility: {} vs {}",
                    descriptor.version, existing.version));
            }
        }
        
        modules.insert(name, descriptor);
        Ok(())
    }

    pub async fn load_module<M: HotSwapModule + 'static>(&self, module: Arc<M>) -> Result<ModuleDescriptor, String> {
        module.init()?;
        let name = module.name().to_string();
        let version = module.version();
        let start = std::time::Instant::now();
        
        if !module.validate()? {
            return Err(format!("Module {} failed health check", name));
        }
        
        let load_time_ms = start.elapsed().as_millis() as u64;
        
        let descriptor = ModuleDescriptor {
            name: name.clone(),
            version,
            dependencies: vec![],
            config: HashMap::new(),
            status: ModuleStatus::Active,
            load_time_ms,
        };
        
        self.register_module(descriptor.clone()).await?;
        Ok(descriptor)
    }

    pub async fn hot_swap<M: HotSwapModule + 'static>(&self, new_module: Arc<M>) -> Result<ModuleDescriptor, String> {
        let name = new_module.name().to_string();
        
        {
            let modules = self.modules.read().await;
            if let Some(old) = modules.get(&name) {
                let mut prev = self.previous_versions.write().await;
                prev.insert(name.clone(), old.clone());
                
                let mut keys: Vec<String> = prev.keys().cloned().collect();
                while keys.len() > self.max_rollback_storage {
                    if let Some(oldest) = keys.first() {
                        prev.remove(oldest);
                        keys.remove(0);
                    }
                }
            }
        }
        
        {
            let mut modules = self.modules.write().await;
            if let Some(old) = modules.get_mut(&name) {
                old.status = ModuleStatus::RollingBack;
            }
        }
        
        let result = self.load_module(new_module).await;
        
        match result {
            Ok(descriptor) => {
                let mut modules = self.modules.write().await;
                if let Some(new) = modules.get_mut(&name) {
                    new.status = ModuleStatus::Active;
                }
                Ok(descriptor)
            }
            Err(e) => {
                self.rollback(&name).await?;
                Err(format!("Hot-swap failed: {}", e))
            }
        }
    }

    pub async fn rollback(&self, name: &str) -> Result<(), String> {
        let mut prev = self.previous_versions.write().await;
        
        if let Some(previous) = prev.remove(name) {
            let mut modules = self.modules.write().await;
            if let Some(current) = modules.get_mut(name) {
                current.status = ModuleStatus::Failed;
            }
            let mut rollback = previous.clone();
            rollback.status = ModuleStatus::Active;
            modules.insert(name.to_string(), rollback);
            Ok(())
        } else {
            Err(format!("No previous version for {}", name))
        }
    }

    pub async fn get_status(&self, name: &str) -> Option<ModuleStatus> {
        let modules = self.modules.read().await;
        modules.get(name).map(|m| m.status.clone())
    }

    pub async fn list_modules(&self) -> Vec<ModuleDescriptor> {
        let modules = self.modules.read().await;
        modules.values().cloned().collect()
    }

    pub async fn unload_module(&self, name: &str) -> Result<(), String> {
        let mut modules = self.modules.write().await;
        if modules.remove(name).is_some() { Ok(()) } else { Err(format!("Module {} not found", name)) }
    }

    pub async fn health_check(&self, name: &str) -> bool {
        let modules = self.modules.read().await;
        modules.get(name).map(|m| m.status == ModuleStatus::Active).unwrap_or(false)
    }

    /// Restart a module by cycling its status through Failed -> Active.
    /// In Autonomous mode, the Copilot calls this to self-heal modules.
    pub async fn restart_module(&self, name: &str) -> Result<(), String> {
        let mut modules = self.modules.write().await;
        if let Some(module) = modules.get_mut(name) {
            module.status = ModuleStatus::Active;
            module.load_time_ms = 0; // reset load time to indicate restart
            Ok(())
        } else {
            Err(format!("Module '{}' not found in registry", name))
        }
    }

    /// List modules as a HashMap (name -> descriptor) for the Copilot System Access layer.
    pub async fn list_modules_map(&self) -> std::collections::HashMap<String, ModuleDescriptor> {
        let modules = self.modules.read().await;
        modules.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadResult {
    pub success: bool,
    pub module_name: String,
    pub version: String,
    pub load_time_ms: u64,
    pub error: Option<String>,
}

impl LoadResult {
    pub fn success(name: &str, version: &str, time_ms: u64) -> Self {
        Self { success: true, module_name: name.to_string(), version: version.to_string(), load_time_ms: time_ms, error: None }
    }
    pub fn failure(name: &str, error: String) -> Self {
        Self { success: false, module_name: name.to_string(), version: String::new(), load_time_ms: 0, error: Some(error) }
    }
}

#[derive(Debug)]
pub struct DefaultHotSwapModule {
    pub name: String,
    pub version: ModuleVersion,
}

impl DefaultHotSwapModule {
    pub fn new(name: &str, major: u16, minor: u16, patch: u16) -> Self {
        Self { name: name.to_string(), version: ModuleVersion::new(major, minor, patch) }
    }
}

impl HotSwapModule for DefaultHotSwapModule {
    fn init(&self) -> Result<(), String> { Ok(()) }
    fn name(&self) -> &str { &self.name }
    fn version(&self) -> ModuleVersion { self.version.clone() }
    fn validate(&self) -> Result<bool, String> { Ok(true) }
    fn shutdown(&self) -> Result<(), String> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        let v1 = ModuleVersion::new(1, 0, 0);
        let v2 = ModuleVersion::new(1, 1, 0);
        let v3 = ModuleVersion::new(2, 0, 0);
        assert!(v2.is_compatible_with(&v1));
        assert!(!v3.is_compatible_with(&v1));
    }

    #[tokio::test]
    async fn test_registry() {
        let registry = HotSwapRegistry::new();
        let descriptor = ModuleDescriptor {
            name: "test".to_string(),
            version: ModuleVersion::new(1, 0, 0),
            dependencies: vec![],
            config: HashMap::new(),
            status: ModuleStatus::Active,
            load_time_ms: 10,
        };
        assert!(registry.register_module(descriptor).await.is_ok());
    }
}
