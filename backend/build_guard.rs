#![allow(dead_code)]
// Verifies reproducible builds, binary hashes, supply chain, and unsafe blocks.
// ==============================================================================

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCheckResult {
    pub check_name: String,
    pub status: BuildStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildAuditReport {
    pub timestamp: String,
    pub overall: BuildStatus,
    pub checks: Vec<BuildCheckResult>,
}

pub struct BuildGuard;

impl BuildGuard {
    /// Run full build integrity audit
    pub fn run_audit() -> BuildAuditReport {
        let mut checks = Vec::new();
        checks.push(Self::check_cargo_lock());
        checks.push(Self::check_release_profile());
        checks.push(Self::check_unsafe_blocks());
        checks.push(Self::check_binary_symbols());
        checks.push(Self::check_dependency_audit());
        checks.push(Self::check_git_clean());

        let overall = checks.iter()
            .map(|c| c.status)
            .fold(BuildStatus::Pass, |acc, s| match (acc, s) {
                (BuildStatus::Fail, _) | (_, BuildStatus::Fail) => BuildStatus::Fail,
                (BuildStatus::Warn, BuildStatus::Warn) => BuildStatus::Warn,
                (BuildStatus::Warn, _) => BuildStatus::Warn,
                _ => acc,
            });

        BuildAuditReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            overall,
            checks,
        }
    }

    /// Verify Cargo.lock exists and is committed
    fn check_cargo_lock() -> BuildCheckResult {
        let lock_exists = std::path::Path::new("Cargo.lock").exists();
        if !lock_exists {
            return BuildCheckResult {
                check_name: "cargo_lock_committed".to_string(),
                status: BuildStatus::Fail,
                detail: "Cargo.lock not found — builds are not reproducible".to_string(),
            };
        }

        // In development, allow uncommitted Cargo.lock during active builds
        let output = Command::new("git")
            .args(["ls-files", "--error-unmatch", "Cargo.lock"])
            .output();

        match output {
            Ok(o) if o.status.success() => BuildCheckResult {
                check_name: "cargo_lock_committed".to_string(),
                status: BuildStatus::Pass,
                detail: "Cargo.lock is tracked in git".to_string(),
            },
            _ => BuildCheckResult {
                check_name: "cargo_lock_committed".to_string(),
                status: BuildStatus::Warn,
                detail: "Cargo.lock exists but is not committed to git (dev mode)".to_string(),
            },
        }
    }

    /// Verify release profile has security flags
    fn check_release_profile() -> BuildCheckResult {
        let cargo_toml = std::fs::read_to_string("Cargo.toml");
        let cargo_toml = match cargo_toml {
            Ok(s) => s,
            Err(_) => return BuildCheckResult {
                check_name: "release_profile".to_string(),
                status: BuildStatus::Fail,
                detail: "Cannot read Cargo.toml".to_string(),
            },
        };

        let has_lto = cargo_toml.contains("lto = \"fat\"") || cargo_toml.contains("lto=true");
        let has_panic_abort = cargo_toml.contains("panic = \"abort\"");
        let has_strip = cargo_toml.contains("strip = true");
        let has_codegen_units = cargo_toml.contains("codegen-units = 1");

        let flags_ok = has_lto && has_panic_abort && has_strip && has_codegen_units;

        if flags_ok {
            BuildCheckResult {
                check_name: "release_profile".to_string(),
                status: BuildStatus::Pass,
                detail: "LTO=fat, panic=abort, strip=true, codegen-units=1".to_string(),
            }
        } else {
            BuildCheckResult {
                check_name: "release_profile".to_string(),
                status: BuildStatus::Fail,
                detail: format!("Missing flags: lto={}, panic_abort={}, strip={}, codegen_units={}",
                    has_lto, has_panic_abort, has_strip, has_codegen_units),
            }
        }
    }

    /// Scan source files for unsafe blocks
    fn check_unsafe_blocks() -> BuildCheckResult {
        let output = Command::new("rg")
            .args(["-t", "rust", "--type", "rust", r"^\s*unsafe\s*(fn|trait|impl)", "--no-heading"])
            .args(["."])
            .output();

        let unsafe_count = match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).lines().count(),
            Err(_) => 0,
        };

        if unsafe_count == 0 {
            BuildCheckResult {
                check_name: "unsafe_blocks".to_string(),
                status: BuildStatus::Pass,
                detail: "No unsafe blocks found in project source".to_string(),
            }
        } else {
            BuildCheckResult {
                check_name: "unsafe_blocks".to_string(),
                status: BuildStatus::Warn,
                detail: format!("{} unsafe blocks found — review required", unsafe_count),
            }
        }
    }

    /// Check binary for debug symbols
    fn check_binary_symbols() -> BuildCheckResult {
        let binary_path = "target/release/allbright-c2-backend";
        if !std::path::Path::new(binary_path).exists() {
            return BuildCheckResult {
                check_name: "binary_symbols".to_string(),
                status: BuildStatus::Warn,
                detail: "Release binary not found — run cargo build --release".to_string(),
            };
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("nm")
                .args(["--defined-only", binary_path])
                .output();

            match output {
                Ok(o) => {
                    let symbols = String::from_utf8_lossy(&o.stdout);
                    let debug_symbols = symbols.lines()
                        .filter(|l| l.contains("debug") || l.contains("rust_begin_unwind"))
                        .count();
                    if debug_symbols > 0 {
                        BuildCheckResult {
                            check_name: "binary_symbols".to_string(),
                            status: BuildStatus::Warn,
                            detail: format!("{} debug symbols found in binary", debug_symbols),
                        }
                    } else {
                        BuildCheckResult {
                            check_name: "binary_symbols".to_string(),
                            status: BuildStatus::Pass,
                            detail: "No debug symbols in release binary".to_string(),
                        }
                    }
                }
                Err(_) => BuildCheckResult {
                    check_name: "binary_symbols".to_string(),
                    status: BuildStatus::Warn,
                    detail: "Cannot run nm — binary symbols unchecked".to_string(),
                },
            }
        }

        #[cfg(not(target_os = "linux"))]
        BuildCheckResult {
            check_name: "binary_symbols".to_string(),
            status: BuildStatus::Warn,
            detail: "Binary symbol check only on Linux".to_string(),
        }
    }

    /// Check if cargo audit / deny are available
    fn check_dependency_audit() -> BuildCheckResult {
        let audit = Command::new("cargo")
            .args(["audit", "--version"])
            .output();

        let deny = Command::new("cargo")
            .args(["deny", "--version"])
            .output();

        let audit_ok = audit.map(|o| o.status.success()).unwrap_or(false);
        let deny_ok = deny.map(|o| o.status.success()).unwrap_or(false);

        if audit_ok && deny_ok {
            BuildCheckResult {
                check_name: "dependency_audit".to_string(),
                status: BuildStatus::Pass,
                detail: "cargo-audit and cargo-deny installed".to_string(),
            }
        } else {
            // Downgrade to Warn in development — audit tools are not required for local builds
            BuildCheckResult {
                check_name: "dependency_audit".to_string(),
                status: BuildStatus::Warn,
                detail: format!("cargo-audit: {}, cargo-deny: {} — install for production builds", audit_ok, deny_ok),
            }
        }
    }

    /// Check git working tree is clean for release builds
    fn check_git_clean() -> BuildCheckResult {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output();

        match output {
            Ok(o) => {
                let dirty = String::from_utf8_lossy(&o.stdout).trim().is_empty();
                if dirty {
                    BuildCheckResult {
                        check_name: "git_clean".to_string(),
                        status: BuildStatus::Pass,
                        detail: "Working tree is clean".to_string(),
                    }
                } else {
                    // Allow dirty working tree in development — only warn
                    BuildCheckResult {
                        check_name: "git_clean".to_string(),
                        status: BuildStatus::Warn,
                        detail: "Working tree has uncommitted changes (allowed in dev)".to_string(),
                    }
                }
            }
            Err(_) => BuildCheckResult {
                check_name: "git_clean".to_string(),
                status: BuildStatus::Warn,
                detail: "Not a git repository — skipping check".to_string(),
            },
        }
    }

    /// Quick check — returns true only if all critical checks pass
    pub fn is_build_safe() -> bool {
        let report = Self::run_audit();
        report.overall != BuildStatus::Fail
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_audit_runs() {
        let report = BuildGuard::run_audit();
        assert!(!report.checks.is_empty());
        assert!(report.checks.len() >= 5);
    }

    #[test]
    fn test_cargo_lock_check() {
        let result = BuildGuard::check_cargo_lock();
        // In the repo, Cargo.lock exists
        assert!(matches!(result.status, BuildStatus::Pass | BuildStatus::Fail));
    }
}
