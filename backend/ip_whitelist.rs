// IP Whitelisting for Production Security
// Restricts access to the system based on IP address whitelist
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpWhitelistConfig {
    pub enabled: bool,
    pub allowed_ips: HashSet<String>,
    pub allowed_cidrs: HashSet<String>,
    pub default_action: WhitelistAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhitelistAction {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpCheckResult {
    pub allowed: bool,
    pub ip: String,
    pub reason: String,
}

pub struct IpWhitelist {
    config: Arc<RwLock<IpWhitelistConfig>>,
}

impl IpWhitelist {
    pub fn new() -> Self {
        let config = IpWhitelistConfig {
            enabled: std::env::var("IP_WHITELIST_ENABLED")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            allowed_ips: Self::parse_allowed_ips(),
            allowed_cidrs: Self::parse_allowed_cidrs(),
            default_action: WhitelistAction::Deny,
        };

        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    fn parse_allowed_ips() -> HashSet<String> {
        std::env::var("ALLOWED_IPS")
            .ok()
            .and_then(|ips| {
                ips.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<HashSet<_>>()
                    .into()
            })
            .unwrap_or_default()
    }

    fn parse_allowed_cidrs() -> HashSet<String> {
        std::env::var("ALLOWED_CIDRS")
            .ok()
            .and_then(|cidrs| {
                cidrs.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<HashSet<_>>()
                    .into()
            })
            .unwrap_or_default()
    }

    /// Check if an IP address is allowed
    pub async fn is_allowed(&self, ip: &str) -> IpCheckResult {
        let config = self.config.read().await;

        if !config.enabled {
            return IpCheckResult {
                allowed: true,
                ip: ip.to_string(),
                reason: "IP whitelist is disabled".to_string(),
            };
        }

        // Check exact IP match
        if config.allowed_ips.contains(ip) {
            return IpCheckResult {
                allowed: true,
                ip: ip.to_string(),
                reason: "IP is in whitelist".to_string(),
            };
        }

        // Check CIDR ranges
        for cidr in &config.allowed_cidrs {
            if self.is_ip_in_cidr(ip, cidr) {
                return IpCheckResult {
                    allowed: true,
                    ip: ip.to_string(),
                    reason: format!("IP is in allowed CIDR: {}", cidr),
                };
            }
        }

        // Check localhost
        if self.is_localhost(ip) {
            return IpCheckResult {
                allowed: true,
                ip: ip.to_string(),
                reason: "IP is localhost".to_string(),
            };
        }

        IpCheckResult {
            allowed: false,
            ip: ip.to_string(),
            reason: "IP is not in whitelist".to_string(),
        }
    }

    /// Check if an IP is in a CIDR range
    fn is_ip_in_cidr(&self, ip: &str, cidr: &str) -> bool {
        let ip_addr: IpAddr = match ip.parse() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        let network: IpAddr = match parts[0].parse() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        let prefix_len: u32 = match parts[1].parse() {
            Ok(len) => len,
            Err(_) => return false,
        };

        match (ip_addr, network) {
            (IpAddr::V4(ip_v4), IpAddr::V4(network_v4)) => {
                self.is_ipv4_in_cidr(ip_v4, network_v4, prefix_len)
            }
            (IpAddr::V6(ip_v6), IpAddr::V6(network_v6)) => {
                self.is_ipv6_in_cidr(ip_v6, network_v6, prefix_len)
            }
            _ => false,
        }
    }

    fn is_ipv4_in_cidr(&self, ip: std::net::Ipv4Addr, network: std::net::Ipv4Addr, prefix_len: u32) -> bool {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        let network_u32 = u32::from_be_bytes(network.octets());
        let mask = if prefix_len == 0 {
            0
        } else {
            0xFFFFFFFF << (32 - prefix_len)
        };

        (ip_u32 & mask) == (network_u32 & mask)
    }

    fn is_ipv6_in_cidr(&self, ip: std::net::Ipv6Addr, network: std::net::Ipv6Addr, prefix_len: u32) -> bool {
        let ip_u128 = u128::from_be_bytes(ip.octets());
        let network_u128 = u128::from_be_bytes(network.octets());
        let mask = if prefix_len == 0 {
            0
        } else {
            0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF << (128 - prefix_len)
        };

        (ip_u128 & mask) == (network_u128 & mask)
    }

    fn is_localhost(&self, ip: &str) -> bool {
        matches!(ip, "127.0.0.1" | "::1" | "localhost")
    }

    /// Add an IP to the whitelist
    pub async fn add_ip(&self, ip: String) {
        let mut config = self.config.write().await;
        config.allowed_ips.insert(ip.clone());
        info!("Added IP to whitelist: {}", ip);
    }

    /// Remove an IP from the whitelist
    pub async fn remove_ip(&self, ip: &str) {
        let mut config = self.config.write().await;
        config.allowed_ips.remove(ip);
        info!("Removed IP from whitelist: {}", ip);
    }

    /// Add a CIDR range to the whitelist
    pub async fn add_cidr(&self, cidr: String) {
        let mut config = self.config.write().await;
        config.allowed_cidrs.insert(cidr.clone());
        info!("Added CIDR to whitelist: {}", cidr);
    }

    /// Remove a CIDR range from the whitelist
    pub async fn remove_cidr(&self, cidr: &str) {
        let mut config = self.config.write().await;
        config.allowed_cidrs.remove(cidr);
        info!("Removed CIDR from whitelist: {}", cidr);
    }

    /// Enable or disable the whitelist
    pub async fn set_enabled(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.enabled = enabled;
        info!("IP whitelist {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Get the current whitelist configuration
    pub async fn get_config(&self) -> IpWhitelistConfig {
        let config = self.config.read().await;
        IpWhitelistConfig {
            enabled: config.enabled,
            allowed_ips: config.allowed_ips.clone(),
            allowed_cidrs: config.allowed_cidrs.clone(),
            default_action: config.default_action.clone(),
        }
    }

    /// Reload configuration from environment variables
    pub async fn reload_config(&self) {
        let mut config = self.config.write().await;
        config.enabled = std::env::var("IP_WHITELIST_ENABLED")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        config.allowed_ips = Self::parse_allowed_ips();
        config.allowed_cidrs = Self::parse_allowed_cidrs();
        info!("IP whitelist configuration reloaded");
    }
}

impl Clone for IpWhitelist {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_localhost_allowed() {
        let whitelist = IpWhitelist::new();
        let result = whitelist.is_allowed("127.0.0.1").await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_ipv6_localhost_allowed() {
        let whitelist = IpWhitelist::new();
        let result = whitelist.is_allowed("::1").await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_add_ip() {
        let whitelist = IpWhitelist::new();
        whitelist.add_ip("192.168.1.100".to_string()).await;
        let result = whitelist.is_allowed("192.168.1.100").await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_cidr_range() {
        let whitelist = IpWhitelist::new();
        whitelist.add_cidr("192.168.1.0/24".to_string()).await;
        
        let result = whitelist.is_allowed("192.168.1.50").await;
        assert!(result.allowed);
        
        let result = whitelist.is_allowed("192.168.2.50").await;
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_remove_ip() {
        let whitelist = IpWhitelist::new();
        whitelist.add_ip("192.168.1.100".to_string()).await;
        whitelist.remove_ip("192.168.1.100").await;
        
        let result = whitelist.is_allowed("192.168.1.100").await;
        assert!(!result.allowed);
    }
}
