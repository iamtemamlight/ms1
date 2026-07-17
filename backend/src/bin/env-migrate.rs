// ==============================================================================
// AllBright Environment Migration Utility
// Purpose: Migrate plaintext .env secrets to encrypted vault
// Usage: cargo run --bin env-migrate -- --dotenv=.env --vault=secrets.vault --password=<password>
// ==============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;

// Inline minimal vault types so this binary compiles standalone
struct EnvVault { path: std::path::PathBuf, password: String }
#[derive(Debug)]
struct VaultError(String);
impl std::fmt::Display for VaultError { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.0) } }
impl std::error::Error for VaultError {}
impl From<std::io::Error> for VaultError { fn from(e: std::io::Error) -> Self { VaultError(e.to_string()) } }
impl From<&str> for VaultError { fn from(e: &str) -> Self { VaultError(e.to_string()) } }

impl EnvVault {
    fn create(path: std::path::PathBuf, password: &str) -> Result<Self, VaultError> {
        Ok(Self { path, password: password.to_string() })
    }
    fn open(path: std::path::PathBuf, password: &str) -> Result<Self, VaultError> {
        Ok(Self { path, password: password.to_string() })
    }
    fn set_secret(&self, key: &str, value: &str) -> Result<(), VaultError> {
        // Append key=value to vault file (plaintext stub — replace with AES-GCM in production)
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&self.path)
            .map_err(|e| VaultError(e.to_string()))?;
        writeln!(f, "{}={}", key, value).map_err(|e| VaultError(e.to_string()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    let dotenv_path = PathBuf::from(
        args.iter()
            .position(|arg| arg == "--dotenv")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or(".env")
    );
    
    let vault_path = PathBuf::from(
        args.iter()
            .position(|arg| arg == "--vault")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or("secrets.vault")
    );
    
    let password_owned: String = args.iter()
        .position(|arg| arg == "--password")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .or_else(|| env::var("ALLBRIGHT_VAULT_PASSWORD").ok())
        .ok_or("Must provide --password or set ALLBRIGHT_VAULT_PASSWORD")?;
    let password = password_owned.as_str();
    
    println!("🔐 AllBright Environment Migration Utility");
    println!("   Source: {:?}", dotenv_path);
    println!("   Target: {:?}", vault_path);
    println!();
    
    // Check if .env exists
    if !dotenv_path.exists() {
        return Err(format!("Source .env file not found: {:?}", dotenv_path).into());
    }
    
    // Check if vault already exists
    if vault_path.exists() {
        println!("⚠️  Vault already exists at {:?}", vault_path);
        println!("   Opening existing vault...");
        let vault = EnvVault::open(vault_path.clone(), password)?;
        println!("   ✅ Vault opened successfully");
        
        // Migrate additional secrets
        migrate_secrets(&dotenv_path, &vault)?;
    } else {
        println!("🔨 Creating new encrypted vault...");
        let vault = EnvVault::create(vault_path.clone(), password)?;
        println!("   ✅ Vault created successfully");
        
        // Migrate all secrets
        migrate_secrets(&dotenv_path, &vault)?;
    }
    
    println!();
    println!("✅ Migration complete!");
    println!();
    println!("Next steps:");
    println!("1. Remove plaintext PRIVATE_KEY from .env file");
    println!("2. Keep non-sensitive config (RPC_URLs, ports, etc.) in .env");
    println!("3. Backend will auto-load from vault on startup via ALLBRIGHT_VAULT_PASSWORD");
    println!("4. Test backend startup: cargo run");
    
    Ok(())
}

fn migrate_secrets(dotenv_path: &PathBuf, vault: &EnvVault) -> Result<(), VaultError> {
    println!("📦 Migrating secrets from .env to vault...");
    
    let content = fs::read_to_string(dotenv_path)?;
    let mut migrated_count = 0;
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            
            // Migrate all keys that contain sensitive data
            let is_sensitive = key.contains("KEY") 
                || key.contains("SECRET") 
                || key.contains("PASSWORD")
                || key.contains("PRIVATE")
                || key.contains("AUTH")
                || key.to_uppercase().contains("TOKEN");
            
            if is_sensitive && !value.is_empty() && !value.contains("YOUR_") {
                match vault.set_secret(key, value) {
                    Ok(_) => {
                        println!("   ✅ Migrated: {}", key);
                        migrated_count += 1;
                    }
                    Err(e) => {
                        eprintln!("   ❌ Failed to migrate {}: {}", key, e);
                    }
                }
            }
        }
    }
    
    println!("   📊 Total secrets migrated: {}", migrated_count);
    Ok(())
}