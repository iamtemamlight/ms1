//! Append-only, persistence-backed audit trail (spec §8.3: "Complete record of actions").
//!
//! Every governance action (reflection submitted, gatekeeper decision, deployment
//! check) is appended as one JSON line to the audit log file and reloaded on startup
//! so the trail survives process restarts.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub seq: u64,
    pub timestamp: i64,
    pub actor: String,        // who performed the action (AEA / IAA / Commander)
    pub action: String,       // e.g. "reflection.submit", "gatekeeper.decision"
    pub target: String,       // resource / reflection id
    pub outcome: String,      // e.g. "approved", "rejected", "flagged"
    pub evidence_hash: String, // sha256 of the evidence payload (integrity)
}

/// Thread-safe audit store backed by a JSON-lines file.
pub struct AuditStore {
    path: PathBuf,
    inner: Mutex<Vec<AuditEntry>>,
    seq: Mutex<u64>,
}

impl AuditStore {
    /// Open (or create) the audit log at `path`, replaying existing entries.
    pub fn open(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut entries = Vec::new();
        let mut max_seq = 0u64;
        if path.exists() {
            let file = File::open(&path)?;
            for line in BufReader::new(file).lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<AuditEntry>(&line) {
                    max_seq = max_seq.max(entry.seq);
                    entries.push(entry);
                }
            }
        }
        Ok(Self {
            path,
            inner: Mutex::new(entries),
            seq: Mutex::new(max_seq + 1),
        })
    }

    /// Append an entry; returns the assigned sequence number. Persists immediately.
    pub fn append(
        &self,
        actor: &str,
        action: &str,
        target: &str,
        outcome: &str,
        evidence_hash: &str,
    ) -> std::io::Result<u64> {
        let seq = {
            let mut s = self.seq.lock().unwrap();
            let n = *s;
            *s += 1;
            n
        };
        let entry = AuditEntry {
            seq,
            timestamp: chrono::Utc::now().timestamp(),
            actor: actor.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            outcome: outcome.to_string(),
            evidence_hash: evidence_hash.to_string(),
        };
        // Persist (append) before recording in memory.
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        writeln!(file, "{}", serde_json::to_string(&entry).unwrap())?;
        file.flush()?;
        self.inner.lock().unwrap().push(entry);
        Ok(seq)
    }

    pub fn entries(&self) -> Vec<AuditEntry> {
        self.inner.lock().unwrap().clone()
    }

    pub fn count(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
}